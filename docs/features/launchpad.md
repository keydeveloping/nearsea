# Feature — Launchpad (Create Collection & Mint per Phase)

> Launchpad ala OpenSea Studio: creator mendefinisikan koleksi + fase mint sendiri via factory (ADR-008). Allowlist = set on-chain; validasi mint dieksekusi on-chain saat dipicu — tanpa cron.

## Objective

Kreator meluncurkan koleksi NFT **tanpa izin/kurasi platform**: pilih slug, isi metadata, set royalti (≤ 10%), definisikan **berapa pun fase mint** (tiap fase: harga, alokasi, allowlist opsional, window waktu), lalu deploy kontrak koleksi via factory. Minter mint mengikuti aturan fase yang tervalidasi on-chain. Koleksi baru langsung tampil di discovery (open collections — [ADR-006](../decisions/ADR-006-open-collections-factory.md)); supply bebas tanpa cap platform (✅ DIPUTUSKAN — [ADR-008](../decisions/ADR-008-launchpad-phases.md)).

**Non-goal MVP**: lazy minting = **M3** (tidak di MVP). Auction = fase 2 ([marketplace.md](./marketplace.md)).

## Preconditions

| Aktor | Syarat |
|---|---|
| Creator | Wallet terhubung; saldo Ⓝ untuk **storage deploy** (kontrak koleksi + set allowlist) — creator membayar storage-nya sendiri (NEP-145, INV-020). |
| Minter | Wallet terhubung; deposit = harga fase (**storage mint dibayar pemicu mint** — INV-019). |
| Platform | Factory sudah ter-deploy; koleksi = sub-akun `<slug>.<factory>` (contoh: `punks.nearsea.testnet` — [environments.md](../deployment/environments.md)). |

## Flow — Create Collection (wizard `/create`)

```text
1. SLUG      → creator pilih slug → identitas koleksi = sub-akun <slug>.<factory>
               (cek ketersediaan; slug terpakai → pilih lain — lihat Error/Edge Cases)
2. METADATA  → nama, simbol, deskripsi, media (media = IPFS; on-chain hanya URL + hash)
3. ROYALTI   → royalty_bps ≤ 1000 (cap 10%/token — INV-027); payout ≤ 10 receiver,
               sisa pembulatan ≤ 1 yocto (validasi payout mengikuti [nft-collection.md](../contracts/nft-collection.md) §Royalti)
4. PHASES    → berapa pun fase (✅ DIPUTUSKAN ADR-008); per fase:
               name · price_yocto · allocation · max_per_wallet ·
               allowlist_required · starts_at / ends_at
               Validasi wizard: ends_at > starts_at; fase berurutan TIDAK overlap (INV-029)
5. ALLOWLIST → per fase allowlist_required: upload CSV → set on-chain (lihat § Upload Allowlist)
6. DEPLOY    → Review & Deploy → creator sign create_collection di factory
               (deposit = storage koleksi + set allowlist — NEP-145, creator bayar)
               State: DRAFT → DEPLOYING → LIVE; event factory_collection_created
               → koleksi langsung tampil di discovery (tanpa kurasi di muka)
```

Argumen `create_collection` (skema kanonik — [02-product-requirements.md](../02-product-requirements.md) § CREATE COLLECTION; detail lengkap di [factory.md](../contracts/factory.md)):

```json
{
  "name": "Genesis",
  "symbol": "GNS",
  "royalty_bps": 500,
  "supply": { "kind": "open" },
  "phases": [
    {
      "index": 0,
      "name": "Whitelist",
      "price_yocto": "2000000000000000000000000",
      "allocation": 100,
      "max_per_wallet": 2,
      "allowlist_required": true,
      "starts_at": 1799016000000000000,
      "ends_at": 1799102400000000000
    }
  ]
}
```

- Semua nilai Ⓝ = **string yoctoNEAR (u128)** — bukan number.
- Royalti > 10% ditolak dua lapis: validasi UI **dan** kontrak (`INVALID_ROYALTY` — AC-COLL-2).

## Flow — Mint per Phase

