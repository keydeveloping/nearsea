# Contract Reference — Market (Listing / Offer / Bundle / Settlement)

> Reference implementasi (ronde 16). Perilaku/settlement = SSOT di [features/marketplace.md](../features/marketplace.md); dokumen ini memiliki signature, layout state, konstanta, dan pemetaan method→event→error.

## Peran & batas dokumen

Kontrak **Market** = satu kontrak gabungan untuk listing + offer + private listing + bundle ([ADR-003](../decisions/ADR-003-single-market-contract.md)); satu otoritas settlement on-chain ([ADR-012](../decisions/ADR-012-order-settlement-authority.md)). Non-custodial: NFT tetap di wallet seller, market hanya memegang approval NEP-178 ([ADR-007](../decisions/ADR-007-approval-listing-model.md)).

| Fakta | Pemilik (dokumen ini hanya merujuk) |
|---|---|
| Alur listing 2-tx, buy, offer, bundle, stale | [features/marketplace.md](../features/marketplace.md) |
| Algoritma payout, fee 2%, merge bundle, refund | [features/payments.md](../features/payments.md) |
| Skema payload event (field-level) | [api/webhooks.md](../api/webhooks.md) |
| Registry kode error & pemetaan panic→kode | [development/error-handling.md](../development/error-handling.md) §3/§4 |
| Invariant INV-001..030 | [smart-contract-invariants.md](../security/smart-contract-invariants.md) |
| Tabel akses per-method (prosa) | [smart-contract-security-architecture.md](../security/smart-contract-security-architecture.md) §11 |

Basis tooling: Rust + `near-sdk` + `near-sdk-contract-tools` (`Owner` + `Pausable` + NEP-145 derive). Upgrade path & rencana test migrasi: contract-architecture §2/§14.

---

## 1. Init — `new`

```rust
#[near(contract_state)]
#[derive(PanicOnDefault)]                 // SEC-CONTRACT-002 (TC-001)
pub struct MarketContract { /* config + persistent collections, §7 */ }

#[near]
impl MarketContract {
    #[init]
    pub fn new(
        owner_id: AccountId,              // Platform Owner (MVP single-key; mainnet = DAO — ADR-013)
        fee_bps: Option<u16>,             // default FEE_BPS_DEFAULT = 200; wajib ≤ MAX_FEE_BPS
        treasury: Option<AccountId>,      // ⏳ open-by-design — default = owner_id (MVP, payments.md)
    ) -> Self;

    /// Upgrade path — hanya dipanggil saat redeploy + migrate (contract-arch §2/§14).
    #[init(ignore_state)]
    pub fn migrate() -> Self;
}
```

| Arg | Status | Catatan |
|---|---|---|
| `owner_id` | ✅ DECIDED | MVP = single owner key (interim, ADR-013); mainnet = Sputnik DAO V2 council 2-of-3 + timelock (SEC-CONTRACT-012). |
| `fee_bps` | ✅ DECIDED (default 200) | Assert `fee_bps ≤ MAX_FEE_BPS (500)` saat init — gagal → panic (`CHAIN_REVERT`). Cap immutable ditegakkan juga di `update_fee_bps` (INV-004). |
| `treasury` | ⏳ open-by-design | Alamat treasury ditetapkan saat deploy testnet; sementara = owner market ([features/payments.md](../features/payments.md) §MVP). Bisa diganti owner via `update_treasury` (§4). |

> **Status implementasi (TASK-005, ronde 23):** `new(owner_id, fee_bps: Option<u16>, treasury: Option<AccountId>)`.
> `fee_bps` default `FEE_BPS_DEFAULT` (200) dan wajib ≤ `MAX_FEE_BPS` saat init (gagal → panic
> `CHAIN_REVERT`); `treasury` default = `owner_id` (⏳ open-by-design — alamat treasury final ditetapkan
> saat deploy testnet). NEP-145 (`storage_deposit`/`storage_withdraw`/`storage_balance_of`/
> `storage_balance_bounds`) aktif dengan bounds `min = storage_per_sale()` (§6), `max = None`.
>
> **Koreksi `withdraw_fees` (ronde 23):** §4 sebelumnya memuat `withdraw_fees()` (fee terakumulasi di
> saldo kontrak, ditarik owner). **Method itu tidak diimplementasikan dan tidak diperlukan**: sesuai
> algoritma distribusi (§3, [features/payments.md](../features/payments.md) §Algoritma Distribusi Payout),
> fee ditransfer **langsung ke `treasury` di dalam settlement yang sama** — kontrak tidak pernah
> memegang dana fee. Lihat §4a untuk alasan lengkap dan konsekuensinya.

---

## 2. Method reference (mutasi order)

> Signature = kanonik; skema argumen field-by-field = SSOT [features/marketplace.md](../features/marketplace.md) § Skema Argumen — tidak diduplikasi di sini. Gas PROPOSED = estimasi, diukur sandbox; FACT = terukur ([features/payments.md](../features/payments.md) §Anggaran Gas). Batas keras **300 Tgas/call** (FACT).

