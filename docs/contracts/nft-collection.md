# Contract Reference — NFT Collection (NEP-171/177/178/181/199 + Launchpad)

> Reference implementasi (ronde 16). Perilaku/settlement = SSOT di [features/marketplace.md](../features/marketplace.md); dokumen ini memiliki signature, layout state, konstanta, dan pemetaan method→event→error.

## Peran & batas dokumen

Kontrak **NFT Collection** = satu kontrak per koleksi (pola NEAR — [RESEARCH.md](../../RESEARCH.md) §1), di-deploy oleh factory ke sub-akun `<slug>.<factory>` ([factory.md](./factory.md)). Kontrak ini memegang:

1. **State NFT standar** — token, metadata, approval, enumerasi (NEP-171/177/178/181/199) via derive `near-sdk-contract-tools`.
2. **Royalti per-kontrak** — `royalty_bps` + derivasi payout map (ronde 16, DECIDED).
3. **State launchpad** — fase mint berurutan + allowlist on-chain ([ADR-008](../decisions/ADR-008-launchpad-phases.md)).

Yang **TIDAK** dimiliki dokumen ini (hanya merujuk):

| Fakta | Pemilik |
|---|---|
| Alur launchpad (create → mint → allowlist upload) | [features/launchpad.md](../features/launchpad.md) |
| Validasi payout di settlement (sisi market) | [features/payments.md](../features/payments.md) |
| Teks assert → kode user (FE) | [development/error-handling.md](../development/error-handling.md) §4 |
| Nama & payload event | [api/webhooks.md](../api/webhooks.md) |
| Storage layout Market (pembanding) | [smart-contract-security-architecture.md](../security/smart-contract-security-architecture.md) §13 + [market.md](./market.md) |

Basis tooling: **Rust + `near-sdk` + `near-sdk-contract-tools`** (derive NEP-171/177/178/181/199/297 + `Owner` + `StorageManagement`), build reproducible NEP-330 ([smart-contract-security-architecture.md](../security/smart-contract-security-architecture.md) §2b).

---

## 1. Init — `new`

```rust
#[near(contract_state)]
#[derive(PanicOnDefault)]                 // SEC-CONTRACT-002: tanpa Default → tidak ada init takeover
pub struct CollectionContract {
    // standar (derive) + field NearSea di bawah
}

#[near]
impl CollectionContract {
    /// Init satu-satunya (PanicOnDefault → memanggil tanpa init = panic).
    #[init]
    pub fn new(
        owner_id: AccountId,              // pengelola kontrak (set_phases, allowlist)
        creator_id: AccountId,            // penerima royalti (ronde 16)
        royalty_bps: u16,                 // 1..=1000 (cap 10%/token, INV-027)
        name: String,                     // NEP-177
        symbol: String,                   // NEP-177
        base_uri: Option<String>,         // NEP-177 (gateway IPFS; on-chain hanya URL + hash)
        market_id: Option<AccountId>,     // PROPOSED — id market yang di-list FE
        metadata: Option<NFTContractMetadata>, // PROPOSED — field NEP-177 tambahan (spec/icon/reference)
    ) -> Self;
}
```

| Arg | Tipe | Status | Catatan |
|---|---|---|---|
| `owner_id` | AccountId | ✅ DECIDED | MVP = akun creator (diisi factory = `predecessor`). Pengelola config launchpad; BUKAN platform. Non-custodial: platform tidak punya backdoor ([ADR-006](../decisions/ADR-006-open-collections-factory.md)). |
| `creator_id` | AccountId | ✅ DECIDED | Penerima royalti — derivasi ronde 16 (lihat §4). MVP = sama dengan `owner_id`; dipisah agar label/kolektif bisa menerima royalti berbeda dari pengelola (PROPOSED untuk nilai berbeda). |
| `royalty_bps` | u16 | ✅ DECIDED (≤1000) | Valid `1..=1000`. Bawah `1` diusulkan karena payout settlement wajib ≥1 penerima ber-amount >0 (INV-003) — payout kosong/0 ditolak market ([payments.md](../features/payments.md) §Algoritma). **PROPOSED**: nilai minimum 1 bps; kreator yang tak ingin royalti menyetel 1 bps (dust 0.01%). |
| `name`, `symbol` | String | ✅ DECIDED | NEP-177; media aset di IPFS, on-chain URL + hash (AGENTS.md Blockchain Rules). |
| `base_uri` | Option | ✅ DECIDED | NEP-177 `base_uri` (off-chain JSON per token = `<base_uri>/<token_id>.json`). |
| `market_id` | Option | ⏳ PROPOSED | Referensi market sah untuk FE/indexer. Tidak dipakai otorisasi (market terbuka — koleksi pihak ketiga tetap bisa di-list via dual verification, [ADR-002](../decisions/ADR-002-listing-two-tx.md)). Boleh `None`. |
| `metadata` | Option | ⏳ PROPOSED | Sisa field NEP-177 (`spec`, `icon`, `reference`, `reference_hash`). Default `spec = "nft-1.0.0"`. |

