# Smart Contract Security Architecture

> Arsitektur keamanan kontrak sebelum implementasi. Kontrak: (1) **NFT Collection Contract** (di-deploy factory, NEP-171/177/178/181/199/297 + launchpad phases), (2) **Market Contract** (listing/offer/bundle/settlement), (3) **Factory** (deploy koleksi). Basis: near-sdk-contract-tools (derive NEP + Pausable + Owner).

## 1. Responsibility split (on-chain WAJIB vs boleh off-chain)

| Fungsi | Wajib on-chain | Alasan |
|---|---|---|
| Settlement & payout | ✅ | dana |
| Escrow offer & refund | ✅ | dana |
| Fee split (2%) | ✅ | dana |
| Validasi phase launchpad & allowlist | ✅ | pembeda akses mint |
| Royalti | ✅ (dari `nft_transfer_payout`) | dipaksa on-chain |
| Stale detection | ✅ cek saat sentuh | konsistensi |
| Pencarian/sort/trait | ❌ off-chain (API/fase 2) | performa |
| Notifikasi | ❌ off-chain | non-fundamental |
| Moderasi/verified badge | ❌ off-chain (display-only) | non-custodial |

## 2. Upgrade strategy

- **MVP (testnet)**: redeploy ke akun sama — state persist (FACT docs: "updating the code does not erase the state"). Perubahan struktur state → method `migrate` dengan `#[init(ignore_state)]` (pola resmi docs smart-contracts/release/upgrade) — bersihkan key orphan agar tidak buang storage.
- **Emergency tool resmi**: **State Cleaner** (tools/clear-state) — wipe state kontrak tanpa hapus akun (terakhir, merusak — hanya via break-glass).
- **Mainnet (DECIDED — ADR-013)**: upgrade via redeploy + `#[init(ignore_state)]` migrate, dengan owner actions finansial di-guard Sputnik DAO V2 council 2-of-3 + timelock; `fee_bps` di-cap oleh `MAX_FEE_BPS` (immutable — INV-004) dan `treasury` di-guard governance DAO (bukan konstanta), sehingga upgrade tidak bisa menyalin dana sekehendak hati. ~~Proxy pattern~~ bukan pola standar NEAR — tidak dipakai.
- Storage layout: near-sdk persistent collections TIDAK boleh diubah prefix/struktur sembarangan saat upgrade → dokumen layout versi di setiap release (SEC-CONTRACT-008).
- **Global contracts**: deploy by hash = immutable, by account = updatable — relevan jika kita pakai global contract untuk hemat storage (perhatikan konsekuensi masing-masing).

## 2b. Verifikasi build & source (NEP-330 — FACT docs, diputuskan dipakai)

- **Reproducible builds** = fitur first-class NEAR: `cargo near new` menyertakan metadata `[package.metadata.near.reproducible_build]` (image Docker + digest + command); proses **build** (reproducible) menyematkan method `contract_source_metadata` (NEP-330) ke wasm tanpa mengubah logika; `cargo near deploy` meng-upload hasil build tersebut.
- **Verifikasi**: siapa pun bisa cek `near view <contract> contract_source_metadata` + build ulang di Docker → hash identik; SourceScan (sourcescan.dev) bisa jadi jembatan publik.
- **Konsekuensi keamanan**: SEC-CONTRACT-006 diperkuat — bukan cuma banding hash artifact CI, tapi **publish source + verifikasi reproduksi siapa pun**. Wajib pakai workflow reproducible sejak commit pertama kontrak.

## 3. Access control

- `Owner` pattern (near-sdk-contract-tools): owner = Platform Owner; fungsi owner: `pause`/`unpause`, `update_fee_bps` (≤ MAX_FEE_BPS), `update_treasury`, `withdraw_fees` (tarik akumulasi fee — MVP default: owner-only; mainnet via DAO), (factory) config.
- **Mainnet**: `pause` dipanggil **guardian `pause_callers`** (terpisah dari owner-DAO) — lihat key-management §3.
- **Tidak ada** role "admin" on-chain selain owner — moderasi TIDAK pernah on-chain (display-only, ADR-006).
- Mutasi seller/buyer = `predecessor_account_id` + ownership/approval check + `assert_one_yocto()`.

## 4. Pause / emergency

- `Pausable` di Market & Factory (DECIDED ronde 13): `paused → assert!` di semua method mutasi KECUALI `storage_withdraw`, `cancel_offer`/`cancel_bundle` (pengembalian aset), `cancel`/`remove_stale_listing` (refund/withdraw path).
- Owner-only; tidak ada timelock untuk pause (kecepatan respons > formalitas); unpause boleh ada delay (PROPOSED).

