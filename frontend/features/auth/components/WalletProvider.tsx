"use client";

import { NearProvider, useNearWallet } from "near-connect-hooks";
import {
  createContext,
  useEffect,
  useMemo,
  useState,
  useSyncExternalStore,
  type ReactNode,
} from "react";

import { buildConnectorConfig } from "@/lib/near/wallet-connector";
import type { WalletConnectErrorCode } from "@/lib/near/wallet-errors";

import { useAccountProbe } from "../hooks/useAccountProbe";
import { useWalletActions, type WalletAction } from "../hooks/useWalletActions";
import { buildWalletApi } from "../wallet-api";

import type { WalletApi, WalletStatus } from "../types/wallet.types";

/** State pra-hidrasi: belum ada wallet, belum ada yang bisa diklik. */
export const IDLE_WALLET_API: WalletApi = buildWalletApi({
  status: "loading",
  transactionsDisabled: true,
});

export const WalletContext = createContext<WalletApi>(IDLE_WALLET_API);

/** Dibuat sekali di level modul supaya identitasnya stabil antar-render. */
const connectorConfig = buildConnectorConfig();

// `useSyncExternalStore` = cara resmi membedakan server vs client tanpa setState di effect.
const subscribeToNothing = () => () => undefined;
const getClientSnapshot = () => true;
const getServerSnapshot = () => false;

/**
 * Menyediakan state wallet ke seluruh app.
 *
 * `NearProvider` sengaja **hanya di-mount di browser**: constructor near-connect memakai
 * `window`/IndexedDB dan mengambil manifest lewat fetch, jadi merendernya saat SSR salah
 * (dan bikin build butuh jaringan). `children` tetap berada di posisi pohon yang sama
 * sebelum & sesudah mount, sehingga konten halaman tidak ikut ter-remount.
 */
export function WalletProvider({ children }: { children: ReactNode }) {
  const [api, setApi] = useState<WalletApi>(IDLE_WALLET_API);
  const isClient = useSyncExternalStore(subscribeToNothing, getClientSnapshot, getServerSnapshot);

  return (
    <WalletContext.Provider value={api}>
      {children}
      {isClient ? (
        <NearProvider config={connectorConfig}>
          <WalletBridge onApi={setApi} />
        </NearProvider>
      ) : null}
    </WalletContext.Provider>
  );
}

/** Komponen tanpa tampilan: menerjemahkan API near-connect menjadi `WalletApi`. */
function WalletBridge({ onApi }: { onApi: (api: WalletApi) => void }) {
  const { signedAccountId, loading, network, getBalance, signIn, signOut } = useNearWallet();
  const probe = useAccountProbe(signedAccountId, getBalance);
  const actions = useWalletActions(signIn, signOut);

  const api = useMemo<WalletApi>(
    () => ({
      status: deriveStatus(actions.pending, signedAccountId, actions.errorCode, loading),
      accountId: signedAccountId || null,
      network,
      balanceYocto: probe.balanceYocto,
      accountStatus: probe.status,
      // Akun yang belum terbukti ada di jaringan ini = transaksi di-disable (hard block).
      transactionsDisabled: signedAccountId !== "" && probe.status !== "ready",
      errorCode: actions.errorCode,
      connect: actions.connect,
      disconnect: actions.disconnect,
      retry: actions.retry,
    }),
    [actions, signedAccountId, loading, network, probe],
  );

  useEffect(() => onApi(api), [api, onApi]);

  return null;
}

function deriveStatus(
  pending: WalletAction | null,
  signedAccountId: string,
  errorCode: WalletConnectErrorCode | null,
  loading: boolean,
): WalletStatus {
  if (pending) return pending === "connect" ? "connecting" : "disconnecting";
  if (signedAccountId) return "connected";
  if (errorCode) return "error";
  return loading ? "loading" : "disconnected";
}
