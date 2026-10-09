/**
 * Konfigurasi jaringan NEAR — dibaca dari env, tidak pernah di-hardcode di komponen
 * (frontend-architecture.md §7, ticket 09 AC "network config follows env").
 */

export type NearNetwork = "testnet" | "mainnet";

/** MVP menargetkan testnet (ADR-016); env kosong = testnet. */
const DEFAULT_NETWORK: NearNetwork = "testnet";

/** Urutan = urutan failover (docs/deployment/environments.md). */
const DEFAULT_RPC_URLS: Record<NearNetwork, readonly string[]> = {
  testnet: ["https://test.rpc.fastnear.com", "https://rpc.testnet.near.org"],
  mainnet: ["https://free.rpc.fastnear.com", "https://rpc.mainnet.near.org"],
};

/**
 * Nilai tak dikenal **dilempar**, bukan diam-diam jatuh ke testnet: salah tulis
 * jaringan = transaksi di jaringan yang salah, jadi harus gagal keras saat start.
 */
export function resolveNetwork(raw: string | undefined): NearNetwork {
  const value = raw?.trim();
  if (!value) return DEFAULT_NETWORK;
  if (value === "testnet" || value === "mainnet") return value;
  throw new Error(`Invalid NEXT_PUBLIC_NEAR_NETWORK: "${value}" (expected "testnet" or "mainnet")`);
}

function splitList(raw: string | undefined): string[] {
  return (raw ?? "")
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

/** Env RPC bila diset, selain itu daftar default dokumen. */
export function rpcProviderUrls(
  network: NearNetwork,
  primary: string | undefined,
  fallbacks: string | undefined,
): string[] {
  const configured = splitList(primary);
  if (configured.length === 0) return [...DEFAULT_RPC_URLS[network]];
  return [...configured, ...splitList(fallbacks)];
}

export const NEAR_NETWORK: NearNetwork = resolveNetwork(process.env.NEXT_PUBLIC_NEAR_NETWORK);

export function configuredRpcUrls(): string[] {
  return rpcProviderUrls(
    NEAR_NETWORK,
    process.env.NEXT_PUBLIC_NEAR_RPC_URL,
    process.env.NEXT_PUBLIC_NEAR_RPC_FALLBACKS,
  );
}