## 5. Fee & royalty model

- `fee_bps` default 200 (2%); **MAX_FEE_BPS = 500 immutable** (SEC-CONTRACT-004) — upgrade tidak bisa menaikkan > 5%.
- Treasury = account owner pada MVP (DECIDED untuk MVP; ditinjau saat mainnet bersama migrasi Sputnik DAO); ganti treasury = owner-only + event.
- Royalti: dibaca dari `nft_transfer_payout` (bukan dihitung market) → sumber = kontrak koleksi; market memvalidasi (≤10 akun, sum, remainder).

## 6. Approval model

- NEP-178 per-token (bukan setApprovalForAll ala EVM — FACT: NEAR tidak punya setApprovalForAll di standar; lebih granular).
- Market selalu cek `nft_is_approved(token, market, approval_id)` + `nft_token(token).owner_id` sebelum settle (dual verification — ADR-002).
- Cancel listing → market **hanya menghapus entry `Sale`**; **tidak** mencabut approval. NEP-178 hanya punya `nft_revoke`/`nft_revoke_all` dan keduanya owner-only ("MUST panic if called by someone other than token owner") — tidak ada `nft_revoke_token` di standar, dan `near-sdk-contract-tools` 4.0 mengikuti itu. Aman karena tanpa entry `Sale` market tidak punya jalur memindahkan token, dan transfer oleh owner otomatis mencabut semua approval. Re-list = seller `nft_revoke` dulu, baru `nft_approve` baru (derive panic bila akun sudah di-approve). Lihat [contracts/market.md](../contracts/market.md) §2a.

## 7. Conduit/operator model

- **TIDAK ADA** padanan Seaport conduit. Settlement transfer = `nft_transfer_payout` dipanggil market sebagai approved operator. Satu jalur transfer per koleksi. (FACT — sederhanakan threat surface.)

## 8. Settlement & withdrawal model

- **Buy (listing aktif)**: buyer attach Ⓝ = harga ke `buy` → dalam receipt yang sama market memanggil `nft_transfer_payout` (1 yocto attach, 15 Tgas) → resolve callback distribusi (115 Tgas).
- **Offer (escrow)**: buyer attach Ⓝ ke `make_offer` → Ⓝ tersimpan sebagai escrow; TIDAK ada transfer saat itu. Transfer hanya terjadi saat seller `accept_offer`.
- Offer escrow: Ⓝ tersimpan di saldo kontrak per offer entry; refund = `Promise::new(buyer).transfer(amount)`.
- Tidak ada "withdrawal balance" seller (proceeds langsung transfer) → mengurangi custodial state. FACT by design.

## 9. Initialization

- `#[init]` + `PanicOnDefault` (FACT pattern); argumen: owner_id, fee_bps, treasury. Takeover init = tidak mungkin jika PanicOnDefault aktif — wajib di-review saat code review (SEC-CONTRACT-002).

## 10. Multisig / timelock

- MVP: tidak ada (single owner key; key-management.md).
- Mainnet: **DECIDED (ADR-013)** — owner actions finansial (fee/treasury/upgrade) via **Sputnik DAO V2 council 2-of-3** + timelock; pause tetap single-key (reaksi cepat). Legacy wallet-2FA multisig = DEPRECATED, ditolak.

## 11. Tabel akses per-method (method → auth → checks → event → gas)

> Kolom **Deposit** = aturan `attached_deposit` (1 yocto / exact / storage). Kolom **Checks** = pemeriksaan wajib sebelum mutasi. Gas: **FACT** = terukur, lainnya **PROPOSED** (diukur sandbox). Sumber skema argumen: [marketplace.md](../features/marketplace.md) § Skema Argumen; event: [webhooks.md](../api/webhooks.md).