- Validasi init: `royalty_bps` di luar `1..=1000` → panic (map FE: `INVALID_ROYALTY`); panic init → map `CHAIN_REVERT` ([error-handling.md](../development/error-handling.md) §4).
- Tidak ada `Pausable` di kontrak koleksi (Pausable hanya Market & Factory — [smart-contract-security-architecture.md](../security/smart-contract-security-architecture.md) §4).

---

## 2. Surface standar NEP (ringkas — signature resmi di-derive)

Implementasi via derive `near-sdk-contract-tools` (bukan tulis manual) untuk NEP-171/177/178/181;
**NEP-199 tidak punya derive di `near-sdk-contract-tools` 4.0**, jadi `nft_transfer_payout` ditulis
manual mengikuti signature standar (lihat §4 "Status implementasi" untuk keputusan implementasinya).

| NEP | Method | Catatan |
|---|---|---|
| NEP-171 core | `nft_token`, `nft_transfer`, `nft_transfer_call`, `nft_resolve_transfer` | `nft_transfer` wajib `assert_one_yocto()` (FACT — [RESEARCH.md](../../RESEARCH.md) §3). |
| NEP-177 metadata | `nft_metadata` + metadata per token | Media IPFS; on-chain hanya URL + hash. |
| NEP-178 approval | `nft_approve`, `nft_revoke`, `nft_revoke_all`, `nft_is_approved`, `nft_on_approve` | Kunci listing 2-tx ([ADR-002](../decisions/ADR-002-listing-two-tx.md)); market memverifikasi sendiri, callback tidak membuat listing. |
| NEP-199 royalty | `nft_transfer_payout(receiver_id, token_id, approval_id, memo?, balance, max_len_payout?)` | Dipanggil **market** dengan attach 1 yocto + 15 Tgas (FACT); return `Payout = HashMap<AccountId, U128>` dari §4. **Diimplementasikan (TASK-003)** — lihat §4 "Status implementasi". |
| NEP-181 enumeration | `nft_total_supply`, `nft_tokens`, `nft_tokens_for_owner` | Halaman koleksi & profil (FE baca langsung RPC — MVP, [ADR-004](../decisions/ADR-004-mvp-data-layer.md)). |
| NEP-145 storage | `storage_deposit`, `storage_withdraw`, `storage_minimum_balance`, `storage_balance_of` | Pre-deposit untuk mint + allowlist (INV-019/020). |
| NEP-297 events | — | Envelope `EVENT_JSON:` satu baris; skema [api/webhooks.md](../api/webhooks.md). |

- `nft_mint` **bukan** method standar bebas — di NearSea launchpad-aware (§3). Tidak ada mint publik di luar aturan fase (INV-017).
- **`set_phases` WAJIB ada di M1 (koreksi ronde 17b).** Karena mint hanya lewat fase, koleksi tanpa fase
  **tidak bisa di-mint sama sekali**. Untuk vertical slice M1 (tanpa factory/launchpad UI), kreator
  memanggil `set_phases` **sekali** dengan **satu fase publik** (allowlist_required = false, window
  terbuka) — itu jalur mint minimum yang membuat tesis slice bisa dijalankan. Kemampuan ini **milik
  TASK-002**, bukan TASK-020 (yang menambah fase bebas + allowlist + UI).
