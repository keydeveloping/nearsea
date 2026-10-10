# Feature — Marketplace (List / Buy / Offer / Private & Bundle)

> Fitur inti. Kontrak + UI + settlement dalam satu dokumen agar agent fokus.

## Objective

Jual-beli NFT: fixed-price listing, offers, private listing, dan bundle (auction = fase 2). Settlement + royalti on-chain, model approval non-custodial (ADR-007).

## Preconditions

- Seller: pemilik token; kontrak NFT mendukung NEP-178; storage deposit market terpenuhi (NEP-145); **harga min listing 0.01 Ⓝ** (ronde 4).
- Buyer: saldo cukup; token tidak milik sendiri (INV-023); deposit = harga — **kelebihan deposit (harga berubah saat signing) dikembalikan ke buyer via resolve**.

## Flow — List (DIPUTUSKAN ronde 3 / ADR-002: 2 transaksi + dual verification)

1. Seller klik Sell → set harga (Ⓝ).
2. **Tx 1** — Seller `nft_approve(market, msg = harga)` di NFT contract (deposit: 1 yocto + biaya storage approval).
3. **Tx 2** — Seller `list_nft_for_sale { nft_contract_id, token_id, approval_id, price }` di market (wajib storage deposit terpenuhi).
4. Market melakukan **dual verification** (2 cross-contract call): `nft_token` (pemilik benar) + `nft_is_approved` (approval valid) → callback `process_listing` → listing aktif.

Referensi: RESEARCH.md 10.4. Catatan UX: dua kali signing — modal frontend wajib menjelaskan langkah 1/2.

## Flow — Buy

> Draft lengkap di docs/02-product-requirements.md (BUY NFT). Settlement: `buy` → `nft_transfer_payout` → `resolve_purchase` (refund otomatis jika gagal).

## Flow — Offer (MVP, ronde 1)

1. Buyer `make_offer(contract, token_id)` dengan attach Ⓝ = jumlah tawaran → escrow di market contract.
2. Seller melihat offer → `accept_offer` → market memanggil `nft_transfer_payout` (settlement + royalti).
3. Dana escrow terdistribusi: harga − fee 2% → seller + royalti; NFT pindah ke buyer.
4. Offer expired / dibatalkan → refund penuh ke buyer.
5. **Perilaku**: saat sebuah offer di-accept, offer AKTIF LAIN pada token yang sama auto-cancel + refund (didefinisikan di sini — dirujuk acceptance-criteria).

Parameter offer (DIPUTUSKAN ronde 4 & 6):
- Expire default **7 hari**; buyer bisa pilih durasi custom.
- Maks **1 offer aktif per buyer per token** (cancel lalu buat ulang diperbolehkan).
- Harga min offer **0.01 Ⓝ**.
- Counter-offer **tidak** di MVP — nego via private listing ke buyer tersebut.

## Flow — Private Listing & Bundle (MVP, ronde 1 + 6)

**Private listing**: seller menandai listing dengan `allowed_buyer` tertentu → hanya alamat itu yang bisa membeli (divalidasi di `buy`).

**Bundle** (gaya OpenSea — DIPUTUSKAN ronde 6; **maks 10 token** — INV-021):
1. Seller pilih maks 10 token miliknya → set **satu harga untuk seluruh bundle**.
2. Approve semua token ke market (NEP-178 per token; approval_id tercatat per token).
3. Buyer `buy_bundle` dengan deposit = harga bundle.
4. Settlement: pre-validasi semua item (INV-025) → loop `nft_transfer_payout` per token → **payout royalti per token dijumlahkan, di-merge per receiver (maks 10 receiver unik — ditolak saat create_bundle bila lebih)** → validasi total ≤ harga → fee 2% dipotong sekali → semua NFT pindah ke buyer.
5. Gagal pre-validasi → abort bersih + refund; kegagalan residual mid-loop → status PARTIAL + kompensasi G7 (order-protocol-security.md §6).

## Flow — Update Price

