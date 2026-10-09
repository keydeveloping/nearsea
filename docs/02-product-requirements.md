# 02 — Product Requirements (Behavior Specification)

> Menjabarkan tiap kemampuan PRD jadi langkah behavior yang bisa diuji. Satu blok per kemampuan.
>
> **Struktur tiap kemampuan**: blok ASCII (perilaku ringkas) di awal, lalu **Spec Engineering** (pre/postcondition, validasi per langkah, skema method, state machine, edge case numerik, event/gas/storage, UI copy EN). Skema kontrak lengkap & aturan kanonik dimiliki [features/marketplace.md](./features/marketplace.md); dokumen ini **tidak** menyalin ulang field kontrak yang sudah ada — ia menambahkan kontrak perilaku (pre/post + validasi + copy).

## Konvensi

| Simbol / label | Arti |
|---|---|
| **Pre** | Kondisi yang wajib benar sebelum langkah dimulai |
| **Post** | Kondisi yang dijamin benar setelah langkah sukses |
| **Invariant** | INV-xxx yang menjaga langkah ([security/smart-contract-invariants.md](./security/smart-contract-invariants.md)) |
| **Event** | Event NEP-297 yang di-emit ([api/webhooks.md](./api/webhooks.md)) |
| `⏳ open-by-design` | Nilai/aturan belum diputuskan — dilarang menebak di kode |
| **PROPOSED** | Estimasi engineering (bukan fakta terkunci) |
| string yoctoNEAR | Semua nilai uang = JSON string u128 (dilarang `number`) |
| Kode error | Dari registry [development/error-handling.md](./development/error-handling.md) §3 |

### Matriks Lintas-Kemampuan — Status, Event, Gas, Storage

> Ringkasan satu layar. Detail per kemampuan ada di bagian Spec Engineering masing-masing. Gas bertanda PROPOSED diukur di sandbox; storage = NEP-145 (user bayar).

| Kemampuan | Method utama | Status awal → akhir | Event | Gas (Tgas) | Storage dibayar |
|---|---|---|---|---|---|
| Connect wallet | (wallet, bukan kontrak) | disconnected → connected | — | — | — |
| Buy NFT | `buy` | ACTIVE → SOLD | `market_sale` | <150 PROPOSED | — |
| Mint NFT | `nft_mint` | phase → minted | `launchpad_mint` | PROPOSED | pemicu mint (INV-019) |
| Create collection | factory `create_collection` | — → deployed | ⏳ open-by-design | PROPOSED | creator (allowlist/collection) |
| List NFT | `list_nft_for_sale` | — → ACTIVE | `market_list` | **≈10–20 total** (tulis ~5 + 2 view ~3–5) PROPOSED | entry `Sale` |
| Cancel/update listing | `remove_sale` / `update_price` | ACTIVE → CANCELLED / ACTIVE(harga baru) | `market_delist` / `market_update_price` | ~5 PROPOSED | — |
| Make/accept offer | `make_offer` / `accept_offer` | ACTIVE → ACCEPTED/EXPIRED/CANCELLED/SUPERSEDED | `market_offer` / `market_offer_accept` | ~5 / <150 PROPOSED | entry `Offer` |
| Private listing | `list_nft_for_sale{allowed_buyer}` | ACTIVE(private) → SOLD | `market_list` | **≈10–20 total** PROPOSED | entry `Sale` |
| Bundle | `create_bundle` / `buy_bundle` | ACTIVE → SOLD/PARTIAL | `market_sale` / `market_bundle_partial` | wajib diukur | entry `Bundle` + item |
| Stale listing | (deteksi) / `remove_stale_listing` | ACTIVE → STALE | `market_stale_detected` / `market_delist` | ~5 PROPOSED | — |
| Search & filter | view/RPC | — | — | — | — |
| View profile | view/RPC + NearBlocks | — | — | — | — |
| Profil custom | `PATCH /api/…/profile` | — → updated | — | — | — |
| Notifications | polling | — | (konsumsi event) | — | — |
| Admin/moderation | `PATCH /api/v1/admin/*` | open → resolved | — | — | — |

## CONNECT WALLET

```text
1. User klik Connect Wallet (header)
2. Modal near-connect: pilih wallet (HOT / Meteor / Nightly / …)
3. User approve di wallet
4. Alamat + saldo tampil di header; state tersimpan (reconnect otomatis)
Error: tolak popup → toast; wallet tak terpasang → disabled + hint; network mismatch → banner + disable tx
```

## BUY NFT

```text
1. User membuka detail NFT
2. Sistem re-verify via view call: status listing, harga, ownership (SEC-ORDER-003)
3. Sistem menampilkan harga + breakdown fee/royalti
4. User klik Buy → modal konfirmasi
5. Wallet signing (attach deposit = harga)
6. Market contract memanggil nft_transfer_payout (settlement + royalti NEP-199, fee 2%)
7. Sistem menunggu confirmation on-chain
8. Ownership berubah, listing jadi SOLD
9. Seller + penerima royalti terbayar; fee → treasury
10. Gagal settlement → refund otomatis ke buyer via resolve_purchase
```

## MINT NFT (launchpad)

```text
1. User membuka halaman koleksi → tab Mint
2. Sistem membaca config phase: fase aktif (waktu), harga, max mint/wallet, allowlist
3. Sistem menampilkan fase yang berlaku untuk user (whitelist: cek keanggotaan on-chain)
4. User pilih jumlah → sign nft_mint dengan deposit = harga fase (+ biaya storage mint ditanggung pemicu mint)
5. Kontrak validasi: fase aktif, allowance, alokasi, deposit exact
6. Token ter-mint ke wallet user
7. Event nft_mint → notifikasi + aktivitas/feed ter-update
Error: bukan allowlist / fase belum mulai / alokasi habis / deposit salah → tx ditolak
```

## CREATE COLLECTION (factory + launchpad)

> DIPUTUSKAN ronde 6+10: via **factory** (launchpad ala OpenSea Studio). Creator:
> (1) isi metadata koleksi (nama, simbol, deskripsi, media — penyimpanan media menyusul keputusan IPFS),
> (2) pilih kebijakan supply — fixed max supply atau open — tanpa cap platform,
> (3) royalti ≤ 10%,
> (4) definisikan **phase mint sebanyak apa pun** (tiap phase: harga, alokasi/max per wallet, allowlist opsional, waktu mulai–selesai),
> (5) upload allowlist per phase (on-chain set; storage pre-deposit creator).
> Koleksi langsung tampil di discovery setelah deploy.