- `max_len_payout` yang dikirim market = 10 (batas gas sehat — [RESEARCH.md](../../RESEARCH.md) §10.5); payout koleksi selalu 1 receiver, selalu muat.

---

## 3. Ekstensi NearSea — `nft_mint` (launchpad-aware)

```rust
#[near]
impl CollectionContract {
    /// Mint mengikuti fase aktif. Deposit EXACT = phase.price × quantity (INV-018).
    /// Storage mint dibayar pemicu mint (INV-019).
    #[payable]
    pub fn nft_mint(&mut self, phase_index: Option<u16>, quantity: u32) {
        // urutan checks di bawah
    }
}
```

| Arg | Tipe | Deskripsi |
|---|---|---|
| `phase_index` | Option\<u16\> | `None` = resolve fase aktif tunggal (INV-029); `Some(i)` = paksa fase i (tetap wajib fase i aktif). |
| `quantity` | u32 | 1..=`MAX_MINT_PER_CALL` (konstanta statis, §6 — INV-021). |

### Checks (urutan eksekusi, semua on-chain saat dipicu — tanpa cron, FACT)

| # | Check | Invariant | Gagal → kode user |
|---|---|---|---|
| 1 | Fase aktif: `phase_index` diberikan → fase i dalam window; `None` → resolve **satu** fase dengan `starts_at ≤ now < ends_at` (maksimum satu — INV-029) | INV-017 | `LAUNCHPAD_PHASE_INACTIVE` |
| 2 | `quantity ≤ MAX_MINT_PER_CALL` (batas statis) | INV-021 | `CHAIN_REVERT` |
| 3 | `phase.allowlist_required ⇒ allowlist(predecessor) == true` | INV-017 | `LAUNCHPAD_NOT_ALLOWED` |
| 4 | `phase.alloc_left ≥ quantity` | INV-017 | `LAUNCHPAD_ALLOCATION_EXHAUSTED` |
| 5 | `minted_by_wallet(predecessor, phase) + quantity ≤ max_per_wallet` | INV-017 | `LAUNCHPAD_MAX_PER_WALLET` |
| 6 | `attached_deposit == phase.price × quantity` (**exact match** — bukan ≥) | INV-018 | `LAUNCHPAD_PRICE_MISMATCH` |
| 7 | Storage pre-deposit minter ≥ `storage_usage` delta token+metadata (NEP-145) | INV-019 | `CHAIN_REVERT` (storage) |
| 8 | Mint: `token_id` = counter berurutan; kurangi `alloc_left`; tambah `minted_by_wallet`; mint ke `predecessor` | — | — |

- Gagal di step mana pun = revert seluruh tx → deposit kembali otomatis via rollback (INV-018; tidak ada refund manual — [features/launchpad.md](../features/launchpad.md) §Flow Mint).
- Semantik `max_per_wallet = 0`: **PROPOSED** — ditolak saat konfigurasi fase (`set_phases`), sehingga 0 tidak pernah muncul di state. Alternatif (0 = tanpa batas) ditolak karena ambigu dan justru membuka mint tak terbatas ([features/launchpad.md](../features/launchpad.md) §Edge mendelegasikan semantik final ke dokumen ini).
- Batas window fase (inklusif/eksklusif): **DECIDED (ronde 19, saat implementasi TASK-002)** `[starts_at, ends_at)` — `starts_at` inklusif, `ends_at` eksklusif; fase berdampingan (`next.starts_at == cur.ends_at`) valid, tidak dianggap overlap. Diuji di dua sisi batas (`test_mint_rejected_before_window_opens`, `test_mint_rejected_at_window_end_exclusive`).
- Waktu = **u64 nanodetik** (selaras `env::block_timestamp()`); FE konversi ke ISO-8601 hanya untuk tampilan.
- `phase_index: None` dengan **lebih dari satu fase aktif** (mustahil bila `set_phases` dijaga) → panic `CONFLICT_PHASE_OVERLAP`, bukan memilih diam-diam (INV-029).

### Event yang di-emit `nft_mint`

