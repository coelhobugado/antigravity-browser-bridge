document.addEventListener("DOMContentLoaded", async () => {
  const statusBadge = document.getElementById("statusBadge");
  const currentTabTitle = document.getElementById("currentTabTitle");
  const currentTabOrigin = document.getElementById("currentTabOrigin");
  const btnAuthorize = document.getElementById("btnAuthorize");
  const btnRevoke = document.getElementById("btnRevoke");
  const btnRevokeAll = document.getElementById("btnRevokeAll");
  const btnDiagnostics = document.getElementById("btnDiagnostics");
  const authorizedTabList = document.getElementById("authorizedTabList");
  const recentActivityList = document.getElementById("recentActivityList");

  let currentTab = null;

  async function updateUI() {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    currentTab = tab;

    if (tab && tab.url) {
      currentTabTitle.textContent = tab.title || tab.url;
      try {
        currentTabOrigin.textContent = new URL(tab.url).origin;
      } catch {
        currentTabOrigin.textContent = tab.url;
      }
    } else {
      currentTabTitle.textContent = "Aba não disponível";
      currentTabOrigin.textContent = "-";
    }

    // Query background status
    chrome.runtime.sendMessage({ action: "popup.get_status" }, (response) => {
      if (chrome.runtime.lastError || !response) {
        statusBadge.textContent = "OFF";
        statusBadge.className = "badge off";
        return;
      }

      const status = response.status || "OFF";
      statusBadge.textContent = status;
      statusBadge.className = `badge ${status.toLowerCase()}`;

      renderAuthorizedTabs(response.authorizedTabs || []);
      renderRecentActivity(response.recentActivity || []);
    });
  }

  function renderAuthorizedTabs(tabs) {
    if (!tabs || tabs.length === 0) {
      authorizedTabList.innerHTML = '<div style="font-size:12px; color:var(--text-muted);">Nenhuma aba autorizada.</div>';
      return;
    }

    authorizedTabList.replaceChildren();
    for (const tab of tabs) {
      const item = document.createElement("div");
      item.className = "tab-item";
      const title = document.createElement("div");
      title.className = "tab-title";
      title.title = tab.url;
      title.textContent = tab.title || tab.url;
      const button = document.createElement("button");
      button.className = "btn-icon";
      button.type = "button";
      button.textContent = "×";
      button.setAttribute("aria-label", `Revogar acesso a ${title.textContent}`);
      button.addEventListener("click", () => {
        chrome.runtime.sendMessage(
          { action: "popup.revoke_tab", tabId: tab.id },
          updateUI,
        );
      });
      item.append(title, button);
      authorizedTabList.append(item);
    }
  }

  function renderRecentActivity(activities) {
    if (!activities || activities.length === 0) {
      recentActivityList.innerHTML = '<div>Nenhuma atividade registrada.</div>';
      return;
    }

    recentActivityList.replaceChildren();
    for (const activity of activities.slice(-5).reverse()) {
      const item = document.createElement("div");
      item.className = "activity-item";
      const description = document.createElement("span");
      description.textContent = `${activity.action} (${activity.origin || "page"})`;
      const time = document.createElement("span");
      time.textContent = activity.time;
      item.append(description, time);
      recentActivityList.append(item);
    }
  }

  btnAuthorize.addEventListener("click", () => {
    chrome.runtime.sendMessage({ action: "popup.authorize_current" }, updateUI);
  });

  btnRevoke.addEventListener("click", () => {
    if (currentTab?.id) {
      chrome.runtime.sendMessage({ action: "popup.revoke_tab", tabId: currentTab.id }, updateUI);
    }
  });

  btnRevokeAll.addEventListener("click", () => {
    chrome.runtime.sendMessage({ action: "popup.revoke_all" }, updateUI);
  });

  btnDiagnostics.addEventListener("click", () => {
    chrome.tabs.create({ url: "chrome://extensions/?id=" + chrome.runtime.id });
  });

  updateUI();
  setInterval(updateUI, 2000);
});