```text
1. Seller buka listing miliknya → Edit Price → set harga baru (Ⓝ).
2. FE re-verify (SEC-ORDER-003): get_sale → pemanggil = seller, listing masih ACTIVE, bukan stale.
3. Seller sign update_price { nft_contract_id, token_id, new_price } — attach 1 yocto (assert_one_yocto).
4. Kontrak: predecessor == sale.owner_id; new_price ≥ 0.01 Ⓝ (INV-030); sale_conditions.price diganti in-place.
5. Event market_update_price (old_price_yocto, new_price_yocto) — skema [webhooks.md](../api/webhooks.md).
6. FE invalidate ['listing',*], ['listings',*] (frontend-architecture.md §4).
```

- `approval_id` **tidak** berubah — hanya state harga milik market yang diganti; approval NEP-178 di NFT contract tetap valid.
- Bundle memakai **satu harga** di `bundles.price_yocto` (bukan per item) → `update_price` per item bundle tidak berlaku (INV-028); jalur ubah harga bundle = `cancel_bundle` + `create_bundle`. Method khusus edit-harga-bundle: ⏳ open-by-design.

## Flow — Remove Sale (Cancel Listing)

```text
1. Seller buka listing miliknya → Cancel.
2. FE re-verify get_sale (pemanggil = seller, status ACTIVE).
3. Seller sign remove_sale { nft_contract_id, token_id } — attach 1 yocto.
4. Kontrak: predecessor == sale.owner_id; hapus entry Sale; storage seller dibebaskan.
5. Event market_delist. NFT TIDAK pernah berpindah wallet.
6. Storage deposit seller dapat ditarik kembali via storage_withdraw (NEP-145).
```

- `remove_sale` tetap diizinkan saat kontrak `paused` (jalur pengembalian aset/refund — INV-022).
- **Market tidak mencabut approval** (koreksi ronde 22 — lihat [contracts/market.md](../contracts/market.md) §2a):
  NEP-178 hanya punya `nft_revoke`/`nft_revoke_all`, dan keduanya **owner-only** — tidak ada method
  untuk approved account. Approval yang tersisa tidak berbahaya: tanpa entry `Sale` market tidak punya
  alasan memindahkan token, dan transfer berikutnya oleh owner otomatis mencabut semua approval.
- Re-list **wajib** `nft_revoke` (atau `nft_revoke_all`) dulu, baru `nft_approve` baru → `approval_id`
  baru. `nft_approve` pada akun yang sudah di-approve panic (`AccountAlreadyApprovedError`), jadi
  langkah revoke itu wajib, bukan opsional.

## Flow — Cancel Offer

```text
1. Buyer buka offer miliknya → Cancel.
2. Buyer sign cancel_offer { nft_contract_id, token_id } — attach 1 yocto.
3. Kontrak: predecessor == offer.buyer_id; hapus entry Offer; refund penuh escrow → offer.buyer_id (hardcoded, INV-005).
4. Event market_offer_cancel.
5. Storage deposit offer TIDAK otomatis kembali — ditarik buyer via storage_withdraw (mencegah gas-DoS refund massal).
```

- `cancel_offer` tetap diizinkan saat `paused` (INV-022).
- Offer yang **expired** dapat di-cancel siapa pun lewat jalur lazy yang sama (lihat § Expiry Sweep); refund tetap ke `buyer_id`.

## Flow — Cancel Bundle

```text
1. Seller buka bundle → Cancel.
2. Seller sign cancel_bundle { bundle_id } — attach 1 yocto.
3. Kontrak: predecessor == bundle.seller; status bundle → CANCELLED; membership dihapus
   (market **tidak** mencabut approval — NEP-178 owner-only, §2a [contracts/market.md](../contracts/market.md))
4. Semua token bebas di-list/di-offer terpisah kembali (INV-028 berhenti berlaku).
5. `cancel_bundle` tetap diizinkan saat paused (INV-022).
```

- Event agregat bundle untuk cancel: ⏳ open-by-design — belum ada nama event bundle di daftar kanonik [webhooks.md](../api/webhooks.md); kandidat: `market_delist` per token.

## Auction — TIDAK di MVP (ronde 1) → fase 2

Desain nanti mengacu pola auction contract (RESEARCH.md, tutorial mastering-near 3.x): bid escrow, bidder lama refund langsung, end + claim setelah deadline.

## UI

- Tombol konteks di Detail NFT: **Buy** (ada listing) / **Make Offer** (selalu, jika bukan pemilik) / **Sell** (jika pemilik) / **Cancel / Edit Price** (listing milik sendiri).
- Modal List 2-langkah dengan progress (approve → list); breakdown fee/royalti di modal Buy.
- Badge listing stale disembunyikan dari discovery; di halaman token tampil "no longer available" jika stale.

