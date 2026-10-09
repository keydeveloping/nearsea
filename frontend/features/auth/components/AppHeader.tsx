"use client";

import { t } from "@/i18n";

import { useWallet } from "../hooks/useWallet";

import { NetworkBanner } from "./NetworkBanner";
import { WalletButton } from "./WalletButton";

/** Header app: identitas, indikator jaringan, kontrol wallet. */
export function AppHeader() {
  const wallet = useWallet();

  return (
    <header className="border-b border-zinc-200 dark:border-zinc-800">
      <NetworkBanner wallet={wallet} />
      <div className="flex items-center justify-between gap-4 px-6 py-3">
        <span className="text-lg font-semibold tracking-tight">{t("common.appName")}</span>
        <WalletButton wallet={wallet} />
      </div>
    </header>
  );
}