```text
1. User buka halaman mint koleksi → FE baca view get_launchpad(collection)
   (fase, harga, alokasi tersisa, allowlist_required, window)
2. FE tampilkan fase yang berlaku; fase whitelist → FE cek keanggotaan on-chain
   (set allowlist publik — bisa diverifikasi siapa pun, ADR-008)
3. User pilih jumlah → sign nft_mint — deposit EXACT = price × quantity
   (+ storage mint ditanggung pemicu mint — INV-019)
4. Kontrak validasi ulang ON-CHAIN SAAT DIPICU (tanpa cron — FACT):
   fase aktif (maksimum satu fase ACTIVE — INV-029)
   → minter ∈ set allowlist (bila allowlist_required)
   → alloc_left ≥ quantity
   → jumlah mint ≤ max_per_wallet            (semua = INV-017)
   → attached_deposit == price × quantity    (INV-018)
   → storage cukup                           (INV-019)
5. Sukses → token ter-mint ke wallet minter; event launchpad_mint
   Gagal   → tx revert; deposit kembali otomatis via rollback tx (deposit tidak hangus)
```

- State machine fase: `SCHEDULED → ACTIVE → ENDED`; hanya satu `ACTIVE` pada satu waktu (INV-029) — mencegah double-mint lintas fase.
- FE menampilkan status fase dari **baca lokal** (`starts_at`/`ends_at` vs now) tanpa menunggu tx; keputusan akhir tetap milik kontrak.

## Flow — Upload Allowlist

1. Pada fase `allowlist_required`, creator upload **CSV** (`account_id[, max_mints]`) di wizard.
2. Validasi pra-upload: baris duplikat ditolak; `max_mints ≤ max_per_wallet` fase tsb per baris — bila dilebihi, tolak dengan **pesan baris yang salah** (AC-COLL-4).
3. Upload ke set on-chain **per batch** (ukuran batch = ⏳ open-by-design — mengikuti batas gas 300 Tgas/call); sisa upload ulang batch berikutnya.
4. Storage set dibayar **pre-deposit creator** (NEP-145) — bila set tumbuh melebihi deposit, creator menambah deposit; kontrak tidak menanggung storage (INV-019/020).
5. Set bersifat idempoten per anggota (menambah akun yang sudah terdaftar = no-op).

## UI

- **`/create`** — wizard Create Collection: step Metadata → Supply & Royalty → Mint phases (+ upload allowlist) → Review & Deploy. Guard `wallet`. Wireframe: [04-ux-ui-spec.md](../04-ux-ui-spec.md) § Halaman 5.
- **`/mint/:collection`** — halaman mint per koleksi: fase aktif, harga, countdown, sisa alokasi, status allowlist wallet, tombol Mint; masuk dari halaman koleksi → tab Mint. Rincian state (loading/empty/error/pending) mengikuti [04-ux-ui-spec.md](../04-ux-ui-spec.md) § State per Halaman.
- Copy UI bahasa Inggris via i18n — kunci kanonik (`create.*`, `mint.*`, `errors.*`) ada di [02-product-requirements.md](../02-product-requirements.md) § Spec Engineering.

## Contract / API

> Nama method kanonik dipakai konsisten di semua dokumen; skema lengkap (init args, layout, konstanta) dimiliki dokumen kontrak.

```text
factory:  create_collection(metadata, royalty_bps, supply, phases[], allowlists[])  — payable (storage creator)
koleksi:  nft_mint(collection, phase_index?, quantity)                              — payable (exact = price × qty + storage)
views:    get_launchpad(collection) — fase, harga, alokasi, allowlist, window
storage:  storage_deposit / storage_withdraw (NEP-145)
```

- Skema lengkap: [nft-collection.md](../contracts/nft-collection.md) (metadata, royalti, fase, set allowlist, `nft_mint`) dan [factory.md](../contracts/factory.md) (`create_collection`, sub-akun `<slug>.<factory>`).
- Event: `factory_collection_created` (factory) · `launchpad_phase_start` · `launchpad_mint` (NFT collection contract) — skema field dimiliki [webhooks.md](../api/webhooks.md).
- MVP tanpa API discovery: FE baca langsung RPC view; endpoint discovery = ★ fase 2 (ADR-004, [endpoints.md](../api/endpoints.md)).

## Error Cases

> Kode = registry [error-handling.md](../development/error-handling.md) §3; pesan = kanal registry.

