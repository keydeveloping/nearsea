"use client";

import { useQuery } from "@tanstack/react-query";

import type { YoctoNear } from "@/lib/format/money";
import { storageShortfall } from "@/lib/format/storage";
import { MARKET_CONTRACT_ID } from "@/lib/near/contracts";

import { fetchStorageAvailable, fetchStorageBounds } from "../api/market-api";
import { marketKeys } from "../api/query-keys";
import { useMarketChain } from "./useMarketChain";

/**
 * Deposit storage yang harus dilampirkan ke `list_nft_for_sale` (NEP-145): selisih antara
 * batas minimum entry `Sale` dan saldo yang sudah dimiliki seller.
 *
 * Ditampilkan di modal **sebelum** signing supaya user tidak dikejutkan popup deposit
 * (temuan UX B4, docs/02-product-requirements.md §Koreksi label langkah).
 */
export function useStorageRequirement(accountId: string | null) {
  const chain = useMarketChain();
  const marketContractId = MARKET_CONTRACT_ID;

  return useQuery<YoctoNear | null>({
    queryKey: marketKeys.storageRequirement(accountId ?? ""),
    enabled: marketContractId !== null && accountId !== null,
    queryFn: async () => {
      if (marketContractId === null || accountId === null) return null;
      const [bounds, available] = await Promise.all([
        fetchStorageBounds(chain, marketContractId),
        fetchStorageAvailable(chain, marketContractId, accountId),
      ]);
      return storageShortfall(bounds.minYocto, available);
    },
  });
}