## LIST NFT FOR SALE

```text
1. Seller klik Sell pada NFT miliknya → set harga (min 0.01 Ⓝ)
2. Tx 1 — nft_approve(market, msg=harga) di NFT contract (NFT TETAP di wallet — approval model)
3. Tx 2 — list_nft_for_sale di market (storage deposit seller terpenuhi)
4. Dual verification (ownership + approval) → listing aktif, tampil di discovery
Error: harga < 0.01 Ⓝ ditolak; sudah terlist → ditolak; storage kurang → minta deposit dulu
```

## CANCEL LISTING / UPDATE PRICE

```text
1. Seller buka listing miliknya → Cancel / Edit Price
2. assert_one_yocto; hanya owner listing yang bisa
3. Cancel → entry `Sale` dihapus (market **tidak** mencabut approval — NEP-178 owner-only) + hapus listing
   Update → sale_conditions diganti
Error: bukan owner → revert
```

## MAKE OFFER / ACCEPT OFFER

```text
MAKE OFFER (token apa pun — di-list atau tidak; 1 aktif per buyer per token; min 0.01 Ⓝ; default 7 hari):
1. Buyer klik Make Offer → set jumlah + durasi (custom, default 7 hari)
2. make_offer → Ⓝ escrow di market
ACCEPT:
3. Seller lihat offer (halaman token / notifikasi) → accept_offer
4. accept_offer → nft_transfer_payout → NFT ke buyer, escrow terdistribusi
   (seller + royalti − fee 2%); offer lain pada token yang sama auto-cancel + refund
5. Expire/cancel → refund penuh ke buyer
Error: offer kedua oleh buyer sama → ditolak (batal dulu); self-offer → ditolak; sudah expire → refund
```

## PRIVATE LISTING

```text
1. Seller set listing dengan allowed_buyer = alamat tertentu
2. Discovery menandai listing private (tidak di-grid publik / tampil khusus)
3. Buyer lain mencoba buy → ditolak kontrak ("bukan buyer yang ditunjuk")
4. allowed_buyer membeli → settlement normal (fee + royalti on-chain)
```

## BUNDLE (satu harga — ronde 6; maks 10 token — INV-021)

```text
1. Seller pilih maks 10 token miliknya → set SATU harga untuk seluruh bundle
2. Approve semua token ke market (NEP-178 per token)
3. Buyer buy_bundle → deposit = harga bundle
4. Pre-validasi SEMUA item (INV-025) → loop nft_transfer_payout per token → royalti per token dijumlahkan,
   di-merge per receiver (maks 10 receiver unik — ditolak saat create_bundle bila lebih)
   → validasi total ≤ harga → fee 2% dipotong sekali → semua NFT pindah ke buyer
5. Gagal pre-validasi → abort bersih + refund penuh; gagal residual mid-loop → status PARTIAL + kompensasi G7
Error: > 10 token ditolak; salah satu token stale/dipindah/di-list terpisah → bundle tidak bisa dibeli
```

## STALE LISTING (auto)

```text
1. Seller memindah NFT di luar market saat listing aktif
2. Marketplace deteksi ownership mismatch (saat view/offer/buy)
3. Listing auto-stale → sembunyi dari discovery; tidak bisa dibeli
```

## SEARCH & FILTER

```text
1. Search bar: nama koleksi/token → **MVP: FE baca langsung dari view call RPC
   (client-side filter atas hasil yang sudah dimuat — TANPA endpoint discovery; ADR-004).
   Endpoint + proyeksi penuh dari indexer Neardata = ★ fase 2** (endpoints.md).
   NearBlocks untuk riwayat
2. Filter harga min/max; sort: terbaru / termurah / termahal
3. Trait filter & rarity = fase 2 (butuh custom indexer)
```

> Koreksi ronde 16: blok lama menyebut "query ke Report API" — bertentangan dengan
> [ADR-004](./decisions/ADR-004-mvp-data-layer.md) (API MVP = auth/profil/report/admin saja).
> Sumber discovery MVP = RPC langsung dari FE.

## VIEW PROFILE & ACTIVITY

```text
1. Buka /profile/:account (atau klik alamat mana pun)
2. Tab: Owned (NEP-181) / Created / Offers / Activity (NearBlocks + market views)
```

## PROFILE CUSTOM (alias/bio/avatar)

```text
1. User (login wallet) buka Settings → edit alias/bio/avatar
2. Frontend minta challenge → wallet sign (NEP-413 utama / fallback custom)
3. PATCH /api/accounts/me/profile → server verifikasi signature + validasi (alias ≤32, bio ≤280, avatar URL allowlist)
4. Profil publik menampilkan data custom; alamat & aset tetap on-chain
```

## NOTIFICATIONS (in-app — ronde 6)

```text
1. FE polling NearBlocks (tx user) + view call market (offers aktif) tiap 30–60 dtk saat tab aktif
2. Diff → daftar notifikasi (offer diterima, offer di-accept, token terjual, mint berhasil, offer expire via cek expiry lokal)
3. Read-state di localStorage; **idempotency via `(receipt_id, event_index)`** (satu tx bisa memuat banyak event — bukan `tx_hash`; selaras [database-schema.md](./database/database-schema.md) & [webhooks.md](./api/webhooks.md))
```

## ADMIN / MODERATION (report system — ronde 3 + revisi ronde 5)

1. User klik "Report" pada koleksi/token → pilih alasan → kirim.
2. Report tersimpan di PostgreSQL kecil di VPS (ronde 5) — satu-satunya backend MVP.
3. Admin membuka antrean review (panel /admin — allowlist + wallet signature, ronde 11; step-up signature per aksi).
4. Admin memutuskan: abaikan / sembunyikan dari discovery (blocklist layer tampilan).
5. Takedown TIDAK menyentuh chain — kepemilikan & transaksi on-chain tidak pernah berubah.

---

## Spesifikasi Engineering per Kemampuan

> Bagian ini menambahkan kontrak perilaku (pre/post + validasi + skema + state machine + edge case + copy) untuk tiap kemampuan di atas. Aturan kontrak lengkap dimiliki [features/marketplace.md](./features/marketplace.md) & [features/payments.md](./features/payments.md).

### CONNECT WALLET — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre** | Browser modern; near-connect terpasang; `NEAR_NETWORK` ter-set |
| **Post** | Alamat publik tersimpan (localStorage); saldo tampil; sesi API **tidak** dibuat (transaksi chain tak butuh sesi) |
| **Post (gagal)** | State kembali `disconnected`; tidak ada tx terkirim |