## Contract API (draft — nama kanonik; dipakai konsisten di semua dokumen)

```text
market:  list_nft_for_sale, remove_sale, update_price, remove_stale_listing,
         make_offer, cancel_offer, accept_offer, buy (settle langsung listing aktif),
         create_bundle, buy_bundle, cancel_bundle,
         storage_deposit / storage_withdraw (NEP-145), nft_on_approve
views:   get_sale(s), get_supply_sales, get_offer(s), get_bundle(s), is_stale?, get_launchpad?
```

Aturan kanonik: `make_offer` = buat offer (escrow); `offer` TIDAK dipakai sebagai nama method (hanya istilah umum); `buy` = beli listing aktif; `accept_offer` = seller menerima offer.

## Skema Argumen per Method

> Tipe mengikuti near-sdk: `AccountId`/`String` → JSON `string`, `u128`/`U128` → JSON **string** yoctoNEAR, `u64`/`u32` → JSON number, `Option<T>` → nullable. Field `#[payable]` = method menerima `attached_deposit`. Nama field di bawah = **kontrak target** (kanonik); event mengikuti [webhooks.md](../api/webhooks.md).

### `list_nft_for_sale` — payable, assert_one_yocto? **TIDAK** (lihat catatan)

```rust
#[payable]
pub fn list_nft_for_sale(
    &mut self,
    nft_contract_id: AccountId,   // koleksi NFT
    token_id: String,             // token di koleksi tsb
    approval_id: Option<u64>,     // approval_id dari nft_approve (NEP-178); None = approval tanpa id
    price: U128,                  // harga listing (yoctoNEAR), ≥ MIN_PRICE
    allowed_buyer: Option<AccountId>, // private listing; None = publik
)
```

| Field | Tipe | Wajib | Deskripsi |
|---|---|---|---|
| `nft_contract_id` | string (AccountId) | ya | Kontrak NFT pemilik token. |
| `token_id` | string | ya | ID token; unik per koleksi. |
| `approval_id` | number\|null | tidak | ID approval dari `nft_approve(market, msg)`. Di-echo kontrak NFT; `null` bila approval tanpa id. |
| `price` | string (U128) | ya | Harga yoctoNEAR; ditolak `< 0.01 Ⓝ` (INV-030). |
| `allowed_buyer` | string\|null | tidak | Bila diisi → private listing; hanya alamat ini bisa `buy` (INV-026). |

- Deposit: **bukan 1 yocto** — `list_nft_for_sale` menerima **storage deposit NEP-145** untuk entry `Sale` (map tumbuh → user bayar, INV-020). Jumlah pasti per-entry ⏳ open-by-design (mengikuti `storage_usage` near-sdk; FE memanggil `storage_deposit` bila kurang). `assert_one_yocto` **tidak** berlaku di sini karena deposit variabel.
- Dual verification (2 view call): `nft_token(token_id).owner_id == predecessor` **dan** `nft_is_approved(token_id, market, approval_id)` → callback `process_listing` (`#[private]`, INV-013).
- Event: `market_list`. Gas: lihat tabel di bawah.

### `remove_sale` — assert_one_yocto

```rust
#[payable]
pub fn remove_sale(&mut self, nft_contract_id: AccountId, token_id: String)
```

| Field | Tipe | Wajib | Deskripsi |
|---|---|---|---|
| `nft_contract_id` | string | ya | Kontrak NFT. |
| `token_id` | string | ya | Token yang listing-nya dibatalkan. |

- Deposit **1 yocto** (assert_one_yocto); predecessor == `sale.owner_id`. Event: `market_delist`.
  Market **tidak** mencabut approval — NEP-178 tidak punya revoke untuk approved account
  ([contracts/market.md](../contracts/market.md) §2a).

### `update_price` — assert_one_yocto

```rust
#[payable]
pub fn update_price(&mut self, nft_contract_id: AccountId, token_id: String, new_price: U128)
```

