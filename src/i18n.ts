import { getEl } from "./dom";
import { renderAccounts } from "./ui";
import { state } from "./state";

export const i18nDict: Record<"zh" | "en", Record<string, any>> = {
  zh: {
    title: "Devin 账号无感切号器",
    subtitle: "隔离沙箱与账户配置文件管理控制台",
    addAccount: "添加账号",
    registeredProfiles: "已配置的沙箱账户",
    profilesCountOne: "1 个配置文件",
    profilesCountMany: (count: number) => `${count} 个配置文件`,
    noAccounts: "暂无配置的账户",
    noAccountsDesc: "创建一个独立的沙箱配置文件，使用不同的 GitHub 会话登录 Devin AI。",
    createProfile: "创建账号卡片",
    renameProfile: "重命名账号卡片",
    configSandbox: "只需输入备注名，网页端与客户端登录后，邮箱、密码和凭证将全自动捕获！",
    profileName: "账号备注名 (必填)",
    profileEmail: "登录邮箱 (网页登录后自动捕获)",
    profilePassword: "登录密码 (网页登录后自动捕获)",
    profileToken: "Windsurf Token (客户端启动时自动捕获)",
    importCreds: "手动导入",
    profileOrgId: "组织 ID / Org ID (自动捕获)",
    profilePlan: "订阅套餐",
    planAuto: "AUTO (自动检测)",
    detecting: "检测中",
    placeholderName: "例如：工作账号、个人账户、客户 A...",
    placeholderEmail: "网页登录后将自动填入此处...",
    placeholderPassword: "网页登录后将自动填入此处...",
    placeholderToken: "客户端启动时将自动填入此处...",
    placeholderOrgId: "客户端启动时将自动填入此处...",
    cancel: "取消",
    confirm: "确认",
    ready: "正常",
    launch: "网页端",
    ide: "客户端",
    bindIde: "绑定客户端",
    rename: "重命名",
    delete: "删除",
    deleteConfirm: "确定要删除此账户及其隔离环境的所有登录态与 Cookie 吗？此操作无法撤销。",
    errorTitle: "检测到系统致命异常",
    dismiss: "忽略",
    nameEmptyAlert: "请输入有效的账户名称",
    fetchFailed: "拉取账户列表失败",
    launchFailed: "无法拉起 Devin 账号窗口",
    launchIdeFailed: "切换并拉起本地 IDE 客户端失败",
    deleteFailed: "删除账户失败",
    saveFailed: "保存账户配置失败",
    dataEmpty: "数据拉取返回了空结果",
    copyEmail: "复制邮箱",
    copyPassword: "复制密码",
    copySuccess: "已成功复制到剪贴板！",
    importSuccess: "已成功从本地 IDE 导入当前登录凭证！",
    importFailed: "导入当前凭证失败",
    silentLoginProgress: "正在尝试后台自动登录并绑定凭证，请稍候...",
    silentLoginSuccess: "后台自动登录并绑定凭证成功！正在拉起客户端...",
    silentLoginTimeout: "自动登录超时（可能需要人机验证）。已为您直接拉起 IDE 客户端，请在弹出的界面中完成登录。"
  },
  en: {
    title: "Devin Account Switcher",
    subtitle: "Isolated Sandbox & Profile Management Console",
    addAccount: "Add Account",
    registeredProfiles: "Registered Sandbox Profiles",
    profilesCountOne: "1 Profile",
    profilesCountMany: (count: number) => `${count} Profiles`,
    noAccounts: "No accounts configured yet",
    noAccountsDesc: "Create an isolated sandbox profile to log into Devin AI with separate GitHub sessions.",
    createProfile: "Create Profile",
    renameProfile: "Rename Profile",
    configSandbox: "Enter name only. Email, password, and tokens will be auto-captured upon login!",
    profileName: "Profile Name / Memo (Required)",
    profileEmail: "Email Address (Auto-captured in Web)",
    profilePassword: "Login Password (Auto-captured in Web)",
    profileToken: "Windsurf Token (Auto-captured on IDE launch)",
    importCreds: "Import",
    profileOrgId: "Organization ID (Auto-captured)",
    profilePlan: "Subscription Plan",
    planAuto: "AUTO (Auto Detect)",
    detecting: "Detecting...",
    placeholderName: "e.g. Work, Personal, Client A...",
    placeholderEmail: "Will be auto-captured after web login...",
    placeholderPassword: "Will be auto-captured after web login...",
    placeholderToken: "Will be auto-captured on client launch...",
    placeholderOrgId: "Will be auto-captured on client launch...",
    cancel: "Cancel",
    confirm: "Confirm",
    ready: "Ready",
    launch: "Web",
    ide: "IDE",
    bindIde: "Bind IDE",
    rename: "Rename",
    delete: "Delete",
    deleteConfirm: "Are you sure you want to delete this profile and all its isolated cookies/storage? This cannot be undone.",
    errorTitle: "CRITICAL ERROR DETECTED",
    dismiss: "Dismiss",
    nameEmptyAlert: "Please enter a valid account name",
    fetchFailed: "Failed to fetch account list",
    launchFailed: "Failed to launch Devin account window",
    launchIdeFailed: "Failed to switch and launch desktop IDE",
    deleteFailed: "Failed to delete account",
    saveFailed: "Failed to save account configuration",
    dataEmpty: "Data fetch returned empty result",
    copyEmail: "Copy Email",
    copyPassword: "Copy Password",
    copySuccess: "Copied to clipboard successfully!",
    importSuccess: "Successfully imported credentials from local IDE!",
    importFailed: "Failed to import current credentials",
    silentLoginProgress: "Attempting background login and key binding, please wait...",
    silentLoginSuccess: "Background login and key binding succeeded! Launching client...",
    silentLoginTimeout: "Background login timed out. Launched client for manual login."
  }
};

