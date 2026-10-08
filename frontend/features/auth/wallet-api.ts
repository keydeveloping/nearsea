import { NEAR_NETWORK } from "@/lib/near/network";

import type { WalletApi } from "./types/wallet.types";

/**
 * Bentuk state wallet yang "tidak ada apa-apa": belum ter-connect, belum ada aksi.
 *
 * Dipakai sebagai default konteks **dan** sebagai dasar fixture test — satu tempat
 * mendefinisikan bentuk `WalletApi`, sehingga menambah field tidak perlu diubah di dua file.
 */
export function buildWalletApi(overrides: Partial<WalletApi> = {}): WalletApi {
  return {
    status: "disconnected",
    accountId: null,
    network: NEAR_NETWORK,
    balanceYocto: null,
    accountStatus: "unknown",
    transactionsDisabled: false,
    errorCode: null,
    connect: async () => undefined,
    disconnect: async () => undefined,
    retry: async () => undefined,
    ...overrides,
  };
}