| Method | Signature (ringkas) | Deposit | Auth (predecessor) | Checks (INV) | Event | Gas (Tgas) | Error utama |
|---|---|---|---|---|---|---|---|
| `list_nft_for_sale` | `(nft_contract_id, token_id, approval_id: Option<u64>, price: U128, allowed_buyer: Option<AccountId>)` | **storage NEP-145** (bukan 1 yocto) | seller = owner token | dual verify via XCC: `nft_token().owner_id == predecessor` + `nft_is_approved(market, approval_id)`; `price ≥ MIN_PRICE` (INV-030); belum ter-list (INV-007 → `CONFLICT_ALREADY_LISTED`); tidak di bundle aktif (INV-028 → `CONFLICT_BUNDLE_ITEM_INVALID` — **belum**, bundle = TASK-010); callback `process_listing` | `market_list` (di callback) | ≈10–20 total (PROPOSED): 2 view ~5 + callback ~10 | `INVALID_PRICE`, `CONFLICT_ALREADY_LISTED`, `CONFLICT_BUNDLE_ITEM_INVALID` (TASK-010), `CHAIN_REVERT` (storage), `CHAIN_PAUSED` |
| `remove_sale` | `(nft_contract_id, token_id)` | 1 yocto (`assert_one_yocto`) | `sale.owner_id` | entry ada; hapus entry `Sale`; **tidak** memanggil revoke (lihat §2a — NEP-178 tak punya revoke untuk approved account); boleh saat paused (INV-022) | `market_delist` | ~5 PROPOSED | `CHAIN_REVERT` |
| `update_price` | `(nft_contract_id, token_id, new_price: U128)` | 1 yocto | `sale.owner_id` | `new_price ≥ MIN_PRICE` (INV-030); entry ada (harga diganti in-place; `approval_id` tidak berubah) | `market_update_price` | ~5 PROPOSED | `INVALID_PRICE`, `CHAIN_REVERT` |
| `buy` | `(nft_contract_id, token_id)` | **≥ price** (kelebihan di-refund via resolve) | buyer ≠ seller | listing ada (INV-008 → `CONFLICT_SOLD`); bukan stale — ownership cocok **dan** approval masih valid (INV-016 → `CONFLICT_STALE`); private → `allowed_buyer` (INV-026 → `FORBIDDEN_BUYER`); **tulis `pending_purchases` SEBELUM optimistic removal** (INV-031 — jalur pemulihan bila callback gagal); callback `process_purchase` menegakkan stale **sebelum** transfer (stale ⇒ refund + `market_stale_detected`, **bukan** panic tx — lihat §3) | `market_sale` (di resolve) | < 150 PROPOSED (15 + 115 FACT) | `CONFLICT_SOLD`, `CONFLICT_STALE` (FE-facing, §3), `FORBIDDEN_SELF_BUY`, `FORBIDDEN_BUYER`, `CHAIN_INSUFFICIENT_DEPOSIT` |
| `make_offer` | `(nft_contract_id, token_id, expires_at: Option<u64>)` | **= amount** (exact escrow, INV-006); storage offer = pre-deposit NEP-145 terpisah | buyer ≠ owner token | `amount ≥ MIN_PRICE` (INV-030); belum ada offer aktif buyer/token (INV-024 → `CONFLICT_OFFER_EXISTS`); bukan owner token **saat itu** (INV-023); `expires_at` None → `now + DEFAULT_OFFER_DURATION_NS` | `market_offer` | ~5 PROPOSED | `CONFLICT_OFFER_EXISTS`, `FORBIDDEN_SELF_BUY`, `INVALID_PRICE`, `CHAIN_REVERT` (storage) |
| `cancel_offer` | `(nft_contract_id, token_id)` | 1 yocto | `offer.buyer_id` | entry ada; refund penuh escrow ke `offer.buyer_id` (hardcoded, INV-005/014); boleh saat paused (INV-022) | `market_offer_cancel` | ~5–10 PROPOSED | `CHAIN_REVERT` |
| `accept_offer` | `(nft_contract_id, token_id, buyer_id)` | 1 yocto | owner token **saat itu** | `env::block_timestamp() < expires_at` (INV-010); ownership saat itu (INV-016); entry dihapus atomik sebelum settle (INV-009); offer lain pada token sama → auto-cancel + refund (SUPERSEDED) | `market_offer_accept` (di resolve) | ~15 + 115 (15 FACT; total < 150 PROPOSED) | `CONFLICT_SOLD` (offer hilang), `CONFLICT_STALE`, `CHAIN_REVERT` |
| `create_bundle` | `(items: Vec<BundleItem>, price: U128)` | **storage NEP-145** | seller = owner SEMUA token | `1 ≤ items.len() ≤ MAX_BUNDLE_TOKENS` (INV-021 → `CONFLICT_BUNDLE_TOO_MANY`); tiap token milik seller + approval valid (XCC); token tidak ter-list/di-offer terpisah (INV-028 → `CONFLICT_BUNDLE_ITEM_INVALID`); receiver unik hasil merge ≤ `MAX_PAYOUT_RECEIVERS` (INV-021/027); **pre-validasi royalti ronde 16** (basis(token) — §3): `Σroyalti + fee ≤ price` (INV-025 → `CONFLICT_PRE_VALIDATE_FAILED`) | `market_bundle_create` | PROPOSED — diukur (XCC pre-validation) | `CONFLICT_BUNDLE_TOO_MANY`, `CONFLICT_BUNDLE_ITEM_INVALID`, `CONFLICT_PRE_VALIDATE_FAILED`, `INVALID_PRICE` |
| `buy_bundle` | `(bundle_id: u64)` | **≥ bundle price** | buyer ≠ seller (INV-023) | bundle `ACTIVE`; **pre-validasi SEMUA item sebelum transfer pertama** (ownership + approval + payout simulasi — INV-025 → `CONFLICT_PRE_VALIDATE_FAILED`); satu token bundle stale/pindah → seluruh bundle tak bisa dibeli (INV-028); loop `nft_transfer_payout` per token; fee dipotong **sekali**; merge ≤ 10 receiver | `market_sale` per token (bentuk payload ⏳ — §3) / `market_bundle_partial` | PROPOSED — **wajib diukur** (jalur paling rawan gas, INV-021) | `CONFLICT_PRE_VALIDATE_FAILED`, `FORBIDDEN_SELF_BUY`, `NOT_FOUND_BUNDLE`, `CHAIN_INSUFFICIENT_DEPOSIT` |
| `cancel_bundle` | `(bundle_id: u64)` | 1 yocto | `bundle.seller` | bundle ada; status → CANCELLED; hapus membership (approval **tidak** dicabut market — §2a); token bebas di-list lagi (INV-028 berhenti); boleh saat paused (INV-022) | `market_bundle_cancel` | ~5–10 PROPOSED | `NOT_FOUND_BUNDLE`, `CHAIN_REVERT` |
| `remove_stale_listing` | `(nft_contract_id, token_id)` | 1 yocto + gas XCC | **siapa pun** (permissionless) | kontrak **wajib** membuktikan staleness on-chain via XCC (bukan percaya klaim pemanggil): **ownership mismatch** `nft_token().owner_id != sale.owner_id` **ATAU** **approval tidak valid** `!nft_is_approved(sale.owner_id, market, approval_id)` (INV-016 dua kasus — §3a); bila tidak stale → tidak ada efek; boleh saat paused (INV-022) | `market_delist` (+ `market_stale_detected` saat deteksi) | ~10–15 PROPOSED (2 XCC) | `CONFLICT_STALE` (bila bukan stale — dipanggil sia-sia), `CHAIN_REVERT` |
| `recover_stuck_purchase` | `(nft_contract_id, token_id)` | 1 yocto | **siapa pun** (permissionless) | ada entri `pending_purchases` untuk sale key tsb **dan** `env::block_height() > pending.created_height + RECOVERY_DELAY_BLOCKS` (callback pasti sudah selesai/gagal) → restore `Sale` (bila token masih milik seller) + **refund penuh ke `pending.buyer`** + hapus entri; boleh saat paused (INV-022) | `market_purchase_recovered` | ~5 PROPOSED | `NOT_FOUND_*` (tidak ada pending), `CHAIN_REVERT` (terlalu dini) |
| `nft_on_approve` | `(token_id, owner_id, approval_id: u32, msg)` — masuk NEP-178 | 1 yocto (dikirim derive koleksi) | predecessor = kontrak NFT (pengirim approval) | **bukan** `#[private]` (INV-013); validasi bentuk payload; **TIDAK membuat listing** (ADR-002) — method ini tidak menulis state apa pun, jadi notifikasi palsu tak berefek | — | minimal | `CHAIN_REVERT` (payload) |

- `nft_on_approve` **tidak** membuat listing (ADR-002). Karena ia tidak menulis state, kontrak tidak perlu memverifikasi bahwa pemanggil benar-benar kontrak NFT — tidak ada yang bisa dieksploitasi; yang dijaga hanya bentuk payload (`token_id` non-kosong, `msg` non-kosong, `owner_id ≠ market`). Verifikasi sebenarnya terjadi di `list_nft_for_sale` → `process_listing`.

- `assert_one_yocto()` hanya untuk mutasi berbasis state yang tidak menerima deposit variabel (SEC-CONTRACT-001) — `list_nft_for_sale` & `create_bundle` **tidak** memakainya (deposit = storage).
- Stale = **lazy evaluation** (MVP tanpa cron — FACT): expiry offer & mismatch ownership dievaluasi saat disentuh; refund lazy selalu ke `buyer_id` (INV-005) + `market_offer_expire` saat refund dieksekusi ([features/marketplace.md](../features/marketplace.md) §Expiry Sweep).

### §2a. Koreksi: `remove_sale` **tidak** mencabut approval (ronde 22, saat implementasi TASK-004)

> Dokumen ini (dan 4 dokumen lain) sebelumnya menyebut `remove_sale` memanggil **`nft_revoke_token`**
> "sebagai approved account, bukan `nft_revoke` yang owner-only". **Method itu tidak ada di NEP-178.**
> NEP-178 hanya punya `nft_revoke` dan `nft_revoke_all`, dan keduanya mensyaratkan **pemilik token**
> sebagai pemanggil ("Contract MUST panic if called by someone other than token owner") — tidak ada
> varian untuk approved account, dan tidak ada `nft_on_revoke`. `near-sdk-contract-tools` 4.0
> (derive `NonFungibleToken` yang dipakai koleksi NearSea) mengikuti standar itu apa adanya.

**Konsekuensi implementasi (dipilih):** `remove_sale` hanya menghapus entry `Sale` dan mengembalikan
storage ke seller. Market **tidak** mencoba mencabut approval. Alasan ini aman:

- **Tanpa listing, approval tidak berguna.** Market hanya memindahkan token lewat jalur yang
  berangkat dari entry `Sale` (`buy`/`accept_offer`, TASK-005). Tidak ada entry → tidak ada alasan
  market menyentuh token; tidak ada method publik di market yang menerima `(contract, token)` tanpa
  membaca `Sale` lebih dulu.
