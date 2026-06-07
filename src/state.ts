import type { Account } from "./types";

export const state = {
  accounts: [] as Account[],
  dialogMode: "create" as "create" | "rename",
  activeAccountId: null as string | null,
  currentLang: "zh" as "zh" | "en",
};
