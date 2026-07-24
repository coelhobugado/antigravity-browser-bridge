const HOST_NAME = "com.antigravity.agent_browser";
const PROTOCOL_VERSION = "1.0";
const RECONNECT_DELAY_MS = 5000;
const CONTENT_READY_TIMEOUT_MS = 5000;

let nativePort = null;
let reconnectTimer = null;
const authorizedTabs = new Map(); // tabId -> { origin, nonce, authorizedAt }
const recentActivity = []; // Array of { action, origin, time, success }

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

function logActivity(action, origin, success) {
  const time = new Date().toLocaleTimeString();
  recentActivity.push({ action, origin, time, success });
  if (recentActivity.length > 20) {
    recentActivity.shift();
  }
}

async function updateBadge(tabId = null, text = "ON", color = "#2563eb") {
  try {
    if (tabId) {
      await chrome.action.setBadgeText({ tabId, text });
      await chrome.action.setBadgeBackgroundColor({ tabId, color });
    } else {
      await chrome.action.setBadgeText({ text });
      await chrome.action.setBadgeBackgroundColor({ color });
    }
  } catch {}
}

async function persistSessionState() {
  try {
    const list = Array.from(authorizedTabs.entries()).map(([id, info]) => ({
      id,
      origin: info.origin,
      nonce: info.nonce,
    }));
    await chrome.storage.session.set({ authorizedTabs: list });
  } catch {}
}

async function restoreSessionState() {
  try {
    const data = await chrome.storage.session.get("authorizedTabs");
    if (Array.isArray(data?.authorizedTabs)) {
      for (const item of data.authorizedTabs) {
        try {
          const tab = await chrome.tabs.get(item.id);
          const currentOrigin = new URL(tab.url).origin;
          if (currentOrigin === item.origin) {
            authorizedTabs.set(item.id, {
              origin: item.origin,
              nonce: item.nonce,
              authorizedAt: Date.now(),
            });
            await updateBadge(item.id, "ON", "#2563eb");
          }
        } catch {
          // Tab closed or origin changed
        }
      }
    }
  } catch {}
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

async function authorizeTab(tab) {
  if (!tab?.id || !tab.url) {
    return;
  }
  try {
    await ensureContentScript(tab.id);
    const origin = new URL(tab.url).origin;
    const nonceBytes = new Uint8Array(16);
    crypto.getRandomValues(nonceBytes);
    const nonce = Array.from(nonceBytes, (byte) =>
      byte.toString(16).padStart(2, "0"),
    ).join("");
    authorizedTabs.set(tab.id, { origin, nonce, authorizedAt: Date.now() });
    await updateBadge(tab.id, "ON", "#2563eb");
    await persistSessionState();

    sendToNative({
      protocolVersion: PROTOCOL_VERSION,
      type: "tab.authorized",
      tab: { id: tab.id, url: tab.url, title: tab.title ?? "" },
    });
    logActivity("authorize_tab", origin, true);
  } catch (error) {
    sendToNative(structuredError(null, "tab_authorization_failed", String(error)));
  }
}

async function revokeTab(tabId) {
  if (authorizedTabs.has(tabId)) {
    const info = authorizedTabs.get(tabId);
    authorizedTabs.delete(tabId);
    await updateBadge(tabId, "", "#000000");
    await persistSessionState();
    sendToNative({
      protocolVersion: PROTOCOL_VERSION,
      type: "tab.authorization_revoked",
      tabId,
      reason: "user_revoked",
    });
    logActivity("revoke_tab", info?.origin, true);
  }
}

async function revokeAllTabs() {
  for (const tabId of Array.from(authorizedTabs.keys())) {
    await revokeTab(tabId);
  }
}

chrome.tabs.onRemoved.addListener((tabId) => {
  revokeTab(tabId);
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
  const current = authorizedTabs.get(tabId);
  if (nextOrigin !== current?.origin) {
    revokeTab(tabId);
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
    await revokeTab(request.tabId);
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
    await updateBadge(tab.id, "BUSY", "#3b82f6");
    const response = await chrome.tabs.sendMessage(tab.id, request);
    await updateBadge(tab.id, "ON", "#2563eb");
    logActivity(request.action, new URL(tab.url).origin, !response?.error);
    return {
      protocolVersion: PROTOCOL_VERSION,
      ref,
      tabId: tab.id,
      isError: Boolean(response?.error),
      ...(response?.error ? { error: response.error } : { data: response }),
    };
  } catch (error) {
    await updateBadge(tab.id, "ERR", "#ef4444");
    logActivity(request.action, tab.url, false);
    return structuredError(ref, "content_script_error", String(error));
  }
}

function connectToNativeHost() {
  if (nativePort) {
    return;
  }

  try {
    const port = chrome.runtime.connectNative(HOST_NAME);
    nativePort = port;
    updateBadge(null, "ON", "#22c55e");
    sendToNative({
      protocolVersion: PROTOCOL_VERSION,
      type: "extension.ready",
      extensionId: chrome.runtime.id,
      extensionVersion: chrome.runtime.getManifest().version,
    });

    port.onMessage.addListener((message) => {
      handleNativeMessage(message)
        .then(sendToNative)
        .catch((error) => sendToNative(structuredError(message?.id, "internal_error", String(error))));
    });

    port.onDisconnect.addListener(() => {
      const message =
        chrome.runtime.lastError?.message ?? "Native host disconnected";
      nativePort = null;
      updateBadge(null, "OFF", "#ef4444");
      console.warn(`Native host disconnected: ${message}`);
      scheduleNativeReconnect();
    });
  } catch {
    nativePort = null;
    updateBadge(null, "OFF", "#ef4444");
    scheduleNativeReconnect();
  }
}

// Popup message listener
chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
  if (msg.action === "popup.get_status") {
    (async () => {
      const tabs = [];
      for (const tabId of authorizedTabs.keys()) {
        try {
          const tab = await chrome.tabs.get(tabId);
          tabs.push({ id: tab.id, title: tab.title || tab.url, url: tab.url });
        } catch {}
      }
      sendResponse({
        status: nativePort ? "ON" : "OFF",
        authorizedTabs: tabs,
        recentActivity: recentActivity,
      });
    })();
    return true;
  }

  if (msg.action === "popup.authorize_current") {
    (async () => {
      const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
      if (tab) {
        await authorizeTab(tab);
      }
      sendResponse({ ok: true });
    })();
    return true;
  }

  if (msg.action === "popup.revoke_tab") {
    (async () => {
      if (msg.tabId) {
        await revokeTab(msg.tabId);
      }
      sendResponse({ ok: true });
    })();
    return true;
  }

  if (msg.action === "popup.revoke_all") {
    (async () => {
      await revokeAllTabs();
      sendResponse({ ok: true });
    })();
    return true;
  }
});

restoreSessionState();
connectToNativeHost();