| Event | Standard | Trigger | Data (SSOT field-level: [webhooks.md](../api/webhooks.md)) |
|---|---|---|---|
| `nft_mint` | `nep171` 1.0.0 | tiap mint sukses | `owner_id`, `token_ids[]` |
| `launchpad_mint` | `x-nearsea-market` 1.0.0 | tiap mint sukses | `collection`, `phase_index`, `account_id`, `token_ids[]`, `price_yocto` |

- **Bentuk `data` = array** (`[{…}]`) di semua event custom `x-nearsea-market`, sesuai [webhooks.md](../api/webhooks.md). Catatan implementasi (ronde 19): derive `near_sdk_contract_tools::event` memancarkan `data` sebagai **objek**, jadi kontrak memakai helper envelope `SingleEvent<T>` (di `contract/src/lib.rs`) yang membungkus payload ke array satu entri. Helper ini dipakai ulang kontrak market/factory agar bentuk event seragam.
- Event `nft_mint`/`nft_transfer` NEP-171 datang dari derive `NonFungibleToken`; envelope-nya `EVENT_JSON:` satu baris (NEP-297).

### `launchpad_phase_start` (lazy, tanpa cron)

- Di-emit pada **transisi fase pertama kali terdeteksi** oleh panggilan state-changing (`nft_mint` yang me-resolve fase baru): kontrak membandingkan `emitted_phase_index` tersimpan vs fase yang di-resolve; bila lebih maju → emit `launchpad_phase_start` (data: `collection`, `phase_index`, `name`, `price_yocto`, `allocation`, `max_per_wallet`, `allowlist_required`) lalu update penanda. **PROPOSED** mekanisme lazy-nya (nama event & payload kanonik — [webhooks.md](../api/webhooks.md)).
- Konsekuensi tanpa cron: fase yang tidak pernah disentuh tidak memancarkan event — indexer/FE mend derivasi status dari window fase (selaras pola lazy [features/marketplace.md](../features/marketplace.md) §Expiry Sweep).

---

## 4. Royalti — penyimpanan & derivasi payout (ronde 16)

### Penyimpanan

- **Per-kontrak** (dipilih, bukan per-token): field `royalty_bps: u16` + `creator_id: AccountId` di state kontrak.
- **Alasan memilih contract-level, bukan `TokenMetadata.extra`**: (1) derivasi ronde 16 seragam untuk semua token (per-token override = fase 2); (2) `extra` adalah string bebas yang harus di-parse — rawan divergensi; (3) `nft_transfer_payout` membutuhkan pembacaan cepat deterministik.
- Per-token override royalti = **fase 2** (⏳ open-by-design, bukan MVP).
- `TokenMetadata.extra` tetap boleh dipakai untuk trait non-royalti (IPFS JSON).

### Derivasi payout map (floor ke yocto)

```rust
/// Payout NEP-199 untuk satu token pada `balance` (harga wajar token).
/// INV-003: payout ≥ 1 receiver, amount > 0 → royalty_bps ≥ 1 (lihat §1).
fn royalty_payout(&self, balance: U128) -> Payout {
    let amount = u128::from(balance.0) * self.royalty_bps as u128 / 10_000; // floor
    // amount == 0 hanya mungkin bila `balance` di bawah granularitas rate (mis. < 20 yocto
    // di 500 bps). Entri TETAP dikembalikan apa adanya — kontrak tidak menyembunyikan
    // penerima dust; market yang menolaknya lewat validasi INV-003 (`∀p.amount > 0`)
    // pada payout FINAL. Lihat §4 "Status implementasi" untuk alasan lengkap.
    Payout::from([(self.creator_id.clone(), U128(amount))])
}
```

- `royalti_token = rate × basis` (floor per token) — definisi `basis(token)` (ronde 16, DECIDED — SSOT [features/payments.md](../features/payments.md) §Algoritma Merge):
  1. harga listing aktif token itu di market, bila ada;
  2. bila tidak → **harga mint fase** tempat token di-mint;
  3. bila tidak keduanya → sisa harga bundle dibagi rata (floor) ke token yang belum punya basis.