- **Transfer oleh owner otomatis mencabut semua approval** (FACT NEP-178: "the approval is cleared
  when the token is transferred"). Jadi approval yang tertinggal tidak bisa dipakai setelah token
  pindah tangan.
- **Seller yang ingin mencabut sendiri bisa** — `nft_revoke(token_id, market)` adalah panggilan
  pemilik token, dan UI seller memang memegang kuncinya.

**Yang perlu diketahui FE:** approval lama tetap `true` on-chain setelah `remove_sale`. Karena
`nft_approve` pada akun yang sudah di-approve **panic** (`AccountAlreadyApprovedError` di derive),
re-list **wajib** memanggil `nft_revoke` (atau `nft_revoke_all`) dulu baru `nft_approve` baru.
`approval_id` untuk re-list selalu **baru** (M ≠ N) — klaim lama tetap benar, hanya jalurnya yang
berbeda: bukan "approval dicabut market", tapi "approval dicabut seller". `approval_id` yang dipegang
market bisa dibaca dari `nft_token(token_id).approved_account_ids[market]` (view `nft_token` derive
memuat field itu).

**Alternatif yang ditolak:** menambah method revoke khusus-approved-account di kontrak koleksi.
Ditolak karena (a) itu **memperluas NEP-178 di luar standar** — koleksi pihak ketiga tidak akan
punya, sehingga market jadi tidak portabel; (b) memberi market kemampuan mencabut approval sepihak
menambah permukaan otorisasi tanpa manfaat nyata (butir di atas).

### `BundleItem` (tipe argumen)

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct BundleItem {
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub approval_id: Option<u64>,
}
```

---

## 3. Callback `#[private]` (INV-013 — predecessor = kontrak sendiri)

### `process_listing` — hasil dual verification listing

```rust
#[private]
pub fn process_listing(
    &mut self,
    #[callback_result] token: Result<Option<Token>, PromiseError>,   // hasil nft_token(token_id)
    #[callback_result] approved: Result<bool, PromiseError>,         // hasil nft_is_approved(token_id, market, approval_id)
    seller_id: AccountId,            // pemanggil asli list_nft_for_sale (predecessor di sini = market)
    nft_contract_id: AccountId,
    token_id: String,
    approval_id: Option<u64>,
    price: U128,
    allowed_buyer: Option<AccountId>,
) {
    // token.ok && token.owner_id == seller_id && approved == Ok(true) → else panic CHAIN_REVERT
    // re-cek duplikat (INV-007) → simpan Sale + charge storage seller + emit market_list
    // gagal → tidak ada state ditulis (panic = revert receipt callback; deposit storage tetap milik user)
}
```

- `nft_token` mengembalikan `Option<Token>` (NEP-171), jadi callback-nya `Result<Option<Token>, _>`;
  `None` (token tidak ada) diperlakukan sebagai verifikasi gagal → `CHAIN_REVERT`.
- Pesan panic = **kode registry** (`CHAIN_REVERT`), bukan teks library ("Signer is not NFT owner"
  dari RESEARCH.md §10.4 adalah contoh tutorial) — [error-handling.md](../development/error-handling.md) §4.
- Gas: pakai sisa budget `list_nft_for_sale`; `GAS_FOR_PROCESS_LISTING` (§6) — PROPOSED.

### `process_purchase` — dual verification saat settle (TASK-005)

> **Mengapa ada callback ini.** `buy` **tidak bisa** memanggil `nft_token`/`nft_is_approved` secara
> sinkron (view call lintas-kontrak selalu lewat receipt terpisah — FACT). Tanpa langkah ini, satu-satunya
> pilihan adalah mempercayai klaim pemanggil — yang dilarang (SEC-ORDER-004, ADR-002). Jadi `buy` menulis
> `pending_purchases` + menghapus `Sale` (optimistic), lalu verifikasi dijalankan di callback **sebelum**
> `nft_transfer_payout`. Pola yang sama dipakai jalur listing (`process_listing`).

```rust
#[private]
pub fn process_purchase(
    &mut self,
    #[callback_result] token: Result<Option<Token>, PromiseError>,   // nft_token
    #[callback_result] approved: Result<bool, PromiseError>,         // nft_is_approved(market, approval_id)
    nft_contract_id: AccountId,
    token_id: String,
) -> Promise {
    // Konteks pembelian dibaca dari pending_purchases (bukan argumen) → tak ada yang bisa dipalsukan.
    // (A) stale  : token.owner_id != sale.owner_id  → ownership_mismatch
    //              token.owner_id == sale.owner_id ∧ approved == Ok(false) → approval_revoked
    //              → hapus pending, emit market_stale_detected, refund penuh. Sale TIDAK dipulihkan.
    // (B) tidak pasti: koleksi tak bisa dihubungi / token tidak ada / approved bukan Ok(true)
    //              → hapus pending, pulihkan Sale (kegagalan bukan stale), refund penuh.
    // (C) valid  : nft_transfer_payout(buyer, token, approval_id, null, price, Some(10))  // 1 yocto
    //              .then(resolve_purchase)
}
```

- **Stale tidak bisa mem-panic tx `buy`.** Saat stale terdeteksi, deposit buyer sudah ada di kontrak;
  panic hanya akan menahan dana itu (temuan C3). Karena itu stale = **refund penuh + event
  `market_stale_detected`**, bukan revert. `CONFLICT_STALE` tetap kode kanonik — tapi dipetakan FE dari
  **hasil** (token tidak pindah + event/`get_sale` kosong), bukan dari panic receipt (§4/§8).
- **Pause ditegakkan di callback juga** (INV-022). Callback = receipt terpisah, jadi kontrak bisa
  di-pause antara `buy` dan settle; dalam kondisi itu pembelian dibatalkan **tanpa menyentuh NFT**
  (pending dihapus, `Sale` dipulihkan, refund penuh) — pola yang sama dengan `process_listing`.
- **`fee_bps` di-snapshot saat `buy`.** `PendingPurchase.fee_bps` menyimpan fee yang berlaku ketika
  pembeli menandatangani, dan `resolve_purchase` memakainya (bukan `self.fee_bps` saat settle) —
  split distribusi tidak bisa berubah di antara dua receipt oleh `update_fee_bps` owner-only.
- `max_len_payout = Some(MAX_PAYOUT_RECEIVERS)` (10) — plafon NEP-199 sekaligus kontrol gas
  (INV-003/021, SEC-CONTRACT-010).
- **Eksposur H1 yang tersisa (dicatat, bukan bug baru).** `nft_transfer_payout` memindahkan NFT **dan**
  mengembalikan payout dalam panggilan yang sama, jadi payout invalid = NFT sudah di buyer dan `Sale`
  yang dipulihkan langsung stale (temuan H1). Untuk koleksi NearSea jalur ini **praktis tak terjangkau**:
  payout-nya hanya royalti kreator dengan rate ≤10% (dijaga saat init koleksi), sehingga
  `Σpayout ≤ harga − fee` (fee ≤5%) selalu benar. Jalur tetap ada sebagai fail-closed untuk koleksi
  pihak ketiga — yang memang di luar scope M1.
- Gas callback = `GAS_FOR_PROCESS_PURCHASE` (§6) = `GAS_FOR_NFT_TRANSFER` + `GAS_FOR_RESOLVE_PURCHASE`
  + overhead; `buy` meng-attach-nya ke callback ini sehingga total jalur buy = 2 view + 135 = 145 Tgas.

### `resolve_purchase` — settlement buy / accept_offer

Algoritma distribusi = SSOT [features/payments.md](../features/payments.md) §Algoritma Distribusi Payout; di sini yang dipegang: **validasi + urutan transfer + revert/refund**.

```rust
#[private]
pub fn resolve_purchase(
    &mut self,
    #[callback_result] payout: Result<Payout, PromiseError>,
    nft_contract_id: AccountId,
    token_id: String,
) {
    // Konteks (harga, buyer, deposit, seller) dibaca dari pending_purchases[sale_key].
    // GAGAL promise (revert NFT contract / gas) ATAU payout invalid →
    //   hapus pending + restore Sale + refund penuh ke pending.buyer (SEC-ORDER-001)
    // SUKSES → validasi payout (UNTRUSTED, dari NFT contract):
    //   1 ≤ len(payout) ≤ MAX_PAYOUT_RECEIVERS (10)          (INV-003)
    //   setiap amount > 0                                     (INV-003)
    //   fee = floor(price × fee_bps / 10_000);  Σpayout ≤ price − fee   (INV-002)
    // → transfer: payout → receiver (merge per receiver); fee → treasury; residual → seller
    //   (sisa = price − fee − Σpayout → seller; aritmetika internal exact: fee + Σ + seller == price)
    // → refund kelebihan deposit ke buyer (harga berubah saat signing — CONFLICT_PRICE_CHANGED preview)
    // → hapus pending_purchases[sale_key]
    // → emit market_sale (payout = map penerima final; fee TIDAK masuk map — Σpayout == price − fee)
}
```

- **Sisa pembulatan vs payout koleksi.** Toleransi `sisa ∈ {0,1}` yocto (INV-002) berlaku untuk
  aritmetika **eksternal**; yang divalidasi market hanyalah `Σpayout ≤ price − fee` + `amount > 0`.
  Karena koleksi NearSea mengembalikan royalti yang jauh di bawah plafon (mis. 5% vs 98% plafon),
  sisa besar = **proceeds seller** yang sah, bukan pelanggaran. Tidak ada batas atas pada sisa.
- Dust ≤1 yocto tidak direfund terpisah (toleransi pembulatan — [features/payments.md](../features/payments.md)).
- Validasi gagal di langkah mana pun → refund penuh buyer + restore Sale (listing tetap ada bila kegagalan bukan stale — [features/payments.md](../features/payments.md) §Refund Path).

### `resolve_purchase_bundle` — settlement bundle

- **Pre-validasi semua item SEBELUM transfer pertama** (INV-025): ownership + approval + simulasi `Σroyalti(basis ronde 16) + fee ≤ price` — gagal → abort bersih, **nol transfer**, refund penuh (deposit otomatis via rollback tx).
- Basis royalti per token (DECIDED ronde 16 — SSOT [features/payments.md](../features/payments.md) §Algoritma Merge): 1) harga listing aktif (state market lokal); 2) harga mint fase (`mint_price_of` XCC ke koleksi — [nft-collection.md](./nft-collection.md) §5); 3) sisa harga bundle dibagi rata (floor). `royalti_token = rate × basis` — rate dari `royalty_config()` koleksi.
- Loop `nft_transfer_payout` per token (≤10) → payout per token **di-merge per receiver** (tanpa cap agregat, INV-027) → fee dipotong sekali → residual → seller.
- Kegagalan residual mid-loop (NEAR tidak punya rollback lintas-receipt — FACT): status bundle → `PARTIAL`, dana belum terpakai di-refund, event `market_bundle_partial`, kompensasi G7 ([order-protocol-security.md](../security/order-protocol-security.md) §6; INV-025). Buyer tidak pernah kehilangan dana tanpa kompensasi tercatat.
- Pola eksekusi (chaining callback per langkah vs promise aggregate): **PROPOSED** — dipilih saat implementasi; nama method & jaminan di atas tetap kanonik.

