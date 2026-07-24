const HOST_NAME = "com.antigravity.agent_browser";
const PROTOCOL_VERSION = "1.0";
const RECONNECT_DELAY_MS = 5000;
const CONTENT_READY_TIMEOUT_MS = 5000;

let nativePort = null;
let reconnectTimer = null;
const authorizedTabs = new Map();

function sendToNative(message) {
  if (!nativePort) {
    return;
  }
  nativePort.postMessage(message);
}

function structuredError(ref, code, message, details = undefined) {
  return {
    protocolVersion: PROTOCOL_VERSION,
    ref,
    isError: true,
    error: { code, message, details },
  };
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function scheduleNativeReconnect() {
  if (reconnectTimer !== null) {
    return;
  }
  reconnectTimer = setTimeout(() => {
    reconnectTimer = null;
    connectToNativeHost();
  }, RECONNECT_DELAY_MS);
}

async function showNativeConnectionError(message) {
  await chrome.action.setBadgeText({ text: "ERR" }).catch(() => {});
  await chrome.action
    .setBadgeBackgroundColor({ color: "#dc2626" })
    .catch(() => {});
  console.error(
    `Native host connection failed for extension ${chrome.runtime.id}: ${message}`,
  );
}

function isInjectableUrl(url) {
  return typeof url === "string" && /^https?:\/\//i.test(url);
}

async function waitForTabReady(tabId) {
  const tab = await chrome.tabs.get(tabId);
  if (tab.status !== "loading") {
    return tab;
  }

  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      chrome.tabs.onUpdated.removeListener(listener);
      reject(new Error("tab_load_timeout"));
    }, CONTENT_READY_TIMEOUT_MS);

    function listener(updatedTabId, changeInfo, updatedTab) {
      if (updatedTabId === tabId && changeInfo.status === "complete") {
        clearTimeout(timeout);
        chrome.tabs.onUpdated.removeListener(listener);
        resolve(updatedTab);
      }
    }

    chrome.tabs.onUpdated.addListener(listener);
  });
}

async function pingContentScript(tabId) {
  try {
    const response = await chrome.tabs.sendMessage(tabId, { action: "ping" });
    return response?.ok === true;
  } catch {
    return false;
  }
}

async function ensureContentScript(tabId) {
  const tab = await waitForTabReady(tabId);
  if (!isInjectableUrl(tab.url)) {
    throw new Error(`unsupported_tab_url:${tab.url || "unknown"}`);
  }

  if (await pingContentScript(tabId)) {
    return;
  }

  await chrome.scripting.executeScript({
    target: { tabId, allFrames: false },
    files: ["content.js"],
  });

  for (let attempt = 0; attempt < 3; attempt += 1) {
    if (await pingContentScript(tabId)) {
      return;
    }
    await delay(100 * (attempt + 1));
  }

  throw new Error("content_script_not_ready");
}

async function authorizeActiveTab() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab?.id || !tab.url) {
    return;
  }

  try {
    await ensureContentScript(tab.id);
    authorizedTabs.set(tab.id, new URL(tab.url).origin);
    await chrome.action.setBadgeText({ tabId: tab.id, text: "ON" });
    await chrome.action.setBadgeBackgroundColor({ tabId: tab.id, color: "#2563eb" });
    sendToNative({
      protocolVersion: PROTOCOL_VERSION,
      type: "tab.authorized",
      tab: { id: tab.id, url: tab.url, title: tab.title ?? "" },
    });
  } catch (error) {
    sendToNative(structuredError(null, "tab_authorization_failed", String(error)));
  }
}

chrome.action.onClicked.addListener(() => {
  authorizeActiveTab();
});

chrome.tabs.onRemoved.addListener((tabId) => {
  authorizedTabs.delete(tabId);
});

chrome.tabs.onUpdated.addListener((tabId, changeInfo) => {
  if (!authorizedTabs.has(tabId) || !changeInfo.url) {
    return;
  }
  let nextOrigin;
  try {
    nextOrigin = new URL(changeInfo.url).origin;
  } catch {
    nextOrigin = null;
  }
  if (nextOrigin !== authorizedTabs.get(tabId)) {
    authorizedTabs.delete(tabId);
    chrome.action.setBadgeText({ tabId, text: "" }).catch(() => {});
    sendToNative({
      protocolVersion: PROTOCOL_VERSION,
      type: "tab.authorization_revoked",
      tabId,
      reason: "navigation",
    });
  }
});

async function resolveAuthorizedTab(request) {
  if (Number.isInteger(request.tabId)) {
    if (!authorizedTabs.has(request.tabId)) {
      throw new Error("tab_not_authorized");
    }
    return chrome.tabs.get(request.tabId);
  }

  const [active] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!active?.id || !authorizedTabs.has(active.id)) {
    throw new Error("active_tab_not_authorized");
  }
  return active;
}

async function handleNativeMessage(request) {
  const ref = request?.id ?? null;
  if (!request || request.protocolVersion !== PROTOCOL_VERSION) {
    return structuredError(ref, "unsupported_protocol", "Protocol version is missing or unsupported");
  }

  if (request.action === "tabs.authorized") {
    const tabs = [];
    for (const tabId of authorizedTabs.keys()) {
      try {
        const tab = await chrome.tabs.get(tabId);
        tabs.push({ id: tab.id, url: tab.url, title: tab.title ?? "", active: tab.active });
      } catch {
        authorizedTabs.delete(tabId);
      }
    }
    return { protocolVersion: PROTOCOL_VERSION, ref, isError: false, data: { tabs } };
  }

  if (request.action === "tab.detach") {
    if (!Number.isInteger(request.tabId)) {
      return structuredError(ref, "invalid_request", "tabId is required");
    }
    authorizedTabs.delete(request.tabId);
    await chrome.action.setBadgeText({ tabId: request.tabId, text: "" }).catch(() => {});
    return { protocolVersion: PROTOCOL_VERSION, ref, isError: false, data: { detached: true } };
  }

  let tab;
  try {
    tab = await resolveAuthorizedTab(request);
    await ensureContentScript(tab.id);
  } catch (error) {
    const code = String(error.message || error);
    return structuredError(
      ref,
      code,
      "Click the Antigravity Connector icon in the target tab to authorize it",
    );
  }

  try {
    const response = await chrome.tabs.sendMessage(tab.id, request);
    return {
      protocolVersion: PROTOCOL_VERSION,
      ref,
      tabId: tab.id,
      isError: Boolean(response?.error),
      ...(response?.error ? { error: response.error } : { data: response }),
    };
  } catch (error) {
    return structuredError(ref, "content_script_error", String(error));
  }
}

function connectToNativeHost() {
  if (nativePort) {
    return;
  }

  const port = chrome.runtime.connectNative(HOST_NAME);
  nativePort = port;
  port.onMessage.addListener((message) => {
    handleNativeMessage(message)
      .then(sendToNative)
      .catch((error) => sendToNative(structuredError(message?.id, "internal_error", String(error))));
  });

  port.onDisconnect.addListener(() => {
    const lastError = chrome.runtime.lastError;
    const message = lastError?.message ?? "Native host disconnected";
    if (nativePort === port) {
      nativePort = null;
    }
    showNativeConnectionError(message);
    scheduleNativeReconnect();
  });

  chrome.action.setBadgeText({ text: "" }).catch(() => {});
  sendToNative({
    protocolVersion: PROTOCOL_VERSION,
    type: "extension.ready",
    extensionId: chrome.runtime.id,
    extensionVersion: chrome.runtime.getManifest().version,
  });
}

connectToNativeHost();