export function t(key: string): any {
  return i18nDict[state.currentLang][key] || key;
}

export function toggleLanguage() {
  state.currentLang = state.currentLang === "zh" ? "en" : "zh";
  localStorage.setItem("devin-switcher-lang", state.currentLang);
  applyLanguage();
}

export function applyLanguage() {
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const key = el.getAttribute("data-i18n")!;
    if (i18nDict[state.currentLang][key]) {
      el.textContent = i18nDict[state.currentLang][key];
    }
  });

  const inputAccountName = getEl<HTMLInputElement>("#input-account-name");
  inputAccountName.placeholder = t("placeholderName");
  
  const inputAccountEmail = getEl<HTMLInputElement>("#input-account-email");
  inputAccountEmail.placeholder = t("placeholderEmail");
  
  const inputAccountPassword = getEl<HTMLInputElement>("#input-account-password");
  inputAccountPassword.placeholder = t("placeholderPassword");
  
  const inputAccountToken = getEl<HTMLInputElement>("#input-account-token");
  inputAccountToken.placeholder = t("placeholderToken") || "";
  
  const inputAccountOrg = getEl<HTMLInputElement>("#input-account-org");
  inputAccountOrg.placeholder = t("placeholderOrgId") || "";

  const langBtnText = getEl<HTMLElement>("#lang-btn-text");
  langBtnText.textContent = state.currentLang === "zh" ? "EN" : "中文";

  const dialogTitle = getEl<HTMLElement>("#dialog-title");
  const dialogDesc = getEl<HTMLElement>("#dialog-desc");
  if (state.dialogMode === "create") {
    dialogTitle.textContent = t("createProfile");
  } else {
    dialogTitle.textContent = t("renameProfile");
  }
  dialogDesc.textContent = t("configSandbox");

  renderAccounts();
}

export function initLanguage() {
  const savedLang = localStorage.getItem("devin-switcher-lang") as "zh" | "en" | null;
  if (savedLang) {
    state.currentLang = savedLang;
  } else {
    const sysLang = navigator.language.toLowerCase();
    state.currentLang = sysLang.startsWith("zh") ? "zh" : "en";
  }
}
