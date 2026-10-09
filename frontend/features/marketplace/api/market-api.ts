/**
 * Modul view-call marketplace — tipis, tanpa JSX (code-standards.md §7).
 *
 * Semua nilai Ⓝ dipetakan dari string yoctoNEAR kontrak ke `YoctoNear` bertipe; nilai yang
 * tidak berbentuk angka yocto **ditolak keras** (`asYoctoNear` melempar) alih-alih diam-diam
 * di-`Number()` — data kontrak yang menyimpang harus terlihat, bukan jadi harga palsu.
 */

import { asYoctoNear, type YoctoNear } from "@/lib/format/money";

import type {
  MarketChain,
  NftToken,
  RoyaltyConfig,
  Sale,
  StorageBounds,
} from "../types/marketplace.types";

/** Batas `get_sales` di kontrak (u8 → clamp 100 di sisi kontrak; kita minta maksimum). */
export const SALES_PAGE_LIMIT = 100;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function fail(what: string, value: unknown): never {
  throw new Error(`Unexpected ${what} from contract: ${JSON.stringify(value)}`);
}

function asString(value: unknown, what: string): string {
  if (typeof value !== "string") fail(what, value);
  return value;
}

function asOptionalString(value: unknown): string | null {
  return typeof value === "string" ? value : null;
}

function asYocto(value: unknown, what: string): YoctoNear {
  if (typeof value !== "string") fail(what, value);
  try {
    return asYoctoNear(value);
  } catch {
    return fail(what, value);
  }
}

function asU64Number(value: unknown, what: string): number {
  if (typeof value !== "number" || !Number.isInteger(value) || value < 0) fail(what, value);
  return value;
}

function asOptionalU64(value: unknown, what: string): number | null {
  return value === null || value === undefined ? null : asU64Number(value, what);
}

function asApprovalMap(value: unknown): Record<string, number> {
  if (!isRecord(value)) fail("approved_account_ids", value);

  const map: Record<string, number> = {};
  for (const [accountId, approvalId] of Object.entries(value)) {
    map[accountId] = asU64Number(approvalId, "approval_id");
  }
  return map;
}

export function parseSale(raw: unknown): Sale {
  if (!isRecord(raw)) fail("sale", raw);

  return {
    nftContractId: asString(raw.nft_contract_id, "sale.nft_contract_id"),
    tokenId: asString(raw.token_id, "sale.token_id"),
    ownerId: asString(raw.owner_id, "sale.owner_id"),
    approvalId: asOptionalU64(raw.approval_id, "sale.approval_id"),
    priceYocto: asYocto(raw.price_yocto, "sale.price_yocto"),
    allowedBuyer: asOptionalString(raw.allowed_buyer),
    listedAt: asU64Number(raw.listed_at, "sale.listed_at"),
  };
}

export function parseSaleList(raw: unknown): Sale[] {
  if (!Array.isArray(raw)) fail("sales list", raw);
  return raw.map(parseSale);
}

export function parseNftToken(raw: unknown): NftToken | null {
  if (raw === null || raw === undefined) return null;
  if (!isRecord(raw)) fail("token", raw);

  return {
    tokenId: asString(raw.token_id, "token.token_id"),
    ownerId: asString(raw.owner_id, "token.owner_id"),
    approvedAccountIds: asApprovalMap(raw.approved_account_ids ?? {}),
    title: asOptionalString(raw.title),
    description: asOptionalString(raw.description),
    media: asOptionalString(raw.media),
  };
}

export function parseRoyaltyConfig(raw: unknown): RoyaltyConfig | null {
  if (raw === null || raw === undefined) return null;
  if (!isRecord(raw)) fail("royalty config", raw);

  return {
    receiverId: asString(raw.receiver, "royalty_config.receiver"),
    bps: asU64Number(raw.bps, "royalty_config.bps"),
  };
}

export function parseStorageBounds(raw: unknown): StorageBounds {
  if (!isRecord(raw)) fail("storage bounds", raw);

  return {
    minYocto: asYocto(raw.min, "storage_balance_bounds.min"),
    maxYocto: raw.max === null || raw.max === undefined ? null : asYocto(raw.max, "storage max"),
  };
}

/** Saldo storage **tersedia** milik akun di market (NEP-145); `0` bila belum terdaftar. */
export function parseStorageAvailable(raw: unknown): YoctoNear {
  if (raw === null || raw === undefined) return asYoctoNear("0");
  if (!isRecord(raw)) fail("storage balance", raw);
  return asYocto(raw.available, "storage_balance.available");
}

/**
 * `nft_token` koleksi. `null` = token tidak ada (kontrak menjawab null) — **bukan** kegagalan
 * RPC. Pembedaan ini penting: "tidak ada" berarti listing stale, sedangkan RPC gagal berarti
 * kita tidak tahu apa-apa dan tidak boleh menyembunyikan listing yang mungkin masih valid.
 */
export async function fetchToken(
  chain: MarketChain,
  nftContractId: string,
  tokenId: string,
): Promise<NftToken | null> {
  const raw = await chain.viewFunction({
    contractId: nftContractId,
    method: "nft_token",
    args: { token_id: tokenId },
  });
  return parseNftToken(raw);
}

export async function fetchSale(
  chain: MarketChain,
  marketContractId: string,
  nftContractId: string,
  tokenId: string,
): Promise<Sale | null> {
  const raw = await chain.viewFunction({
    contractId: marketContractId,
    method: "get_sale",
    args: { nft_contract_id: nftContractId, token_id: tokenId },
  });
  return raw === null || raw === undefined ? null : parseSale(raw);
}

/**
 * Umpan listing. Urutan dari kontrak **tidak dijamin** (iterasi map), jadi pengurutan
 * dilakukan di pemanggil (docs/features/marketplace.md §Discovery).
 */
export async function fetchSales(
  chain: MarketChain,
  marketContractId: string,
  limit = SALES_PAGE_LIMIT,
): Promise<Sale[]> {
  const raw = await chain.viewFunction({
    contractId: marketContractId,
    method: "get_sales",
    args: { offset: 0, limit },
  });
  return parseSaleList(raw);
}

export async function fetchRoyaltyConfig(
  chain: MarketChain,
  nftContractId: string,
): Promise<RoyaltyConfig | null> {
  const raw = await chain.viewFunction({
    contractId: nftContractId,
    method: "royalty_config",
    args: {},
  });
  return parseRoyaltyConfig(raw);
}

export async function fetchStorageBounds(
  chain: MarketChain,
  marketContractId: string,
): Promise<StorageBounds> {
  const raw = await chain.viewFunction({
    contractId: marketContractId,
    method: "storage_balance_bounds",
    args: {},
  });
  return parseStorageBounds(raw);
}

/** Saldo storage seller di market; akun tanpa entri = `0` (belum pernah deposit). */
export async function fetchStorageAvailable(
  chain: MarketChain,
  marketContractId: string,
  accountId: string,
): Promise<YoctoNear> {
  const raw = await chain.viewFunction({
    contractId: marketContractId,
    method: "storage_balance_of",
    args: { account_id: accountId },
  });
  return parseStorageAvailable(raw);
}

/** Fee platform dalam basis poin — dibaca dari kontrak, tidak di-hardcode (2% = 200). */
export async function fetchFeeBps(chain: MarketChain, marketContractId: string): Promise<number> {
  const raw = await chain.viewFunction({
    contractId: marketContractId,
    method: "get_fee_bps",
    args: {},
  });
  return asU64Number(raw, "fee_bps");
}