- Aturan 2 & 3 dieksekusi **market** (pre-validasi `create_bundle`); kontrak koleksi menyediakan data basis via dua view PROPOSED di §5.
- Validasi `Σroyalti + fee ≤ harga_bundle` = bagian pre-validasi `create_bundle` (INV-025) di market ([market.md](./market.md) §Callbacks); kegagalan → `CONFLICT_PRE_VALIDATE_FAILED`.
- Cap royalti **per token** 10% (INV-027); agregat bundle = penjumlahan tanpa cap; penerima unik hasil merge ≤10 (INV-021).

### Status implementasi (TASK-003, ronde 21)

`nft_transfer_payout` + `royalty_payout` ada di `contract/src/lib.rs`. Keputusan implementasi:

| Hal | Perilaku yang dipilih | Alasan |
|---|---|---|
| Otorisasi | Sama dengan `nft_transfer`: `approval_id` → jalur approval NEP-178; tanpa `approval_id` → jalur owner | NEP-199 "menerima semua argumen `nft_transfer`"; market selalu mengirim `approval_id` listing |
| Tipe `approval_id` | `Option<u64>` di ABI (sesuai [market.md](./market.md) §Skema Argumen), dipetakan ke `u32` internal derive NEP-178 | Id di luar rentang `u32` tidak mungkin valid → diperlakukan sebagai "bukan approval" (jalur owner), bukan panic |
| 1 yocto | `assert_one_yocto()` | NEP-199 + SEC-CONTRACT-001 |
| `max_len_payout` | `Some(n)` dengan `payout.len() > n` → panic `CHAIN_REVERT`; `None` = tanpa plafon | NEP-199 mensyaratkan panic bila payout melebihi plafon; payout koleksi selalu 1 penerima sehingga `max_len_payout = 10` dari market selalu lolos |
| Aritmetika | `checked_mul` → `/ 10_000` (floor); overflow → `CHAIN_REVERT` | SEC-CONTRACT-005 (checked math) |
| `amount == 0` | Tetap dikembalikan sebagai satu entri ber-amount `0` (tidak dihilangkan) | INV-003 divalidasi market pada payout **final**; kontrak tidak menyembunyikan penerima dust |
| Pesan panic | Semua jalur gagal memakai kode registry — termasuk kegagalan transfer dari derive: `format!("{}: {e}", err::CHAIN_REVERT)` | [error-handling.md](../development/error-handling.md) §4: pemetaan FE tidak boleh bergantung pada teks pesan library |
| Gas | Tanpa XCC, tanpa tulis state baru | Terukur **~1,70 Tgas** host gas di unit test (batas bawah — tanpa gas CPU wasm); anggaran market 15 Tgas, angka final di sandbox TASK-006 |

- **Konsekuensi dust (`amount == 0`)**: karena `amount == 0` hanya muncul bila `balance` di bawah
  granularitas rate (mis. `< 20` yocto di 500 bps), dan market menolak payout ber-amount `0`
  (`∀p.amount > 0`, INV-003), token seperti itu **tidak bisa di-settle** — ditolak + refund. Ini
  fail-closed dan praktis tak terjangkau (harga NFT tidak pernah se-fraksi itu). Alternatif
  "hilangkan entri" menghasilkan payout **kosong**, yang juga ditolak market (`1 ≤ len`), jadi
  hasilnya sama; memilih mengembalikan entri apa adanya lebih sederhana (tanpa logika filter)
  dan membuat dust terlihat, bukan tersembunyi.

Bukti unit (11 test): perpindahan kepemilikan + payout kreator, batas ≤10% untuk 4 rate × 6 basis, dust (`19` → `0`, `20` → `1` di 500 bps), tepat 10% di cap, turunan dari konfigurasi level kontrak, wajib 1 yocto, penolakan pengirim tanpa approval, token tidak ada, `max_len_payout` terlalu kecil, dan approval lama invalid setelah transfer (INV-011).

- **Belum dibuktikan di tiket ini:** TC-003 versi sandbox — yaitu validasi payout di sisi **market** (≥1 penerima, Σ ≤ harga−fee, sisa ≤1 yocto, refund saat invalid) dan angka gas penuh (butuh dua kontrak). Sama seperti TC-001 di TASK-002, bagian yang bisa dibuktikan di unit sudah dibuktikan di unit.

---

## 5. Surface launchpad (config) + view