| Method | Auth (predecessor) | Deposit | Checks | Event | Gas (Tgas) |
|---|---|---|---|---|---|
| `list_nft_for_sale` | seller = owner token | storage NEP-145 (bukan 1 yocto) | dual verify: `nft_token().owner_id == predecessor` + `nft_is_approved(market, approval_id)`; `price ≥ MIN` (INV-030); token belum di-list (INV-007); token tidak di bundle aktif (INV-028) | `market_list` | ~10–20 PROPOSED |
| `remove_sale` | `sale.owner_id` | 1 yocto (`assert_one_yocto`) | entry ada; hapus entry `Sale` + kembalikan storage; **tidak** mencabut approval (NEP-178 owner-only — [contracts/market.md](../contracts/market.md) §2a) | `market_delist` | ~5 PROPOSED |
| `update_price` | `sale.owner_id` | 1 yocto | `new_price ≥ MIN` (INV-030); entry ACTIVE | `market_update_price` | ~5 PROPOSED |
| `buy` | buyer ≠ seller | `≥ price` | listing ACTIVE & bukan stale (INV-016); private → `allowed_buyer` (INV-026); optimistic removal sale sebelum settle | `market_sale` | < 150 PROPOSED (15+115 FACT) |
| `make_offer` | buyer ≠ owner token | `= amount` (exact escrow, INV-006) | `amount ≥ MIN` (INV-030); belum ada offer aktif buyer/token (INV-024); token bukan milik pemanggil (INV-023) | `market_offer` | ~5 PROPOSED |
| `cancel_offer` | `offer.buyer_id` | 1 yocto | entry ada; refund penuh ke `buyer_id` (INV-005) | `market_offer_cancel` | ~5–10 PROPOSED |
| `accept_offer` | owner token **saat itu** | 1 yocto | `block_timestamp < expires_at` (INV-010); ownership token saat itu; offer lain → SUPERSEDED | `market_offer_accept` | < 150 PROPOSED |
| `create_bundle` | seller = owner semua token | storage NEP-145 (bukan 1 yocto) | 1..=10 item (INV-021); tiap token milik seller + approval valid; receiver unik hasil merge ≤10 (INV-027); token tidak di-list/di-offer terpisah (INV-028) | ⏳ open-by-design | PROPOSED |
| `buy_bundle` | buyer ≠ seller | `≥ bundle price` | pre-validasi SEMUA item sebelum transfer pertama (INV-025); loop payout per token → merge per receiver | `market_sale` / `market_bundle_partial` | wajib diukur PROPOSED |
| `cancel_bundle` | `bundle.seller` | 1 yocto | entry ada; revoke approval per token | ⏳ open-by-design | ~5–10 PROPOSED |
| `remove_stale_listing` | siapa pun (permissionless) | 1 yocto | kontrak **wajib** verifikasi `nft_token().owner_id != sale.owner_id` sebelum hapus | `market_delist` (+ `market_stale_detected`) | ~5 PROPOSED |
| `nft_mint` (launchpad) | minter | `= phase.price` (exact, INV-018) + storage (INV-019) | fase aktif tunggal & berurutan (INV-029); alokasi/`max_per_wallet`; allowlist bila `allowlist_required` (INV-017) | `launchpad_mint` | PROPOSED |
| `storage_withdraw` | pemilik saldo storage | 1 yocto | NEP-145; tetap diizinkan saat `paused` (INV-022) | — | ~5 PROPOSED |
| `withdraw_fees` | owner (MVP) / DAO (mainnet) | 1 yocto | hanya dana fee, **bukan escrow** (INV-005) | ⏳ open-by-design | PROPOSED |
| `pause` / `unpause` | owner (MVP) / guardian `pause_callers` (mainnet, pause saja) | 1 yocto | — | `market_pause` / `market_unpause` | ~3 PROPOSED |
| `nft_on_approve` (masuk) | `predecessor` = kontrak NFT sah | 1 yocto | payload NEP-178 valid; **bukan** `#[private]` (INV-013) | — | PROPOSED |
| `process_listing` / `resolve_purchase` / `nft_resolve_transfer` (callback) | kontrak sendiri (`#[private]`) | — | predecessor = self; state konsisten (INV-013) | — | 115 (resolve) FACT |

- **Catatan**: `assert_one_yocto()` hanya untuk mutasi seller/buyer berbasis state (SEC-CONTRACT-001); method yang menerima storage (`list_nft_for_sale`, `create_bundle`) **tidak** memakai 1 yocto.
- Pause memblokir semua mutasi baru KECUALI `storage_withdraw`, `cancel_offer`/`cancel_bundle`, `remove_sale`/`remove_stale_listing` (jalur refund/aset — INV-022).

## 12. Sequence diagram — settlement (buy listing)