| Field | Tipe | Wajib | Deskripsi |
|---|---|---|---|
| `nft_contract_id` | string | ya | Kontrak NFT. |
| `token_id` | string | ya | Token listing milik pemanggil. |
| `new_price` | string (U128) | ya | Harga baru; ≥ 0.01 Ⓝ (INV-030). |

- Deposit **1 yocto**; predecessor == `sale.owner_id`. Event: `market_update_price`.

### `remove_stale_listing` — assert_one_yocto

```rust
#[payable]
pub fn remove_stale_listing(&mut self, nft_contract_id: AccountId, token_id: String)
```

| Field | Tipe | Wajib | Deskripsi |
|---|---|---|---|
| `nft_contract_id` | string | ya | Kontrak NFT. |
| `token_id` | string | ya | Token listing yang terbukti stale. |

- Deposit **1 yocto**; kontrak **wajib** memverifikasi stale on-chain (`nft_token().owner_id != sale.owner_id`) sebelum hapus — bukan percaya klaim pemanggil. Siapa pun boleh memanggil (permissionless cleanup) **hanya bila** mismatch terbukti; caller yang bukan owner tidak bisa memakai method ini untuk menghapus listing valid. Event: `market_delist` (plus `market_stale_detected` saat deteksi). Tetap diizinkan saat `paused` (INV-022).

### `make_offer` — payable, escrow exact-match

```rust
#[payable]
pub fn make_offer(
    &mut self,
    nft_contract_id: AccountId,
    token_id: String,
    expires_at: Option<u64>,   // Unix **nanodetik** (u64, sama dengan env::block_timestamp()); None = now + 7 hari
)
```

| Field | Tipe | Wajib | Deskripsi |
|---|---|---|---|
| `nft_contract_id` | string | ya | Kontrak NFT (token boleh ter-list maupun tidak). |
| `token_id` | string | ya | Token yang di-offer. |
| `expires_at` | number\|null | tidak | Unix epoch **nanodetik (u64)** — bukan ms; `null` → now + 7 hari. FE mengonversi ke ISO-8601 hanya untuk tampilan; satuan ns agar INV-010 dapat membandingkan langsung dengan `env::block_timestamp()`. |

- Deposit = jumlah tawaran (escrow **exact match**, INV-006) dan harus ≥ 0.01 Ⓝ (INV-030). Maks **1 offer aktif per buyer per token** (INV-024). Self-offer ditolak (INV-023).
- Storage deposit offer dibayar buyer terpisah (NEP-145) — tidak otomatis kembali saat refund (ditarik via `storage_withdraw`).
- Event: `market_offer`.

### `cancel_offer` — assert_one_yocto

```rust
#[payable]
pub fn cancel_offer(&mut self, nft_contract_id: AccountId, token_id: String)
```

| Field | Tipe | Wajib | Deskripsi |
|---|---|---|---|
| `nft_contract_id` | string | ya | Kontrak NFT. |
| `token_id` | string | ya | Token offer milik pemanggil. |

- Deposit **1 yocto**; predecessor == `offer.buyer_id`; refund penuh ke `buyer_id`. Event: `market_offer_cancel`.

### `accept_offer` — assert_one_yocto

```rust
#[payable]
pub fn accept_offer(&mut self, nft_contract_id: AccountId, token_id: String, buyer_id: AccountId)
```

| Field | Tipe | Wajib | Deskripsi |
|---|---|---|---|
| `nft_contract_id` | string | ya | Kontrak NFT. |
| `token_id` | string | ya | Token yang di-accept offer-nya. |
| `buyer_id` | string | ya | Pembuat offer (kunci offer = token + buyer). |

- Deposit **1 yocto**; predecessor == owner token **saat itu** & `block_timestamp < expires_at` (INV-010). Offer lain pada token sama auto-cancel + refund (SUPERSEDED). Event: `market_offer_accept`.

### `buy` — payable, deposit = harga

```rust
#[payable]
pub fn buy(&mut self, nft_contract_id: AccountId, token_id: String)
```

| Field | Tipe | Wajib | Deskripsi |
|---|---|---|---|
| `nft_contract_id` | string | ya | Kontrak NFT. |
| `token_id` | string | ya | Token dengan listing aktif. |

- Deposit ≥ harga; kelebihan (harga berubah saat signing) dikembalikan via `resolve_purchase`. Buyer ≠ seller (INV-023); private listing → hanya `allowed_buyer` (INV-026). Jalur: `nft_transfer_payout` → `resolve_purchase`. Event: `market_sale`.

