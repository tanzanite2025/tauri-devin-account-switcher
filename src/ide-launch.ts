import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import { launchIdeApi } from "./api";

const silentLoginStatus: Record<string, "idle" | "in_progress"> = {};
const silentLoginTimers: Record<string, number> = {};
const ideLaunchInProgress: Record<string, boolean> = {};
let silentListenerInitialized = false;

function initSilentLoginListenerOnce() {
  if (silentListenerInitialized) return;
  silentListenerInitialized = true;

  listen<string>("silent-login-success", (event) => {
    const id = event.payload as string;
    if (!id) return;
    if (silentLoginStatus[id] === "in_progress") {
      silentLoginStatus[id] = "idle";
      const timerId = silentLoginTimers[id];
      if (timerId) {
        clearTimeout(timerId);
        delete silentLoginTimers[id];
      }
      alert(t("silentLoginSuccess"));
      if (ideLaunchInProgress[id]) {
        return;
      }
      ideLaunchInProgress[id] = true;
      launchIdeApi(id).finally(() => {
        ideLaunchInProgress[id] = false;
      });
    }
  });
}

export function handleLaunchIde(acc: any) {
  const id = acc.id as string;
  if (!id) {
    return;
  }

  if (ideLaunchInProgress[id]) {
    return;
  }

  // 如果已经有 token，直接拉起客户端
  if (acc.token && typeof acc.token === "string" && acc.token.trim().length > 0) {
    ideLaunchInProgress[id] = true;
    launchIdeApi(id).finally(() => {
      ideLaunchInProgress[id] = false;
    });
    return;
  }

  // 没有 token，尝试后台静默登录
  initSilentLoginListenerOnce();

  if (silentLoginStatus[id] === "in_progress") {
    return;
  }

  // 如果缺少邮箱或密码，则无法静默登录，直接拉起客户端让用户手动登录
  if (!acc.email || !acc.password) {
    alert(t("silentLoginTimeout"));
    ideLaunchInProgress[id] = true;
    launchIdeApi(id).finally(() => {
      ideLaunchInProgress[id] = false;
    });
    return;
  }

  silentLoginStatus[id] = "in_progress";
  alert(t("silentLoginProgress"));

  const oldTimer = silentLoginTimers[id];
  if (oldTimer) {
    clearTimeout(oldTimer);
  }

  invoke("start_silent_login", { id, name: acc.name })
    .catch((err) => {
      console.error("[CRITICAL] Failed to start silent login", err);
      silentLoginStatus[id] = "idle";
      const timerId = silentLoginTimers[id];
      if (timerId) {
        clearTimeout(timerId);
        delete silentLoginTimers[id];
      }
      alert(t("silentLoginTimeout"));
      ideLaunchInProgress[id] = true;
      launchIdeApi(id).finally(() => {
        ideLaunchInProgress[id] = false;
      });
    });

  silentLoginTimers[id] = window.setTimeout(() => {
    if (silentLoginStatus[id] === "in_progress") {
      silentLoginStatus[id] = "idle";
      delete silentLoginTimers[id];
      alert(t("silentLoginTimeout"));
      if (!ideLaunchInProgress[id]) {
        ideLaunchInProgress[id] = true;
        launchIdeApi(id).finally(() => {
          ideLaunchInProgress[id] = false;
        });
      }
    }
  }, 30000);
}
