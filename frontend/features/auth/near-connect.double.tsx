"use client";

/**
 * Double untuk `near-connect-hooks` di unit test.
 *
 * near-connect nyata tidak boleh dijalankan di sini: constructor-nya memakai `window`/IndexedDB
 * dan mengambil manifest lewat jaringan (testing-strategy.md: "FE — wallet (near-connect) |
 * mock signer/`signMessage`").
 *
 * Modul ini **tidak boleh** mengimpor apa pun dari aplikasi: ia dipakai sebagai target
 * `vi.mock("near-connect-hooks")`, dan mengimpor komponen aplikasi akan membuat siklus
 * (test-utils → WalletProvider → near-connect-hooks → test-utils) yang menggantung runner.
 *
 * Ia meniru **kontrak** `useNearWallet()` dan memberi tahu subscriber saat akun berubah —
 * persis seperti event `wallet:signIn`/`wallet:signOut` near-connect, sehingga logika turunan
 * aplikasi benar-benar dieksekusi (bukan sekadar mengulang nilai yang disuntik).
 */
import { useSyncExternalStore, type ReactNode } from "react";
import { vi } from "vitest";

const listeners = new Set<() => void>();
const notify = () => listeners.forEach((listener) => listener());

export const nearDouble = {
  state: { accountId: "", loading: false },
  /** Saldo yoctoNEAR yang dikembalikan probe. */
  balance: 0n,
  /** Error probe (mis. akun tidak ada / RPC timeout); `null` = probe sukses. */
  probeError: null as Error | null,
  /** Error yang dilempar `signIn`/`signOut` — untuk menguji jalur gagal + Retry. */
  signInError: null as Error | null,
  signOutError: null as Error | null,
  /** Akun yang "disetujui di wallet" saat `signIn` berhasil. */
  signInAccountId: "alice.testnet",
  signIn: vi.fn(async () => undefined),
  signOut: vi.fn(async () => undefined),
  subscribe(listener: () => void) {
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
    };
  },
  reset() {
    nearDouble.state.accountId = "";
    nearDouble.state.loading = false;
    nearDouble.balance = 0n;
    nearDouble.probeError = null;
    nearDouble.signInError = null;
    nearDouble.signOutError = null;
    nearDouble.signInAccountId = "alice.testnet";
    nearDouble.signIn.mockClear();
    nearDouble.signOut.mockClear();
  },
};

export function NearProvider({ children }: { children: ReactNode }) {
  return children;
}

export function useNearWallet() {
  const signedAccountId = useSyncExternalStore(
    nearDouble.subscribe,
    () => nearDouble.state.accountId,
    () => "",
  );
  const loading = useSyncExternalStore(
    nearDouble.subscribe,
    () => nearDouble.state.loading,
    () => false,
  );

  return {
    signedAccountId,
    loading,
    network: "testnet" as const,
    getBalance: async () => {
      if (nearDouble.probeError) throw nearDouble.probeError;
      return nearDouble.balance;
    },
    signIn: async () => {
      await nearDouble.signIn();
      if (nearDouble.signInError) throw nearDouble.signInError;
      // Wallet menyetujui → near-connect memancarkan `wallet:signIn`.
      nearDouble.state.accountId = nearDouble.signInAccountId;
      notify();
    },
    signOut: async () => {
      await nearDouble.signOut();
      if (nearDouble.signOutError) throw nearDouble.signOutError;
      nearDouble.state.accountId = "";
      notify();
    },
  };
}