### `create_bundle` — payable (storage), assert_one_yocto? **TIDAK**

```rust
#[payable]
pub fn create_bundle(
    &mut self,
    items: Vec<BundleItem>,   // 1..=10 token
    price: U128,              // SATU harga untuk seluruh bundle
)
// BundleItem { nft_contract_id: AccountId, token_id: String, approval_id: Option<u64> }
```

| Field | Tipe | Wajib | Deskripsi |
|---|---|---|---|
| `items` | array<object> | ya | 1..10 item; tiap item punya `nft_contract_id`, `token_id`, `approval_id`. |
| `items[].nft_contract_id` | string | ya | Kontrak NFT item. |
| `items[].token_id` | string | ya | Token item. |
| `items[].approval_id` | number\|null | tidak | Approval NEP-178 item. |
| `price` | string (U128) | ya | Satu harga bundle; ≥ 0.01 Ⓝ. |

- Deposit = storage NEP-145 untuk entry bundle + membership item (bukan 1 yocto). Pre-validasi saat create: tiap token milik seller + approval valid; **receiver royalti unik hasil merge ≤ 10** (jika > 10 → tolak di sini, INV-021/027). Event agregat create bundle ⏳ open-by-design.

### `buy_bundle` — payable, deposit = harga bundle

```rust
#[payable]
pub fn buy_bundle(&mut self, bundle_id: u64)
```

| Field | Tipe | Wajib | Deskripsi |
|---|---|---|---|
| `bundle_id` | number (u64) | ya | ID bundle (counter kontrak). |

- Deposit ≥ harga bundle. Pre-validasi SEMUA item sebelum transfer pertama (INV-025); loop `nft_transfer_payout` per token → merge royalti per receiver → fee 2% dipotong sekali. Event: `market_sale` (atau `market_bundle_partial` bila residual failure).

### `cancel_bundle` — assert_one_yocto

```rust
#[payable]
pub fn cancel_bundle(&mut self, bundle_id: u64)
```

| Field | Tipe | Wajib | Deskripsi |
|---|---|---|---|
| `bundle_id` | number (u64) | ya | ID bundle milik pemanggil. |

- Deposit **1 yocto**; predecessor == `bundle.seller`; revoke approval per token.

### Method owner (bukan order)

| Method | Signature | Deposit | Catatan |
|---|---|---|---|
| `storage_deposit` | `storage_deposit(account_id: Option<AccountId>)` | Ⓝ sesuai kebutuhan | NEP-145; `account_id` null → pemanggil. |
| `storage_withdraw` | `storage_withdraw(amount: Option<U128>)` | 1 yocto | NEP-145; tetap diizinkan saat `paused`. |
| `nft_mint` | `nft_mint(...)` | harga fase + storage | Launchpad; exact-match deposit (INV-018). |
| `update_fee_bps` / `update_treasury` | `update_fee_bps(fee_bps: u16)` / `update_treasury(treasury)` | 1 yocto | Owner-only (MVP) / DAO (mainnet); `fee_bps ≤ MAX_FEE_BPS`; event `fee_update`/`treasury_update`. |
| `recover_stuck_purchase` | `recover_stuck_purchase(nft_contract_id, token_id)` | 1 yocto | **Siapa pun** (permissionless) setelah `RECOVERY_DELAY_BLOCKS`; refund penuh ke buyer (INV-031). |
| `pause` / `unpause` | `pause()` / `unpause()` | 1 yocto | Owner (MVP) / guardian `pause_callers` (mainnet). |

> **Tidak ada `withdraw_fees`.** Fee ditransfer langsung ke treasury saat settlement — market tidak pernah memegang dana fee ([contracts/market.md](../contracts/market.md) §4a, koreksi ronde 23).

## Event per Aksi (ringkas)

> Skema field lengkap dimiliki [webhooks.md](../api/webhooks.md) — tabel ini hanya memetakan aksi → event. `standard: "x-nearsea-market"`, `version: "1.0.0"`.

