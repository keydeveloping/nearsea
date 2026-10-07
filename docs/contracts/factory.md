# Contract Reference — Factory (Deploy Koleksi Launchpad)

> Reference implementasi (ronde 16). Perilaku/settlement = SSOT di [features/marketplace.md](../features/marketplace.md); dokumen ini memiliki signature, layout state, konstanta, dan pemetaan method→event→error.

## Peran & batas dokumen

Kontrak **Factory** men-deploy kontrak NFT Collection ke sub-akun `<slug>.<factory>` — launchpad ala OpenSea Studio, open-minting tanpa kurasi di muka ([ADR-006](../decisions/ADR-006-open-collections-factory.md); fase & allowlist = [ADR-008](../decisions/ADR-008-launchpad-phases.md)). Satu akun factory untuk semua koleksi; siapa pun boleh deploy (open factory) selama deposit storage cukup.

| Fakta | Pemilik (dokumen ini hanya merujuk) |
|---|---|
| Alur create collection & mint | [features/launchpad.md](../features/launchpad.md) |
| Validasi fase/royalti di dalam koleksi (setelah deploy) | [nft-collection.md](./nft-collection.md) |
| Nama & payload event | [api/webhooks.md](../api/webhooks.md) |
| Registry kode error | [development/error-handling.md](../development/error-handling.md) §3 |
| Penamaan akun per environment | [deployment/environments.md](../deployment/environments.md) §Konvensi penamaan |

- Factory memakai **Pausable** (deploy koleksi baru dihentikan saat insiden — INV-022, konsekuensi [ADR-006](../decisions/ADR-006-open-collections-factory.md)); pause tidak memblokir method baca.
- Kontrak koleksi yang di-deploy factory = kontrak standar [nft-collection.md](./nft-collection.md); factory **tidak** punya backdoor ke kontrak koleksi setelah deploy (non-custodial — koleksi milik creator).

---

## 1. Init — `new`

```rust
#[near(contract_state)]
#[derive(PanicOnDefault)]                 // SEC-CONTRACT-002
pub struct FactoryContract { /* state §6 */ }

#[near]
impl FactoryContract {
    #[init]
    pub fn new(
        owner_id: AccountId,          // Platform Owner (MVP single-key; mainnet DAO — ADR-013)
        market_id: AccountId,         // id market kanonik (diteruskan ke init koleksi sebagai market_id)
        wasm_code: Option<Vec<u8>>,   // ⏳ PROPOSED — hanya jika pola factory-stored (§2)
    ) -> Self;

    /// Upgrade path koleksi masa depan: deploy ulang kode koleksi ke sub-akun eksisting.
    /// PROPOSED — tidak dipakai di MVP (koleksi kreator dikelola sendiri).
    #[payable]
    pub fn update_collection_code(&mut self, slug: String, wasm_code: Vec<u8>); // owner-only
}
```

| Arg | Status | Catatan |
|---|---|---|
| `owner_id` | ✅ DECIDED | Owner pattern (pause/unpause, config). |
| `market_id` | ✅ DECIDED (nilainya ⏳ per env) | Diteruskan ke `CollectionContract::new(market_id)` ([nft-collection.md](./nft-collection.md) §1). Nilai per environment = [environments.md](../deployment/environments.md): `market.dev.nearsea.testnet` (dev) / `market.nearsea.testnet` (testnet). |
| `wasm_code` | ⏳ PROPOSED | Hanya dipakai bila pola penyimpanan kode = factory-stored (§2). |

---

## 2. Wiring deploy — `create_collection`

### Pilihan penyimpanan kode (PROPOSED)

| Pola | Mekanisme | Trade-off |
|---|---|---|
| **Embedded (dipilih PROPOSED)** | `include_bytes!("../res/collection.wasm")` saat build factory | Sederhana, satu artefak; ukuran factory naik; update kode koleksi = redeploy factory |
| Factory-stored | Kode di-upload sekali ke state factory (arg `wasm_code` di `new` / method owner), lalu dipakai ulang per deploy | Update kode tanpa redeploy factory; state factory membengkak (storage dibayar platform) |
| Global contract (by hash/account) | Deploy sekali sebagai global contract; koleksi mereferensikan kode | Paling hemat per-koleksi; UX deploy baru; dua variasi hash/account punya konsekuensi update berbeda |

