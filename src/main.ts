import { listen } from "@tauri-apps/api/event";
import { initLanguage, toggleLanguage, t } from "./i18n";
import { getEl } from "./dom";
import { state } from "./state";
import { fetchAccountsApi, saveProfileApi, importCurrentCredentialsApi, triggerQuotaRefreshApi } from "./api";
import { openDialog, closeDialog, togglePasswordVisibility } from "./ui";

window.addEventListener("DOMContentLoaded", () => {
  initLanguage();

  const btnLangToggle = getEl<HTMLButtonElement>("#btn-lang-toggle");
  const btnAddAccount = getEl<HTMLButtonElement>("#btn-add-account");
  const btnCloseAlert = getEl<HTMLButtonElement>("#btn-close-alert");
  const btnDialogCancel = getEl<HTMLButtonElement>("#btn-dialog-cancel");
  const btnDialogSubmit = getEl<HTMLButtonElement>("#btn-dialog-submit");
  const btnPasswordToggle = getEl<HTMLButtonElement>("#btn-password-toggle");
  const btnImportCreds = getEl<HTMLButtonElement>("#btn-import-creds");
  const criticalAlertBar = getEl<HTMLElement>("#critical-alert-bar");

  const inputAccountName = getEl<HTMLInputElement>("#input-account-name");
  const inputAccountEmail = getEl<HTMLInputElement>("#input-account-email");
  const inputAccountPassword = getEl<HTMLInputElement>("#input-account-password");
  const inputAccountToken = getEl<HTMLInputElement>("#input-account-token");
  const inputAccountOrg = getEl<HTMLInputElement>("#input-account-org");
  const selectAccountPlan = getEl<HTMLSelectElement>("#select-account-plan");

  btnLangToggle.addEventListener("click", toggleLanguage);

  btnAddAccount.addEventListener("click", () => {
    openDialog("create", t("createProfile"));
  });

  btnDialogCancel.addEventListener("click", closeDialog);

  const handleDialogSubmit = async () => {
    const name = inputAccountName.value.trim();
    const email = inputAccountEmail.value.trim() || null;
    const password = inputAccountPassword.value.trim() || null;
    const token = inputAccountToken.value.trim() || null;
    const orgId = inputAccountOrg.value.trim() || null;
    const plan = selectAccountPlan.value;
    
    if (!name) {
      alert(t("nameEmptyAlert"));
      return;
    }

    await saveProfileApi(state.dialogMode, state.activeAccountId, name, email, password, token, orgId, plan);
    closeDialog();
  };

  btnDialogSubmit.addEventListener("click", handleDialogSubmit);
  
  btnPasswordToggle.addEventListener("click", togglePasswordVisibility);
  
  btnImportCreds.addEventListener("click", async () => {
    const res = await importCurrentCredentialsApi();
    if (res) {
      if (res.token) {
        inputAccountToken.value = res.token;
      }
      if (res.org_id) {
        inputAccountOrg.value = res.org_id;
      }
      alert(t("importSuccess"));
    }
  });
  
  const handleInputsKeydown = (e: KeyboardEvent) => {
    if (e.key === "Enter") {
      handleDialogSubmit();
    } else if (e.key === "Escape") {
      closeDialog();
    }
  };

  inputAccountName.addEventListener("keydown", handleInputsKeydown);
  inputAccountEmail.addEventListener("keydown", handleInputsKeydown);
  inputAccountPassword.addEventListener("keydown", handleInputsKeydown);
  inputAccountToken.addEventListener("keydown", handleInputsKeydown);
  inputAccountOrg.addEventListener("keydown", handleInputsKeydown);

  btnCloseAlert.addEventListener("click", () => {
    criticalAlertBar.classList.add("hidden");
  });

  let fetchTimer: any = null;
  const debouncedFetch = () => {
    if (fetchTimer) clearTimeout(fetchTimer);
    fetchTimer = setTimeout(() => {
      fetchAccountsApi();
    }, 200);
  };

  listen("account-plan-updated", debouncedFetch);
  listen("account-quota-updated", debouncedFetch);

  fetchAccountsApi().then(() => {
    triggerQuotaRefreshApi();
  });

  window.addEventListener("focus", () => {
    triggerQuotaRefreshApi();
  });
});
