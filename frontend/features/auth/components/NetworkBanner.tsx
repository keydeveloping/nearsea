"use client";

import { format, t } from "@/i18n";

import type { WalletApi } from "../types/wallet.types";

/**
 * Indikator jaringan permanen + peringatan akun tidak ditemukan.
 *
 * Mismatch jaringan tidak bisa dibaca langsung dari wallet: near-connect tidak mengekspos
 * network wallet. Yang bisa dibuktikan on-chain adalah **akun tidak ada di jaringan yang
 * dikonfigurasi** — sebabnya bisa wallet di jaringan lain **atau** akun belum dibuat/di-fund.
 * Copy UI menyebut kedua kemungkinan, bukan mengklaim jaringan yang salah
 * (features/auth.md §Account / Network Switching).
 */
export function NetworkBanner({ wallet }: { wallet: WalletApi }) {
  const warning = wallet.status === "connected" && wallet.accountStatus === "not_found";

  return (
    <div
      role="status"
      className={
        warning
          ? "w-full bg-amber-100 px-4 py-2 text-center text-sm text-amber-900 dark:bg-amber-900/40 dark:text-amber-100"
          : "w-full bg-zinc-100 px-4 py-1.5 text-center text-xs text-zinc-600 dark:bg-zinc-900 dark:text-zinc-400"
      }
    >
      {warning ? (
        <>
          <strong>{format(t("auth.networkMismatchTitle"), { network: wallet.network })}</strong>{" "}
          {format(t("auth.networkMismatchBody"), { network: wallet.network })}
        </>
      ) : (
        format(t("common.networkBanner"), { network: wallet.network })
      )}
    </div>
  );
}
