import { Suspense } from "react";

import { MarketChainProvider } from "@/app/MarketChainProvider";
import { TokenPage } from "@/features/marketplace/TokenPage";
import { t } from "@/i18n";

/**
 * Route `/token/[contract]/[tokenId]` (frontend-architecture.md §2).
 *
 * `params` adalah data runtime, jadi aksesnya berada di dalam `<Suspense>`: shell halaman
 * ter-render seketika dan detailnya menyusul — pola "SSR shell + CSR data" yang sama dengan
 * tabel route (loading state = skeleton detail), bukan halaman yang memblokir prerender.
 *
 * Kedua segmen adalah `AccountId` dan `TokenId` dari path, jadi diteruskan apa adanya —
 * validasi bentuknya dilakukan kontrak saat view call, bukan ditebak di sini.
 */
export default function TokenRoute({
  params,
}: {
  params: Promise<{ contract: string; tokenId: string }>;
}) {
  return (
    <Suspense fallback={<TokenFallback />}>
      <TokenRouteContent params={params} />
    </Suspense>
  );
}

function TokenFallback() {
  return (
    <main className="mx-auto w-full max-w-3xl flex-1 px-6 py-10">
      <p role="status" className="text-sm text-zinc-500">
        {t("marketplace.detail.loading")}
      </p>
    </main>
  );
}

async function TokenRouteContent({
  params,
}: {
  params: Promise<{ contract: string; tokenId: string }>;
}) {
  const { contract, tokenId } = await params;

  return (
    <MarketChainProvider>
      <TokenPage nftContractId={contract} tokenId={tokenId} />
    </MarketChainProvider>
  );
}