```text
Buyer            Market Contract                 NFT Contract
  │                    │                              │
  │  buy(nft, token)   │                              │
  │  attach Ⓝ = price  │                              │
  ├───────────────────►│                              │
  │                    │ 1. cek predecessor ≠ seller  │
  │                    │ 2. ambil Sale; cek ACTIVE    │
  │                    │    & bukan stale (INV-016)   │
  │                    │ 3. optimistic: hapus Sale     │
  │                    │                              │
  │                    │  nft_is_approved(token,market,approval_id)
  │                    ├─────────────────────────────►│
  │                    │◄─────────────────────────────┤ bool
  │                    │  nft_token(token)            │
  │                    ├─────────────────────────────►│
  │                    │◄─────────────────────────────┤ owner_id
  │                    │ assert owner == sale.owner   │
  │                    │                              │
  │                    │ nft_transfer_payout(buyer, token, approval_id, price, max=10)
  │                    │ attach 1 yocto · 15 Tgas     │
  │                    ├─────────────────────────────►│  ┌─ payout NEP-199
  │                    │                              │  │  (royalti; UNTRUSTED)
  │                    │◄─────────────────────────────┤  └─ NFT pindah ke buyer
  │                    │ resolve_purchase (callback, #[private]) 115 Tgas
  │                    │  ┌─ validasi payout (≤10, Σ ≤ price−fee, sisa ≤1 yocto)
  │                    │  ├─ fee 2% → treasury
  │                    │  ├─ royalti → receiver (merge per receiver)
  │                    │  ├─ residual → seller
  │                    │  └─ refund selisih deposit ke buyer
  │                    │ emit market_sale              │
  │◄───────────────────┤ (receipt_id, event_index)     │
  │  hasil: NFT di wallet buyer; fee+royalti+seller = price (INV-001/002)
```

- **Gagal promise / payout invalid** → `resolve_purchase` mengembalikan state Sale + refund buyer atomik (SEC-ORDER-001).
- Untuk **bundle**, langkah `nft_transfer_payout` diulang per token (≤10) setelah pre-validasi semua item; tidak ada rollback lintas-receipt di NEAR → kegagalan residual = status `PARTIAL` + `market_bundle_partial` + kompensasi G7 (INV-025).

## 13. Storage layout (persistent collections)

> near-sdk persistent collections (`LookupMap`/`UnorderedMap`/`Vector`) menyimpan per **prefix byte** di state trie. **Prefix & struktur TIDAK boleh berubah sembarangan saat upgrade** (SEC-CONTRACT-008) — dokumen layout versi di setiap release. Layout ini **PROPOSED** (ditetapkan saat implementasi; kunci stabil).

```text
STATE ROOT (kontrak Market)
├── config
│   ├── owner_id            : AccountId        (MVP single-key; mainnet = DAO account)
│   ├── treasury            : AccountId        (fee destination)
│   ├── fee_bps             : u16 = 200        (≤ MAX_FEE_BPS const = 500, INV-004)
│   ├── paused              : bool
│   └── pause_callers       : UnorderedSet<AccountId>   (guardian mainnet)
├── sales                   : UnorderedMap<SaleKey, Sale>
│      SaleKey  = (nft_contract_id, token_id)
│      Sale     = { owner_id, approval_id, price, allowed_buyer, ... }
├── offers                  : UnorderedMap<OfferKey, Offer>
│      OfferKey = (nft_contract_id, token_id, buyer_id)   ← unique key = 1 offer/buyer/token (INV-024)
│      Offer    = { buyer_id, amount(escrow), expires_at, ... }
├── bundles                 : UnorderedMap<u64, Bundle>     (bundle_id = counter)
│      Bundle   = { seller, price, status }
├── bundle_items            : UnorderedMap<u64, Vector<BundleItem>>   (≤10 per bundle, INV-021)
├── storage_deposits        : LookupMap<AccountId, u128>   (NEP-145; user bayar storage — INV-020)
├── next_bundle_id          : u64
└── launchpad (NFT collection contract)
    ├── phases              : Vector<Phase>       (berurutan, tidak overlap — INV-029)
    └── allowlist           : LookupSet<AccountId> per fase
```

| Zona | Prefix stabil? | Risiko upgrade | Kontrol |
|---|---|---|---|
| `sales` / `offers` / `bundles` / `bundle_items` | **Ya** (dokumen per release) | ubah prefix → data "hilang" (orphan) | `migrate` bersihkan key orphan; review release (SEC-CONTRACT-008) |
| `config` | Ya | field baru → `#[init(ignore_state)]` migrate | migrasi eksplisit + test |
| `storage_deposits` | Ya (NEP-145) | refactor → saldo user tidak ketemu | jangan ubah; uji saldo pasca-migrate |
| `launchpad.phases` | Ya | phase berurutan (index = urutan) | validasi INV-029 saat migrate |