### Event sukses bundle

`buy_bundle` sukses = `market_sale` per token (skema per-token [webhooks.md](../api/webhooks.md)). Nilai `price_yocto` per token pada event bundle **⏳ open-by-design** — usulan: basis royalti token ronde 16 bila terdefinisi, selain itu pembagian rata harga bundle (floor). Nama event khusus bundle-sukses belum ada di daftar kanonik (ronde 16 hanya menambah `market_bundle_create`/`cancel`/`partial`).

---

## 3a. Stale — dua kasus (perbaikan H4, ronde 17b)

> **Temuan H4 (review ronde 17):** definisi stale lama hanya mencakup **ownership mismatch**. Seller yang
> mencabut approval **tanpa memindahkan token** menghasilkan listing yang tampil di discovery tapi
> **selalu gagal dibeli**, dan `remove_stale_listing` tidak bisa membersihkannya (ownership cocok) →
> listing "zombie" permanen.

**Definisi kanonik stale (dua kasus, keduanya wajib didukung):**

```text
stale(sale) :=  (A) nft_token(token_id).owner_id != sale.owner_id      -- token dipindah di luar market
             ∨  (B) !nft_is_approved(sale.owner_id, market, approval_id) -- approval dicabut/kedaluwarsa
```

- **(A) ownership mismatch** — kasus lama; token dipindah/dijual di tempat lain.
- **(B) approval tidak valid** — kasus baru; seller memanggil `nft_revoke`, atau approval_id invalid
  setelah transfer (FACT NEP-178).

**Konsekuensi implementasi:**
- `buy` menolak kedua kasus dengan `CONFLICT_STALE` (bukan `CHAIN_REVERT` generik).
- `remove_stale_listing` **wajib** membuktikan salah satu kasus via XCC (`nft_token` + `nft_is_approved`);
  permissionless, pemanggil menanggung gas. Tidak percaya klaim pemanggil.
- Discovery: `is_stale` di FE harus mengecek **kedua** kasus (dua view call), bukan hanya ownership.
- Test: TC-053 (stale karena approval dicabut) + TC-006 (stale karena ownership) — keduanya wajib.

---

## 3b. Pemulihan dana nyangkut (perbaikan C3, ronde 17b)

> **Temuan C3 (review ronde 17):** `resolve_purchase` adalah **receipt terpisah**. Bila callback-nya
> gagal (gas habis, panic), `buy` **tidak** revert; `Sale` sudah dihapus optimistic; deposit tertahan di
> kontrak; dan **tidak ada method recovery publik** → dana buyer hanya bisa dipulihkan lewat upgrade
> governance. Itu kehilangan dana dengan pemulihan yang bergantung pada tim.

**Mekanisme `pending_purchases` (wajib di M1):**

```rust
// state baru (prefix: PendingPurchases)
struct PendingPurchase {
    buyer: AccountId,
    deposit: u128,          // attached deposit saat buy
    created_height: u64,    // env::block_height() saat buy
    fee_bps: u16,           // snapshot fee saat buy — split tidak berubah oleh update_fee_bps
    sale: Sale,             // snapshot listing yang dihapus (verifikasi + restore + distribusi)
}
// LookupMap<SaleKey, PendingPurchase>
```

```text
buy(sale_key):
  1. assert pending_purchases[sale_key] TIDAK ada          -- cegah tumpang tindih
  2. tulis pending_purchases[sale_key] = {buyer, deposit, created_height}
  3. hapus Sale (optimistic removal)
  4. XCC nft_transfer_payout → callback resolve_purchase

resolve_purchase (callback, #[private]):
  - SUKSES → distribusi → HAPUS pending_purchases[sale_key]
  - GAGAL   → restore Sale + refund buyer → HAPUS pending_purchases[sale_key]

recover_stuck_purchase(sale_key):        -- permissionless, siapa pun
  assert pending_purchases[sale_key] ada
  assert env::block_height() > pending.created_height + RECOVERY_DELAY_BLOCKS   -- callback pasti selesai/gagal
  → restore Sale (bila token masih milik sale.owner_id) + refund penuh ke pending.buyer
  → hapus pending_purchases[sale_key]; emit market_purchase_recovered
```

**Jaminan (INV-031):** setiap deposit yang masuk ke `buy` **selalu** punya jalur keluar —
`resolve_purchase` (sukses/gagal) **atau** `recover_stuck_purchase` (permissionless, tanpa syarat
kepercayaan pada tim). Tidak ada dana yang bergantung pada governance untuk kembali.

**Parameter:**
- `RECOVERY_DELAY_BLOCKS` = **PROPOSED** (usulan 20 blok ≈ 20 detik; cukup agar callback pasti selesai,
  cukup pendek agar buyer tidak menunggu lama). Diukur/dikonfirmasi saat implementasi.
- `resolve_purchase` gas budget **wajib diverifikasi** terhadap worst case (10 payout + fee + seller +
  refund + event) — bila 115 Tgas tidak cukup, naikkan dan dokumentasikan (temuan C3 bagian gas).
- **Storage entri `pending_purchases` ditanggung kontrak, bukan buyer** (deviasi sadar dari INV-020).
  Entri ini **transien** — dibuat di `buy`, dihapus di setiap ujung jalur (settle sukses/gagal,
  `process_purchase`, recovery). Alasannya buyer tidak punya saldo NEP-145 di market, dan depositnya
  sendiri (≥ harga ≥ 0.01 Ⓝ) jauh melampaui biaya storage satu entry (~ratusan byte) sehingga kontrak
  tidak pernah insolven. Entry yang tersisa karena callback tak pernah jalan ditutup
  `recover_stuck_purchase`. Biaya nyata diukur TASK-006; bila ternyata signifikan, buyer dapat dikenai
  pre-deposit NEP-145 terpisah.

**Berlaku juga untuk `buy_bundle`** (saat bundle dikerjakan di M1+): satu entri pending per bundle,
dipulihkan dengan pola yang sama.

---

## 4. Owner / guardian (bukan order)

| Method | Signature | Auth | Checks | Event |
|---|---|---|---|---|
| `update_fee_bps` | `update_fee_bps(fee_bps: u16)` | owner → mainnet: DAO 2-of-3 + timelock | `fee_bps ≤ MAX_FEE_BPS` (INV-004 — upgrade pun tak bisa menaikkan cap) | `fee_update` |
| `update_treasury` | `update_treasury(treasury: AccountId)` | owner → mainnet: DAO | — | `treasury_update` |
| `pause` | `pause()` | owner (MVP) / **guardian `pause_callers`** (mainnet — terpisah dari owner-DAO, ADR-013) | — | `market_pause` |
| `unpause` | `unpause()` | owner (MVP) / **hanya** owner-DAO (guardian tidak boleh — mencegah penyanderaan, ADR-013) | — | `market_unpause` |
| `storage_deposit` | `storage_deposit(account_id: Option<AccountId>, registration_only: Option<bool>)` | pemanggil (atau utk `account_id`) | NEP-145 | — |
| `storage_withdraw` | `storage_withdraw(amount: Option<U128>)` | pemilik saldo | NEP-145; **tetap diizinkan saat paused** (INV-022) | — |

- `update_fee_bps`/`update_treasury` = `assert_one_yocto()` + owner-only (SEC-CONTRACT-001/012).
- `withdraw_fees` **tidak ada** — lihat §4a.

### §4a. Koreksi: `withdraw_fees` dihapus — fee masuk treasury di settlement (ronde 23)

