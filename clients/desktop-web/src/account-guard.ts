// Classic head script: preserve real interaction before either module arrives.
(() => {
  if (Object.hasOwn(window, "__consoleAccountGuard")) return;
  let touched = false;
  const initialX = window.scrollX, initialY = window.scrollY;
  const observe = (event: Event) => {
    if (event.isTrusted || event.type === "focusin" || event.type === "submit") touched = true;
  };
  for (const type of ["keydown", "keyup", "pointerdown", "pointerup", "mousedown", "click", "beforeinput", "input", "change", "compositionstart", "compositionupdate", "compositionend", "wheel", "touchstart", "touchmove", "scroll", "focusin", "submit"]) {
    window.addEventListener(type, observe, {capture: true, passive: true});
  }
  Object.defineProperty(window, "__consoleAccountGuard", {
    configurable: false, writable: false,
    value: Object.freeze({canPromote: () => !touched && window.scrollX === initialX && window.scrollY === initialY
      && (document.activeElement === null || document.activeElement === document.body || document.activeElement === document.documentElement)}),
  });
})();
