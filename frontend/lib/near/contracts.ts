/**
 * Alamat kontrak marketplace dari env (frontend-architecture.md §7, environments.md).
 *
 * Kontrak **koleksi NFT tidak** dikonfigurasi di sini: ia bagian dari route
 * (`/token/[contract]/[tokenId]`) dan sudah tercatat di setiap baris listing
 * (`get_sales` mengembalikan `nft_contract_id`), jadi tidak ada alamat global kedua
 * yang bisa menyimpang dari data on-chain.
 */

/** Hanya Next.js yang mengekspos prefix `NEXT_PUBLIC_` ke bundle browser. */
export function resolveMarketContractId(raw: string | undefined): string | null {
  const value = raw?.trim();
  return value ? value : null;
}

/**
 * `null` (bukan throw) bila belum diisi: build & halaman baca harus tetap jalan tanpa
 * deploy. UI menampilkan keadaan "belum dikonfigurasi" alih-alih gagal saat start —
 * berbeda dari `NEXT_PUBLIC_NEAR_NETWORK` yang salah tulis = transaksi di jaringan salah.
 */
export const MARKET_CONTRACT_ID: string | null = resolveMarketContractId(
  process.env.NEXT_PUBLIC_MARKET_CONTRACT_ID,
);