> Pola final ⏳ open-by-design — dipilih saat implementasi TASK-012; ketiganya tidak mengubah signature `create_collection` di bawah. Embedded = default usulan (paling sedikit moving part).

### Signature

```rust
/// Open factory — siapa pun boleh deploy (ADR-006); deposit = biaya seluruh provisioning (§2b).
/// Pausable: ditolak saat paused (INV-022).
#[payable]
pub fn create_collection(
    &mut self,
    slug: String,                      // sub-akun = <slug>.<factory id saat ini>
    name: String,                      // → init koleksi
    symbol: String,                    // → init koleksi
    royalty_bps: u16,                  // → init koleksi (1..=1000; INVALID_ROYALTY)
    base_uri: Option<String>,          // → init koleksi
    phases: Option<Vec<PhaseConfig>>,  // → set_phases koleksi (receipt yang sama, §2a)
    allowlists: Option<Vec<AllowlistBatch>>, // → allowlist_add koleksi (batch kecil; batch lanjutan §3)
) -> Promise;

pub struct AllowlistBatch {           // PROPOSED
    pub phase_index: u16,
    pub entries: Vec<AllowlistEntry>, // ≤ MAX_ALLOWLIST_BATCH per batch (INV-021)
}
```

### 2a. Rangkaian promise (deploy wiring — bagian yang selama ini hilang)

```text
create_collection (factory, deposit = provisioning fee)
  1. validasi lokal: paused? slug valid & belum dipakai (state factory §6)?
     royalti/harga fase hanya divalidasi OLEH KOLEKSI saat init (satu otoritas — §2c)
  2. Promise::new(<slug>.<factory_id>)
       .create_account()                       // gagal bila sub-akun sudah ada (slug taken)
       .transfer(cost_account + cost_code)     // dana storage akun + kode + fase + factory state
       .deploy_contract(WASM)                  // wasm koleksi (pola §2)
       .function_call("new", args_init, 0, GAS_FOR_INIT)
       .function_call("set_phases", phases, 0, GAS_FOR_CONFIG)          // bila phases ada
       .function_call("allowlist_add", batch, 0, GAS_FOR_ALLOWLIST)     // per batch (batch kecil)
       .then(Promise::new(factory).function_call("create_collection_callback", …, 0, GAS_FOR_RESOLVE_CREATE))
```

- `args_init` = argumen `CollectionContract::new` ([nft-collection.md](./nft-collection.md) §1): `owner_id = creator_id = predecessor` (MVP), `royalty_bps`, `name`, `symbol`, `base_uri`, `market_id` (state factory), `phases`.
- Semua aksi di satu receipt batch pada sub-akun → **atomik**: `new` panic (mis. `INVALID_ROYALTY`, `CONFLICT_PHASE_OVERLAP` dari validasi koleksi) = seluruh receipt revert, sub-akun tidak jadi dibuat.
- Validasi fase/royalti **tetap dieksekusi kontrak koleksi** — factory tidak menduplikasi aturan INV-029/027 (satu otoritas validasi: koleksi; wizard FE memvalidasi lebih dulu untuk UX — [features/launchpad.md](../features/launchpad.md) §Wizard).
- `set_phases`/`allowlist_add` dipanggil factory dalam receipt yang sama → creator tidak perlu tx lanjutan untuk konfigurasi awal; upload allowlist besar = tx batch berikutnya (§3).

### 2b. Siapa membayar (formula sketch)

Creator membayar SEMUA provisioning (NEP-145 — user bayar storage-nya sendiri, INV-019/020):

