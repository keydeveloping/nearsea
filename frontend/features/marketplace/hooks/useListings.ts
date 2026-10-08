"use client";

import { useQuery } from "@tanstack/react-query";

import { MARKET_CONTRACT_ID } from "@/lib/near/contracts";

import { loadVisibleListings } from "../api/discovery";
import { marketKeys } from "../api/query-keys";
import { compareNewestFirst } from "../api/sale-validity";
import type { Sale } from "../types/marketplace.types";
import { useMarketChain } from "./useMarketChain";

/**
 * Umpan listing untuk grid discovery: hanya listing publik yang sudah **dibuktikan** masih
 * sah (bukan stale), terurut deterministik `newest` (docs/features/marketplace.md §Discovery).
 */
export function useListings() {
  const chain = useMarketChain();

  return useQuery<Sale[]>({
    queryKey: marketKeys.listings(),
    enabled: MARKET_CONTRACT_ID !== null,
    queryFn: async () => {
      // `enabled` sudah menjamin ini; dijaga eksplisit supaya tipe menyempit ke `string`.
      if (MARKET_CONTRACT_ID === null) return [];
      const sales = await loadVisibleListings(chain, MARKET_CONTRACT_ID);
      return [...sales].sort(compareNewestFirst);
    },
  });
}
