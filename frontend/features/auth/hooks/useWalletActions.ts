"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { classifyConnectError, type WalletConnectErrorCode } from "@/lib/near/wallet-errors";

export type WalletAction = "connect" | "disconnect";

interface WalletActions {
  /** Aksi yang sedang berjalan — `null` bila tidak ada. */
  pending: WalletAction | null;
  errorCode: WalletConnectErrorCode | null;
  /** Mengulang aksi terakhir yang gagal — bukan selalu connect. */
  retry: () => Promise<void>;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
}

/**
 * Membungkus sign-in/sign-out dengan status pending + pemetaan error.
 *
 * `pending` menyimpan **aksi mana** yang berjalan, bukan sekadar boolean: user harus bisa
 * membedakan "sedang menyambung" dari "sedang memutus", dan `Retry` harus mengulang aksi
 * yang gagal.
 *
 * Semua yang dikembalikan identitasnya stabil antar-render — pemanggil memakainya sebagai
 * dependensi `useMemo`/`useEffect`, dan objek baru tiap render akan membuat efek berjalan
 * tanpa henti.
 */
export function useWalletActions(
  signIn: () => Promise<void>,
  signOut: () => Promise<void>,
): WalletActions {
  const [pending, setPending] = useState<WalletAction | null>(null);
  const [errorCode, setErrorCode] = useState<WalletConnectErrorCode | null>(null);
  const failedAction = useRef<WalletAction>("connect");

  const signInRef = useRef(signIn);
  const signOutRef = useRef(signOut);
  useEffect(() => {
    signInRef.current = signIn;
    signOutRef.current = signOut;
  });

  const run = useCallback(async (action: WalletAction) => {
    setErrorCode(null);
    setPending(action);
    try {
      await (action === "connect" ? signInRef.current() : signOutRef.current());
    } catch (error) {
      failedAction.current = action;
      setErrorCode(classifyConnectError(error));
    } finally {
      setPending(null);
    }
  }, []);

  const connect = useCallback(() => run("connect"), [run]);
  const disconnect = useCallback(() => run("disconnect"), [run]);
  const retry = useCallback(() => run(failedAction.current), [run]);

  return useMemo(
    () => ({ pending, errorCode, retry, connect, disconnect }),
    [pending, errorCode, retry, connect, disconnect],
  );
}
