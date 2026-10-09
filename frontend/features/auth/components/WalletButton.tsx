"use client";

import { format, t } from "@/i18n";
import { formatNear } from "@/lib/format/money";
import type { WalletConnectErrorCode } from "@/lib/near/wallet-errors";

import type { WalletApi } from "../types/wallet.types";

type ErrorMessageKey =
  "auth.error.rejected" | "auth.error.wallet_unavailable" | "auth.error.connect_failed";

const ERROR_KEYS: Record<WalletConnectErrorCode, ErrorMessageKey> = {
  rejected: "auth.error.rejected",
  wallet_unavailable: "auth.error.wallet_unavailable",
  connect_failed: "auth.error.connect_failed",
};

function Spinner() {
  return (
    <span
      aria-hidden="true"
      className="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"
    />
  );
}

/**
 * Pesan error + Retry. Dirender di kedua keadaan (belum connect **dan** sudah connect):
 * disconnect yang gagal meninggalkan user tetap ter-connect, jadi pesannya tidak boleh hilang
 * hanya karena status utamanya "connected".
 */
function ErrorLine({ wallet }: { wallet: WalletApi }) {
  if (!wallet.errorCode) return null;

  return (
    <p role="alert" className="flex items-center gap-2 text-xs text-red-600 dark:text-red-400">
      {t(ERROR_KEYS[wallet.errorCode])}
      <button type="button" onClick={() => void wallet.retry()} className="underline">
        {t("auth.retry")}
      </button>
    </p>
  );
}

function ConnectButton({ wallet }: { wallet: WalletApi }) {
  const busy = wallet.status === "loading" || wallet.status === "connecting";

  return (
    <div className="flex flex-col items-end gap-1">
      <button
        type="button"
        onClick={() => void wallet.connect()}
        disabled={busy}
        aria-busy={busy}
        className="inline-flex items-center gap-2 rounded-md bg-zinc-900 px-3 py-1.5 text-sm font-medium text-white disabled:opacity-60 dark:bg-zinc-100 dark:text-zinc-900"
      >
        {busy ? <Spinner /> : null}
        {wallet.status === "connecting" ? t("auth.connecting") : t("auth.connect")}
      </button>

      {wallet.status === "connecting" ? (
        <p className="text-xs text-zinc-500">{t("auth.approvingHint")}</p>
      ) : null}

      <ErrorLine wallet={wallet} />
    </div>
  );
}

function ConnectedAccount({ wallet }: { wallet: WalletApi }) {
  const disconnecting = wallet.status === "disconnecting";

  return (
    <div className="flex flex-col items-end gap-1">
      <div className="flex items-center gap-3">
        <div className="flex flex-col items-end leading-tight">
          <span className="font-mono text-sm">{wallet.accountId}</span>
          {wallet.balanceYocto !== null ? (
            <span className="text-xs text-zinc-500">
              {format(t("common.amountNear"), { amount: formatNear(wallet.balanceYocto) })}
            </span>
          ) : null}
        </div>
        <button
          type="button"
          onClick={() => void wallet.disconnect()}
          disabled={disconnecting}
          aria-busy={disconnecting}
          className="inline-flex items-center gap-2 rounded-md border border-zinc-300 px-3 py-1.5 text-sm font-medium disabled:opacity-60 dark:border-zinc-700"
        >
          {disconnecting ? <Spinner /> : null}
          {disconnecting ? t("auth.disconnecting") : t("auth.disconnect")}
        </button>
      </div>

      <ErrorLine wallet={wallet} />
    </div>
  );
}

/**
 * Kontrol wallet di header. Setiap keadaan punya tampilan berbeda supaya user tidak bingung
 * (features/auth.md §Modal State Detail).
 */
export function WalletButton({ wallet }: { wallet: WalletApi }) {
  if (wallet.status === "connected" || wallet.status === "disconnecting") {
    return <ConnectedAccount wallet={wallet} />;
  }
  return <ConnectButton wallet={wallet} />;
}
