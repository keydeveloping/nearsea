# Feature — Users & Profile

## Objective

Profil publik per akun NEAR: aset dimiliki, dibuat, aktivitas, offers aktif — plus **profil custom** (alias/bio/avatar) yang bisa diedit user.

## Preconditions

- Akun NEAR valid; data agregat dari on-chain views + NearBlocks (indexer pihak ketiga — MVP tanpa indexer sendiri).

## Flow — Profil publik

Buka `/profile/:account` → tab **Owned** (default) / **Created** / **Offers** / **Activity**.

## Flow — Profil Custom (signature auth)

1. User (wallet terhubung) buka Settings → edit alias/bio/avatar.
2. Frontend minta challenge → wallet **signMessage (NEP-413 utama; custom challenge fallback)** — lihat [api/authentication.md](../api/authentication.md).
3. `PATCH /api/accounts/me/profile` → server verifikasi signature + validasi (alias ≤32, bio ≤280, avatar URL allowlist).
4. Profil publik menampilkan data custom; alamat & aset tetap on-chain.

## UI

- Header: avatar (default: identicon dari alamat) + alamat + tombol share.
- Grid NFT Owned/Created (pagination cursor); tabel Activity (sale/transfer/offer, dari NearBlocks + market views).

## Data

- Owned: NEP-181 `nft_tokens_for_owner` (on-chain) + NearBlocks untuk riwayat.
- Created: token `creator_id` — **catatan**: `creator_id` bukan field standar NEP-177 (ekstensi custom kontrak kita); untuk koleksi pihak ketiga fallback = owner awal dari riwayat mint.

## API

- `GET /api/accounts/:id/activity` + `GET/PATCH profile` — lihat [api/endpoints.md](../api/endpoints.md).

## Pagination & Batas Page-Size

| Aspek | Nilai | Catatan |
|---|---|---|
| Model | Cursor opaque (`{items[], nextCursor}`) | Bukan offset — stabil saat data berubah (api-overview.md) |
| Default page size | 20 | — |
| Maks page size | **100** (clamp, bukan error) | Nilai di atas cap dipangkas server |
| Cursor per tab | Owned / Created / Offers / Activity punya cursor **independen** | Ganti tab tidak mereset tab lain |
| URL state | `tab`, `cursor` di query params (shareable) | frontend-architecture.md §2 |
| MVP (tanpa API) | `nft_tokens_for_owner` cursor near-sdk; NearBlocks page/per_page | Endpoint discovery ★ fase 2 |

## Activity — Pemetaan Event → Tipe

> Activity dibangun dari event NEP-297 + NearBlocks (MVP); skema event dimiliki [webhooks.md](../api/webhooks.md). `type` di bawah = nilai yang tampil di UI/API activity.

| Event sumber | `type` activity | Label UI (EN) |
|---|---|---|
| `nft_mint` / `launchpad_mint` | `mint` | "Minted" |
| `nft_transfer` (non-market) | `transfer` | "Transferred" |
| `market_list` | `listing` | "Listed" |
| `market_delist` | `delist` | "Delisted" |
| `market_update_price` | `price_update` | "Price updated" |
| `market_sale` | `sale` | "Sold" |
| `market_offer` | `offer_made` | "Offer made" |
| `market_offer_accept` | `offer_accepted` | "Offer accepted" |
| `market_offer_cancel` | `offer_cancelled` | "Offer cancelled" |
| `market_offer_expire` | `offer_expired` | "Offer expired" |
| `market_stale_detected` | `stale` | "Listing stale" |

- Satu event = satu baris activity; identity dedup `(receipt_id, event_index)` (bukan `tx_hash`).
- `from`/`to` ditentukan dari field event (`seller`/`buyer`/`old_owner_id`/`new_owner_id`); arah (in/out) dihitung relatif akun profil.
- Event bundle agregat ⏳ open-by-design (belum ada nama event kanonik) → activity bundle dapat tampil sebagai beberapa baris per token.

## Avatar — Penyimpanan & Allowlist

| Aspek | Aturan |
|---|---|
| Penyimpanan | **URL saja** di DB kolom `avatar_url` (text) — **tidak ada upload server-side di MVP** (ADR-014) |
| Skema URL | `https://` atau `ipfs://` → ditulis ulang ke gateway allowlist; `http://`, `ftp://`, `data:` **ditolak** |
| Gateway allowlist | `ipfs.dweb.link`, `ipfs.io`, gateway resmi kita (contoh provisional; final = OPEN QUESTION [metadata-security.md](../security/metadata-security.md) §5) |
| Validasi | Server-side saat `PATCH` → gagal = `400 INVALID_AVATAR` |
| Render | Hanya `<img>` (tanpa `innerHTML`); CSP `img-src` allowlist (SEC-META-001) |
| Fallback | Tidak ada avatar / invalid → **identicon** dari alamat |
| Ukuran/format | Batas ukuran respons gambar mengikuti metadata-security.md (≤5 MB); SVG dirender via `<img>` |

## Bio — Aturan Sanitasi

| Aturan | Nilai |
|---|---|
| Panjang maks | **280** karakter (Unicode code point) |
| Normalisasi | Unicode **NFC** sebelum hitung panjang & simpan |
| Trim | Buang whitespace di awal/akhir |
| Karakter kontrol | Tolak karakter kontrol (kecuali `\n`/`\t`) |
| HTML/markup | **Tidak** di-parse — disimpan sebagai teks mentah, dirender sebagai **text node** (tidak pernah `innerHTML`) |
| URL | Boleh teks; **tidak** auto-link di MVP |
| Gagal validasi | `400 INVALID_BIO` — data tidak tersimpan, pesan field spesifik |