**Validasi per langkah**

| Langkah | Validasi | Gagal → |
|---|---|---|
| 1 Klik Connect | — | — |
| 2 Pilih wallet | Wallet terpasang di browser | Tombol disabled + hint install (lokal) |
| 3 Approve | User menyetujui popup | Toast "dibatalkan"; kembali disconnected |
| 4 Terhubung | Network wallet = `NEAR_NETWORK` | `network-mismatch` → banner + disable tx |

**Skema (bukan kontrak — state client)**

```json
{ "connected": true, "accountId": "alice.testnet", "network": "testnet",
  "balanceYocto": "5000000000000000000000000" }
```

**State machine (modal)**

| State | Pemicu | Transisi berikut |
|---|---|---|
| `closed` | default | → `wallet-picker` |
| `wallet-picker` | klik Connect | → `connecting` |
| `connecting` | pilih wallet | → `approving` / `error` |
| `approving` | tunggu wallet | → `connected` / `rejected` |
| `connected` | approve sukses | → `closed` |
| `rejected` | user tolak | → `wallet-picker` |
| `error` | error/timeout | → `wallet-picker` (retry) |
| `network-mismatch` | network beda | → `connected` setelah switch network |

**Edge case**

| Kasus | Perilaku |
|---|---|
| Reload halaman | Wallet tetap connected; sesi API hilang (in-memory) |
| Ganti akun di wallet | Sesi API dicabut + invalidate query ber-scope akun |
| Wallet tanpa `signMessage` | Browse saja; tulis API pakai custom fallback |
| Wallet tanpa `signAndSendTransaction` | Tidak bisa bertransaksi (hanya browse) |

**Event / Gas / Storage**: tidak ada (bukan transaksi kontrak).

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `auth.connect` | "Connect Wallet" |
| `auth.picker.title` | "Select a wallet" |
| `auth.approving` | "Approve the connection in your wallet" |
| `auth.rejected` | "Connection cancelled" |
| `auth.networkMismatch` | "Your wallet is on a different network. Switch to {network} to transact." |
| `auth.installHint` | "Wallet not installed — get it first" |

### BUY NFT — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre** | Listing `ACTIVE`; buyer ≠ seller (INV-023); bila private → buyer = `allowed_buyer` (INV-026); saldo ≥ harga |
| **Pre** | FE re-verify via view call (SEC-ORDER-003): harga, owner, status |
| **Post** | NFT di wallet buyer; seller + royalti terbayar; fee 2% → treasury; listing `SOLD` |
| **Post (gagal)** | Refund 100% ke buyer (`resolve_purchase`); listing tetap `ACTIVE` bila kegagalan bukan stale |

**Validasi per langkah**

| Langkah | Validasi on-chain | Gagal → kode |
|---|---|---|
| `buy` | `sale` ada & `status == ACTIVE` | `CONFLICT_SOLD` |
| `buy` | `predecessor != sale.owner_id` | `FORBIDDEN_SELF_BUY` |
| `buy` | `allowed_buyer == null ∨ predecessor == allowed_buyer` | `FORBIDDEN_BUYER` |
| `buy` | `attached_deposit ≥ price` | `CHAIN_INSUFFICIENT_DEPOSIT` |
| settle | `nft_token().owner_id == sale.owner_id` | `CONFLICT_STALE` |
| resolve | `1 ≤ |payout| ≤ 10`; `Σpayout ≤ price−fee`; sisa ≤1 yocto | `CHAIN_REVERT` |

**Skema method**

```json
// args: buy(nft_contract_id, token_id) — payable, deposit ≥ harga
{ "nft_contract_id": "nearsea-open.testnet", "token_id": "101" }

// return (view): get_sale(nft_contract_id, token_id)
{ "owner_id": "alice.testnet", "approval_id": 7,
  "sale_conditions": { "price": "10000000000000000000000000" },
  "allowed_buyer": null }
```

**State machine**: `ACTIVE → SOLD` (sukses) atau `ACTIVE → ACTIVE` (gagal + refund).

**Edge case numerik**

| Input | Perilaku | Kode |
|---|---|---|
| `attached = price` | Sukses | — |
| `attached = price − 1` | Tolak | `CHAIN_INSUFFICIENT_DEPOSIT` |
| `attached = price_lama > price_baru` | Kelebihan direfund via resolve | `CONFLICT_PRICE_CHANGED` (preview) |
| 20 pembeli paralel | Tepat 1 sukses; 19 revert + refund | `CONFLICT_SOLD` |
| Royalti token > 10% | Tolak + refund | `CHAIN_REVERT` |

**Event / Gas / Storage**

| Aksi | Event | Gas | Storage |
|---|---|---|---|
| `buy` | `market_sale` | <150 PROPOSED (15 + 115 FACT) | — (listing sudah ada) |

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `buy.button` | "Buy now" |
| `buy.confirm.title` | "Confirm purchase" |
| `buy.breakdown.fee` | "Platform fee (2%)" |
| `buy.breakdown.royalty` | "Creator royalty" |
| `buy.breakdown.total` | "Total" |
| `buy.pending` | "Waiting for confirmation…" |
| `buy.success` | "Purchase complete" |
| `errors.CONFLICT_SOLD` | "Already sold — your funds are on the way back" |
| `errors.FORBIDDEN_SELF_BUY` | "You can't buy your own item" |
| `errors.CONFLICT_STALE` | "This listing is no longer valid" |

### MINT NFT (launchpad) — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre** | Ada fase aktif (waktu); alokasi tersisa; jumlah ≤ max/wallet; bila fase whitelist → minter di allowlist |
| **Post** | Token ter-mint ke wallet minter; alokasi fase berkurang; `launchpad_mint` di-emit |
| **Post (gagal)** | Deposit dikembalikan otomatis via rollback tx (INV-018) |

**Validasi per langkah**

| Langkah | Validasi | Gagal → |
|---|---|---|
| 4 `nft_mint` | `∃ active_phase` | Tolak |
| 4 | `alloc_left ≥ jumlah` | Tolak |
| 4 | `count_minted ≤ max_per_wallet` | Tolak |
| 4 | `allowlist_required ⇒ minter ∈ set` | "not in allowlist" |
| 5 | `attached_deposit == phase.price` (exact) | Tolak (INV-018) |
| 5 | storage mint dibayar pemicu mint | Tolak bila kurang (INV-019) |

**Skema method**

