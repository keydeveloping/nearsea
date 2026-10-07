# CONTEXT.md — Glosarium NearSea

> **Glosarium istilah domain. Hanya bahasa, bukan spesifikasi.** Tidak ada detail implementasi di sini.
> Sumber kebenaran perilaku tetap `docs/`; lihat [docs/DOCUMENTATION-MAP.md](./docs/DOCUMENTATION-MAP.md).
>
> Tujuan: mencegah satu kata dipakai untuk dua arti berbeda. Setiap istilah yang pernah ambigu dicatat
> di sini dengan **satu** definisi kanonik.

## Cara pakai

Kalau kamu menemukan istilah yang dipakai berbeda di dua dokumen, tambahkan di sini dan selaraskan
dokumennya. Format: istilah → definisi kanonik → padanan yang **bukan** maksudnya.

---

## Istilah inti

### Marketplace
Produk NearSea: tempat orang mem-list dan membeli NFT. Terdiri dari **kontrak market** (satu, ADR-003),
**kontrak koleksi NFT** (satu per koleksi, dibuat via factory), dan **frontend**.

### Kontrak market
Kontrak tunggal yang menangani listing, offer, private listing, dan bundle (ADR-003). Bukan "marketplace"
— marketplace mencakup frontend + kontrak.

### Kontrak koleksi / kontrak NFT
Kontrak per koleksi yang menyimpan token, metadata, approval (NEP-178), dan royalti (NEP-199).
Dibuat lewat **factory**. Di M1 di-deploy manual.

### Factory
Kontrak yang membuat kontrak koleksi baru untuk kreator (ADR-006, ADR-008). **Tidak dipakai di M1.**

### Listing
Penawaran jual sebuah token dengan harga tetap. **Non-custodial**: NFT tetap di wallet seller; market
hanya memegang approval (ADR-007). Dibuat dengan **2 transaksi** (ADR-002). Permanen (tanpa expiry);
otomatis **stale** bila ownership berubah.

### Stale
Kondisi listing yang tidak lagi valid karena **ownership token berubah** (token dipindah di luar market).
Listing stale disembunyikan dari discovery dan tidak bisa dibeli.

> ⚠️ **Ambigu diketahui (temuan H4, ronde 17):** definisi saat ini hanya mencakup **ownership mismatch**,
> padahal listing juga jadi tidak valid bila **approval dicabut** tanpa memindahkan token. Itu
> menghasilkan listing "zombie" yang tampil tapi selalu gagal dibeli. Definisi final harus mencakup
> kedua kasus.

### Offer
Tawaran beli dengan **escrow Ⓝ** di kontrak market. Boleh untuk token apa pun (ter-list atau tidak).
Default expire 7 hari; maksimum 1 offer aktif per buyer per token.

### Order
⚠️ **Ambigu — jangan pakai tanpa kualifikasi.** Di [order-protocol-security.md](./docs/security/order-protocol-security.md)
"order" kadang berarti **listing**, kadang **offer**, kadang **bundle**. Dokumen itu menyebut "dua jenis
order" lalu menambahkan bundle sebagai jenis ketiga.

**Aturan:** tulis istilah spesifiknya (`listing` / `offer` / `bundle`), jangan `order`. Kalau harus
menyebut ketiganya: **"entri order"** dengan daftar eksplisit.

### Bundle
Beberapa token (maks 10) dijual dengan **satu harga**. Royalti per token dijumlahkan lalu di-merge per
penerima (maks 10 receiver unik). **Ditunda ke M1+** (ADR-016) — dan tiga temuan kritis (C1/C2/C3)
harus diselesaikan sebelum dikerjakan.

### Private listing
Listing yang hanya bisa dibeli oleh **satu alamat** (`allowed_buyer`).

> ⚠️ **Bukan privasi.** Harga dan alamat target tersimpan publik on-chain; hanya *grid* yang
> menyembunyikannya. Ini **access control**, bukan kerahasiaan (temuan M1, ronde 17).

### Launchpad
Fitur kreator membuat koleksi + beberapa **fase mint** (harga/alokasi/allowlist/waktu berbeda) via
factory (ADR-008). **Ditunda ke M1+** (ADR-016).

### Fase (phase)
Satu tahap mint dalam launchpad. Fase **berurutan, tidak overlap** (INV-029) — maksimum satu aktif.
Window `[starts_at, ends_at)`.

### Allowlist
Daftar alamat yang boleh mint pada fase tertentu. Disimpan sebagai **set on-chain**; storage dibayar
kreator. Bukan "whitelist" (istilah lama yang sudah tidak dipakai).

### Settlement
Proses memindahkan NFT ke buyer **dan** mendistribusikan dana (seller + royalti + fee) dalam satu
transaksi. Dipicu `buy` atau `accept_offer`; divalidasi di callback `resolve_purchase`.

