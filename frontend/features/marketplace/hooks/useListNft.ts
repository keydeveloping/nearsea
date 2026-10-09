"use client";

import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useState } from "react";

import { GAS_APPROVE, GAS_LIST, ONE_YOCTO } from "@/lib/constants/near-gas";
import {
  KnownTxError,
  classifyTxError,
  configFailure,
  type TxFailure,
} from "@/lib/errors/market-errors";
import { MARKET_CONTRACT_ID } from "@/lib/near/contracts";

import { fetchToken } from "../api/market-api";
import { invalidateMarketQueries } from "../api/query-keys";

import { useMarketChain } from "./useMarketChain";
import { useMountedRef } from "./useMountedRef";

import type { YoctoNear } from "@/lib/format/money";

/**
 * Langkah signing listing. `approving`/`listing` = **langkah 1/2 dan 2/2** yang ditampilkan
 * modal; harga diisi di form sebelum keduanya (docs/02-product-requirements.md §Koreksi label
 * langkah).
 */
export type ListStep = "idle" | "approving" | "listing" | "success" | "error";

export interface ListNftInput {
  nftContractId: string;
  tokenId: string;
  priceYocto: YoctoNear;
  allowedBuyer: string | null;
  /** Deposit storage yang dihitung pemanggil dari `useStorageRequirement`. */
  storageDepositYocto: YoctoNear;
}

export interface ListNftState {
  step: ListStep;
  failure: TxFailure | null;
  submit: (input: ListNftInput) => Promise<void>;
  reset: () => void;
}

/**
 * Alur listing **dua transaksi** (docs/features/marketplace.md §Listing):
 *
 * 1. `nft_approve` di kontrak koleksi (1 yocto, `msg: null` → tanpa callback),
 * 2. baca `approval_id` konkret dari `nft_token`,
 * 3. `list_nft_for_sale` di market dengan deposit storage.
 *
 * Langkah 2 bukan formalitas: `approval_id` yang dikirim `null` membuat listing **tidak bisa
 * dibeli** — `nft_transfer_payout` memetakan `None` ke otorisasi owner, sedangkan pemanggilnya
 * market, bukan owner. Jadi id konkret wajib dibaca dari state koleksi setelah approve.
 */
export function useListNft(): ListNftState {
  const chain = useMarketChain();
  const queryClient = useQueryClient();
  const mounted = useMountedRef();
  const [step, setStep] = useState<ListStep>("idle");
  const [failure, setFailure] = useState<TxFailure | null>(null);

  const reset = useCallback(() => {
    setStep("idle");
    setFailure(null);
  }, []);

  const submit = useCallback(
    async (input: ListNftInput) => {
      const marketContractId = MARKET_CONTRACT_ID;
      setFailure(null);

      if (marketContractId === null) {
        setStep("error");
        setFailure(configFailure());
        return;
      }

      try {
        setStep("approving");
        await chain.callFunction({
          contractId: input.nftContractId,
          method: "nft_approve",
          args: { token_id: input.tokenId, account_id: marketContractId, msg: null },
          gas: GAS_APPROVE,
          deposit: ONE_YOCTO,
        });

        const token = await fetchToken(chain, input.nftContractId, input.tokenId);
        const approvalId = token?.approvedAccountIds[marketContractId];
        if (approvalId === undefined) {
          // Approval tidak terlihat di state koleksi: listing akan mustahil dibeli, jadi
          // berhenti di sini alih-alih mengirim transaksi kedua yang pasti rusak.
          throw new KnownTxError({ kind: "code", code: "CHAIN_REVERT" });
        }

        setStep("listing");
        await chain.callFunction({
          contractId: marketContractId,
          method: "list_nft_for_sale",
          args: {
            nft_contract_id: input.nftContractId,
            token_id: input.tokenId,
            approval_id: approvalId,
            price: input.priceYocto,
            allowed_buyer: input.allowedBuyer,
          },
          gas: GAS_LIST,
          deposit: input.storageDepositYocto,
        });

        if (!mounted.current) return;
        setStep("success");
        await invalidateMarketQueries(queryClient);
      } catch (error) {
        if (!mounted.current) return;
        setStep("error");
        setFailure(classifyTxError(error));
      }
    },
    [chain, queryClient, mounted],
  );

  return { step, failure, submit, reset };
}
