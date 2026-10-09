"use client";

import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useState } from "react";

import { GAS_BUY } from "@/lib/constants/near-gas";
import { classifyTxError, configFailure, type TxFailure } from "@/lib/errors/market-errors";
import { MARKET_CONTRACT_ID } from "@/lib/near/contracts";

import { invalidateMarketQueries } from "../api/query-keys";
import { verifySaleForPurchase } from "../api/verify-sale";

import { useMarketChain } from "./useMarketChain";
import { useMountedRef } from "./useMountedRef";

import type { Sale } from "../types/marketplace.types";

export type BuyStatus = "idle" | "verifying" | "signing" | "success" | "error";

export interface BuyNftState {
  status: BuyStatus;
  failure: TxFailure | null;
  buy: (sale: Sale) => Promise<void>;
  reset: () => void;
}

/**
 * Pembelian dengan **re-verify wajib sebelum signing** (SEC-ORDER-003, P0): argumen yang
 * ditandatangani harus berasal dari view call segar, bukan dari cache yang bisa sudah basi.
 *
 * Deposit = harga hasil verifikasi (kontrak mengembalikan kelebihan, tapi tidak ada alasan
 * melampirkan lebih).
 */
export function useBuyNft(): BuyNftState {
  const chain = useMarketChain();
  const queryClient = useQueryClient();
  const mounted = useMountedRef();
  const [status, setStatus] = useState<BuyStatus>("idle");
  const [failure, setFailure] = useState<TxFailure | null>(null);

  const reset = useCallback(() => {
    setStatus("idle");
    setFailure(null);
  }, []);

  const buy = useCallback(
    async (sale: Sale) => {
      const marketContractId = MARKET_CONTRACT_ID;
      setFailure(null);

      if (marketContractId === null) {
        setStatus("error");
        setFailure(configFailure());
        return;
      }

      try {
        setStatus("verifying");
        const fresh = await verifySaleForPurchase(chain, marketContractId, sale);

        setStatus("signing");
        await chain.callFunction({
          contractId: marketContractId,
          method: "buy",
          args: { nft_contract_id: sale.nftContractId, token_id: sale.tokenId },
          gas: GAS_BUY,
          deposit: fresh.priceYocto,
        });

        if (!mounted.current) return;
        setStatus("success");
        await invalidateMarketQueries(queryClient);
      } catch (error) {
        if (!mounted.current) return;
        setStatus("error");
        setFailure(classifyTxError(error));
      }
    },
    [chain, queryClient, mounted],
  );

  return { status, failure, buy, reset };
}
