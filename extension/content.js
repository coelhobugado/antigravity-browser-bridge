(() => {
  if (globalThis.__antigravityConnectorInstalled) {
    return;
  }
  globalThis.__antigravityConnectorInstalled = true;

  const MAX_TEXT_LENGTH = 20000;
  const MAX_ELEMENTS = 500;
  const observedElements = new Map();

  function error(code, message, details = undefined) {
    return { error: { code, message, details } };
  }

  function resolveElement(target = {}) {
    if (target.ref && observedElements.has(target.ref)) {
      const observed = observedElements.get(target.ref);
      if (observed?.isConnected) {
        return observed;
      }
    }
    if (target.selector) {
      return document.querySelector(target.selector);
    }

    const candidates = Array.from(
      document.querySelectorAll(
        "a,button,input,textarea,select,[role],[contenteditable='true'],[tabindex]",
      ),
    );
    return candidates.find((element) => {
      const role = element.getAttribute("role") || element.tagName.toLowerCase();
      const name = accessibleName(element);
      return (!target.role || role === target.role) &&
        (!target.name || name.toLowerCase().includes(target.name.toLowerCase()));
    });
  }

  function accessibleName(element) {
    return (
      element.getAttribute("aria-label") ||
      element.getAttribute("title") ||
      element.getAttribute("placeholder") ||
      element.innerText ||
      element.textContent ||
      ""
    ).trim().slice(0, 500);
  }

  function elementSummary(element, index) {
    const rect = element.getBoundingClientRect();
    return {
      ref: `e${index + 1}`,
      tag: element.tagName.toLowerCase(),
      role: element.getAttribute("role") || null,
      name: accessibleName(element),
      type: element.getAttribute("type"),
      disabled: Boolean(element.disabled || element.getAttribute("aria-disabled") === "true"),
      visible: rect.width > 0 && rect.height > 0,
    };
  }

  function observe() {
    observedElements.clear();
    const elements = Array.from(
      document.querySelectorAll(
        "a,button,input,textarea,select,[role],[contenteditable='true'],[tabindex]",
      ),
    )
      .filter((element) => {
        const rect = element.getBoundingClientRect();
        return rect.width > 0 && rect.height > 0;
      })
      .slice(0, MAX_ELEMENTS);
    const interactive = elements.map((element, index) => {
      const summary = elementSummary(element, index);
      observedElements.set(summary.ref, element);
      return summary;
    });

    return {
      url: location.href,
      origin: location.origin,
      title: document.title,
      readyState: document.readyState,
      focused: document.activeElement ? elementSummary(document.activeElement, 0) : null,
      interactive,
      visibleText: (document.body?.innerText || "").slice(0, MAX_TEXT_LENGTH),
      truncated: interactive.length === MAX_ELEMENTS ||
        (document.body?.innerText || "").length > MAX_TEXT_LENGTH,
      timestamp: new Date().toISOString(),
    };
  }

  function setNativeValue(element, value) {
    if (element.isContentEditable) {
      element.focus();
      document.execCommand("selectAll", false);
      document.execCommand("insertText", false, value);
      element.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertText", data: value }));
      return;
    }

    const prototype = element instanceof HTMLTextAreaElement
      ? HTMLTextAreaElement.prototype
      : HTMLInputElement.prototype;
    const descriptor = Object.getOwnPropertyDescriptor(prototype, "value");
    descriptor?.set?.call(element, value);
    element.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertText", data: value }));
    element.dispatchEvent(new Event("change", { bubbles: true }));
  }

  function execute(request) {
    switch (request.action) {
      case "ping":
        return { ok: true };
      case "observe":
        return observe();
      case "get_text": {
        const element = resolveElement(request.target);
        return element
          ? { text: (element.innerText || element.textContent || "").slice(0, MAX_TEXT_LENGTH) }
          : error("target_not_found", "No element matched the requested target");
      }
      case "click": {
        const element = resolveElement(request.target);
        if (!element) {
          return error("target_not_found", "No element matched the requested target");
        }
        element.scrollIntoView({ block: "center", inline: "center" });
        element.click();
        return { clicked: true };
      }
      case "focus": {
        const element = resolveElement(request.target);
        if (!element) {
          return error("target_not_found", "No element matched the requested target");
        }
        element.focus();
        return { focused: document.activeElement === element };
      }
      case "type":
      case "fill": {
        if (typeof request.text !== "string") {
          return error("invalid_request", "text must be a string");
        }
        const element = resolveElement(request.target);
        if (!element) {
          return error("target_not_found", "No element matched the requested target");
        }
        element.focus();
        setNativeValue(element, request.text);
        return { filled: true };
      }
      default:
        return error("unknown_action", `Unsupported action: ${String(request.action)}`);
    }
  }

  chrome.runtime.onMessage.addListener((request, _sender, sendResponse) => {
    try {
      sendResponse(execute(request));
    } catch (caught) {
      sendResponse(error("execution_failed", String(caught)));
    }
    return false;
  });
})();