```json
// args: nft_mint(collection, phase_index?, quantity) — payable, exact = price × qty
{ "phase_index": 0, "quantity": 1 }

// return (view): get_launchpad(collection)
{ "phases": [ { "index": 0, "name": "Whitelist",
    "price_yocto": "2000000000000000000000000",
    "allocation": 100, "max_per_wallet": 2, "allowlist_required": true,
    "starts_at": 1799016000000000000, "ends_at": 1799102400000000000 } ] }
```

**State machine (fase)**: `SCHEDULED → ACTIVE → ENDED`; maksimum satu `ACTIVE` pada satu waktu (INV-029).

**Edge case numerik**

| Input | Perilaku |
|---|---|
| Fase overlap | Tolak saat konfigurasi (INV-029) |
| `deposit = price − 1` | Tolak (INV-018) |
| Alokasi habis | Tolak (INV-017) |
| Bukan allowlist | Tolak, deposit tidak hangus |
| Mint tepat `max_per_wallet` | Diterima (batas inklusif) |

**Event / Gas / Storage**

| Aksi | Event | Gas | Storage |
|---|---|---|---|
| `nft_mint` | `launchpad_mint` (+ `launchpad_phase_start` saat fase mulai) | PROPOSED | token + metadata, dibayar pemicu mint (INV-019) |

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `mint.button` | "Mint" |
| `mint.phase.active` | "{name} — {price} Ⓝ" |
| `mint.phase.endsIn` | "Ends in {time}" |
| `mint.notAllowlisted` | "This wallet is not on the allowlist for this phase" |
| `mint.phaseNotStarted` | "This phase hasn't started yet" |
| `mint.allocationSoldOut` | "This phase is sold out" |
| `mint.maxPerWallet` | "You've reached the mint limit for this phase" |
| `mint.success` | "Minted successfully" |

### CREATE COLLECTION (factory + launchpad) — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre** | Creator wallet terhubung; metadata lengkap; royalti ≤ 10%; phase non-overlap; allowlist terupload (bila perlu) |
| **Post** | Kontrak koleksi ter-deploy via factory; koleksi tampil di discovery |
| **Post (gagal)** | Tidak ada kontrak ter-deploy; state form dipertahankan untuk retry |

**Validasi per langkah**

| Langkah | Validasi | Gagal → |
|---|---|---|
| 1 Form | Nama/simbol wajib; royalti ≤ 10% | Field error |
| 2 Supply | fixed max atau open (tanpa cap platform) | — |
| 4 Phase | `ends_at > starts_at`; phase tidak overlap (INV-029) | Tolak |
| 5 Allowlist | `max_mints ≤ max_per_wallet` per baris | Pesan baris salah |
| Deploy | Storage creator cukup (allowlist set + collection) | Tolak → minta deposit |

**Skema method (factory)**

```json
// args: create_collection(metadata, royalty_bps, supply, phases[], allowlists[])
{ "name": "Genesis", "symbol": "GNS",
  "royalty_bps": 500,
  "supply": { "kind": "open" },
  "phases": [ { "index": 0, "name": "Whitelist",
    "price_yocto": "2000000000000000000000000",
    "allocation": 100, "max_per_wallet": 2, "allowlist_required": true,
    "starts_at": 1799016000000000000, "ends_at": 1799102400000000000 } ] }
```

**State machine (koleksi)**: `DRAFT (form) → DEPLOYING → LIVE`.

**Edge case**

| Kasus | Perilaku |
|---|---|
| Royalti 15% | Ditolak (UI + kontrak) |
| Phase overlap | Ditolak (INV-029) |
| Allowlist baris melebihi max/wallet | Ditolak dengan baris salah |
| Supply open | Bebas, tanpa cap platform |

**Event / Gas / Storage**

| Aksi | Event | Gas | Storage |
|---|---|---|---|
| `create_collection` | ⏳ open-by-design (belum ada nama event kanonik) | PROPOSED | creator (collection + allowlist set) |

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `create.title` | "Create a collection" |
| `create.field.name` | "Name" |
| `create.field.royalty` | "Creator royalty (%)" |
| `create.field.royaltyHint` | "Maximum 10%" |
| `create.phase.add` | "Add phase" |
| `create.allowlist.upload` | "Upload allowlist (CSV)" |
| `create.review` | "Review & deploy" |
| `create.success` | "Collection created" |
| `errors.INVALID_ROYALTY` | "Royalty must be 10% or less" |
| `errors.CONFLICT_PHASE_OVERLAP` | "Phases cannot overlap" |

### LIST NFT FOR SALE — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre** | Seller = owner token; kontrak NFT mendukung NEP-178; storage deposit market terpenuhi (NEP-145); harga ≥ 0.01 Ⓝ (INV-030) |
| **Post** | Listing `ACTIVE`; NFT **tetap** di wallet seller; `market_list` di-emit |
| **Post (gagal Tx-2)** | Approval menggantung; seller bisa list ulang atau `nft_revoke`; tidak ada dana hilang |

**Validasi per langkah**

| Langkah | Validasi | Gagal → kode |
|---|---|---|
| Tx-1 `nft_approve` | Pemanggil = owner token | `CHAIN_REVERT` |
| Tx-2 `list_nft_for_sale` | `storage_deposit ≥ required` | Tolak → minta deposit |
| Tx-2 | `price ≥ 0.01 Ⓝ` | `INVALID_PRICE` |
| Tx-2 | Belum ada listing aktif token ini | `CONFLICT_ALREADY_LISTED` |
| dual verify | `nft_token().owner_id == predecessor` | `CONFLICT_STALE` |
| dual verify | `nft_is_approved(token, market, approval_id)` | `CHAIN_REVERT` |
| (bundle aktif) | Token ∉ bundle aktif | Tolak (INV-028) |

**Skema method**

```json
// args: list_nft_for_sale(nft_contract_id, token_id, approval_id, price, allowed_buyer)
{ "nft_contract_id": "nearsea-open.testnet", "token_id": "101",
  "approval_id": 7, "price": "10000000000000000000000000", "allowed_buyer": null }
```

**State machine**: `(none) → ACTIVE`; `ACTIVE → SOLD | CANCELLED | STALE`.

**Edge case numerik**

| Input | Perilaku |
|---|---|
| `price = 0.01 Ⓝ` | Diterima (inklusif) |
| `price = 0.009…` | Tolak (INV-030) |
| List token yang sama 2× | Tolak (INV-007) |
| Storage kurang 1 yocto | Tolak (INV-020) |
| Token sudah di bundle aktif | Tolak (INV-028) |