| Kasus | Kode | Pesan |
|---|---|---|
| Fase tidak aktif / belum mulai / sudah berakhir | `LAUNCHPAD_PHASE_INACTIVE` | "Mint belum dibuka" |
| Minter di luar allowlist fase | `LAUNCHPAD_NOT_ALLOWED` | "Kamu belum masuk allowlist fase ini" |
| Alokasi fase habis | `LAUNCHPAD_ALLOCATION_EXHAUSTED` | "Alokasi fase ini habis" |
| Melebihi `max_per_wallet` | `LAUNCHPAD_MAX_PER_WALLET` | "Sudah mencapai batas mint per wallet" |
| Deposit ≠ harga fase (× qty) | `LAUNCHPAD_PRICE_MISMATCH` | "Jumlah tidak sesuai harga fase" |
| Dua fase window overlap (konfigurasi) | `CONFLICT_PHASE_OVERLAP` | "Phases cannot overlap" |
| Royalti > 1000 bps | `INVALID_ROYALTY` | "Royalti tidak valid" |
| Harga fase tidak valid (input wizard / preview FE) | `INVALID_PRICE` | — |

- Kegagalan `nft_mint` = revert tx — deposit kembali via rollback (INV-018); tidak ada refund manual.

## Security

| Kontrol | Isi | Rujukan |
|---|---|---|
| INV-017 | Mint ditolak bila: tak ada fase aktif, alokasi habis, melebihi max/wallet, atau minter tidak terdaftar di allowlist phase | [smart-contract-invariants.md](../security/smart-contract-invariants.md) |
| INV-018 | `attached_deposit == price × quantity` (exact-match; selisih → revert) | sama |
| INV-019 | Storage mint dibayar pemicu mint; storage allowlist pre-deposit creator — kontrak tidak menanggung | sama |
| INV-029 | Fase berurutan, tidak overlap — maksimum satu fase aktif (dicek saat `nft_mint`) | sama |
| SEC-CONTRACT-009 | Launchpad: validasi fase/allowlist/alokasi/exact deposit (✅ DECIDED, P0, MVP) | [security-requirements.md](../security/security-requirements.md) |
| Allowlist on-chain | Set on-chain = verifikasi publik, tanpa titik percaya server (✅ ADR-008) | [ADR-008](../decisions/ADR-008-launchpad-phases.md) |
| Tanpa cron | Validasi fase dieksekusi on-chain saat dipicu, bukan pekerjaan background | [backend-architecture.md](../architecture/backend-architecture.md) |

## Edge Cases

| Kasus | Perlakuan |
|---|---|
| Waktu mint tepat di `starts_at` / tepat di `ends_at` | **DECIDED (ronde 19, implementasi TASK-002)** — window `[starts_at, ends_at)`: `starts_at` inklusif, `ends_at` eksklusif (SSOT: [contracts/nft-collection.md](../contracts/nft-collection.md) §3) |
| `allocation = 0` | **DECIDED (ronde 19, implementasi TASK-002)** — ditolak saat konfigurasi fase (`set_phases` → `CHAIN_REVERT`); nilai 0 tidak pernah masuk state |
| `max_per_wallet = 0` | **DECIDED (ronde 19, implementasi TASK-002)** — ditolak saat konfigurasi fase (`set_phases` → `CHAIN_REVERT`); 0 = tanpa batas **tidak** dipakai karena ambigu |
| Slug sudah dipakai | Deploy ditolak (sub-akun `<slug>.<factory>` sudah ada) → creator pilih slug lain; aturan charset/panjang slug = ⏳ open-by-design |
| Baris allowlist `max_mints > max_per_wallet` | Upload ditolak dengan pesan baris yang salah (AC-COLL-4) |
| Mint qty > alokasi tersisa | Ditolak (INV-017, `LAUNCHPAD_ALLOCATION_EXHAUSTED`) |

## Acceptance Criteria

- Definisi "selesai" fitur ini = AC lolos di [acceptance-criteria.md](../testing/acceptance-criteria.md):
  - § Create Collection (launchpad) — **AC-COLL-1..4** (deploy via factory, royalti > 10% ditolak, phase overlap ditolak, allowlist vs max/wallet).
  - § Mint (launchpad) — **AC-MINT-1..5** (mint allowlist sukses, bukan allowlist ditolak, fase belum mulai/berakhir ditolak, deposit ≠ harga ditolak, alokasi/max-wallet ditolak).
