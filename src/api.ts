import { invoke } from "@tauri-apps/api/core";
import { failLoudly } from "./dom";
import type { Account } from "./types";
import { showLoading, hideLoading } from "./ui";
import { renderAccounts } from "./accounts-view";
import { state } from "./state";

export async function fetchAccountsApi() {
  showLoading();
  try {
    const res = await invoke<Account[]>("get_accounts");
    if (!res) {
      failLoudly("dataEmpty");
    }
    state.accounts = res;
    renderAccounts();
  } catch (err) {
    failLoudly("fetchFailed", err);
  } finally {
    hideLoading();
  }
}

export async function launchProfileApi(id: string, name: string) {
  showLoading();
  try {
    await invoke("open_account_window", { id, name });
  } catch (err) {
    failLoudly("launchFailed", err);
  } finally {
    hideLoading();
  }
}

export async function launchIdeApi(id: string) {
  showLoading();
  try {
    await invoke("apply_account_to_default_ide", { id });
  } catch (err) {
    failLoudly("launchIdeFailed", err);
  } finally {
    hideLoading();
  }
}

export async function importCurrentCredentialsApi(): Promise<{token?: string, org_id?: string} | undefined> {
  showLoading();
  try {
    const res = await invoke<{ token?: string; org_id?: string }>("import_current_credentials");
    return res;
  } catch (err) {
    failLoudly("importFailed", err);
  } finally {
    hideLoading();
  }
}

export async function deleteProfileApi(id: string) {
  showLoading();
  try {
    await invoke("delete_account", { id });
    await fetchAccountsApi();
  } catch (err) {
    failLoudly("deleteFailed", err);
  } finally {
    hideLoading();
  }
}

export async function saveProfileApi(
  mode: "create" | "rename",
  id: string | null,
  name: string,
  email: string | null,
  password: string | null,
  token: string | null,
  orgId: string | null,
  plan: string
) {
  showLoading();
  try {
    if (mode === "create") {
      await invoke("add_account", {
        name,
        email,
        password,
        token,
        // Rust: org_id -> JS: orgId (camelCase)
        orgId: orgId,
        // Rust: plan_tier -> JS: planTier (camelCase)
        planTier: plan,
      });
    } else if (mode === "rename" && id) {
      await invoke("rename_account", {
        id,
        // Rust: new_name -> JS: newName (camelCase)
        newName: name,
        email,
        password,
        token,
        // Rust: org_id -> JS: orgId (camelCase)
        orgId: orgId,
        // Rust: plan_tier -> JS: planTier (camelCase)
        planTier: plan,
      });
    }
    await fetchAccountsApi();
  } catch (err) {
    failLoudly("saveFailed", err);
  } finally {
    hideLoading();
  }
}

export async function triggerQuotaRefreshApi() {
  try {
    await invoke("refresh_all_quotas");
  } catch (err) {
    console.error("[CRITICAL] Failed to refresh quotas headlessly:", err);
  }
}