- **Orphan keys**: setelah migrasi struktur, key lama yang tidak terpakai **dihapus** agar tidak membuang storage (State Cleaner hanya break-glass).
- Field baru **aditif** (default value via `Option`) aman; menghapus/mengubah tipe field = migrasi + test khusus.

## 14. Rencana test migrasi (upgrade)

> Setiap upgrade state berisiko; migrasi **tanpa test = tidak boleh** (SEC-CONTRACT-008, AGENTS "main harus buildable"). Migrasi memakai pola resmi `#[init(ignore_state)]` (docs smart-contracts/release/upgrade).

| # | Langkah | Test / bukti |
|---|---|---|
| 1 | Snapshot state sebelum upgrade (jumlah entry tiap map) | skrip sandbox: hitung `sales.len()`, `offers.len()`, `bundles.len()`, `storage_deposits` |
| 2 | Deploy kode baru ke akun yang sama (redeploy) | state persist (FACT docs) — verifikasi jumlah entry tidak berubah |
| 3 | Panggil `migrate()` (`#[init(ignore_state)]`) | assert sukses; migrasi idempoten |
| 4 | Verifikasi integritas pasca-migrate | semua listing/offer/bundle dapat dibaca; saldo storage utuh |
| 5 | Uji jalur uang pasca-migrate | TC-002 (buy), TC-005 (accept offer), TC-009 (bundle) hijau |
| 6 | Verifikasi tidak ada orphan key | iterasi map; banding dengan snapshot langkah 1 |
| 7 | Publikasi hash build reproducible (NEP-330) | `near view … contract_source_metadata` + build ulang → hash identik (SEC-CONTRACT-006) |
| 8 | Rollback plan | redeploy kode lama (state tetap); jika migrasi destruktif → restore dari snapshot testnet |

- Migrasi diuji di **testnet** dulu (drill); mainnet hanya setelah drill hijau + approval user (AGENTS.md).
- Perubahan `MAX_FEE_BPS` / `treasury` tidak lewat migrasi state biasa — lewat governance DAO (ADR-013) dengan timelock.

## 15. Daftar kode error kontrak (assert → pemetaan user)

> Kontrak melempar panic/assert dengan teks; wallet menampilkan mentah. FE **wajib** menormalkan ke kode user ([error-handling.md](../development/error-handling.md) §4). Pemetaan dilakukan dengan **mencocokkan tipe error**, bukan substring.

| Kondisi on-chain (assert) | Kode user | Invariant |
|---|---|---|
| Sale tidak ada / sudah dihapus | `CONFLICT_SOLD` | INV-007/008 |
| `attached < price` | `CHAIN_INSUFFICIENT_DEPOSIT` | INV-001 |
| `predecessor == sale.owner_id` gagal | `FORBIDDEN_SELF_BUY` / revert generik | INV-023 |
| Ownership mismatch (stale) | `CONFLICT_STALE` | INV-016 |
| Private listing, buyer ≠ `allowed_buyer` | `FORBIDDEN_BUYER` | INV-026 |
| Offer aktif sudah ada (buyer/token) | `CONFLICT_OFFER_EXISTS` | INV-024 |
| Payout invalid (>10, Σ > harga−fee, sisa >1 yocto, royalti >10%) | `CHAIN_REVERT` | INV-002/003/027 |
| Harga berubah antar preview & sign | `CONFLICT_PRICE_CHANGED` | INV-001/002 |
| Kontrak `paused` | `CHAIN_PAUSED` | INV-022 |
| Phase launchpad overlap / tidak aktif | `CHAIN_REVERT` | INV-017/029 |
| Deposit launchpad ≠ `phase.price` | `CHAIN_INSUFFICIENT_DEPOSIT` | INV-018 |
| Storage deposit kurang | `CHAIN_REVERT` (storage) | INV-020 |
| Bundle > 10 token / receiver unik > 10 | `CHAIN_REVERT` | INV-021 |
| `assert_one_yocto` gagal | `CHAIN_REVERT` | SEC-CONTRACT-001 |

- Registry lengkap (envelope + namespace + status HTTP) dimiliki [error-handling.md](../development/error-handling.md) §3 — dokumen ini **tidak menduplikasi** nilai, hanya memetakan assert kontrak → kode user.
- Kode error kontrak **stabil**; teks assert boleh berubah tanpa memutus FE karena pemetaan berbasis tipe error.
