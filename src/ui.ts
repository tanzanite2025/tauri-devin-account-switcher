import { getEl } from "./dom";
import { state } from "./state";
import { t } from "./i18n";

// 显示进度条
export function showLoading() {
  const globalProgress = getEl<HTMLElement>("#global-progress");
  const globalProgressBar = getEl<HTMLElement>("#global-progress-bar");
  
  globalProgress.classList.remove("hidden");
  globalProgressBar.style.width = "40%";
  setTimeout(() => {
    if (!globalProgress.classList.contains("hidden")) {
      globalProgressBar.style.width = "75%";
    }
  }, 200);
}

// 隐藏进度条
export function hideLoading() {
  const globalProgress = getEl<HTMLElement>("#global-progress");
  const globalProgressBar = getEl<HTMLElement>("#global-progress-bar");
  
  globalProgressBar.style.width = "100%";
  setTimeout(() => {
    globalProgress.classList.add("hidden");
    globalProgressBar.style.width = "0%";
  }, 150);
}

// 打开弹窗
export function openDialog(
  mode: "create" | "rename", 
  titleText: string, 
  defaultName = "", 
  defaultEmail = "", 
  defaultPassword = "", 
  defaultToken = "", 
  defaultOrgId = "", 
  defaultPlan = "AUTO"
) {
  state.dialogMode = mode;
  
  const dialogTitle = getEl<HTMLElement>("#dialog-title");
  const inputAccountName = getEl<HTMLInputElement>("#input-account-name");
  const inputAccountEmail = getEl<HTMLInputElement>("#input-account-email");
  const inputAccountPassword = getEl<HTMLInputElement>("#input-account-password");
  const inputAccountToken = getEl<HTMLInputElement>("#input-account-token");
  const inputAccountOrg = getEl<HTMLInputElement>("#input-account-org");
  const selectAccountPlan = getEl<HTMLSelectElement>("#select-account-plan");
  const eyeIconShow = getEl<HTMLElement>("#eye-icon-show");
  const eyeIconHide = getEl<HTMLElement>("#eye-icon-hide");
  const dialogDesc = getEl<HTMLElement>("#dialog-desc");
  const dialogOverlay = getEl<HTMLElement>("#dialog-overlay");
  const accountDialog = getEl<HTMLElement>("#account-dialog");

  dialogTitle.textContent = titleText;
  inputAccountName.value = defaultName;
  inputAccountEmail.value = defaultEmail;
  inputAccountPassword.value = defaultPassword;
  inputAccountToken.value = defaultToken;
  inputAccountOrg.value = defaultOrgId;
  selectAccountPlan.value = defaultPlan;
  
  inputAccountPassword.type = "password";
  eyeIconShow.classList.add("hidden");
  eyeIconHide.classList.remove("hidden");

  dialogDesc.textContent = t("configSandbox");
  
  dialogOverlay.classList.remove("hidden");
  
  setTimeout(() => {
    accountDialog.classList.remove("scale-95", "opacity-0");
    inputAccountName.focus();
  }, 50);
}

// 关闭弹窗
export function closeDialog() {
  const accountDialog = getEl<HTMLElement>("#account-dialog");
  const dialogOverlay = getEl<HTMLElement>("#dialog-overlay");

  accountDialog.classList.add("scale-95", "opacity-0");
  setTimeout(() => {
    dialogOverlay.classList.add("hidden");
    state.activeAccountId = null;
  }, 200);
}

// 打开重命名弹窗
export function openRenameDialog(
  id: string, 
  currentName: string, 
  currentEmail: string, 
  currentPassword: string, 
  currentToken: string, 
  currentOrgId: string, 
  currentPlan: string
) {
  state.activeAccountId = id;
  openDialog("rename", t("renameProfile"), currentName, currentEmail, currentPassword, currentToken, currentOrgId, currentPlan);
}

// 监听从账号卡片发出的重命名事件
window.addEventListener("open-rename-dialog", (e: Event) => {
  const custom = e as CustomEvent<{
    id: string;
    name: string;
    email: string;
    password: string;
    token: string;
    org_id: string;
    plan_tier: string;
  }>;
  const detail = custom.detail;
  if (!detail) return;
  openRenameDialog(
    detail.id,
    detail.name,
    detail.email,
    detail.password,
    detail.token,
    detail.org_id,
    detail.plan_tier,
  );
});

// 控制密码框显隐
export function togglePasswordVisibility() {
  const inputAccountPassword = getEl<HTMLInputElement>("#input-account-password");
  const eyeIconShow = getEl<HTMLElement>("#eye-icon-show");
  const eyeIconHide = getEl<HTMLElement>("#eye-icon-hide");

  if (inputAccountPassword.type === "password") {
    inputAccountPassword.type = "text";
    eyeIconShow.classList.remove("hidden");
    eyeIconHide.classList.add("hidden");
  } else {
    inputAccountPassword.type = "password";
    eyeIconShow.classList.add("hidden");
    eyeIconHide.classList.remove("hidden");
  }
}

// 渲染账号列表
