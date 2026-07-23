// content.js - Injetado em todas as páginas para extrair DOM e executar ações

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  console.log("Comando do background:", request);

  try {
    switch (request.action) {
      case "observe":
        sendResponse(capturePageState());
        break;
      
      case "click":
        const clickResult = simulateClick(request.selector || request.id);
        sendResponse({ success: clickResult });
        break;
        
      case "type":
        const typeResult = simulateType(request.selector || request.id, request.text);
        sendResponse({ success: typeResult });
        break;

      default:
        sendResponse({ error: "Ação desconhecida" });
    }
  } catch (err) {
    sendResponse({ error: err.toString() });
  }

  // Retorna true para indicar que a resposta é assíncrona/pode demorar
  return true;
});

function capturePageState() {
  return {
    url: window.location.href,
    title: document.title,
    // Aqui no futuro adicionaremos o extrator completo do WP-3 (accessibility tree)
    nodes_summary: `Página com ${document.querySelectorAll('*').length} elementos.`
  };
}

function simulateClick(selector) {
  const el = document.querySelector(selector);
  if (!el) return false;
  el.click();
  return true;
}

function simulateType(selector, text) {
  const el = document.querySelector(selector);
  if (!el) return false;
  el.value = text;
  el.dispatchEvent(new Event('input', { bubbles: true }));
  el.dispatchEvent(new Event('change', { bubbles: true }));
  return true;
}
