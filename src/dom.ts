import { t } from "./i18n";

export function failLoudly(messageKey: string, error?: any): never {
  const translatedMsg = t(messageKey) || messageKey;
  console.error("[CRITICAL]", translatedMsg, error);
  
  const criticalAlertBar = document.querySelector("#critical-alert-bar");
  const criticalAlertMsg = document.querySelector("#critical-alert-msg");

  if (criticalAlertBar && criticalAlertMsg) {
    criticalAlertMsg.textContent = `[${t("errorTitle")}] ${translatedMsg} ${error ? `(${error})` : ""}`;
    criticalAlertBar.classList.remove("hidden");
  }
  
  throw new Error(`[CRITICAL] ${translatedMsg}`);
}

export function getEl<T extends Element>(selector: string): T {
  const el = document.querySelector<T>(selector);
  if (!el) {
    failLoudly(`[CRITICAL] DOM Element missing: ${selector}`);
  }
  return el;
}
