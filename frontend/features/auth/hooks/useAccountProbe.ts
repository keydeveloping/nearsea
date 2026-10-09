"use client";

import { useEffect, useRef, useState } from "react";

import type { YoctoNear } from "@/lib/format/money";
import {
  classifyAccountProbe,
  classifyAccountProbeError,
  type AccountProbeResult,
} from "@/lib/near/wallet-errors";

import type { AccountStatus } from "../types/wallet.types";

interface AccountProbe {
  status: AccountStatus;
  balanceYocto: YoctoNear | null;
}

const UNKNOWN: AccountProbe = { status: "unknown", balanceYocto: null };

function toProbe(result: AccountProbeResult): AccountProbe {
  if (result.kind === "ready") return { status: "ready", balanceYocto: result.balanceYocto };
  if (result.kind === "missing") return { status: "not_found", balanceYocto: null };
  return { status: "unreachable", balanceYocto: null };
}

/**
 * Membaca saldo akun lewat RPC jaringan yang dikonfigurasi. Efek sampingnya sekaligus
 * memverifikasi akun benar-benar ada di jaringan itu — sinyal yang dipakai untuk peringatan
 * jaringan (features/auth.md §Account / Network Switching).
 *
 * `getBalance` disimpan di ref, **bukan** di daftar dependensi: pemiliknya
 * (`near-connect-hooks`) membuat ulang fungsi itu tiap render, jadi menjadikannya dependensi
 * membuat efek berjalan tanpa henti. Probe hanya perlu diulang saat akunnya berganti.
 *
 * Hasil disimpan bersama `accountId`-nya: tanpa itu, akun yang baru berganti akan menampilkan
 * saldo akun sebelumnya selama probe baru masih berjalan.
 */
export function useAccountProbe(
  accountId: string,
  getBalance: (accountId: string) => Promise<bigint>,
): AccountProbe {
  const [probe, setProbe] = useState<{ accountId: string; value: AccountProbe } | null>(null);

  const getBalanceRef = useRef(getBalance);
  useEffect(() => {
    getBalanceRef.current = getBalance;
  });

  useEffect(() => {
    if (!accountId) return;

    let cancelled = false;
    const settle = (result: AccountProbeResult) => {
      if (!cancelled) setProbe({ accountId, value: toProbe(result) });
    };

    getBalanceRef
      .current(accountId)
      .then((balance) => settle(classifyAccountProbe(balance)))
      .catch((error: unknown) => settle(classifyAccountProbeError(error)));

    return () => {
      cancelled = true;
    };
  }, [accountId]);

  return probe !== null && probe.accountId === accountId ? probe.value : UNKNOWN;
}
