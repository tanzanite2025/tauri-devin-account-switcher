import { getEl } from "./dom";
import { t } from "./i18n";
import { state } from "./state";
import { launchProfileApi, deleteProfileApi } from "./api";
import { handleLaunchIde } from "./ide-launch";

export function renderAccounts() {
  const emptyState = getEl<HTMLElement>("#empty-state");
  const accountsGrid = getEl<HTMLElement>("#accounts-grid");
  const accountCountLabel = getEl<HTMLElement>("#account-count-label");
  
  if (state.accounts.length === 0) {
    emptyState.classList.remove("hidden");
    accountsGrid.classList.add("hidden");
    accountCountLabel.textContent = t("profilesCountMany")(0);
    return;
  }

  emptyState.classList.add("hidden");
  accountsGrid.classList.remove("hidden");
  
  accountCountLabel.textContent = state.accounts.length === 1 
    ? t("profilesCountOne") 
    : t("profilesCountMany")(state.accounts.length);

  accountsGrid.innerHTML = "";

  state.accounts.forEach((acc) => {
    const card = document.createElement("div");
    card.className = "uds-card relative overflow-hidden flex flex-col justify-between gap-5";
    
    const depth = document.createElement("div");
    depth.className = "uds-depth-gradient";
    card.appendChild(depth);

    const cardContent = document.createElement("div");
    cardContent.className = "grid grid-cols-[minmax(0,2fr)_minmax(0,1.2fr)] gap-4 z-10 items-start";

    const headerRow = document.createElement("div");
    headerRow.className = "flex justify-between items-start gap-3 col-span-2";

    const nameTitle = document.createElement("h3");
    nameTitle.className = "uds-h2 text-zinc-100 truncate flex-1";
    nameTitle.textContent = acc.name;
    headerRow.appendChild(nameTitle);

    const badge = document.createElement("span");
    const tier = (acc.plan_tier || "AUTO").toUpperCase();
    
    if (tier === "AUTO") {
      badge.className = "uds-badge uds-status-alert";
      badge.textContent = t("detecting");
    } else if (tier === "FREE") {
      badge.className = "uds-badge bg-zinc-800 text-zinc-400 border border-zinc-700/50";
      badge.textContent = "FREE";
    } else if (tier === "PRO") {
      badge.className = "uds-badge uds-status-healthy";
      badge.textContent = "PRO";
    } else if (tier === "MAX") {
      badge.className = "uds-badge bg-amber-500/15 text-amber-300 border border-amber-500/30";
      badge.textContent = "MAX";
    } else if (tier === "TEAMS") {
      badge.className = "uds-badge bg-emerald-600/20 text-emerald-300 border border-emerald-500/40";
      badge.textContent = "TEAMS";
    } else {
      badge.className = "uds-badge bg-purple-500/20 text-purple-300 border border-purple-500/40 animate-pulse";
      badge.textContent = "ENTERPRISE";
    }
    
    headerRow.appendChild(badge);
    cardContent.appendChild(headerRow);

    const leftCol = document.createElement("div");
    leftCol.className = "flex flex-col gap-2";

    const rightCol = document.createElement("div");
    rightCol.className = "flex flex-col gap-2 items-end pr-3";

    if (acc.email) {
      const emailMeta = document.createElement("div");
      emailMeta.className = "text-[10px] text-emerald-400/80 font-medium truncate mt-1";
      emailMeta.textContent = acc.email;
      leftCol.appendChild(emailMeta);
    }

    if (acc.email || acc.password) {
      const copyRow = document.createElement("div");
      copyRow.className = "flex flex-wrap justify-end gap-2 mt-1 max-w-full";
      
      if (acc.email) {
        const btnCopyEmail = document.createElement("button");
        btnCopyEmail.className = "uds-btn-ghost p-1 h-6 hover:bg-zinc-800 text-[8px] flex items-center gap-1 border-none bg-zinc-900/40 rounded-lg px-2";
        btnCopyEmail.innerHTML = `
          <svg xmlns="http://www.w3.org/2000/svg" width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
          ${t("copyEmail")}
        `;
        btnCopyEmail.addEventListener("click", () => navigator.clipboard.writeText(acc.email!));
        copyRow.appendChild(btnCopyEmail);
      }
      
      if (acc.password) {
        const btnCopyPassword = document.createElement("button");
        btnCopyPassword.className = "uds-btn-ghost p-1 h-6 hover:bg-zinc-800 text-[8px] flex items-center gap-1 border-none bg-zinc-900/40 rounded-lg px-2";
        btnCopyPassword.innerHTML = `
          <svg xmlns="http://www.w3.org/2000/svg" width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
          ${t("copyPassword")}
        `;
        btnCopyPassword.addEventListener("click", () => navigator.clipboard.writeText(acc.password!));
        copyRow.appendChild(btnCopyPassword);
      }
      rightCol.appendChild(copyRow);
    }

    if (acc.available_credits != null || acc.billing_error) {
      const quotaContainer = document.createElement("div");
      quotaContainer.className = "flex flex-col gap-1 mt-2";
      
      const quotaHeader = document.createElement("div");
      quotaHeader.className = "flex justify-between items-center";
      
      const quotaLabel = document.createElement("span");
      quotaLabel.className = "text-[10px] font-black uppercase tracking-widest text-zinc-500/50";
      quotaLabel.textContent = "QUOTA";
      
      const quotaValue = document.createElement("span");
      quotaValue.className = "uds-mono text-[9px] font-bold px-2 py-0.5 rounded-full";
      
      const progressBarBg = document.createElement("div");
      progressBarBg.className = "w-full h-1 rounded-full bg-zinc-800/30 overflow-hidden mt-1";
      
      const progressBar = document.createElement("div");
      progressBar.className = "h-full rounded-full transition-all duration-500";
      
      const isOutOfQuota = acc.billing_error === "out_of_quota";
      
      if (isOutOfQuota || acc.available_credits === 0) {
        quotaValue.textContent = "0 (DEPLETED)";
        quotaValue.className += " bg-rose-500/10 text-rose-600 animate-pulse";
        progressBar.className += " bg-rose-500/80";
        progressBar.style.width = "5%";
      } else {
        const credits = acc.available_credits != null ? acc.available_credits : "∞";
        quotaValue.textContent = `${credits} CREDITS`;
        if (typeof credits === 'number' && credits < 50) {
          quotaValue.className += " bg-amber-500/10 text-amber-600";
          progressBar.className += " bg-amber-500/80";
          progressBar.style.width = `${Math.max(10, Math.min(100, credits))}%`;
        } else {
          quotaValue.className += " bg-emerald-500/10 text-emerald-600";
          progressBar.className += " bg-emerald-500/80";
          progressBar.style.width = typeof credits === 'number' ? `${Math.min(100, credits / 5)}%` : "100%";
        }
      }
      
      quotaHeader.appendChild(quotaLabel);
      quotaHeader.appendChild(quotaValue);
      progressBarBg.appendChild(progressBar);
      
      quotaContainer.appendChild(quotaHeader);
      quotaContainer.appendChild(progressBarBg);
      
      leftCol.appendChild(quotaContainer);
    }

    const idMeta = document.createElement("div");
    idMeta.className = "uds-mono text-zinc-500 mt-2 select-all";
    idMeta.textContent = `ID: ${acc.id.substring(0, 8)}...`;
    leftCol.appendChild(idMeta);

    const date = new Date(acc.created_at * 1000);
    const dateMeta = document.createElement("div");
    dateMeta.className = "uds-mono text-zinc-600 text-[7px]";
    dateMeta.textContent = `CREATED: ${date.toLocaleString()}`;
    leftCol.appendChild(dateMeta);

    cardContent.appendChild(leftCol);
    cardContent.appendChild(rightCol);
    card.appendChild(cardContent);

    const actionsRow = document.createElement("div");
    actionsRow.className = "flex justify-between items-center gap-2 z-10 mt-2";

    const btnLaunch = document.createElement("button");
    btnLaunch.className = "uds-btn-primary flex-1 gap-1 text-[9px]";
    btnLaunch.innerHTML = `
      <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>
      ${t("launch")}
    `;
    btnLaunch.addEventListener("click", () => launchProfileApi(acc.id, acc.name));
    actionsRow.appendChild(btnLaunch);

    const btnLaunchIde = document.createElement("button");
    btnLaunchIde.className = "uds-btn-primary flex-1 gap-1 text-[9px] bg-zinc-800 hover:bg-zinc-700 text-zinc-100";
    btnLaunchIde.innerHTML = `
      <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"></polyline><polyline points="8 6 2 12 8 18"></polyline></svg>
      ${t("ide")}
    `;
    btnLaunchIde.addEventListener("click", () => handleLaunchIde(acc));
    actionsRow.appendChild(btnLaunchIde);

    const btnRename = document.createElement("button");
    btnRename.className = "uds-btn-ghost h-9 px-3 text-[9px]";
    btnRename.textContent = t("rename");
    btnRename.addEventListener("click", () => {
      // 这里仅负责调用弹窗打开逻辑，具体实现保留在 ui.ts 中
      const event = new CustomEvent("open-rename-dialog", {
        detail: {
          id: acc.id,
          name: acc.name,
          email: acc.email || "",
          password: acc.password || "",
          token: acc.token || "",
          org_id: acc.org_id || "",
          plan_tier: acc.plan_tier || "AUTO",
        },
      });
      window.dispatchEvent(event);
    });
    actionsRow.appendChild(btnRename);

    const btnDelete = document.createElement("button");
    btnDelete.className = "uds-btn-danger h-9 px-3 text-[9px]";
    btnDelete.textContent = t("delete");
    btnDelete.addEventListener("click", () => {
      if (confirm(t("deleteConfirm"))) {
        deleteProfileApi(acc.id);
      }
    });
    actionsRow.appendChild(btnDelete);

    card.appendChild(actionsRow);
    accountsGrid.appendChild(card);
  });
}