| Aksi / method | Event |
|---|---|
| `list_nft_for_sale` | `market_list` |
| `remove_sale` / `remove_stale_listing` | `market_delist` |
| `update_price` | `market_update_price` |
| `buy` | `market_sale` |
| `make_offer` | `market_offer` |
| `cancel_offer` | `market_offer_cancel` |
| `accept_offer` | `market_offer_accept` |
| Expiry sweep (lazy) | `market_offer_expire` |
| Deteksi stale | `market_stale_detected` |
| `buy_bundle` residual failure | `market_bundle_partial` |
| `create_bundle` / `buy_bundle` sukses / `cancel_bundle` | ⏳ open-by-design (belum ada nama event bundle kanonik) |

## Storage Deposit per Aksi (NEP-145)

> User membayar storage-nya sendiri; kontrak tidak pernah menanggung (INV-020). Angka pasti per-entry = hasil `storage_usage` near-sdk saat implementasi — kolom "Perkiraan" di bawah **PROPOSED**, bukan nilai bisnis terkunci.

| Aksi | Yang dibayar | Perkiraan |
|---|---|---|
| `list_nft_for_sale` | entry `Sale` (map `sales`) | ✅ aktif — bounds `min = storage_per_sale()` (500 byte, PROPOSED); deposit kurang → revert |
| `make_offer` | entry `Offer` + escrow bookkeeping | ⏳ open-by-design |
| `create_bundle` | entry `Bundle` + membership item | ⏳ open-by-design |
| `nft_mint` | storage token + metadata di NFT contract | dibayar pemicu mint (INV-019) |
| `storage_withdraw` | — (menarik sisa) | refund ke pemilik, bukan tarik paksa |

- Deposit **tidak** otomatis kembali saat refund/expire/cancel — pemilik menarik sendiri via `storage_withdraw` (mencegah gas-DoS refund massal, order-protocol-security.md §5).

## Anggaran Gas per Method (Tgas)

> Batas keras **300 Tgas/call** (FACT NEAR). Nilai bertanda PROPOSED = estimasi, diukur di test sandbox. Sumber lintas-kontrak: [system-architecture.md](../architecture/system-architecture.md) § Anggaran gas.

| Method | Gas (Tgas) | Catatan |
|---|---|---|
| `nft_approve` (NFT contract) | ~10–15 (PROPOSED) | Tx-1 listing; + storage approval. |
| `list_nft_for_sale` | **≈10–20 total** (PROPOSED): 2 view call ~5 + callback ~10 | Dual verification `nft_token` + `nft_is_approved`; callback `process_listing`. Angka total = SSOT lintas-dokumen (order-protocol-security §11, ADR-012). Konstanta: `GAS_FOR_NFT_VIEW`/`GAS_FOR_PROCESS_LISTING` ([contracts/market.md](../contracts/market.md) §6). |
| `remove_sale` / `update_price` | ~5 (PROPOSED) | Tulis state (remove: hapus entry + kembalikan storage; **tanpa** XCC — §2a). |
| `make_offer` | ~5 (PROPOSED) | Tulis state escrow. |
| `cancel_offer` | ~5–10 (PROPOSED) | Transfer refund. |
| `accept_offer` | ~15 (`nft_transfer_payout`) + 115 (`resolve_purchase`) | Sama seperti `buy`. |
| `buy` total (worst case) | < 150 (PROPOSED) | Masih jauh di bawah 300. |
| `buy_bundle` (10 token) | PROPOSED — **wajib diukur** | 10× `nft_transfer_payout` → jalur paling rawan gas (INV-021). |
| `remove_stale_listing` | ~5 (PROPOSED) | Hapus state. |

## Approval_id Lifecycle

```text
nft_approve(market, msg, approval_id=N)   →  approval aktif di NFT contract, id=N
        │
list_nft_for_sale(..., approval_id=N)     →  market mencatat Sale.approval_id = N
        │
        ├── buy / accept_offer  →  nft_transfer_payout memakai N; setelah transfer sukses
        │                          approval N INVALID (FACT NEP-178) → settle cek ulang tiap kali (INV-011)
        ├── remove_sale         →  entry Sale hilang; approval N TETAP ada (market tidak mencabut — §2a)
        └── update_price        →  approval TIDAK berubah (state harga di market saja)

remove_sale lalu list ulang  →  nft_revoke(market) dulu (approval lama masih ada),
                                baru nft_approve BARU → approval_id baru (M ≠ N)
```