```text
attached_deposit ≥
    cost_account      // biaya storage akun baru (balance minimum akun)
  + cost_code         // bytes(wasm) × STORAGE_PRICE_PER_BYTE        (≈ 10^19 yocto/byte — 1 Ⓝ/100 KB, RESEARCH.md §3)
  + cost_collection_state // entry fase + config awal di kontrak koleksi
  + cost_allowlist    // baris allowlist batch awal (bila ada)
  + cost_factory_state// entry CollectionInfo di state factory (INV-020)
```

- Kontrak menghitung kebutuhan eksak dari `wasm.len()` + jumlah fase/baris (angka final dari `storage_usage` nyata saat implementasi — ⏳ open-by-design, pola OQ-007); kurang → panic (`CHAIN_REVERT` storage) SEBELUM promise dibuat.
- Kelebihan deposit di-refund ke creator via callback (§2c).

### 2c. Failure handling & callback

```rust
#[private] // INV-013
pub fn create_collection_callback(
    &mut self,
    #[callback_result] deploy: Result<(), PromiseError>,
    slug: String, creator: AccountId, deposited: U128,
) {
    // SUKSES → emit factory_collection_created (§4)
    // GAGAL (slug taken / deploy gagal / init panic):
    //   dana sisa dari receipt yang gagal sudah kembali ke FACTORY (beneficiary caller),
    //   BUKAN otomatis ke creator → callback WAJIB me-refund sisa deposit ke `creator`
    //   (tujuan hardcoded dari argumen, bukan arbitrer — INV-014)
}
```

- Deposit yang di-transfer ke sub-akun pada langkah `transfer` tidak kembali bila receipt gagal setelahnya? Bila receipt batch revert atomik, seluruh transfer ikut revert — yang mengalir kembali ke factory adalah `attached_deposit` `create_collection`; callback meneruskannya ke creator. Jaminan: **create gagal → dana kembali penuh ke creator** (tidak ada dana menggantung di factory kecuali dust ≤1 yocto).
- Batas gas per receipt 300 Tgas (FACT): `GAS_FOR_INIT`/`GAS_FOR_CONFIG`/`GAS_FOR_ALLOWLIST`/`GAS_FOR_RESOLVE_CREATE` = konstanta terpisah, diukur sandbox (⏳ PROPOSED).

### 2d. Sub-akun & slug validation

| Aturan | Nilai | Status |
|---|---|---|
| Pola nama | `<slug>.<factory>` — contoh: `punks.nearsea.testnet`; dev namespace `<role>.dev.nearsea.testnet` (contoh: koleksi dev = `<slug>.dev.nearsea.testnet`) | ✅ DECIDED ([environments.md](../deployment/environments.md) §Konvensi penamaan) |
| Charset slug | `^[a-z0-9]([a-z0-9_-]*[a-z0-9])?$` (aturan akun NEAR: huruf kecil, digit, `-`, `_`; tidak diawali/diakhiri separator — FACT aturan AccountId) | ✅ DECIDED (aturan protokol) |
| Panjang slug | 2..=48 char (total akun ≤ 64 char — FACT; 48 menyisakan ruang suffix terpanjang; final mengikuti suffix per env) | ⏳ PROPOSED (batas numerik final) |
| Reserved slug | `market`, `dev`, `testnet`, `mainnet`, `app`, `admin`, `www`, `factory`, `nearsea` + minimal 1 entri per-environment ⏳ (daftar final saat deploy; cek juga akun eksisting on-chain) | ⏳ PROPOSED (daftar) |
| Slug sudah dipakai | Cek state factory (cepat) **dan** `create_account` gagal bila akun eksisting (otoritas akhir = protokol NEAR) | ✅ DECIDED |

- Tidak ada kurasi on-chain atas isi metadata — kualitas koleksi = badge verified + report (display-layer, [ADR-006](../decisions/ADR-006-open-collections-factory.md)); factory hanya menegakkan aturan naming & pembayaran.

---

## 3. Allowlist upload (lanjutan, per batch)

Upload awal (batch kecil) ikut `create_collection` (§2a). Batch lanjutan = method **kontrak koleksi** `allowlist_add(phase_index, entries)` ([nft-collection.md](./nft-collection.md) §5) — factory tidak terlibat setelah deploy:

