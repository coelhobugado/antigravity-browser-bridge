(() => {
  if (globalThis.__antigravityConnectorInstalled) {
    return;
  }
  globalThis.__antigravityConnectorInstalled = true;

  const MAX_TEXT_LENGTH = 20000;
  const MAX_ELEMENTS = 500;
  const observedElements = new Map();

  // Document generation UUID refreshed per page lifecycle
  const documentGeneration = `gen-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;

  function error(code, message, details = undefined) {
    return { error: { code, message, details } };
  }

  function delay(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  // --- Accessibility & Shadow DOM Traversal ---

  function getImplicitRole(element) {
    const explicitRole = element.getAttribute("role");
    if (explicitRole) return explicitRole;
    const tag = element.tagName.toLowerCase();
    const type = element.getAttribute("type")?.toLowerCase();
    if (tag === "button") return "button";
    if (tag === "a" && element.hasAttribute("href")) return "link";
    if (tag === "input") {
      if (type === "checkbox") return "checkbox";
      if (type === "radio") return "radio";
      if (type === "button" || type === "submit" || type === "reset") return "button";
      return "textbox";
    }
    if (tag === "textarea") return "textbox";
    if (tag === "select") return "combobox";
    if (tag === "dialog") return "dialog";
    if (tag === "h1" || tag === "h2" || tag === "h3" || tag === "h4" || tag === "h5" || tag === "h6") return "heading";
    return null;
  }

  function getAccessibleName(element) {
    // 1. aria-labelledby
    const labelledby = element.getAttribute("aria-labelledby");
    if (labelledby) {
      const parts = labelledby.split(/\s+/).map((id) => {
        const target = document.getElementById(id);
        return target ? target.innerText || target.textContent : "";
      });
      const name = parts.join(" ").trim();
      if (name) return name.slice(0, 500);
    }

    // 2. aria-label
    const ariaLabel = element.getAttribute("aria-label");
    if (ariaLabel?.trim()) return ariaLabel.trim().slice(0, 500);

    // 3. label[for]
    if (element.id) {
      const labelEl = document.querySelector(
        `label[for="${CSS.escape(element.id)}"]`,
      );
      if (labelEl) {
        const text = (labelEl.innerText || labelEl.textContent || "").trim();
        if (text) return text.slice(0, 500);
      }
    }

    // 4. parent label
    const parentLabel = element.closest("label");
    if (parentLabel) {
      const text = (parentLabel.innerText || parentLabel.textContent || "").trim();
      if (text) return text.slice(0, 500);
    }

    // 5. placeholder / title / alt
    const placeholder = element.getAttribute("placeholder");
    if (placeholder?.trim()) return placeholder.trim().slice(0, 500);
    const title = element.getAttribute("title");
    if (title?.trim()) return title.trim().slice(0, 500);
    const alt = element.getAttribute("alt");
    if (alt?.trim()) return alt.trim().slice(0, 500);

    // 6. innerText / textContent
    return (element.innerText || element.textContent || "").trim().slice(0, 500);
  }

  function isSensitiveElement(element) {
    const type = element.getAttribute("type")?.toLowerCase();
    const name = element.getAttribute("name")?.toLowerCase() || "";
    const id = element.getAttribute("id")?.toLowerCase() || "";
    const autocomplete = element.getAttribute("autocomplete")?.toLowerCase() || "";

    return (
      type === "password" ||
      autocomplete.includes("cc-") ||
      name.includes("password") ||
      name.includes("secret") ||
      name.includes("cvv") ||
      name.includes("card") ||
      id.includes("password") ||
      id.includes("secret")
    );
  }

  function collectElementsFromNode(root, results) {
    if (!root || results.length >= MAX_ELEMENTS) return;

    const selectors = "a,button,input,textarea,select,[role],[contenteditable='true'],[tabindex],dialog,main,nav,header,footer";
    let elements = [];
    try {
      elements = Array.from(root.querySelectorAll(selectors));
    } catch {
      return;
    }

    for (const el of elements) {
      if (results.length >= MAX_ELEMENTS) break;
      const rect = el.getBoundingClientRect();
      if (rect.width > 0 && rect.height > 0) {
        results.push(el);
      }
      if (el.shadowRoot) {
        collectElementsFromNode(el.shadowRoot, results);
      }
      if (el.tagName.toLowerCase() === "iframe") {
        try {
          if (el.contentDocument) {
            collectElementsFromNode(el.contentDocument, results);
          }
        } catch {
          // Cross-origin iframe
        }
      }
    }
  }

  function elementSummary(element, index) {
    const rect = element.getBoundingClientRect();
    const sensitive = isSensitiveElement(element);
    const inViewport = (
      rect.top >= 0 &&
      rect.left >= 0 &&
      rect.bottom <= (window.innerHeight || document.documentElement.clientHeight) &&
      rect.right <= (window.innerWidth || document.documentElement.clientWidth)
    );

    let rawValue = element.value !== undefined ? String(element.value) : null;
    if (sensitive && rawValue) {
      rawValue = "[REDACTED_SENSITIVE]";
    }

    return {
      ref: `e${index + 1}`,
      tag: element.tagName.toLowerCase(),
      role: getImplicitRole(element),
      name: getAccessibleName(element),
      type: element.getAttribute("type") || null,
      disabled: Boolean(element.disabled || element.getAttribute("aria-disabled") === "true"),
      readOnly: Boolean(element.readOnly || element.getAttribute("aria-readonly") === "true"),
      focused: document.activeElement === element,
      sensitive,
      value: rawValue,
      rect: {
        x: Math.round(rect.x),
        y: Math.round(rect.y),
        width: Math.round(rect.width),
        height: Math.round(rect.height),
      },
      inViewport,
    };
  }

  function observe() {
    observedElements.clear();
    const rawElements = [];
    collectElementsFromNode(document, rawElements);

    const interactive = rawElements.slice(0, MAX_ELEMENTS).map((element, index) => {
      const summary = elementSummary(element, index);
      observedElements.set(summary.ref, element);
      return summary;
    });

    const isModalOpen = Boolean(
      document.querySelector("dialog[open], [role='dialog'], .modal.open, .overlay.visible")
    );

    return {
      documentGeneration,
      url: location.href,
      origin: location.origin,
      title: document.title,
      readyState: document.readyState,
      scroll: { x: window.scrollX, y: window.scrollY },
      isModalOpen,
      focused: document.activeElement ? elementSummary(document.activeElement, 0) : null,
      interactive,
      visibleText: (document.body?.innerText || "").slice(0, MAX_TEXT_LENGTH),
      truncated: interactive.length === MAX_ELEMENTS,
      timestamp: new Date().toISOString(),
    };
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

    const candidates = [];
    collectElementsFromNode(document, candidates);
    return candidates.find((element) => {
      const role = getImplicitRole(element);
      const name = getAccessibleName(element);
      return (
        (!target.role || role === target.role) &&
        (!target.name || name.toLowerCase().includes(target.name.toLowerCase()))
      );
    });
  }

  // --- Pipeline: Hit Testing & Geometric Stability ---

  function performHitTest(element) {
    const rect = element.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;

    const hit = document.elementFromPoint(centerX, centerY);
    if (!hit) return { ok: true };

    if (hit === element || element.contains(hit) || hit.contains(element)) {
      return { ok: true };
    }

    return {
      ok: false,
      blocker: hit.tagName.toLowerCase() + (hit.id ? `#${hit.id}` : ""),
    };
  }

  async function validateAndStabilize(element) {
    if (!element || !element.isConnected) {
      throw new Error("element_detached");
    }

    const rect1 = element.getBoundingClientRect();
    if (rect1.width === 0 || rect1.height === 0) {
      throw new Error("element_not_visible");
    }

    if (element.disabled || element.getAttribute("aria-disabled") === "true") {
      throw new Error("element_disabled");
    }

    await delay(50);
    const rect2 = element.getBoundingClientRect();
    if (Math.abs(rect1.x - rect2.x) > 5 || Math.abs(rect1.y - rect2.y) > 5) {
      throw new Error("element_animating");
    }

    const hit = performHitTest(element);
    if (!hit.ok) {
      throw new Error(`element_covered_by:${hit.blocker}`);
    }
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

  async function typeSequentially(element, text) {
    element.focus();
    let currentValue = element.isContentEditable
      ? element.textContent || ""
      : element.value || "";
    for (const char of text) {
      element.dispatchEvent(new KeyboardEvent("keydown", { key: char, bubbles: true }));
      element.dispatchEvent(new KeyboardEvent("keypress", { key: char, bubbles: true }));
      currentValue += char;
      setNativeValue(element, currentValue);
      element.dispatchEvent(new KeyboardEvent("keyup", { key: char, bubbles: true }));
      await delay(10);
    }
  }

  // --- Action Execution ---

  async function execute(request) {
    if (request.documentGeneration && request.documentGeneration !== documentGeneration) {
      return error(
        "stale_document_generation",
        `Target document generation '${request.documentGeneration}' does not match current page generation '${documentGeneration}'`
      );
    }

    switch (request.action) {
      case "ping":
        return { ok: true, documentGeneration };
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
        if (!element) return error("target_not_found", "No element matched the requested target");
        element.scrollIntoView({ block: "center", inline: "center" });
        await validateAndStabilize(element);
        element.click();
        return { clicked: true, documentGeneration, urlAfter: location.href };
      }
      case "focus": {
        const element = resolveElement(request.target);
        if (!element) return error("target_not_found", "No element matched the requested target");
        element.focus();
        return { focused: document.activeElement === element };
      }
      case "fill": {
        if (typeof request.text !== "string") return error("invalid_request", "text must be a string");
        const element = resolveElement(request.target);
        if (!element) return error("target_not_found", "No element matched the requested target");
        element.scrollIntoView({ block: "center", inline: "center" });
        await validateAndStabilize(element);
        element.focus();
        setNativeValue(element, request.text);
        return { filled: true, documentGeneration };
      }
      case "type": {
        if (typeof request.text !== "string") return error("invalid_request", "text must be a string");
        const element = resolveElement(request.target);
        if (!element) return error("target_not_found", "No element matched the requested target");
        element.scrollIntoView({ block: "center", inline: "center" });
        await validateAndStabilize(element);
        await typeSequentially(element, request.text);
        return { typed: true, documentGeneration };
      }
      case "press": {
        const key = request.key || "Enter";
        const targetEl = document.activeElement || document.body;
        targetEl.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true }));
        targetEl.dispatchEvent(new KeyboardEvent("keyup", { key, bubbles: true }));
        return { pressed: key };
      }
      case "hover": {
        const element = resolveElement(request.target);
        if (!element) return error("target_not_found", "No element matched the requested target");
        element.dispatchEvent(new MouseEvent("mouseenter", { bubbles: true }));
        element.dispatchEvent(new MouseEvent("mouseover", { bubbles: true }));
        return { hovered: true };
      }
      case "select": {
        const element = resolveElement(request.target);
        if (!element || element.tagName.toLowerCase() !== "select") {
          return error("invalid_target", "Target must be a <select> element");
        }
        element.value = request.value;
        element.dispatchEvent(new Event("change", { bubbles: true }));
        return { selected: element.value };
      }
      case "scroll_into_view": {
        const element = resolveElement(request.target);
        if (!element) return error("target_not_found", "No element matched the requested target");
        element.scrollIntoView({ block: "center", inline: "center" });
        return { scrolled: true };
      }
      default:
        return error("unknown_action", `Unsupported action: ${String(request.action)}`);
    }
  }

  chrome.runtime.onMessage.addListener((request, _sender, sendResponse) => {
    execute(request)
      .then((res) => sendResponse(res))
      .catch((caught) => sendResponse(error("execution_failed", String(caught.message || caught))));
    return true;
  });
})();