### Method config (owner-only — `owner_id`, MVP = creator)

```rust
#[payable] // menerima storage pre-deposit (NEP-145) utk state fase
pub fn set_phases(&mut self, phases: Vec<PhaseConfig>);            // replace-all (ubah fase belum mulai)

#[payable] // menerima storage pre-deposit utk baris allowlist
pub fn allowlist_add(&mut self, phase_index: u16, entries: Vec<AllowlistEntry>);

/// PROPOSED — data basis royalti utk market (INV-025 pre-validation, ronde 16):
pub fn mint_price_of(&self, token_id: String) -> Option<U128>;     // harga mint fase token (None bila bukan launchpad)
pub fn royalty_config(&self) -> RoyaltyConfig;                     // { receiver: AccountId, bps: u16 }
```

```rust
pub struct PhaseConfig {        // PROPOSED nama field final saat implementasi
    pub name: String,
    pub price_yocto: U128,      // string yoctoNEAR di JSON
    pub allocation: u32,        // ≥ 1 (PROPOSED: 0 ditolak saat config)
    pub max_per_wallet: u32,    // ≥ 1 (PROPOSED: 0 ditolak saat config)
    pub allowlist_required: bool,
    pub starts_at: u64,         // Unix NANOSECONDS (u64)
    pub ends_at: u64,           // Unix NANOSECONDS (u64)
}

pub struct AllowlistEntry {
    pub account_id: AccountId,
    pub max_mints: Option<u32>, // override per-baris (CSV `account_id[, max_mints]`);
                                // efektif = min(max_per_wallet, max_mints bila ada)
}
```

Validasi `set_phases` (INV-029 + [features/launchpad.md](../features/launchpad.md) §Wizard):

| Check | Gagal → kode user |
|---|---|
| `phases` non-kosong; `ends_at > starts_at` per fase; harga fase > 0 | `INVALID_PRICE` (harga) / `CHAIN_REVERT` (struktur) |
| Fase berurutan, tidak overlap (`next.starts_at < cur.ends_at` dgn window `[starts_at, ends_at)`) | `CONFLICT_PHASE_OVERLAP` |
| `allocation ≥ 1`, `max_per_wallet ≥ 1` | `CHAIN_REVERT` (PROPOSED — lihat §3) |
| Loop konfigurasi terikat statis (jumlah fase dibatasi konstanta §6) | INV-021 |
| Fase yang sudah `ACTIVE`/`ENDED` tidak boleh di-replace (PROPOSED — hanya fase SCHEDULED yang boleh diubah) | `CHAIN_REVERT` |

Validasi `allowlist_add`:

| Check | Invariant | Gagal → |
|---|---|---|
| `entries.len() ≤ MAX_ALLOWLIST_BATCH` (statis) | INV-021 | `CHAIN_REVERT` |
| `max_mints ≤ max_per_wallet` fase tsb (per baris, baris penyangkal disebut) | AC-COLL-4 | `CHAIN_REVERT` |
| Storage pre-deposit creator cukup utk baris baru (NEP-145) | INV-019/020 | `CHAIN_REVERT` (storage) |
| Idempoten: akun sudah terdaftar = no-op (update `max_mints` bila diberikan — PROPOSED) | — | — |

- Tidak ada event untuk `set_phases`/`allowlist_add` (config hanya dibaca via view; daftar event kanonik tidak memuatnya — [webhooks.md](../api/webhooks.md)).

### View launchpad — `get_launchpad`

Bentuk `get_launchpad(collection)` di [features/launchpad.md](../features/launchpad.md) = intent FE; kontrak yang menampung view = **koleksi itu sendiri** (view call tidak bisa cross-contract — FACT), jadi signature tanpa arg `collection`:

```rust
pub fn get_launchpad(&self) -> LaunchpadStatus;
pub fn allowlist_contains(&self, phase_index: u16, account_id: AccountId) -> bool;
```

```json
// get_launchpad() → LaunchpadStatus (status fase derivasi block_timestamp; nilai Ⓝ = string yocto)
{
  "phases": [
    {
      "index": 0,
      "name": "Whitelist",
      "price_yocto": "2000000000000000000000000",
      "allocation": 100,
      "alloc_left": 42,
      "max_per_wallet": 2,
      "allowlist_required": true,
      "starts_at": 1799016000000000000,
      "ends_at": 1799102400000000000,
      "status": "ACTIVE"
    }
  ],
  "active_phase_index": 0
}
```

