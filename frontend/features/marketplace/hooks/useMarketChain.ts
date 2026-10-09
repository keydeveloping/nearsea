"use client";

import { createContext, useContext } from "react";

import type { MarketChain } from "../types/marketplace.types";

/**
 * Kanal chain untuk fitur marketplace.
 *
 * Fitur **tidak boleh** saling impor (frontend-architecture.md §1), jadi marketplace tidak
 * boleh memanggil `useWallet()` milik `features/auth` langsung. Nilainya disuntikkan dari
 * `app/` (satu-satunya lapisan yang boleh impor semua fitur) lewat `AppMarketChain`;
 * marketplace hanya bergantung pada interface sempit `MarketChain`.
 */
export const MarketChainContext = createContext<MarketChain | null>(null);

export function useMarketChain(): MarketChain {
  const chain = useContext(MarketChainContext);
  if (chain === null) {
    throw new Error("useMarketChain must be used within AppMarketChain");
  }
  return chain;
}