**Event / Gas / Storage**

| Aksi | Event | Gas | Storage |
|---|---|---|---|
| `nft_approve` | (NEP-178) | ~10–15 PROPOSED | approval |
| `list_nft_for_sale` | `market_list` | **≈10–20 total** (tulis ~5 + 2 view ~3–5) PROPOSED | entry `Sale` (NEP-145, ⏳ nominal) |

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `list.button` | "Sell" |
| `list.step1` | "Step 1 of 2: Approve the marketplace" |
| `list.step2` | "Step 2 of 2: Confirm the listing" |
| `list.price.label` | "Price (Ⓝ)" |
| `list.price.min` | "Minimum price is 0.01 Ⓝ" |
| `list.private.toggle` | "Private listing" |
| `list.success` | "Your item is listed" |
| `errors.CONFLICT_ALREADY_LISTED` | "This item is already listed" |

> **Koreksi ronde 17 (label langkah).** Harga **bukan** salah satu dari 2 langkah — harga diisi di
> **form** sebelum signing, lalu ada **2 tanda tangan**: approve (langkah 1) dan list (langkah 2).
> Versi lama menulis `list.step2` = "Set your price", yang membalik urutan terhadap alur sebenarnya
> ([03-user-flows.md](./03-user-flows.md) FL-LIST-1..3: set harga → approve → list). Progress indicator
> harus mencerminkan **langkah signing**, bukan pengisian form.
>
> **Catatan biaya tersembunyi (temuan UX B4):** ada biaya storage NEP-145 yang bisa memicu **popup
> ketiga** (deposit storage) sebelum `list_nft_for_sale`. Progress "2 langkah" tidak mengantisipasinya.
> Untuk M1 (slice), nominal storage dihitung dari `storage_usage` dan ditampilkan di modal sebelum
> signing — lihat [features/marketplace.md](./features/marketplace.md) §Storage Deposit.

### CANCEL LISTING / UPDATE PRICE — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre** | Pemanggil = `sale.owner_id`; listing `ACTIVE` (INV-012) |
| **Post (cancel)** | Entry `Sale` dihapus; market **tidak** mencabut approval (NEP-178 owner-only — `contracts/market.md` §2a); NFT tetap di wallet; `market_delist` |
| **Post (update)** | `sale_conditions.price` diganti; `approval_id` tidak berubah; `market_update_price` |

**Validasi per langkah**

| Langkah | Validasi | Gagal → |
|---|---|---|
| 2 | Pemanggil = seller, listing ACTIVE | Revert |
| 3 | `assert_one_yocto` | Revert |
| 4 update | `new_price ≥ 0.01 Ⓝ` | `INVALID_PRICE` |
| cancel saat paused | Diizinkan (INV-022) | — |

**Skema method**

```json
// remove_sale(nft_contract_id, token_id) — 1 yocto
{ "nft_contract_id": "nearsea-open.testnet", "token_id": "101" }

// update_price(nft_contract_id, token_id, new_price) — 1 yocto
{ "nft_contract_id": "nearsea-open.testnet", "token_id": "101",
  "new_price": "1500000000000000000000000" }
```

**State machine**: `ACTIVE → CANCELLED` (cancel); `ACTIVE → ACTIVE` (update, harga berubah).

**Edge case**

| Kasus | Perilaku |
|---|---|
| Bukan owner | Revert tanpa efek |
| Update harga < min | Tolak (INV-030) |
| Update harga listing bundle | Tidak berlaku — bundle = satu harga (INV-028) |
| Cancel saat paused | Diizinkan (INV-022) |

**Event / Gas / Storage**

| Aksi | Event | Gas | Storage |
|---|---|---|---|
| `remove_sale` | `market_delist` | ~5 PROPOSED | ditarik via `storage_withdraw` |
| `update_price` | `market_update_price` | ~5 PROPOSED | — |

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `listing.cancel` | "Cancel listing" |
| `listing.editPrice` | "Edit price" |
| `listing.cancel.confirm` | "Remove this listing?" |
| `listing.cancelled` | "Listing removed" |
| `listing.priceUpdated` | "Price updated" |

### MAKE OFFER / ACCEPT OFFER — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre (make)** | Buyer ≠ owner token saat itu (INV-023); tidak ada offer aktif buyer ini pada token (INV-024); jumlah ≥ 0.01 Ⓝ (INV-030) |
| **Post (make)** | Escrow Ⓝ = jumlah (exact, INV-006); offer `ACTIVE`; `market_offer` |
| **Pre (accept)** | Pemanggil = owner token saat itu; `block_timestamp < expires_at` (INV-010) |
| **Post (accept)** | NFT ke buyer; escrow terdistribusi (seller + royalti − fee 2%); offer lain pada token auto-cancel + refund (`SUPERSEDED`) |
| **Post (expire/cancel)** | Refund penuh ke `offer.buyer_id` (hardcoded, INV-005) |

**Validasi per langkah**

| Langkah | Validasi | Gagal → kode |
|---|---|---|
| `make_offer` | `predecessor != nft_token().owner_id` | `FORBIDDEN_SELF_BUY` |
| `make_offer` | Belum ada offer aktif buyer ini | `CONFLICT_OFFER_EXISTS` |
| `make_offer` | `attached_deposit == amount` (exact) & ≥ 0.01 Ⓝ | `CHAIN_INSUFFICIENT_DEPOSIT` / `INVALID_PRICE` |
| `accept_offer` | `predecessor == owner` & belum expire | `CHAIN_REVERT` |
| `cancel_offer` | `predecessor == offer.buyer_id` | Revert |

**Skema method**

```json
// make_offer(nft_contract_id, token_id, expires_at) — payable, deposit = amount
// expires_at = u64 NANOSECONDI (selaras env::block_timestamp() — INV-010); FE tampil ISO-8601
{ "nft_contract_id": "nearsea-open.testnet", "token_id": "101",
  "expires_at": 1799016000000000000 }

// accept_offer(nft_contract_id, token_id, buyer_id) — 1 yocto
{ "nft_contract_id": "nearsea-open.testnet", "token_id": "101",
  "buyer_id": "bob.testnet" }

// return (view): get_offers(nft_contract_id, token_id)
{ "offers": [ { "buyer_id": "bob.testnet",
    "amount_yocto": "900000000000000000000000",
    "expires_at": 1799016000000000000 } ] }
```

**State machine (offer)**: `ACTIVE → ACCEPTED | CANCELLED | EXPIRED | SUPERSEDED`.

**Edge case numerik**