> Dokumen ini (§4), [features/payments.md](../features/payments.md) §Akuntansi, [security/asset-inventory.md](../security/asset-inventory.md),
> [security/smart-contract-security-architecture.md](../security/smart-contract-security-architecture.md) §11, dan
> [decisions/ADR-005-platform-fee.md](../decisions/ADR-005-platform-fee.md) §Consequences sebelumnya
> menyebut model **dua langkah**: fee terakumulasi di saldo kontrak, lalu owner menariknya via
> `withdraw_fees()`. Itu **bertentangan** dengan algoritma distribusi yang menjadi SSOT perilaku
> settlement — [features/payments.md](../features/payments.md) §Algoritma Distribusi Payout:

```text
distributions = nft_payout + [ (seller, seller), (treasury, fee) ]
```

**Keputusan (ronde 23, saat TASK-005):** fee ditransfer **langsung ke `treasury` di dalam settlement
yang sama**. `withdraw_fees` tidak diimplementasikan karena tidak ada dana fee yang pernah tertahan di
kontrak.

**Alasan:**

- **Menghapus temuan M7.** Dengan akumulasi, saldo kontrak bercampur tiga jenis dana (fee, escrow offer,
  storage NEP-145) tanpa akuntansi solvensi — `withdraw_fees` tidak bisa dibuktikan aman tanpa memisahkan
  ketiganya. Model transfer-langsung membuat kontrak **tidak pernah** memegang fee: tidak ada yang perlu
  dipisahkan, tidak ada honeypot saldo yang bisa disedot bila owner key bocor.
- **Selaras SSOT & AC.** [features/payments.md](../features/payments.md) §Algoritma (perilaku settlement),
  §3 dokumen ini, dan AC TASK-005 ("membayar kreator + treasury + seller dalam satu settlement") semuanya
  menyatakan transfer langsung. Yang menyimpang adalah baris-baris ringkasan (akuntansi/asset-inventory/TC-047),
  bukan algoritmanya.
- **Konsisten dengan event.** Skema `market_sale` ([api/webhooks.md](../api/webhooks.md)) mendefinisikan
  `payout` = map penerima **final** dengan `Σpayout + fee ≤ harga`; contohnya tidak memuat treasury.
  Itu hanya konsisten bila fee keluar lewat transfer terpisah.

**Konsekuensi:**

- Cakupan owner-only yang sebelumnya diuji lewat `withdraw_fees` (TC-047) dialihkan ke
  `update_fee_bps`/`update_treasury` — keduanya tetap owner-only + 1 yocto dan punya test unit.
- **`treasury` wajib akun yang ada sebelum deploy.** Bila transfer fee gagal (akun treasury tidak ada),
  receipt itu gagal **terpisah** — penjualan tetap sah (NFT pindah, royalti + seller terbayar), tapi fee
  itu tertinggal di saldo kontrak **tanpa jalur penarikan**. Ini misconfiguration operator, bukan vektor
  serangan (tidak ada pihak lain yang bisa memicunya); dicatat sebagai batasan operasional, sejalan
  ADR-005 §Consequences ("Treasury address harus diisi sebelum deploy").
- Bila suatu saat model akumulasi + `withdraw_fees` diinginkan kembali, temuan M7 (akuntansi solvensi
  fee vs escrow vs storage) **wajib** diselesaikan lebih dulu, dan keputusan itu harus lewat ADR.

- **Pausable (INV-022)**: `paused` memblokir SEMUA mutasi baru KECUALI jalur pengembalian aset/refund: `cancel_offer`, `cancel_bundle`, `remove_sale`, `remove_stale_listing`, `storage_withdraw` (TC-012).
- Governance gating MVP→mainnet: aksi finansial/upgrade = Sputnik DAO V2 council 2-of-3 + timelock 24 jam; pause tetap single-key guardian ([ADR-013](../decisions/ADR-013-key-management.md), SEC-CONTRACT-012; `pause_callers` = `UnorderedSet` di state, di-set saat deploy — PROPOSED mekanisme penambahannya owner-only).
- Owner bukan role on-chain tambahan lain — moderasi TIDAK pernah on-chain ([smart-contract-security-architecture.md](../security/smart-contract-security-architecture.md) §3).

---

## 5. Views (return shape eksplisit)

View call = read-only via RPC, tidak biaya gas user; FE MVP baca langsung + filter client-side ([features/marketplace.md](../features/marketplace.md) §Discovery; [ADR-004](../decisions/ADR-004-mvp-data-layer.md)). Nilai Ⓝ = **string yoctoNEAR (u128)**; waktu = u64 nanodetik.

| View | Signature |
|---|---|
| `get_sale` | `get_sale(nft_contract_id: AccountId, token_id: String) -> Option<Sale>` |
| `get_sales` | `get_sales(offset: Option<u64>, limit: Option<u8>) -> Vec<Sale>` |
| `get_supply_sales` | `get_supply_sales() -> u64` |
| `get_fee_bps` | `get_fee_bps() -> u16` — dipakai ops/monitoring ([deployment.md](../deployment/deployment.md), [monitoring.md](../deployment/monitoring.md)) |
| `get_treasury` | `get_treasury() -> AccountId` |
| `get_pending_purchase` | `get_pending_purchase(nft_contract_id: AccountId, token_id: String) -> Option<PendingPurchase>` — entri `pending_purchases` (§3b); dipakai ops/FE untuk memicu `recover_stuck_purchase` setelah jeda (TC-054) |
| `get_offer` | `get_offer(nft_contract_id: AccountId, token_id: String, buyer_id: AccountId) -> Option<Offer>` |
| `get_offers` | `get_offers(offset: Option<u64>, limit: Option<u8>) -> Vec<Offer>` |
| `get_supply_offers` | `get_supply_offers() -> u64` |
| `get_offers_for_token` | `get_offers_for_token(nft_contract_id: AccountId, token_id: String, offset: Option<u64>, limit: Option<u8>) -> Vec<Offer>` |
| `get_offers_by_account` | `get_offers_by_account(account_id: AccountId, offset: Option<u64>, limit: Option<u8>) -> Vec<Offer>` |
| `get_bundle` | `get_bundle(bundle_id: u64) -> Option<Bundle>` |
| `get_bundles` | `get_bundles(offset: Option<u64>, limit: Option<u8>) -> Vec<Bundle>` |
| `get_supply_bundles` | `get_supply_bundles() -> u64` |
| `is_stale` | `is_stale(nft_contract_id: AccountId, token_id: String, current_owner: AccountId) -> bool` |
| `storage_minimum_balance` / `storage_balance_of` | NEP-145 standar |
| `contract_source_metadata` | NEP-330 (reproducible build) |

```json
// Sale (get_sale / get_sales[].entry exists ⇔ listing aktif)
{
  "nft_contract_id": "nft.example.testnet",
  "token_id": "42",
  "owner_id": "alice.testnet",
  "approval_id": 7,
  "price_yocto": "1000000000000000000000000",
  "allowed_buyer": null,
  "listed_at": 1799016000000000000
}
```

```json
// Offer (get_offer / get_offers[].escrow tersimpan di saldo kontrak — INV-006 exact)
{
  "nft_contract_id": "nft.example.testnet",
  "token_id": "42",
  "buyer_id": "bob.testnet",
  "amount_yocto": "900000000000000000000000",
  "expires_at": 1799016000000000000,
  "created_at": 1798411200000000000
}
```

```json
// Bundle (get_bundle / get_bundles[] — satu harga utk semua item, INV-028)
{
  "bundle_id": 7,
  "seller": "alice.testnet",
  "price_yocto": "3000000000000000000000000",
  "status": "ACTIVE",
  "created_at": 1798411200000000000,
  "items": [
    { "nft_contract_id": "nft.example.testnet", "token_id": "42", "approval_id": 11 },
    { "nft_contract_id": "nft.example.testnet", "token_id": "43", "approval_id": 12 }
  ]
}
```

Semantik status & staleness:

