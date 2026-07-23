// background.js - Gerencia a comunicação com o binário em Rust via Native Messaging

const HOST_NAME = "com.antigravity.agent_browser";
let port = null;

function connectToNativeHost() {
  port = chrome.runtime.connectNative(HOST_NAME);

  port.onMessage.addListener((message) => {
    console.log("Recebido do Rust:", message);
    handleRustMessage(message);
  });

  port.onDisconnect.addListener(() => {
    console.warn("Desconectado do host nativo:", chrome.runtime.lastError?.message);
    port = null;
    // Tenta reconectar após alguns segundos
    setTimeout(connectToNativeHost, 5000);
  });
}

function handleRustMessage(message) {
  // O motor Rust enviou uma instrução (ex: observar tela, clicar, digitar)
  if (!message.action) return;

  chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
    if (tabs.length === 0) return;
    
    // Repassa o comando para o content_script da aba ativa
    chrome.tabs.sendMessage(tabs[0].id, message, (response) => {
      if (chrome.runtime.lastError) {
        console.error("Erro ao falar com a aba:", chrome.runtime.lastError.message);
        sendToRust({ isError: true, error: chrome.runtime.lastError.message, ref: message.id });
      } else {
        // Envia o resultado de volta pro motor Rust
        sendToRust({ isError: false, data: response, ref: message.id });
      }
    });
  });
}

function sendToRust(msg) {
  if (port) {
    port.postMessage(msg);
  } else {
    console.error("Tentativa de enviar mensagem, mas o host nativo não está conectado.");
  }
}

// Iniciar conexão quando a extensão carregar
connectToNativeHost();