| Input | Perilaku |
|---|---|
| `amount = 0.01 Ⓝ` | Diterima (inklusif) |
| `amount < 0.01 Ⓝ` | Tolak (INV-030) |
| Offer kedua buyer sama | Tolak (INV-024) |
| Offer pada token tak di-list | Sah (AC-OFFER-8) |
| Accept setelah expire | Tolak + refund lazy |
| Offer lain saat 1 di-accept | Auto-cancel + refund (`SUPERSEDED`) |

**Event / Gas / Storage**

| Aksi | Event | Gas | Storage |
|---|---|---|---|
| `make_offer` | `market_offer` | ~5 PROPOSED | entry `Offer` + escrow bookkeeping |
| `cancel_offer` | `market_offer_cancel` | ~5–10 PROPOSED | ditarik via `storage_withdraw` |
| `accept_offer` | `market_offer_accept` | <150 PROPOSED | — |
| expiry (lazy) | `market_offer_expire` | — | — |

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `offer.make` | "Make offer" |
| `offer.amount` | "Offer amount (Ⓝ)" |
| `offer.duration` | "Duration (default 7 days)" |
| `offer.accept` | "Accept offer" |
| `offer.cancel` | "Cancel offer" |
| `offer.expiresAt` | "Expires {time}" |
| `offer.success` | "Offer placed" |
| `errors.CONFLICT_OFFER_EXISTS` | "You already have an active offer on this item" |

### PRIVATE LISTING — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre** | Seller = owner; `allowed_buyer` diisi alamat valid |
| **Post** | Listing `ACTIVE` (private); hanya `allowed_buyer` bisa `buy` (INV-026); tidak di-grid publik |

**Validasi per langkah**

| Langkah | Validasi | Gagal → |
|---|---|---|
| 1 | `allowed_buyer` format AccountId valid | Field error |
| 3 | Buyer ≠ `allowed_buyer` | `FORBIDDEN_BUYER` |
| 4 | Buyer = `allowed_buyer` | Settlement normal |

**Skema method**: `list_nft_for_sale{..., allowed_buyer: "alice.testnet"}`; `get_sale` mengembalikan `allowed_buyer`.

**State machine**: `(none) → ACTIVE(private) → SOLD | CANCELLED | STALE`.

**Edge case**

| Kasus | Perilaku |
|---|---|
| Buyer lain mencoba buy | Ditolak kontrak |
| `allowed_buyer` = seller sendiri | Diperlakukan self-buy → tolak (INV-023) |
| Private listing di discovery | Tidak di-grid publik; tampil khusus |

**Event / Gas / Storage**

| Aksi | Event | Gas | Storage |
|---|---|---|---|
| `list_nft_for_sale` | `market_list` (`allowed_buyer` ≠ null) | **≈10–20 total** PROPOSED | entry `Sale` |

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `private.toggle` | "Private listing" |
| `private.buyerAddress` | "Allowed buyer address" |
| `private.badge` | "Private" |
| `private.onlyBuyer` | "Only {address} can buy this listing" |
| `errors.FORBIDDEN_BUYER` | "This listing is reserved for a specific buyer" |

### BUNDLE — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre** | 1..10 token milik seller; approval tiap token valid; receiver royalti unik hasil merge ≤ 10 (INV-021/027) |
| **Post** | Semua NFT pindah ke buyer; royalti per token dijumlahkan (merge per receiver); fee 2% dipotong sekali; `market_sale` |
| **Post (pre-validasi gagal)** | Abort bersih, **nol transfer**, refund penuh (INV-025) |
| **Post (residual mid-loop)** | Status `PARTIAL` + `market_bundle_partial` + refund dana belum terpakai + kompensasi G7 |

**Validasi per langkah**

| Langkah | Validasi | Gagal → |
|---|---|---|
| `create_bundle` | `1 ≤ items ≤ 10` | Tolak ("maks 10 token") |
| `create_bundle` | Tiap token milik seller + approval valid | Tolak |
| `create_bundle` | Receiver unik merge ≤ 10 | Tolak |
| `create_bundle` | Token ∉ bundle lain / tidak di-list terpisah | Tolak (INV-028) |
| `buy_bundle` | Pre-validasi SEMUA item sebelum transfer pertama | Abort + refund (INV-025) |
| `buy_bundle` | `Σpayout ≤ harga − fee`; sisa ≤ 1 yocto | Tolak + refund |

**Skema method**

```json
// create_bundle(items[], price) — payable (storage)
{ "items": [ { "nft_contract_id": "nearsea-open.testnet", "token_id": "101", "approval_id": 7 },
             { "nft_contract_id": "nearsea-open.testnet", "token_id": "102", "approval_id": 8 } ],
  "price": "10000000000000000000000000" }

// buy_bundle(bundle_id) — payable, deposit ≥ harga bundle
{ "bundle_id": 1 }

// return (view): get_bundle(bundle_id)
{ "seller": "alice.testnet", "price_yocto": "10000000000000000000000000",
  "status": "ACTIVE", "items": [ { "nft_contract_id": "nearsea-open.testnet", "token_id": "101" } ] }
```

**State machine (bundle)**: `ACTIVE → SOLD | CANCELLED | PRE_VALIDATE_FAILED | PARTIAL`.

**Edge case numerik**

| Input | Perilaku |
|---|---|
| `items = 10` | Diterima (inklusif) |
| `items = 11` | Tolak (INV-021) |
| Receiver unik = 10 | Diterima |
| Receiver unik = 11 | Tolak saat `create_bundle` |
| Salah satu token stale | Seluruh bundle tak bisa dibeli (INV-028) |

**Event / Gas / Storage**

| Aksi | Event | Gas | Storage |
|---|---|---|---|
| `create_bundle` | ⏳ open-by-design | PROPOSED | entry `Bundle` + membership item |
| `buy_bundle` | `market_sale` / `market_bundle_partial` | wajib diukur (risiko utama) | — |
| `cancel_bundle` | ⏳ open-by-design (kandidat `market_delist` per token) | ~5–10 PROPOSED | ditarik via `storage_withdraw` |

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `bundle.create` | "Sell as bundle" |
| `bundle.select` | "Select up to 10 items" |
| `bundle.price` | "Bundle price (Ⓝ)" |
| `bundle.buy` | "Buy bundle" |
| `bundle.itemCount` | "{count} items" |
| `errors.CONFLICT_BUNDLE_TOO_MANY` | "Maximum 10 items per bundle" |
| `errors.CONFLICT_BUNDLE_ITEM_INVALID` | "This bundle can't be bought — one of the items is no longer valid" |