- `Sale` **tanpa field status** — entry ada ⇔ ACTIVE; hilang ⇔ SOLD/delisted (optimistic removal, INV-009); `allowed_buyer ≠ null` = private listing (disembunyikan dari grid publik).
- `Bundle.status`: `ACTIVE` / `SOLD` / `CANCELLED` / `PARTIAL` (residual failure — §3).
- `is_stale` menerima `current_owner` sebagai argumen karena **view call tidak bisa cross-contract** (FACT) — FE/indexer membaca `nft_token().owner_id` dari koleksi lalu membandingkan di sini; otoritas keputusan stale tetap on-chain saat `buy`/`accept_offer` (INV-016) dan `remove_stale_listing` membuktikan mismatch sendiri.
- **Tidak ada view launchpad di market** — state launchpad hidup di kontrak koleksi; `get_launchpad` = view koleksi ([nft-collection.md](./nft-collection.md) §5). Intent `get_launchpad?` di [features/marketplace.md](../features/marketplace.md) §Contract API dimiliki kontrak koleksi, bukan market (XCC mustahil di view).
- `get_offers_by_account` = scan + filter atas `offers` (MVP tanpa index per-akun); index `offers_by_account` = ⏳ fase 2 (indexer — [ADR-015](../decisions/ADR-015-indexer-consistency.md)); `limit` di-clamp (bukan error) — pola [features/marketplace.md](../features/marketplace.md) §Discovery.
- `storage_minimum_balance()` = hasil `storage_usage` aktual saat implementasi — nominal per-entry **⏳ open-by-design** (OQ-007; angka 0.01 Ⓝ dari tutorial BUKAN keputusan NearSea — [RESEARCH.md](../../RESEARCH.md) §10.3).

---

## 6. Konstanta bernama

```rust
// Gas (Tgas)
pub const GAS_FOR_RESOLVE_PURCHASE: Gas = Gas::from_tgas(115); // FACT — callback settle (RESEARCH.md §10.2)
pub const GAS_FOR_NFT_TRANSFER: Gas    = Gas::from_tgas(15);   // FACT — nft_transfer_payout (1 yocto attach)
pub const GAS_FOR_NFT_VIEW: Gas         = Gas::from_tgas(5);   // PROPOSED — satu view dual verification
pub const GAS_FOR_PROCESS_LISTING: Gas  = Gas::from_tgas(10);  // PROPOSED — callback process_listing
pub const GAS_FOR_DUAL_VERIFY: Gas      = Gas::from_gas(2 * GAS_FOR_NFT_VIEW.as_gas() + GAS_FOR_PROCESS_LISTING.as_gas());
                                                               // PROPOSED (usulan 15–20 Tgas; total listing ≈10–20)
pub const GAS_FOR_PROCESS_PURCHASE: Gas = Gas::from_gas(GAS_FOR_NFT_TRANSFER.as_gas() + GAS_FOR_RESOLVE_PURCHASE.as_gas() + Gas::from_tgas(5).as_gas());
                                                               // PROPOSED — callback process_purchase (dual verify saat buy)
                                                               //   total jalur buy = 2 view (10) + 135 = 145 Tgas (<150)
pub const GAS_FOR_PROCESS_RECOVERY: Gas = Gas::from_tgas(10);  // PROPOSED — callback process_recovery
pub const NO_DEPOSIT: Balance = 0;

// Ekonomi (bps & yoctoNEAR — string u128 di JSON)
pub const MAX_FEE_BPS: u16          = 500;   // immutable cap — INV-004 (ubah hanya via upgrade lolos governance)
pub const FEE_BPS_DEFAULT: u16      = 200;   // 2%
pub const MIN_PRICE_YOCTO: u128     = 10_000_000_000_000_000_000_000; // 0.01 Ⓝ = 10^22 yocto (INV-030)
pub const STORAGE_PER_SALE_BYTES: u64 = 500; // PROPOSED — plafon byte entry `Sale` utk bounds NEP-145 (INV-020)

// Batas statis (INV-021)
pub const MAX_PAYOUT_RECEIVERS: u32 = 10;   // penerima payout setelah merge (batas gas NEP-199/300 Tgas)
pub const MAX_BUNDLE_TOKENS: u32    = 10;   // item per bundle

// Waktu / blok (nanodetik — selaras env::block_timestamp())
pub const RECOVERY_DELAY_BLOCKS: u64 = 20;  // PROPOSED — jeda blok sebelum recover_stuck_purchase (INV-031)
pub const DEFAULT_OFFER_DURATION_NS: u64 = 7 * 24 * 60 * 60 * 1_000_000_000; // 604_800_000_000_000 ns = 7 hari
```

| Konstanta | Nilai | Status |
|---|---|---|
| `GAS_FOR_RESOLVE_PURCHASE` | 115 Tgas | ✅ FACT |
| `GAS_FOR_NFT_TRANSFER` | 15 Tgas | ✅ FACT |
| `GAS_FOR_DUAL_VERIFY` | 20 Tgas (2×5 view + 10 callback) | ⏳ PROPOSED (usulan 15–20 Tgas utk 2 promise + callback; diukur sandbox TASK-006) |
| `GAS_FOR_NFT_VIEW` / `GAS_FOR_PROCESS_LISTING` | 5 / 10 Tgas | ⏳ PROPOSED — komponen `GAS_FOR_DUAL_VERIFY` (total listing ≈10–20 Tgas) |
| `GAS_FOR_PROCESS_PURCHASE` | 135 Tgas (15 + 115 + 5) | ⏳ PROPOSED — callback `process_purchase`; total jalur buy = 2 view (10) + 135 = **145 Tgas** (<150) |
| `GAS_FOR_PROCESS_RECOVERY` | 10 Tgas | ⏳ PROPOSED — callback `process_recovery` |
| `STORAGE_PER_SALE_BYTES` | 500 | ⏳ PROPOSED — bounds NEP-145 `min`; nominal final = `storage_usage` aktual (OQ-007) |
| `MAX_FEE_BPS` | 500 | ✅ DECIDED (INV-004) |
| `FEE_BPS_DEFAULT` | 200 (2%) | ✅ DECIDED (ADR-005) |
| `MIN_PRICE_YOCTO` | `10000000000000000000000` (0.01 Ⓝ) | ✅ DECIDED (INV-030; edge table [features/marketplace.md](../features/marketplace.md)) |
| `RECOVERY_DELAY_BLOCKS` | 20 | ⏳ PROPOSED — jeda blok `recover_stuck_purchase` (INV-031, §3b) |
| `DEFAULT_OFFER_DURATION_NS` | 604_800_000_000_000 (7 hari) | ✅ DECIDED (ronde 4/6) |
| `MAX_PAYOUT_RECEIVERS` | 10 | ✅ DECIDED (INV-003/021) |
| `MAX_BUNDLE_TOKENS` | 10 | ✅ DECIDED (ronde 6) |
| Total gas per tx | maks 300 Tgas (FACT); `buy` < 150 PROPOSED | ✅ FACT / PROPOSED |

---

## 7. Storage-key prefix design (SEC-CONTRACT-008)

```rust
#[derive(BorshStorageKey)]
pub enum StorageKey {
    Sales,            // UnorderedMap<SaleKey, Sale>;  SaleKey  = (AccountId, String)
                      //   ⇔ satu token tidak bisa punya 2 listing aktif (INV-007)
    PendingPurchases, // LookupMap<SaleKey, PendingPurchase>; state transien INV-031 (§3b)
                      //   (buyer, deposit, created_height, snapshot Sale) — satu entri per settle berjalan
    Offers,           // UnorderedMap<OfferKey, Offer>; OfferKey = (AccountId, String, AccountId)
                      //   (nft_contract_id, token_id, buyer_id) ⇔ 1 offer aktif /buyer/token (INV-024)
    Bundles,          // UnorderedMap<u64, Bundle>     (key = bundle_id)
    BundleItems,      // UnorderedMap<u64, Vec<BundleItem>>  (≤ MAX_BUNDLE_TOKENS per bundle)
                      //   PROPOSED: Vec di nilai map (contract-arch §13 menulis Vector<BundleItem>;
                      //   ≤10 item = satu Vec cukup — final saat implementasi)
    StorageDeposits,  // LookupMap<AccountId, u128> (NEP-145 — user bayar storage, INV-020)
    Paused,           // flag bool (kunci eksplisit agar layout stabil & ter-audit per release)
    PauseCallers,     // UnorderedSet<AccountId> (guardian mainnet — ADR-013; kosong di MVP testnet)
    BundleCounter,    // u64 next_bundle_id (counter — get_supply_bundles turunannya)
}
```

- `config` (`owner_id`, `treasury`, `fee_bps`) = **field struct biasa di root state** (bukan persistent collection — tidak butuh prefix) — ikut didokumentasikan per release.
- Layout = **PROPOSED** (ditetapkan saat implementasi; kunci & urutan enum stabil) — [smart-contract-security-architecture.md](../security/smart-contract-security-architecture.md) §13; versi layout dipublikasikan tiap release (SEC-CONTRACT-008), migrate via `#[init(ignore_state)]` (§1).
- Semua map yang tumbuh dibayar user via NEP-145: entry `Sale`, `Offer`, `Bundle`+membership (INV-020); deposit tidak otomatis kembali saat refund/cancel — ditarik via `storage_withdraw` (anti gas-DoS — [features/marketplace.md](../features/marketplace.md) §Storage Deposit).

