"use client";

import { useWallet } from "@/features/auth/hooks/useWallet";
import { MarketChainContext } from "@/features/marketplace/hooks/useMarketChain";

import type { ReactNode } from "react";

/**
 * Menyambungkan wallet ke fitur marketplace. Tinggal di `app/` karena hanya lapisan ini
 * yang boleh mengimpor beberapa fitur sekaligus (frontend-architecture.md §1).
 */
export function MarketChainProvider({ children }: { children: ReactNode }) {
  const wallet = useWallet();

  return <MarketChainContext.Provider value={wallet}>{children}</MarketChainContext.Provider>;
}
