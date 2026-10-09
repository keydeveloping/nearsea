// TC-040 · AC-WALLET-1, AC-WALLET-4
import { render, type RenderResult } from "@testing-library/react";

import { AppHeader } from "./components/AppHeader";
import { WalletContext } from "./components/WalletProvider";
import { buildWalletApi } from "./wallet-api";

import type { WalletApi } from "./types/wallet.types";

/**
 * Merender **header app sungguhan** (banner jaringan + kontrol wallet) dengan state wallet
 * yang ditentukan test. Komponen ini yang membaca konteks wallet, jadi test menguji jalur
 * yang sama dengan app — bukan prop yang disuntik langsung.
 */
export function renderHeader(overrides: Partial<WalletApi> = {}): RenderResult {
  return render(
    <WalletContext.Provider value={buildWalletApi(overrides)}>
      <AppHeader />
    </WalletContext.Provider>,
  );
}