> **Status implementasi (TASK-005, ronde 23):** `Sales` + `PendingPurchases` ada di enum kontrak
> (`market/src/lib.rs`). `Offers`/`Bundles`/`BundleItems`/`BundleCounter` menyusul di TASK-009/010.
> `Paused` dan prefix NEP-145 (`~$145`) dikelola derive (`Pause`, `Nep145`), bukan enum kontrak —
> prefix milik derive tidak boleh bertabrakan dengan nama enum di atas. `PauseCallers` (guardian
> mainnet, ADR-013) ditambahkan saat mekanisme penambahannya diputuskan. `StorageDeposits` tidak
> lagi entry enum: saldo storage NEP-145 disimpan derive di slot `~$145`.
> `config` (`owner_id`, `treasury`, `fee_bps`) = field struct biasa di root state (§7 bawah).

---

## 8. Pemetaan assert → kode user (ringkas)

Registry = [error-handling.md](../development/error-handling.md) §3; pemetaan FE berdasar **tipe error** ([§4](../development/error-handling.md)) — tabel ini = peta kasus per method (padanan contract-arch §15).

| Kondisi on-chain (assert) | Kode user | Method |
|---|---|---|
| Sale tidak ada / sudah dihapus (optimistic removal) | `CONFLICT_SOLD` | `buy`, `accept_offer` |
| `attached < price` / `≠ price` (exact) | `CHAIN_INSUFFICIENT_DEPOSIT` | `buy`, `buy_bundle`, `make_offer` |
| Pemanggil = seller / owner token | `FORBIDDEN_SELF_BUY` | `buy`, `buy_bundle`, `make_offer` |
| Ownership mismatch **atau** approval dicabut (stale) | `CONFLICT_STALE` | `buy`/`accept_offer`/`buy_bundle` — **hasil**, bukan panic tx (§3 `process_purchase`) |
| Private listing, buyer ≠ `allowed_buyer` | `FORBIDDEN_BUYER` | `buy` |
| Offer aktif sudah ada (buyer/token) | `CONFLICT_OFFER_EXISTS` | `make_offer` |
| Harga < 0.01 Ⓝ | `INVALID_PRICE` | `list_nft_for_sale`, `update_price`, `make_offer`, `create_bundle` |
| Duplikat listing aktif | `CONFLICT_ALREADY_LISTED` | `list_nft_for_sale` |
| Item sudah di bundle aktif lain / token bundle di-list terpisah | `CONFLICT_BUNDLE_ITEM_INVALID` | `create_bundle`, `list_nft_for_sale`, `make_offer` |
| Bundle > `MAX_BUNDLE_TOKENS` | `CONFLICT_BUNDLE_TOO_MANY` | `create_bundle` |
| Pre-validasi bundle gagal (item/payout/Σroyalti+fee > harga) | `CONFLICT_PRE_VALIDATE_FAILED` | `create_bundle`, `buy_bundle` |
| Payout invalid struktur (0 / >10 receiver, amount 0, Σ > harga−fee) | `CHAIN_REVERT` | `resolve_purchase` (internal: tolak + refund; FE melihat hasilnya sebagai `CONFLICT_*`/refund) |
| Royalti per token > 10% | `INVALID_ROYALTY` | resolve paths (map [error-handling.md](../development/error-handling.md) §4) |
| Receiver unik hasil merge > 10 | `CHAIN_REVERT` (kandidat kode khusus — wajib didaftarkan dulu di registry §3 bila FE butuh) | `create_bundle` |
| Kontrak paused, mutasi baru | `CHAIN_PAUSED` | semua mutasi kecuali daftar INV-022 |
| `assert_one_yocto` gagal / auth gagal / bundle_id tidak ada | `CHAIN_REVERT` / `NOT_FOUND_BUNDLE` | umum |
| Harga berubah antar preview & sign (kelebihan deposit di-refund via resolve) | `CONFLICT_PRICE_CHANGED` (preview FE — bukan assert) | `buy` |

- Refund selalu ke penerima hardcoded dari state (`buyer_id`) — tidak ada transfer ke alamat arbitrer (INV-014). Refund tidak terhalang paused (INV-022).
- Dust ≤1 yocto = toleransi pembulatan, tidak di-refund terpisah ([features/payments.md](../features/payments.md) §Presisi).
- **Status (ronde 23):** baris yang **sudah** berlaku = `list_nft_for_sale`, `remove_sale`, `update_price` (TASK-004), `buy`/`process_purchase`/`resolve_purchase`/`recover_stuck_purchase`, `update_fee_bps`/`update_treasury` (TASK-005). Baris offer, bundle, `remove_stale_listing` menyusul TASK-009/010/022 — kode errornya tetap kanonik dan dipetakan FE seperti di atas.
- **`CONFLICT_STALE` bukan panic tx.** Karena verifikasi stale butuh XCC (receipt terpisah), `buy`
  tidak bisa menolaknya secara sinkron: stale diputuskan di `process_purchase` → refund penuh +
  event `market_stale_detected`. FE memetakan `CONFLICT_STALE` dari **hasil** (dana kembali +
  listing hilang/`get_sale` kosong), bukan dari panic.

## Catatan review desain (ronde 17) — temuan belum tertutup

> Review kritis desain (3 agent, 2026-10-07) menemukan 7 kelemahan nyata. **Status pasca-keputusan
> ADR-016** (M1 = vertical slice, bundle ditunda ke M1+): tiga yang kritis **tertunda bersama bundle**;
> yang lain punya jalur penyelesaian. **Tidak ada yang boleh dianggap selesai sebelum diperbaiki** —
> dokumen ini mencatatnya supaya tidak hilang.

| # | Temuan | Sev | Status pasca-cut |
|---|---|---|---|
| **C1** | **`PARTIAL` tanpa alokasi kerugian.** Untuk bundle satu harga, "dana belum terpakai di-refund" **tidak terdefinisi** (pro-rata? penuh? nol?). Yang paling mungkin = buyer dapat refund penuh sambil menyimpan k NFT → **seller kehilangan NFT tanpa dibayar**. Ditambah: fund G7 (0.1% × fee 2% = **0.002% volume**) **tidak punya jalur pendanaan** di algoritma payout mana pun, dan `PARTIAL` **tidak punya method keluar** (tidak ada `sweep`/`finalize`). | **Kritis** | **Tertunda** — bundle = TASK-010 (M1+). Wajib diperbaiki sebelum TASK-010. |
| **C2** | **Gas `buy_bundle` melebihi 300 Tgas.** Angka desain sendiri: 10×15 + 115 = 265, sisa **35 Tgas** — padahal pre-validasi INV-025 butuh ownership + approval + simulasi payout **per item** (~3–4 XCC × 10 ≈ 90–150 Tgas) dari pohon gas yang sama. Estimasi total **355–430 Tgas** → bundle 10 token **tidak bisa dibeli**. | **Kritis** | **Tertunda** (M1+). Wajib **diukur di sandbox** + turunkan `MAX_BUNDLE_TOKENS` atau pecah settlement. |
| **C3** | **Dana buyer bisa nyangkut.** `resolve_purchase` receipt terpisah; bila callback-nya gagal, `buy` **tidak** revert, Sale sudah dihapus optimistic, deposit tertahan di kontrak, dan **tidak ada method recovery** publik (hanya `withdraw_fees` owner + `storage_withdraw`). | **Kritis** | ✅ **SELESAI (ronde 17b)** — §3b: `pending_purchases` + `recover_stuck_purchase` permissionless + INV-031 + TC-054. |
| **H1** | **Payout pihak ketiga divalidasi setelah NFT pindah.** `nft_transfer_payout` memindahkan NFT **dan** mengembalikan payout dalam panggilan yang sama → payout invalid = NFT sudah di buyer, buyer di-refund, **seller kehilangan NFT**. | **Tinggi** | **Dimatikan di M1** — slice hanya menerima koleksi NearSea (view royalty tersedia). Untuk membuka koleksi pihak ketiga (M1+), butuh desain baru. |
| **H2** | **`accept_offer` loop tak terbatas.** Auto-cancel + refund **semua** offer lain di token; jumlah offer tidak dibatasi → penyerang bisa menaruh ratusan offer murah sehingga `accept_offer` kehabisan gas (griefing) + escrow nyangkut. INV-021 mengklaim "semua loop terbatas statis" — loop ini tidak. | **Tinggi** | **Tertunda** — offers = TASK-009 (M1+). Perlu batas statis atau batalkan per-offer saat cancel. |
| **H3** | **Wiring factory tidak menutup storage + `new()` salah argumen.** `args_init` didokumentasikan memuat `phases`, tapi `CollectionContract::new()` **tidak punya parameter `phases`** (dan `phases` juga dikirim via `set_phases` terpisah). `.transfer()` hanya menutup `cost_account + cost_code`, sementara `set_phases`/`allowlist_add` dipanggil dengan **deposit 0** padahal butuh bayar storage. | **Tinggi** | **Tidak relevan di M1** (factory tidak dipakai). Wajib diperbaiki sebelum TASK-012/020. |
| **H4** | **Listing "zombie".** Stale hanya dicek dari **ownership**; seller yang mencabut approval (tanpa pindah token) menghasilkan listing yang tampil di discovery tapi **selalu gagal dibeli**, dan `remove_stale_listing` tidak bisa membersihkannya (ownership cocok). | **Tinggi** | ✅ **SELESAI (ronde 17b)** — §3a: definisi stale dua kasus + TC-053. |

