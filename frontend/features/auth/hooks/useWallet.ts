"use client";

import { useContext } from "react";

import { WalletContext } from "../components/WalletProvider";
import type { WalletApi } from "../types/wallet.types";

/** Satu-satunya cara komponen membaca state wallet. */
export function useWallet(): WalletApi {
  return useContext(WalletContext);
}