### Fee platform
2% dari harga penjualan, dipotong **on-chain saat settlement**, dikirim ke treasury (ADR-005).
Nilai default `fee_bps = 200`; **cap immutable** `MAX_FEE_BPS = 500`. Cap ≠ nilai fee.

### Royalti
Bagian hasil penjualan untuk kreator, maksimum **10% per token** (NEP-199). Dipaksa on-chain oleh
NearSea — tapi lihat catatan: ini **pilihan ideologis/teknis**, bukan keunggulan kompetitif
(perang royalti 2022–2023 sudah selesai; lihat [research/near-nft-market-2026.md](./docs/research/near-nft-market-2026.md)).

### Escrow
Dana Ⓝ yang ditahan kontrak market selama **offer aktif**. Selalu bisa diambil kembali oleh pemiliknya
(cancel/expire). Bukan kustodi NFT — NFT tidak pernah dipegang kontrak.

### Storage deposit (NEP-145)
Deposit Ⓝ yang dibayar **user** untuk menutup biaya penyimpanan state-nya di kontrak. Tidak otomatis
kembali saat refund/cancel — ditarik manual via `storage_withdraw`. Nominal dihitung dari
`storage_usage` (open-by-design).

### Treasury
Akun penerima fee platform. Alamatnya **open-by-design** (ditetapkan saat deploy).

### MVP
⚠️ **Ambigu — jangan pakai tanpa kualifikasi (koreksi ronde 17).** Kata ini pernah dipakai untuk dua
hal berbeda:
- **M1 (vertical slice)** — 8 task: mint → list → buy + royalti + fee, testnet. **Ini yang wajib.**
- **M1+ (MVP completion)** — sisa fitur MVP lama (offers, bundle, launchpad, moderasi, infra).

**Aturan:** tulis `M1` atau `M1+`, jangan `MVP` sendirian.

### M1 / M1+ / M2 / M3 / M4
- **M1** — vertical slice (8 task). Milestone pertama yang wajib lulus.
- **M1+** — MVP completion (lanjutan eksplisit).
- **M2** — trading lanjutan (auction, FT, indexer awal).
- **M3** — ekosistem (lazy minting, indexer penuh, onboarding).
- **M4** — mainnet ⚠️ **hanya bila proyek beralih ke jalur produk** (ADR-016).

### Niat proyek
**Pembelajaran/portofolio** (ADR-016). Bukan usaha komersial; tidak ada target pengguna/pendapatan.
Sukses = tesis inti terbukti + penulisnya paham cara kerjanya.

---

## Singkatan & istilah teknis

| Istilah | Arti |
|---|---|
| **NEP-171** | Standar NFT inti NEAR (transfer, approval dasar) |
| **NEP-177** | Metadata NFT (title, media, extra) |
| **NEP-178** | Approval management (dasar model listing non-custodial) |
| **NEP-181** | Enumeration (daftar token per owner) |
| **NEP-199** | Royalti (`nft_transfer_payout`) |
| **NEP-145** | Storage management (user bayar storage sendiri) |
| **NEP-297** | Event standard (`EVENT_JSON:`) |
| **NEP-413** | Sign-message auth (login wallet off-chain) |
| **NEP-330** | Reproducible build (`contract_source_metadata`) |
| **yoctoNEAR** | Satuan terkecil NEAR (1 Ⓝ = 10²⁴ yocto). Selalu **string** di JSON |
| **Tgas** | Satuan gas (300 Tgas = batas keras per transaksi) |
| **stale** | Lihat §Stale |
| **dual verification** | Cek ownership **dan** approval saat listing & saat settle |
| **optimistic removal** | Hapus entri listing dulu, lalu settle; dipulihkan bila gagal |
| **PARTIAL** | Status bundle saat sebagian token sudah pindah tapi settlement gagal (temuan C1 — belum tertutup) |
| **G7** | Kebijakan kompensasi (insurance fund 0.1% fee) — **tidak punya jalur pendanaan** (temuan C1) |

---

## Istilah yang DILARANG dipakai (sudah usang)

| Jangan pakai | Pakai ini | Alasan |
|---|---|---|
| `whitelist` | `allowlist` | istilah lama |
| `order` (tanpa kualifikasi) | `listing` / `offer` / `bundle` | ambigu (3 arti) |
| `MVP` (tanpa kualifikasi) | `M1` atau `M1+` | ambigu (2 arti) |
| `nft_revoke_token` | `nft_revoke` | nama method NEP-178 yang benar |
| `NEAR Lake` | `Neardata` | NEAR Lake deprecated 2026-03 |
| `Vercel`/`Supabase` | VPS self-hosted | digantikan ADR-009 |
| `2,5%` (fee OpenSea) | ~1% | data usang |