- `approval_id` = `BIGINT` di DB ([data-model.md](../database/data-model.md)); `null` bila approval tanpa id.
- **Invalidasi pasca-transfer**: market tidak boleh settle dengan `approval_id` lama — cek ulang `nft_is_approved` setiap settle (INV-011/016).
- Satu `approval_id` per (token, market); revoke/transfer → invalid (signature-architecture.md).
- **Re-list**: `nft_approve` pada akun yang sudah di-approve **panic** (`AccountAlreadyApprovedError`),
  jadi revoke dulu wajib. `remove_sale` tidak melakukannya untuk seller (§2a).

## Discovery — Query / Filter / Sort / Pagination / Search

> **MVP**: FE baca langsung dari RPC view (`get_sales`, `get_offers`, `nft_tokens_for_owner`) + NearBlocks untuk riwayat; **tanpa endpoint discovery** (★ fase 2 — [endpoints.md](../api/endpoints.md), ADR-004). Sort/filter/pagination MVP dilakukan client-side atas hasil view; spesifikasi di bawah = kontrak target saat indexer aktif (fase 2).

| Aspek | Spesifikasi | MVP | Fase 2 |
|---|---|---|---|
| **Pagination** | Cursor opaque `{items[], nextCursor}`; `limit` default 20, maks 100 (clamp, bukan error) | Cursor view near-sdk (`nft_tokens_for_owner`) | Cursor endpoint `/api/v1/*` |
| **Sort** | `newest` (default) · `price_asc` · `price_desc` | Client-side atas hasil view | Server-side (index) |
| **Filter** | `collection` (contract id) · `status` (active/sold/cancelled/stale) · rentang harga | Client-side | Server-side |
| **Search** | `q` min 2 char; full-text nama koleksi/token; `type` = all/collection/token | Client-side (koleksi terbatas) | `GET /api/v1/search` |
| **Private listing** | `allowed_buyer != null` → tidak di-grid publik / tampil khusus | View field `allowed_buyer` | Sama |
| **Stale** | Listing stale **disembunyikan** dari discovery (badge stale tidak muncul di grid) | Cek ownership saat render | Flag `isStale` dari indexer |

- **URL state**: search/filter/sort/pagination di query params (shareable) — frontend-architecture.md §2.
- **Trait filter & rarity** = fase 3 (butuh custom indexer) — bukan MVP/fase 2.
- Urutan deterministik: `newest` → `listed_at` desc; tie-break `token_id` asc.

## Expiry Sweep & Stale Detection (Lazy Evaluation)

> MVP **tidak punya cron** (FACT). Expiry & stale dievaluasi **saat disentuh** — bukan oleh background job.

**Expiry offer (lazy):**

| Pemicu | Efek |
|---|---|
| `accept_offer` | Cek `block_timestamp < expires_at`; jika lewat → tolak, refund lazy. |
| `cancel_offer` (oleh buyer) | Refund penuh (jalur normal). |
| `storage_withdraw` / sentuhan view | Offer lewat expire ditandai EXPIRED + refund saat refund dieksekusi. |
| Siapa pun | Boleh memicu refund offer expired (lazy) — refund selalu ke `buyer_id` (INV-005). |

- Event `market_offer_expire` di-emit **saat refund lazy dieksekusi**, bukan saat `expires_at` terlewat ([webhooks.md](../api/webhooks.md)).
- FE menampilkan "expired" dari **cek expiry lokal** (`expires_at < now`) tanpa menunggu tx on-chain ([notifications.md](./notifications.md)).

**Stale detection (lazy):**

| Pemicu | Efek |
|---|---|
| `buy` / `buy_bundle` | Cek `nft_token().owner_id != sale.owner_id` → tolak + listing stale. |
| View discovery (`get_sales`) | FE bandingkan owner on-chain → sembunyikan listing stale dari grid. |
| `remove_stale_listing` | Permissionless cleanup setelah mismatch terbukti on-chain. |

- Timing deteksi: **saat view/beli** (bukan proaktif). Konsekuensi: listing stale bisa sempat tampak sampai view berikutnya — FE wajib re-verify sebelum sign (SEC-ORDER-003).
- `market_stale_detected` di-emit saat mismatch terdeteksi dan listing ditandai; setelah itu listing tidak bisa dibeli (INV-008).