### STALE LISTING — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre** | Listing `ACTIVE`; ownership berubah di luar market |
| **Post** | Listing `STALE`; disembunyikan dari discovery; tidak bisa dibeli; `market_stale_detected` |
| **Post (cleanup)** | `remove_stale_listing` (permissionless, mismatch terbukti on-chain) → `market_delist` |

**Validasi per langkah**

| Langkah | Validasi | Gagal → |
|---|---|---|
| deteksi | `nft_token().owner_id != sale.owner_id` | (tetap ACTIVE) |
| `buy`/`buy_bundle` | mismatch → tolak | `CONFLICT_STALE` |
| `remove_stale_listing` | Mismatch terbukti on-chain | Revert (bukan owner tak bisa hapus listing valid) |
| saat paused | `remove_stale_listing` diizinkan (INV-022) | — |

**Skema method**: `remove_stale_listing(nft_contract_id, token_id)` — 1 yocto.

**State machine**: `ACTIVE → STALE → (removed)`.

**Edge case**

| Kasus | Perilaku |
|---|---|
| Listing stale tampil sampai view berikutnya | FE wajib re-verify sebelum sign (SEC-ORDER-003) |
| Siapa pun cleanup | Boleh, hanya bila mismatch terbukti |
| Token bundle dipindah | Seluruh bundle tak bisa dibeli (INV-028) |

**Event / Gas / Storage**

| Aksi | Event | Gas | Storage |
|---|---|---|---|
| deteksi | `market_stale_detected` | — | — |
| `remove_stale_listing` | `market_delist` | ~5 PROPOSED | — |

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `stale.badge` | "No longer available" |
| `stale.notice` | "This listing is no longer valid" |
| `errors.CONFLICT_STALE` | "This listing is no longer valid" |

### SEARCH & FILTER — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre** | Halaman discovery terbuka; query params (`q`, `sort`, `filter`, `cursor`) valid |
| **Post** | Hasil terurut deterministik + cursor; URL shareable |
| **Post (gagal)** | Inline error + retry; UI tetap hidup |

**Validasi per langkah**

| Langkah | Validasi | Gagal → |
|---|---|---|
| 1 Search | `q` min 2 char; verifikasi hasil via view call | Empty state (bukan error) |
| 2 Filter | Rentang harga valid; `status ∈ {active,sold,cancelled,stale}` | Clamp/normalisasi |
| 2 Sort | `newest` (default) / `price_asc` / `price_desc` | Fallback default |
| Pagination | `limit` default 20, maks 100 (clamp, bukan error) | Clamp |

**Skema (query params)**

```json
{ "q": "genesis", "sort": "price_asc", "minPrice": "10000000000000000000000",
  "maxPrice": "5000000000000000000000000", "cursor": "opaque", "limit": 20 }
```

**State machine (hasil)**: `idle → loading → success | empty | error`.

**Edge case**

| Kasus | Perilaku |
|---|---|
| `q` 1 char | Tidak query (min 2) |
| `limit` > 100 | Dipangkas ke 100 |
| Koleksi tanpa listing | Empty state |
| Trait filter | Fase 2/3 (bukan MVP) |

**Event / Gas / Storage**: tidak ada (read-only).

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `search.placeholder` | "Search collections and items" |
| `filter.price` | "Price range" |
| `sort.newest` | "Recently listed" |
| `sort.priceAsc` | "Price: low to high" |
| `sort.priceDesc` | "Price: high to low" |
| `search.empty` | "No results found" |
| `search.tooShort` | "Type at least 2 characters" |

### VIEW PROFILE & ACTIVITY — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre** | Akun NEAR valid (named/implicit/0x) |
| **Post** | Tab Owned/Created/Offers/Activity terisi; cursor independen per tab |

**Validasi per langkah**

| Langkah | Validasi | Gagal → |
|---|---|---|
| 1 | Akun valid | Empty state "akun tidak ditemukan" |
| 2 | Data per tab | Empty state per tab |
| Created | Resolusi `creator_id` (fallback owner awal mint) | "Unknown creator" (bukan error) |

**Skema (view/API)**

```json
// GET /api/v1/accounts/:id/activity → {items[], nextCursor}
{ "items": [ { "type": "sale", "contract_account_id": "nearsea-open.testnet",
  "token_id": "101", "from": "alice.testnet", "to": "bob.testnet",
  "price_yocto": "1000000000000000000000000", "block_height": 123456789,
  "timestamp": "2026-10-02T11:00:00Z" } ], "nextCursor": null }
```

**State machine**: `loading → success | empty | error`.

**Edge case**

| Kasus | Perilaku |
|---|---|
| Akun implisit (64-hex) | Tampil apa adanya (full address) |
| Belum ada aktivitas | Empty state per tab |
| Riwayat tak tersedia (RPC/NearBlocks gagal) | Empty state, bukan error |

**Event / Gas / Storage**: tidak ada (read-only).

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `profile.tab.owned` | "Owned" |
| `profile.tab.created` | "Created" |
| `profile.tab.offers` | "Offers" |
| `profile.tab.activity` | "Activity" |
| `profile.empty` | "Nothing here yet" |
| `profile.notFound` | "Account not found" |
| `profile.share` | "Share" |

### PROFILE CUSTOM — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre** | Wallet terhubung; sesi scope `profile` (NEP-413/fallback) |
| **Post** | Profil publik menampilkan alias/bio/avatar; alamat & aset tetap on-chain |

**Validasi per langkah**

| Langkah | Validasi | Gagal → kode |
|---|---|---|
| 2 | Signature valid atas challenge kanonik | `AUTH_SIGNATURE` |
| 3 | `alias ≤ 32` char (NFC, trim) | `INVALID_ALIAS` |
| 3 | `bio ≤ 280` char (NFC, trim, tanpa karakter kontrol) | `INVALID_BIO` |
| 3 | `avatar_url` https/ipfs di gateway allowlist | `INVALID_AVATAR` |
| 3 | Scope `profile` | `FORBIDDEN_SCOPE` |

**Skema method (API)**

```json
// PATCH /api/v1/accounts/me/profile
{ "alias": "Alice", "bio": "NEAR NFT collector",
  "avatar_url": "https://ipfs.io/ipfs/bafy..." }
```

**State machine (form)**: `idle → signing → saving → saved | error`.

**Edge case**