| Aspek | Nilai |
|---|---|
| Ukuran batch | ≤ `MAX_ALLOWLIST_BATCH` (statis, usulan 200 baris/call — INV-021, ⏳ diukur) |
| Storage | Pre-deposit creator di **kontrak koleksi** via `storage_deposit` (NEP-145) — bila set tumbuh melebihi deposit, creator menambah; kontrak tidak menanggung (INV-019/020) |
| Gas per batch | PROPOSED — diukur (usulan ~10 Tgas + per-rows); loop terikat statis |
| Idempoten | Akun sudah terdaftar = no-op ([features/launchpad.md](../features/launchpad.md) §Upload Allowlist) |
| Verifikasi publik | `allowlist_contains(phase_index, account_id)` (view — [ADR-008](../decisions/ADR-008-launchpad-phases.md)) |

- CSV `account_id[, max_mints]` diparse di FE; baris `max_mints > max_per_wallet` ditolak sebelum upload (pesan baris yang salah — AC-COLL-4); kontrak memvalidasi ulang per baris saat `allowlist_add`.

---

## 4. Method → event

| Method | Event | `data[]` ([webhooks.md](../api/webhooks.md)) |
|---|---|---|
| `create_collection` (sukses, di `create_collection_callback`) | `factory_collection_created` | `collection` (sub-akun baru), `creator`, `royalty_bps`, `name` |
| `create_collection` (gagal) | — (tanpa event; refund via callback; FE tahu dari tx failed + pesan assert) | — |
| `pause` / `unpause` (factory) | ⏳ open-by-design — daftar kanonik [webhooks.md](../api/webhooks.md) belum memuat event pause factory; usulan: pola `market_pause` (`standard: x-nearsea-market`, `event: factory_pause`) — wajib didaftarkan dulu sebelum dipakai | — |

- Satu event per deploy; identity/dedup = `(receipt_id, event_index)` ([webhooks.md](../api/webhooks.md) §Envelope).
- `launchpad_*` events di-emit **kontrak koleksi** (bukan factory) — [nft-collection.md](./nft-collection.md) §3.

---

## 5. Error → kode user

Kode = registry [error-handling.md](../development/error-handling.md) §3. Tidak ada kode khusus factory di registry saat ini → dipetakan ke kode terdaftar terdekat; kode baru hanya boleh dipakai setelah didaftarkan di §3 (aturan registry).

| Kondisi on-chain (assert) | Kode user (mapping) | Catatan |
|---|---|---|
| Slug tidak valid (charset/panjang) | `INVALID_ACCOUNT_ID` (terdaftar, HTTP 400) | Slug = komponen AccountId; aturan NEAR — FACT |
| Slug sudah dipakai (state factory) | `CONFLICT_ALREADY_TAKEN` (terdaftar di registry §3 — ronde 16; HTTP 409) | FE wizard pre-check ketersediaan slug agar jarang sampai on-chain |
| Slug taken oleh akun di luar factory (create_account gagal) | idem | Otoritas akhir = protokol NEAR (§2d) |
| Deposit < kebutuhan provisioning | `CHAIN_REVERT` (storage) | Kalkulasi §2b |
| Royalti di luar 1..=1000 (dari init koleksi) | `INVALID_ROYALTY` (bubble dari panic koleksi) | Validasi koleksi = [nft-collection.md](./nft-collection.md) §1 |
| Fase overlap / config fase invalid (dari `set_phases` koleksi) | `CONFLICT_PHASE_OVERLAP` / `INVALID_PRICE` | idem |
| Kontrak paused (deploy baru) | `CHAIN_PAUSED` | INV-022 |
| Callback bukan dari factory | — (tidak terjadi; `#[private]` — INV-013) | |

- Kasus "slug sudah dipakai" didefinisikan perilakunya di [features/launchpad.md](../features/launchpad.md) §Edge (creator pilih slug lain); dokumen ini yang menetapkan mapping kode & titik penolakan (§2a/§2d).