**Temuan Medium** (dicatat, tidak memblokir): M1 private listing bukan rahasia (hanya access control — harga & target buyer publik on-chain); M2 allowlist tanpa `remove` + tanpa cap total + reuse phase-index; M3 stale menumpuk tanpa insentif pembersihan; M4 spam listing murah tanpa rate limit; M5 `accept_offer` tidak menghapus entry `Sale`; M6 basis royalti bundle rule-1 praktis mati (INV-028 melarang item bundle di-list terpisah); M7 akuntansi solvensi `withdraw_fees` tidak dispesifikasi (fee vs escrow vs storage dalam satu saldo).

## Status

- Signature, konstanta, layout prefix, callback validation, return shape view — **di-pertanggung-jawab-kan dokumen ini** (reference implementasi, ronde 16).
- Behavior/settlement (alur, kapan refund, merge, partial, lazy sweep) = SSOT [features/marketplace.md](../features/marketplace.md) + [features/payments.md](../features/payments.md) — di sini hanya dirujuk, tidak dinyatakan ulang.
- Invariant relevan: INV-001..003, INV-004, INV-005/006, INV-007..011, INV-013, INV-016, INV-020..028, INV-030, INV-031; requirement SEC-CONTRACT-001..005, 007, 008, 010, 011, 012.

### Status implementasi (ronde 23 — TASK-005: buy + settlement + refund)

`market/src/lib.rs` mengimplementasikan **§1, §2 jalur listing + buy, §2a, §3 `process_listing`/
`process_purchase`/`resolve_purchase`, §3a stale, §3b recovery, §4/§4a fee-treasury, §5 view, §6, §7**:

| Sudah ada | Bukti |
|---|---|
| `new(owner_id, fee_bps, treasury)` + bounds NEP-145 (`min = storage_per_sale()`, `max = None`); `fee_bps` default 200, cap `MAX_FEE_BPS` saat init | `test_new_defaults_fee_and_treasury`, `test_new_accepts_fee_at_cap_and_custom_treasury`, `test_new_rejects_fee_above_cap` |
| `update_fee_bps` / `update_treasury` — 1 yocto, owner-only, cap fee, event `fee_update`/`treasury_update` | `test_update_fee_bps_*` (4 test), `test_update_treasury_*` (2 test) |
| `list_nft_for_sale` — harga min (INV-030), duplikat (INV-007), storage NEP-145 (INV-020), paused (INV-022), `approval_id` u64→u32 | `test_list_*` (10 test) |
| Dual verification via 2 view XCC + `process_listing` `#[private]` (INV-013) — ownership **dan** approval, bukan klaim pemanggil (ADR-002) | `test_list_creates_sale_after_dual_verification`, `test_list_rejected_when_*` |
| Non-custodial: jalur listing hanya membuat view call; tidak ada `nft_transfer`/`nft_transfer_payout` | `test_list_creates_sale_after_dual_verification` (assert receipt) |
| `remove_sale` — 1 yocto, owner-only, hapus entry + kembalikan storage, boleh saat paused, **tanpa** revoke approval (§2a) | `test_remove_sale_*` (6 test) |
| `update_price` — 1 yocto, owner-only, in-place, min harga, paused | `test_update_price_*` (4 test) |
| **`buy`** — self-buy (INV-023), private listing (INV-026), deposit ≥ harga, `CONFLICT_SOLD` saat sale/pending sudah ada, **`pending_purchases` ditulis sebelum optimistic removal** (INV-031) | `test_buy_rejected_for_*`, `test_buy_rejected_when_*`, `test_buy_settles_*` |
| **`process_purchase`** `#[private]` — dual verification saat settle; **stale dua kasus** (INV-016: ownership mismatch **dan** approval dicabut) → refund + `market_stale_detected`; verifikasi tak pasti → refund + restore `Sale`; valid → `nft_transfer_payout` (1 yocto, `max_len_payout = 10`) | `test_buy_refunds_and_marks_stale_when_*` (2), `test_buy_restores_sale_when_verification_is_inconclusive`, `test_buy_attaches_one_yocto_and_payout_cap_to_transfer` |
| **`resolve_purchase`** `#[private]` — payout UNTRUSTED divalidasi (`1..=10` penerima, `amount > 0`, `Σ ≤ harga−fee`); distribusi fee → treasury, royalti → receiver, residual → seller (INV-001/002/003); refund saat invalid/gagal | `test_resolve_refunds_*` (6), `test_resolve_accepts_*` (2), `test_resolve_accepts_small_royalty_and_pays_residual_to_seller` |
| Kelebihan deposit di-refund ke buyer (INV-001) | `test_buy_refunds_excess_deposit_to_buyer` |
| Event `market_sale` dengan payout map final + `market_stale_detected` + `market_purchase_recovered` | assert payload di test terkait |
| **`recover_stuck_purchase` + `process_recovery`** — permissionless, jeda `RECOVERY_DELAY_BLOCKS`, refund penuh + restore opsional, boleh saat paused (INV-031/022) | `test_recover_*` (5 test) |
| Anggaran gas `resolve_purchase` diverifikasi vs worst case (10 penerima + fee + seller + refund + event) | `test_resolve_purchase_gas_within_budget` |
| View `get_sale` / `get_sales` / `get_supply_sales` / `get_fee_bps` / `get_treasury` / `get_pending_purchase` | `test_get_sales_pagination_and_supply`, `test_new_defaults_fee_and_treasury` |

Belum diimplementasikan (sengaja, bukan kelalaian):

| Bagian | Milik | Catatan |
|---|---|---|
| `remove_stale_listing` + `is_stale` | **TASK-022** (M1+) | Butuh XCC pembuktian stale (INV-016 dua kasus). `buy` sudah menolak kedua kasus stale (refund + event). |
| Offers / bundle | **TASK-009/010** (M1+) | Termasuk temuan C1/C2/H2 yang belum tertutup. |
| `withdraw_fees` | **dihapus** | Fee masuk treasury di settlement (§4a) — tidak ada dana fee yang tertahan di kontrak. |
| Kalibrasi `GAS_FOR_*` / `STORAGE_PER_SALE_BYTES` / `RECOVERY_DELAY_BLOCKS` + pengukuran sandbox | **TASK-006** | Nilai saat ini = usulan dokumen. |
| Deploy ke testnet | butuh **persetujuan user** | [git-workflow.md](../development/git-workflow.md) §3. |

**Tiga sifat implementasi yang perlu diketahui (fail-closed, belum diuji end-to-end di unit):**

1. **Storage = plafon, bukan nominal eksak.** `list_nft_for_sale` mengecek `available ≥ storage_per_sale()`
   **sinkron** (supaya deposit kurang = revert tx, bukan callback gagal senyap); pertumbuhan **aktual**
   entry `Sale` ditagih di callback. Bila entry tumbuh melebihi plafon (mis. `token_id` sangat panjang),
   callback panic → listing tidak jadi, **tanpa kehilangan dana** (deposit tetap di saldo seller).
   Nominal final = `storage_usage` aktual (OQ-007), diukur TASK-006.
2. **Urutan hasil promise = urutan `.and()`.** `process_listing`/`process_purchase` membaca `token`
   (indeks 0) lalu `approved` (indeks 1); kalau tertukar, deserialisasi gagal → `CHAIN_REVERT`
   (gagal aman). Unit test memanggil callback langsung, jadi urutan nyata promise dibuktikan di
   sandbox TASK-006.
3. **`buy` = 2 receipt untuk verifikasi, bukan 1.** Verifikasi kepemilikan/approval mustahil sinkron
   (view XCC selalu receipt terpisah), jadi `buy` **tidak** menolak tx untuk stale — ia menulis
   `pending_purchases`, menghapus `Sale`, lalu `process_purchase` memutuskan (stale → refund + event).
   Deposit buyer tidak pernah nyangkut: setiap jalur menghapus `pending_purchases` (§3b, INV-031).