- Alias: panjang maks **32** karakter, aturan normalisasi/trim/kontrol sama; gagal → `400 INVALID_ALIAS`.
- Alias **bukan** identifier unik (boleh sama); identitas tetap `account_id`.

## Profil — Penghapusan & Privasi

- **Reset field**: user dapat mengosongkan alias/bio/avatar (kirim `null`/string kosong via `PATCH`) → profil kembali ke default (identicon + alamat).
- **Data on-chain tidak bisa dihapus**: kepemilikan, listing, transaksi bersifat publik & permanen — reset profil off-chain tidak menyentuh chain.
- **Penghapusan baris profil DB (akun) sepenuhnya** = ⏳ open-by-design (belum ada keputusan produk; on-chain tetap ada).
- **Privasi**: MVP tidak meminta PII (nama/email); hanya alias/bio/avatar opsional. `account_id` & aset sudah publik by design.
- Log/analytics tidak menyimpan signature/seed (error-handling.md §7).

## Settings Page Spec

> Route `/settings` (guard: `wallet`) — [frontend-architecture.md](../architecture/frontend-architecture.md) §2.

| Section | Isi | Aksi |
|---|---|---|
| **Profile** | Form alias (≤32), bio (≤280, counter), avatar URL | Simpan → signature (NEP-413) → `PATCH /api/accounts/me/profile`; error per field (`INVALID_*`) |
| **Wallet** | Alamat (copy), network, tombol Disconnect | Disconnect → hapus state koneksi + sesi API in-memory |
| **Notifications** | Reset read-state (mark all read / clear) | Tulis localStorage per akun ([notifications.md](./notifications.md)) |
| **Danger / advanced** | ⏳ open-by-design | Belum ada keputusan (mis. hapus profil) |

- Simpan profil **wajib** signature ulang (bukan memakai sesi lama tanpa verifikasi) — alur kanonik di [security/wallet-authentication.md](../security/wallet-authentication.md) §3.
- Loading: form skeleton; error: toast + inline field error; sukses: toast + invalidate `['profile', account]`.

## Identicon (Fallback Avatar)

> Dipakai saat avatar tidak ada/invalid. Deterministik dari `account_id` — tidak butuh network.

```text
1. h = SHA-256(account_id)                      # 32 byte
2. warna  = h[0..3] → HSL (hue dari byte, S/L tetap agar kontras)
3. pola   = bit dari h[4..] → grid 5×5, dicerminkan horizontal (simetris)
4. render = SVG (atau canvas) → data-URI lokal; dirender via <img> (bukan innerHTML)
```

- Algoritma spesifik (hash, ukuran grid, palet) = **PROPOSED** — dikunci saat sesi branding (TASK-008b); yang tetap: deterministik, tanpa jaringan, tanpa PII.
- Akun implisit (`64-hex`) / `0x` juga menghasilkan identicon dari string alamatnya.

## API Response Shapes

**`GET /api/v1/accounts/:id/profile`** (lihat [endpoints.md](../api/endpoints.md) § Profil):

```json
{
  "account_id": "alice.testnet",
  "alias": "Alice",
  "bio": "NEAR NFT collector",
  "avatar_url": "https://ipfs.io/ipfs/bafy...",
  "updated_at": "2026-10-02T12:00:00Z"
}
```

**`GET /api/v1/accounts/:id/activity`** (★ fase 2 untuk endpoint; MVP FE langsung NearBlocks):

```json
{
  "items": [
    {
      "type": "sale",
      "contract_account_id": "nft.example.testnet",
      "token_id": "42",
      "from": "alice.testnet",
      "to": "bob.testnet",
      "price_yocto": "1000000000000000000000000",
      "tx_hash": "9f8E...",
      "block_height": 123456789,
      "timestamp": "2026-10-02T11:00:00Z"
    }
  ],
  "nextCursor": null
}
```

- Field `payout` (bila ada) memakai bentuk [data-model.md](../database/data-model.md) § PayoutSplit; semua nilai = string yoctoNEAR.
- Field tambahan bersifat aditif; klien **wajib** mengabaikan field tak dikenal (api-overview.md).

## Creator_id — Aturan Fallback

`creator_id` **bukan** field standar NEP-177 — ekstensi custom kontrak NearSea. Untuk koleksi pihak ketiga, aturan resolusi:

```text
1. Jika token/koleksi punya creator_id (ekstensi NearSea) → pakai itu.
2. Else → fallback: owner awal dari riwayat mint (event nft_mint pertama untuk token tsb).
3. Else (riwayat tidak tersedia) → tampilkan owner koleksi / "Unknown creator" (jangan kosongkan baris).
```

- Tab **Created** = token dengan `creator_id` = akun tersebut (hasil resolusi di atas).
- Untuk koleksi pihak ketiga tanpa riwayat mint yang bisa dibaca (RPC/NearBlocks gagal) → empty state "creator tidak diketahui", bukan error.

## Error Cases

| Kasus | Perlakuan |
|---|---|
| Akun tidak ada / invalid | empty state "akun tidak ditemukan" |
| Belum ada aktivitas | empty state per tab |
| Akun implisit (64-hex) / 0x | tampil apa adanya (full address) |
| Signature profil gagal | retry; data tidak tersimpan |
| Validasi gagal (alias >32 / bio >280 / avatar di luar allowlist) | 400 — data tidak tersimpan, pesan field spesifik |

## Acceptance Criteria

- Given akun memiliki 3 NFT, When buka profil, Then 3 NFT tampil di tab Owned dan activity terisi jika ada riwayat.
- Given user terhubung, When set alias/bio/avatar (signature), Then profil publik menampilkan data tsb.