- `status` per fase: `SCHEDULED` / `ACTIVE` / `ENDED` — derivasi dari `env::block_timestamp()` dgn window `[starts_at, ends_at)` (§3, PROPOSED).
- `allowlist_contains` = verifikasi publik keanggotaan on-chain ([ADR-008](../decisions/ADR-008-launchpad-phases.md) — set dapat diverifikasi siapa pun).
- Tidak ada view launchpad di market — penjelasan di [market.md](./market.md) §Views.

---

## 6. Konstanta

| Konstanta | Nilai | Status |
|---|---|---|
| `MAX_ROYALTY_BPS` | 1000 (10% per token) | ✅ DECIDED (INV-027) |
| `MIN_ROYALTY_BPS` | 1 | ⏳ PROPOSED (konsistensi INV-003 — §1) |
| `MAX_MINT_PER_CALL` | PROPOSED — diukur sandbox (usulan 10; batas statis loop mint, INV-021) | ⏳ PROPOSED |
| `MAX_ALLOWLIST_BATCH` | PROPOSED — diukur sandbox (usulan 200 baris/call; INV-021) | ⏳ PROPOSED |
| `MAX_PHASES` | PROPOSED (usulan 20; membatasi loop konfigurasi & `get_launchpad`, INV-021) | ⏳ PROPOSED |
| `GAS_FOR_MINT` | PROPOSED — diukur; mint tanpa XCC (royalti dihitung lokal) | ⏳ PROPOSED |
| `NO_DEPOSIT` | 0 (mint/config tidak menerima deposit selain harga fase + storage) | ✅ |

- **Cross-contract call keluar**: kontrak koleksi **tidak melakukan XCC di MVP** (FACT by design — royalti dihitung lokal; transfer settlement dipicu market). Gas 15 Tgas yang di-attach market ke `nft_transfer_payout` (FACT) wajib diuji cukup untuk jalur payout (TC-002/TC-003).
- Semua konstanta PROPOSED diukur di sandbox sebelum rilis (gate CI [ci-cd.md](../development/ci-cd.md)).

---

## 7. Storage-key prefix design (SEC-CONTRACT-008)

```rust
#[derive(BorshStorageKey)]
pub enum StorageKey {
    // --- NearSea extension (dimiliki dokumen ini) ---
    Phases,             // Vector<Phase> — berurutan; index = urutan fase (INV-029)
    Allowlist,          // LookupMap<AllowlistKey, Option<u32>>; AllowlistKey = (u16, AccountId)
                        //   inner key di-hash (pola by_owner_id — RESEARCH.md §10.2)
    MintedByWallet,     // LookupMap<MintKey, u32>; MintKey = (u16, AccountId) — inner hashed
    NextTokenId,        // u64 counter (token_id berurutan "0","1",…)
    EmittedPhaseIndex,  // penanda lazy `launchpad_phase_start` (§3)
    MintPriceByToken,   // LookupMap<String, U128> — basis royalti ronde 16 utk `mint_price_of`

    // --- Prefix internal NEP (dikelola derive near-sdk-contract-tools) ---
    // TokenMetadata, TokensPerOwner, Approvals (NEP-171/177/178/181),
    // StorageDeposits (NEP-145)
}
```

- Prefix & struktur **tidak boleh berubah sembarangan saat upgrade** — didokumentasikan ulang (versi layout) di setiap release (SEC-CONTRACT-008; pola migrate `#[init(ignore_state)]` — [smart-contract-security-architecture.md](../security/smart-contract-security-architecture.md) §2/§14).
- Storage semua map tumbuh dibayar user: mint → minter (INV-019), allowlist → pre-deposit creator (INV-019/020).
- Nama enum = desain; literal byte prefix mengikuti derive Borsh (diskriminasi enum) — yang mengikat adalah **urutan & makna entri** di atas.

---

## 8. Pemetaan method → event → error (ringkas)