| Kasus | Perilaku |
|---|---|
| Alias 33 char | 400 `INVALID_ALIAS`, data tak tersimpan |
| Bio 281 char | 400 `INVALID_BIO` |
| Avatar di luar allowlist | 400 `INVALID_AVATAR` |
| Reset field (null) | Kembali ke identicon + alamat |
| Tulis ulang tanpa signature | Ditolak |

**Event / Gas / Storage**: tidak ada (off-chain DB).

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `settings.profile.title` | "Profile" |
| `settings.profile.alias` | "Display name" |
| `settings.profile.bio` | "Bio" |
| `settings.profile.avatar` | "Avatar URL" |
| `settings.profile.save` | "Save" |
| `settings.profile.saved` | "Profile updated" |
| `errors.INVALID_ALIAS` | "Display name must be 32 characters or fewer" |
| `errors.INVALID_BIO` | "Bio must be 280 characters or fewer" |
| `errors.INVALID_AVATAR` | "Avatar URL is not allowed" |

### NOTIFICATIONS (in-app) — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre** | Wallet terhubung; tab aktif; interval polling 30–60 dtk |
| **Post** | Daftar notifikasi ter-diff dari event; read-state di localStorage per akun; unread badge akurat |
| **Post (gagal)** | Badge berhenti update + retry backoff; UI tetap hidup |

**Validasi per langkah**

| Langkah | Validasi | Gagal → |
|---|---|---|
| 1 Polling | NearBlocks + view `get_offers` | Backoff (AC-NOTIF-3) |
| 2 Diff | Dedup by `(receipt_id, event_index)` — bukan `tx_hash` | Tanpa duplikat |
| 3 Read-state | Union ke readSet; prune 500 id terbaru / 30 hari | Nilai invalid → anggap kosong |

**Skema payload (client)**

```json
{ "id": "9f8E3a1b...:2",
  "event_id": { "receipt_id": "9f8E3a1b...", "event_index": 2 },
  "type": "offer_received", "account_id": "alice.testnet",
  "created_at": "2026-10-02T12:00:00Z", "block_height": 123456789,
  "read": false,
  "payload": { "contract_account_id": "nearsea-open.testnet", "token_id": "101",
    "counterparty": "bob.testnet", "amount_yocto": "900000000000000000000000" },
  "destination": "/token/nearsea-open.testnet/101" }
```

**State machine (bell)**: `idle-empty | idle-read | unread | loading | stale | error`.

**Edge case**

| Kasus | Perilaku |
|---|---|
| Event sama dari polling ulang | Tidak duplikat (idempoten) |
| Offer expire | Dari cek expiry lokal (tanpa tx) |
| NearBlocks 429 | Badge berhenti update, UI hidup |
| Dua tab | Satu polling; tab lain via `storage` event |

**Event / Gas / Storage**: mengonsumsi event on-chain; tidak ada gas (read-only). Read-state di localStorage.

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `notif.title` | "Notifications" |
| `notif.empty` | "No notifications yet" |
| `notif.offerReceived` | "New offer received" |
| `notif.offerAccepted` | "Your offer was accepted" |
| `notif.offerCancelled` | "Offer cancelled" |
| `notif.offerExpired` | "Your offer expired — funds returned" |
| `notif.tokenSold` | "Your item sold" |
| `notif.mintSuccess` | "Mint successful" |
| `notif.markAllRead` | "Mark all as read" |
| `notif.staleRefresh` | "Couldn't refresh — showing older items" |

### ADMIN / MODERATION — Spec Engineering

**Preconditions / Postconditions**

| | Kondisi |
|---|---|
| **Pre** | Akun ∈ allowlist admin DB; scope `admin`; step-up signature untuk aksi destruktif (SEC-ADMIN-003) |
| **Post** | Report resolved; target disembunyikan dari discovery; baris `admin_audit` append-only |
| **Post (gagal)** | Aksi ditolak; tidak ada perubahan state; tidak ada audit sukses |

**Validasi per langkah**

| Langkah | Validasi | Gagal → kode |
|---|---|---|
| 3 Login | Akun ∈ allowlist | `FORBIDDEN_ADMIN` |
| 3 | Scope `admin` | `FORBIDDEN_SCOPE` |
| 4 Aksi destruktif | `step_up` signature valid, nonce belum dipakai, belum expire, action/target cocok | `INVALID_*` / `AUTH_*` |
| 4 | Session ≤ 15 menit (tanpa silent refresh) | `AUTH_SESSION_EXPIRED` |
| 5 | Takedown hanya layer tampilan | — (chain tak tersentuh) |

**Skema method (API)**

```json
// PATCH /api/v1/admin/reports/:id — aksi step-up = kanonik (A6 ronde 16); expires = ISO-8601
{ "action": "report_decide", "decision": "hide_target", "reason": "fraud",
  "step_up": { "action": "report_decide", "target": "nearsea-open.testnet",
    "nonce": "a1b2…", "expires": "2026-10-09T12:00:00Z", "signature": "ed25519:…" } }

// PATCH /api/v1/admin/collections/:id/verified
{ "verified": true, "reason": "official collection",
  "step_up": { "action": "verified_set", "target": "nearsea-genesis.testnet",
    "nonce": "c3d4…", "expires": "2026-10-09T12:00:00Z", "signature": "ed25519:…" } }
```

**State machine (report)**: `open → resolved (hidden | ignored)`.

**Edge case**

| Kasus | Perilaku |
|---|---|
| Admin di luar allowlist | 403 `FORBIDDEN_ADMIN` |
| Step-up nonce dipakai ulang | 401 `AUTH_NONCE_USED` |
| Step-up expired | 401 `AUTH_NONCE_EXPIRED` |
| Report sudah diputuskan | 409 `CONFLICT_ALREADY_DECIDED` |
| > 30 aksi/menit | 429 `RATE_LIMITED` |

**Event / Gas / Storage**: tidak ada event on-chain; audit di DB (`admin_audit`, append-only). Tidak menyentuh chain.

**UI copy (EN)**

| Kunci | Teks |
|---|---|
| `admin.title` | "Admin" |
| `admin.queue` | "Review queue" |
| `admin.action.hide` | "Hide from discovery" |
| `admin.action.ignore` | "Ignore" |
| `admin.action.verify` | "Mark verified" |
| `admin.reason` | "Reason" |
| `admin.stepUp` | "Confirm with your wallet" |
| `admin.success` | "Action applied" |
| `errors.FORBIDDEN_ADMIN` | "You don't have admin access" |
| `errors.CONFLICT_ALREADY_DECIDED` | "This report was already decided" |