## Tabel Edge Case Numerik

> Semua nilai = yoctoNEAR; 1 Ⓝ = `1000000000000000000000000`; 0.01 Ⓝ = `10000000000000000000000`.

| Kasus | Input | Perilaku kontrak | Kode user |
|---|---|---|---|
| Harga tepat di minimum | `price = 10000000000000000000000` (0.01 Ⓝ) | **Diterima** (batas inklusif ≥ MIN) | — |
| Harga di bawah minimum | `price = 9999999999999999999999` | Tolak (INV-030) | `INVALID_PRICE` |
| Deposit kurang 1 yocto | `attached = price − 1` | Tolak | `CHAIN_INSUFFICIENT_DEPOSIT` |
| Deposit tepat harga | `attached = price` | Sukses, sisa 0 | — |
| Deposit lebih (harga berubah) | `attached = price_lama > price_baru` | Selisih dikembalikan via `resolve_purchase` | `CONFLICT_PRICE_CHANGED` (preview) |
| Bundle tepat 10 token | `items.len() = 10` | Diterima (batas inklusif) | — |
| Bundle 11 token | `items.len() = 11` | Tolak | — |
| Penerima royalti unik = 10 | merge menghasilkan 10 receiver | Diterima (batas inklusif) | — |
| Penerima royalti unik = 11 | merge > 10 | Tolak saat `create_bundle` | — |
| Royalti per token tepat 10% | rate = 1000 bps | Diterima (cap inklusif) | — |
| Royalti per token > 10% | rate = 1001 bps | Tolak → refund | `CHAIN_REVERT` |
| Payout melebihi plafon | Σpayout > harga − fee | Tolak → refund | `CHAIN_REVERT` |
| Payout kosong / >10 penerima / amount 0 | struktur payout tidak sah | Tolak → refund | `CHAIN_REVERT` |
| Offer tepat minimum | `attached = 10000000000000000000000` | Diterima | — |
| Offer kedua buyer sama | 1 offer aktif sudah ada | Tolak (INV-024) | `CONFLICT_OFFER_EXISTS` |

- Boundary "tepat di batas" (min price, 10 token, 10 receiver, 10% royalti, Σpayout = harga − fee) **selalu inklusif** kecuali dinyatakan lain.
- `INVALID_PRICE` **terdaftar** di registry [error-handling.md](../development/error-handling.md) §3.

## Error Cases (pesan user-facing)

| Kasus | Pesan |
|---|---|
| Bukan allowed_buyer (private listing) | "Listing ini khusus pembeli tertentu" |
| Bundle > 10 token | "Maksimum 10 token per bundle" |
| Token bundle dipindah/di-list terpisah | "Bundle tidak bisa dibeli — salah satu item tidak valid" |
| Harga berubah saat signing | "Harga berubah — periksa ulang" (kontrak assert deposit ≥ price; kelebihan deposit dikembalikan via resolve) |
| Token terjual duluan | "Sudah terjual — dana kamu kembali" (refund otomatis) |
| Deposit kurang | "Jumlah kurang dari harga" |
| Self-buy | "Tidak bisa membeli milik sendiri" |
| Offer kedua oleh buyer sama | "Kamu sudah punya offer aktif di token ini" |
| Bukan owner aksi seller | revert kontrak (tanpa efek) |
| Kontrak di-pause (mutasi baru) | "Marketplace sedang dijeda sementara" (`CHAIN_PAUSED`) |
| Offer sudah expire | "Offer sudah kedaluwarsa — dana kembali" (refund lazy ke buyer) |
| Listing stale (ownership mismatch) | "Listing tidak lagi valid" (`CONFLICT_STALE`) |

## Security

> Draft: assert_one_yocto pada mutasi seller; callback `#[private]`; payout ≤ 10 akun; validasi total payout; stale check `nft_token.owner_id` saat offer/buy.

## Acceptance Criteria

- Given listing aktif milik orang lain, When buyer mengirim deposit = harga, Then NFT pindah, seller+royalti terbayar (fee 2% dipotong), listing jadi SOLD.
- Given offer aktif, When expire, Then escrow kembali penuh ke buyer.
- Given token dipindah di luar market, When sistem mendeteksi mismatch, Then listing stale & sembunyi dari discovery.
