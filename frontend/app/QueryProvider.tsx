"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";

/**
 * Provider server-state (TanStack Query) — dipasang di root layout.
 *
 * `QueryClient` dibuat sekali per instance browser lewat `useState` (bukan di level modul):
 * instance modul akan dibagi antar-request saat SSR dan membocorkan cache satu user ke user lain.
 */
export function QueryProvider({ children }: { children: ReactNode }) {
  const [client] = useState(() => new QueryClient());

  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}
