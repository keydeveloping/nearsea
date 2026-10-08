// TC-040 · jalur emas M1 (browse → list → buy) — komponen & hook marketplace
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, type RenderResult } from "@testing-library/react";
import { vi } from "vitest";
import type { ReactElement, ReactNode } from "react";

import { MarketChainContext } from "./hooks/useMarketChain";
import type { MarketChain } from "./types/marketplace.types";

export interface RecordedView {
  contractId: string;
  method: string;
  args?: Record<string, unknown>;
}

export interface RecordedCall extends RecordedView {
  gas?: string;
  deposit?: string;
}

/** Chain palsu dengan pencatatan panggilan — menggantikan wallet tanpa near-connect. */
export interface FakeChain extends MarketChain {
  views: RecordedView[];
  calls: RecordedCall[];
  viewResults: Record<string, unknown>;
  callResults: Record<string, unknown>;
  /** Error untuk **semua** view call. */
  viewError: Error | null;
  /** Error per nama method — dipakai saat hanya satu view yang boleh gagal. */
  viewErrors: Record<string, Error>;
  callError: Error | null;
}

/**
 * Chain palsu yang meniru kontrak: hasil view/tx dipilih per nama method, dan setiap
 * panggilan dicatat sehingga test bisa memeriksa **urutan dua langkah** listing beserta
 * gas & deposit yang dilampirkan.
 */
export function buildFakeChain(overrides: Partial<MarketChain> = {}): FakeChain {
  // Closure membaca `chain` (bukan salinan) supaya hasil & error bisa diubah test
  // di tengah skenario tanpa membangun ulang objeknya.
  const chain: FakeChain = {
    accountId: null,
    transactionsDisabled: false,
    views: [],
    calls: [],
    viewResults: {},
    callResults: {},
    viewError: null,
    viewErrors: {},
    callError: null,
    viewFunction: vi.fn(async (params) => {
      chain.views.push(params);
      const specific = chain.viewErrors[params.method];
      if (specific) throw specific;
      if (chain.viewError) throw chain.viewError;
      return chain.viewResults[params.method] ?? null;
    }),
    callFunction: vi.fn(async (params) => {
      chain.calls.push(params);
      if (chain.callError) throw chain.callError;
      return chain.callResults[params.method] ?? null;
    }),
  };

  Object.assign(chain, overrides);
  return chain;
}

/**
 * Merender komponen marketplace dengan provider yang benar: QueryClient **baru per test**
 * (tanpa retry, supaya kegagalan tidak menunggu backoff) dan chain palsu.
 */
export function renderWithMarket(
  ui: ReactElement,
  chain: FakeChain = buildFakeChain(),
): RenderResult & { chain: FakeChain; queryClient: QueryClient } {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 }, mutations: { retry: false } },
  });

  const result = render(ui, {
    wrapper: ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>
        <MarketChainContext.Provider value={chain}>{children}</MarketChainContext.Provider>
      </QueryClientProvider>
    ),
  });

  return { ...result, chain, queryClient };
}