---

## 6. Storage-key prefix design (SEC-CONTRACT-008)

```rust
#[derive(BorshStorageKey)]
pub enum StorageKey {
    Collections,           // UnorderedMap<String /*slug*/, CollectionInfo>
                           //   CollectionInfo = { account_id: AccountId, creator: AccountId,
                           //                      royalty_bps: u16, name: String, created_at: u64 (ns) }
                           //   ⇔ cek slug taken + indeks koleksi per factory (discovery tampil tanpa kurasi)
    CollectionsByCreator,  // LookupMap<AccountId, UnorderedSet<String /*slug*/>>
                           //   inner key di-hash (pola by_owner_id — RESEARCH.md §10.2); halaman profil creator
    WasmCode,              // PROPOSED — hanya bila pola factory-stored (§2): Vec<u8> kode koleksi
    // Pause flag & owner config = field struct root (bukan persistent collection)
}
```

- Layout **PROPOSED** (final saat implementasi; urutan enum & makna entri stabil) — dokumentasikan versi layout per release (SEC-CONTRACT-008).
- State factory tumbuh 1 entry per koleksi → dibayar dari deposit creator (`cost_factory_state`, §2b — INV-020); owner dapat menarik kelebihan via NEP-145 storage management bila derive dipakai (PROPOSED: factory pakai `StorageManagement` untuk saldo provisioning).

---

## 7. Konstanta

| Konstanta | Nilai | Status |
|---|---|---|
| `STORAGE_PRICE_PER_BYTE` | dibaca `env::storage_byte_cost()` (≈ 10^19 yocto/byte — 1 Ⓝ/100 KB) | ✅ FACT (jangan hardcode — baca dari env) |
| `GAS_FOR_INIT` | PROPOSED — diukur (init koleksi + metadata fase) | ⏳ PROPOSED |
| `GAS_FOR_CONFIG` | PROPOSED — diukur (`set_phases` dalam receipt deploy) | ⏳ PROPOSED |
| `GAS_FOR_ALLOWLIST` | PROPOSED — diukur (per batch) | ⏳ PROPOSED |
| `GAS_FOR_RESOLVE_CREATE` | PROPOSED — diukur (callback refund/event) | ⏳ PROPOSED |
| `MAX_SLUG_LEN` / `MIN_SLUG_LEN` | 48 / 2 | ⏳ PROPOSED (§2d) |
| `MAX_ALLOWLIST_BATCH` | 200 baris/call (sama dgn koleksi — [nft-collection.md](./nft-collection.md) §6) | ⏳ PROPOSED |
| `MAX_PHASES_PER_CREATE` | 20 (sama dgn `MAX_PHASES` koleksi) | ⏳ PROPOSED |
| `RESERVED_SLUGS` | daftar §2d | ⏳ PROPOSED |

- Total gas satu `create_collection` (5 sub-call + callback) **wajib diukur** dan tetap < 300 Tgas/receipt (FACT — INV-021).

## Status

- Signature `create_collection`, wiring promise, formula biaya, aturan slug & prefix storage — **di-pertanggung-jawabkan dokumen ini** (reference implementasi, ronde 16).
- Pola penyimpanan kode (§2), batas numerik (§2d, §7), mekanisme event pause factory (§4) — **PROPOSED/⏳**, final saat implementasi TASK-012 + diukur sandbox.
- Invariant relevan: INV-013 (`#[private]` callback), INV-014 (tujuan refund dari state), INV-020/021 (storage user + batas statis), INV-022 (Pausable); requirement SEC-CONTRACT-002/008/010/011.
- Terkait: [ADR-006](../decisions/ADR-006-open-collections-factory.md) · [ADR-008](../decisions/ADR-008-launchpad-phases.md) · [features/launchpad.md](../features/launchpad.md) · [nft-collection.md](./nft-collection.md) · [market.md](./market.md) · [environments.md](../deployment/environments.md) §Konvensi penamaan.