| Method | Auth | Event | Kode error utama (map FE — [error-handling.md](../development/error-handling.md) §4) |
|---|---|---|---|
| `new` | init sekali (PanicOnDefault) | — | `INVALID_ROYALTY`, `CHAIN_REVERT` |
| `nft_mint` | siapa pun yang lolos validasi | `nft_mint` + `launchpad_mint` (+ lazy `launchpad_phase_start`) | `LAUNCHPAD_PHASE_INACTIVE`, `LAUNCHPAD_NOT_ALLOWED`, `LAUNCHPAD_ALLOCATION_EXHAUSTED`, `LAUNCHPAD_MAX_PER_WALLET`, `LAUNCHPAD_PRICE_MISMATCH`, `CHAIN_REVERT` (storage) |
| `set_phases` | owner | — (tanpa event) | `CONFLICT_PHASE_OVERLAP`, `INVALID_PRICE`, `CHAIN_REVERT` |
| `allowlist_add` | owner | — (tanpa event) | `CHAIN_REVERT` (batch/storage/baris > max_per_wallet) |
| `nft_transfer_payout` | market (approval NEP-178) atau owner | `nft_transfer` (standar, oleh transfer) | `CHAIN_REVERT` (assert 1 yocto; `max_len_payout` < jumlah penerima; overflow aritmetika) — payout divalidasi market: [payments.md](../features/payments.md) |
| `storage_deposit` / `storage_withdraw` | pemilik saldo (`account_id` bila menyetor untuk orang lain) | — | `CHAIN_REVERT` |
| View (`get_launchpad`, `allowlist_contains`, `mint_price_of`, `royalty_config`, NEP-181) | publik, gratis (RPC read) | — | — |

- Pemetaan FE dilakukan **mencocokkan tipe error**, bukan substring teks panic ([error-handling.md](../development/error-handling.md) §4; teks assert kontrak stabil-nya kode, bukan pesannya).
- Tidak ada kode error baru dari kontrak ini di luar registry §3 [error-handling.md](../development/error-handling.md).

## Status

- Init args, `creator_id` contract-level, derivasi royalti ronde 16, layout fase/allowlist, prefix storage — **di-pertanggungjawabkan dokumen ini** (reference implementasi).
- Nilai bertanda PROPOSED/⏳ = final saat implementasi + diukur sandbox; tidak boleh dianggap keputusan bisnis.
- Invariant relevan: INV-017, INV-018, INV-019, INV-021, INV-027, INV-029 ([smart-contract-invariants.md](../security/smart-contract-invariants.md)); requirement SEC-CONTRACT-002/008/009/010/011.

### Status implementasi (ronde 21 — TASK-002 + TASK-003)

`contract/src/lib.rs` sudah mengimplementasikan **§1–§5, §7**: init, surface NEP-171/177/178/181 via
derive `NonFungibleToken`, NEP-145 storage, event NEP-297, `nft_mint` launchpad-aware, `set_phases`,
`allowlist_add`, `get_launchpad`, `allowlist_contains`, `royalty_config`, `market_id`, dan
**`nft_transfer_payout` NEP-199 + derivasi payout §4** (TASK-003). Bukti: 42 unit test di crate
(fmt + clippy `-D warnings` bersih, wasm ter-build 272 KB dengan `nft_transfer_payout` di ABI).

Belum diimplementasikan (sengaja, bukan kelalaian):

| Bagian | Milik | Catatan |
|---|---|---|
| `mint_price_of` (basis royalti bundle) | **TASK-010** | View PROPOSED; dipakai pre-validasi bundle, bukan slice M1. |
| Fase bebas penuh + `Pausable` + `MAX_FEE_BPS` | **TASK-020** | Tiket ini cukup satu fase publik untuk slice. |
| Kalibrasi konstanta §6 (`MAX_MINT_PER_CALL`, `MAX_ALLOWLIST_BATCH`, `MAX_PHASES`, `GAS_FOR_MINT`) + gas payout 15 Tgas | **TASK-006** | Nilai saat ini = usulan dokumen; diukur di sandbox. |
| Deploy ke testnet | butuh **persetujuan user** | [git-workflow.md](../development/git-workflow.md) §3. |
