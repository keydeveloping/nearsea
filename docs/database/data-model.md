# Data Model & Relationships

> Entitas logis dan hubungannya (di luar kebenaran on-chain).

## Entity map (final ronde 13 + audit)

```text
Account (wallet NEAR)
 │
 ├── owns many Token            (on-chain: NEP-171 owner_id)
 ├── creates many Collection    (owner_id kontrak)
 ├── creates many Listing       (seller)
 ├── makes many Offer           (buyer; maks 1 aktif per buyer per token)
 ├── creates many Bundle        (seller; berisi N Token, N ≤ 10 — INV-021)
 │
 ├── has one Profile            (alias/bio/avatar — DB kecil, signature auth)
 ├── submits many Report        (moderasi)
 │
 └── receives many PayoutSplit  (royalti — nilai dalam sales.payout jsonb)

Collection (1 kontrak NFT)
 ├── has many Token
 └── has many LaunchpadPhase   (phase mint: harga/alokasi/allowlist/waktu; allowlist on-chain)

Listing ─── resolves to ─── Sales (0..1)
Offer   ─── accepts to ──── Sales (0..1)
Bundle  ─── settles to ──── Sales (1 sale per bundle)
```

## Kebenaran sumber

| Data | Sumber kebenaran | DB indexer hanya |
|---|---|---|
| Kepemilikan token | NFT contract (`nft_token`) | cache untuk pencarian |
| Harga listing / offer | Market contract | cache + agregat |
| Penjualan | event custom **`market_sale`** (buyer, price, payout — lihat [../api/webhooks.md](../api/webhooks.md)) | riwayat permanen |
| Phase launchpad | kontrak koleksi | cache progres |
| Profil user | DB kecil (alias/bio/avatar, signature-auth); alamat & aset tetap on-chain | profile table |

---

## 1. ER diagram formal (kardinalitas + key)

Notasi: `1 ──< N` = one-to-many; `1 ── 0..1` = one-to-zero-or-one; `(PK)` primary key; `(UQ)` unique; `(FK)` foreign key; `★` = entitas chain-derived (hanya terisi fase 2); `●` = app-mutable (backup wajib); `○` = transient; `■` = append-only.

```text
Account (identitas chain, tanpa tabel wajib; `accounts(account_id)` = cache)
 │
 │ 1
 ├────< owns (0..N) ──────────────► Token ★
 ├────< creates (0..N) ────────────► Collection ★
 ├────< lists (0..N) ──────────────► Listing ★
 ├────< bids (0..N, ≤1 ACTIVE/token)► Offer ★
 ├────< sells (0..N) ──────────────► Bundle ★
 ├────< buys (0..N) ───────────────► Sale ★
 ├──── 1 ── 0..1 ──────────────────► Profile ●
 ├────< reports (0..N) ────────────► Report ●
 └────< receives (0..N) ───────────► PayoutSplit ★

Collection ★ (1 kontrak NFT, PK contract_account_id)
 │ 1 ──< N Token ★
 │ 1 ──< N LaunchpadPhase ★   (UQ collection+phase_index)
 │ 1 ── 0..1 VerifiedFlag ●    (kolom collections.verified)
 │ 1 ── 0..1 BlocklistEntry ●  (mirror blocklist target_type='collection')

Token ★ (PK contract_account_id + token_id)
 │ 1 ── 0..1 Listing ★         (INV-007: maks 1 listing per token)
 │ 1 ──< N Offer ★             (aktif ≤1 per buyer — INV-024)
 └ 1 ──< N PayoutSplit ★       (via Sale.payout)

Bundle ★ (PK id)
 │ 1 ──< N BundleItem ★        (N ≤ 10 — INV-021)
 │ 1 ── 0..1 Sale ★
 └ 1 ──< N Listing ★           (bundle_id FK; ON DELETE SET NULL)

Listing ★ ── 0..1 ──► Sale ★      (SOLD)
Offer   ★ ── 0..1 ──► Sale ★      (ACCEPTED)
Bundle  ★ ── 0..1 ──► Sale ★      (SOLD)

LaunchpadPhase ★ (PK id; UQ collection+phase_index)
 └ 1 ──< N LaunchpadMint ★    (UQ phase_id+account_id)

Sale ★ (PK id) ── 1 ──< N PayoutSplit ★   (komponen distribusi, di dalam sales.payout)

Report ● (PK id) ── 0..1 ──► BlocklistEntry ● (keputusan hide)
AdminAudit ■ (PK id) ── 0..N ──► (Report | Collection | BlocklistEntry)  [polimorfik, tanpa FK]
```

> Entitas `PayoutSplit` ★ kini didefinisikan eksplisit (dulu hanya disebut "many PayoutSplit"): lihat §8.

## 2. Atribut per entitas

> Tipe detail & constraint ada di [database-schema.md](./database-schema.md) §DDL — tabel di sini merangkum atribut logis.

### Account (identitas chain)

| Atribut | Tipe | Null | Keterangan |
|---|---|---|---|
| `account_id` | TEXT (PK) | tidak | named / implicit 64-hex / `0x`+40hex / `0s`+40hex |
| `joined_at_block` | bigint | tidak | blok pertama terlihat (★, proyeksi) |

Bukan entitas app-mutable; `accounts` hanya cache. Profil (alias/bio/avatar) entitas terpisah.

### Profile ●

| Atribut | Tipe | Null | Keterangan |
|---|---|---|---|
| `account_id` | TEXT (PK) | tidak | 1 profil per akun |
| `alias` | TEXT | ya | ≤ 32 (validasi API) |
| `bio` | TEXT | ya | ≤ 280 |
| `avatar_url` | TEXT | ya | `https://`, gateway allowlist |
| `updated_at` | timestamptz | tidak | `now()` default |

### Collection ★ (+ flag app-mutable)

| Atribut | Tipe | Null | Keterangan |
|---|---|---|---|
| `contract_account_id` | TEXT (PK) | tidak | alamat kontrak koleksi |
| `name` | TEXT | ya | nama tampilan |
| `symbol` | TEXT | ya | simbol |
| `base_uri` | TEXT | ya | basis metadata |
| `owner` | TEXT | ya | owner kontrak; NULL sampai indexer fase 2 |
| `verified` ● | boolean | tidak | badge kurasi admin (default false) |
| `blocklisted` ● | boolean | tidak | mirror blocklist (default false) |
| `created_at_block` | bigint | ya | blok deploy; NULL sampai indexer fase 2 |

### Token ★

| Atribut | Tipe | Null | Keterangan |
|---|---|---|---|
| `contract_account_id` | TEXT (PK, FK) | tidak | → collections |
| `token_id` | TEXT (PK) | tidak | NEP-171 |
| `owner_id` | TEXT | tidak | pemilik saat ini (cache; otoritas on-chain) |
| `creator_id` | TEXT | ya | ekstensi custom; NULL koleksi pihak ketiga |
| `metadata` | jsonb | ya | NEP-177 (§Skema JSONB) |
| `minted_at_block` | bigint | tidak | blok mint |
| `burned_at` | timestamptz | ya | NULL = hidup |

### Listing ★

| Atribut | Tipe | Null | Keterangan |
|---|---|---|---|
| `contract_account_id`, `token_id` | TEXT (PK, FK) | tidak | → tokens |
| `seller` | TEXT | tidak | owner saat listing |
| `price_yocto` | NUMERIC(39,0) | tidak | diabaikan bila `bundle_id` terisi |
| `approval_id` | BIGINT | ya | NEP-178; invalid setelah transfer |
| `allowed_buyer` | TEXT | ya | private listing (INV-026) |
| `bundle_id` | BIGINT (FK) | ya | → bundles |
| `is_stale` | boolean | tidak | ownership mismatch |
| `status` | listing_status | tidak | ACTIVE/SOLD/CANCELLED/STALE |
| `listed_at` / `removed_at` | timestamptz | tidak / ya | |

### Offer ★

| Atribut | Tipe | Null | Keterangan |
|---|---|---|---|
| `id` | BIGINT (PK) | tidak | identity |
| `contract_account_id`, `token_id` | TEXT | tidak | target |
| `buyer` | TEXT | tidak | pemberi offer |
| `amount_yocto` | NUMERIC(39,0) | tidak | escrow |
| `expires_at` | timestamptz | tidak | default now + 7 hari |
| `status` | offer_status | tidak | ACTIVE/ACCEPTED/CANCELLED/EXPIRED/SUPERSEDED |
| `created_at` | timestamptz | tidak | |

### Bundle ★ & BundleItem ★

| Bundle | Tipe | Null | Keterangan |
|---|---|---|---|
| `id` | BIGINT (PK) | tidak | |
| `seller` | TEXT | tidak | pemilik semua item |
| `price_yocto` | NUMERIC(39,0) | tidak | harga tunggal |
| `status` | bundle_status | tidak | termasuk PRE_VALIDATE_FAILED/PARTIAL |
| `created_at` / `removed_at` | timestamptz | tidak / ya | |

| BundleItem | Tipe | Null | Keterangan |
|---|---|---|---|
| `bundle_id` | BIGINT (PK, FK) | tidak | → bundles |
| `contract_account_id`, `token_id` | TEXT (PK) | tidak | item (N ≤ 10) |

### Sale ★ & PayoutSplit ★

| Sale | Tipe | Null | Keterangan |
|---|---|---|---|
| `id` | BIGINT (PK) | tidak | |
| `contract_account_id` | TEXT | tidak | koleksi |
| `token_id` | TEXT | ya | NULL untuk sale bundle |
| `bundle_id` | BIGINT (FK) | ya | NULL untuk sale token |
| `buyer` / `seller` | TEXT | tidak | |
| `price_yocto` | NUMERIC(39,0) | tidak | |
| `payout` | jsonb | tidak | array PayoutSplit (§8) |
| `block_height` | bigint | tidak | ordering chain |

### LaunchpadPhase ★ & LaunchpadMint ★

| LaunchpadPhase | Tipe | Null | Keterangan |
|---|---|---|---|
| `id` | BIGINT (PK) | tidak | |
| `collection` | TEXT (FK) | tidak | → collections |
| `phase_index` | integer | tidak | urutan; UQ(collection, phase_index) |
| `name` | TEXT | ya | |
| `price_yocto` | NUMERIC(39,0) | tidak | exact-match deposit (INV-018) |
| `allocation` / `max_per_wallet` | integer | tidak | |
| `starts_at` / `ends_at` | timestamptz | tidak | window; tidak overlap (INV-029) |
| `allowlist_required` | boolean | tidak | |

| LaunchpadMint | Tipe | Null | Keterangan |
|---|---|---|---|
| `phase_id` | BIGINT (PK, FK) | tidak | → launchpad_phases |
| `account_id` | TEXT (PK) | tidak | |
| `minted_count` | integer | tidak | progres per wallet |

### Report ● / BlocklistEntry ● / AdminAudit ■ / AuthNonce ○ / Session ○

| Entitas | Atribut kunci | Keterangan |
|---|---|---|
| Report | `id` PK, `reporter`, `contract_account_id`, `token_id?`, `reason`, `status`, `decided_by?`, `decided_at?`, `created_at` | 1 report/target/hari |
| BlocklistEntry | `id` PK, `target_type`, `target_id`, `reason`, `decided_by`, `decided_at`, UQ(target_type,target_id) | display-layer |
| AdminAudit | `id` PK, `actor_account`, `action`, `target_type?`, `target_id?`, `reason`, `decision?`, `created_at` | append-only |
| AuthNonce | `nonce` PK, `account_id`, `scope`, `expires_at`, `used`, `created_at` | TTL 5 mnt |
| Session | `token_id` PK, `account_id`, `scope`, `expires_at`, `rotated_from?`, `revoked_at?`, `created_at` | TTL 15 mnt |

### EventRaw ★ (bukan entitas domain — sumber replay)

`(block_height, receipt_id, event_index)` PK, `tx_hash`, `event_json` jsonb, `ingested_at`, `finality`. Dedup kanonik `(receipt_id, event_index)`.

## 3. Entity → table mapping

| Entitas | Tabel | Kelas | Rebuild-able? |
|---|---|---|---|
| Account | `accounts` | Chain-derived ★ | ya |
| Profile | `profiles` | App-mutable ● | tidak |
| Collection | `collections` | Chain-derived ★ + flag ● | sebagian (flag tidak) |
| Token | `tokens` | Chain-derived ★ | ya |
| Listing | `listings` | Chain-derived ★ | ya |
| Offer | `offers` | Chain-derived ★ | ya |
| Bundle | `bundles` | Chain-derived ★ | ya |
| BundleItem | `bundle_items` | Chain-derived ★ | ya |
| Sale | `sales` | Chain-derived ★ (riwayat) | ya |
| PayoutSplit | komponen `sales.payout` (jsonb) | Chain-derived ★ | ya |
| LaunchpadPhase | `launchpad_phases` | Chain-derived ★ | ya |
| LaunchpadMint | `launchpad_mints` | Chain-derived ★ | ya |
| Report | `reports` | App-mutable ● | tidak |
| BlocklistEntry | `blocklist` | App-mutable ● | tidak |
| VerifiedFlag | kolom `collections.verified` | App-mutable ● | tidak |
| AdminAudit | `admin_audit` | Append-only ■ | tidak |
| AuthNonce | `auth_nonce` | Transient ○ | ya (expire) |
| Session | `sessions` | Transient ○ | ya (expire) |
| EventRaw | `events_raw` | Chain-derived ★ (sumber) | ya |

## 4. Lifecycle / state per entitas

State kanonik milik [../security/order-protocol-security.md](../security/order-protocol-security.md) §2 — di sini dipetakan ke kolom status DB. **Tidak ada transisi keluar dari state terminal.**

```text
Listing.status
  ACTIVE ──► SOLD        (settlement sukses)
  ACTIVE ──► CANCELLED   (seller; remove_sale)
  ACTIVE ──► STALE       (ownership mismatch; is_stale=true)
  terminal: SOLD | CANCELLED | STALE

Offer.status
  ACTIVE ──► ACCEPTED    (accept_offer sukses)
  ACTIVE ──► CANCELLED   (buyer)
  ACTIVE ──► EXPIRED     (expires_at lewat)
  ACTIVE ──► SUPERSEDED  (offer lain di-accept pada token sama)
  terminal: ACCEPTED | CANCELLED | EXPIRED | SUPERSEDED

Bundle.status
  ACTIVE ──► SOLD                 (semua token pindah)
  ACTIVE ──► CANCELLED            (seller)
  ACTIVE ──► PRE_VALIDATE_FAILED  (gagal sebelum transfer pertama; abort + refund)
  ACTIVE ──► PARTIAL              (residual mid-loop; INV-025; kompensasi G7)
  terminal: SOLD | CANCELLED | PRE_VALIDATE_FAILED | PARTIAL

Token
  (minted) ──► (owned) ──► (burned: burned_at diisi)
  owner_id berubah setiap transfer (proyeksi)

Collection
  (deployed) ──► verified true/false (admin)   [orthogonal]
  (deployed) ──► blocklisted true/false (admin) [orthogonal]

Report.status
  pending ──► hidden   (admin hide_target)
  pending ──► ignored  (admin ignore)
  terminal: hidden | ignored

LaunchpadPhase
  (scheduled) ──► (active: starts_at ≤ now < ends_at) ──► (ended)
  Satu fase aktif pada satu waktu (INV-029)
```

**Aturan transisi:**
- Transisi terminal tidak boleh dibalik (mis. `SOLD → ACTIVE` dilarang). Koreksi = rebuild dari `events_raw`.
- `is_stale` adalah flag tambahan pada listing `ACTIVE`; saat stale, `status` menjadi `STALE` dan listing tak bisa dibeli.
- `Report.status` hanya berpindah dari `pending` sekali (keputusan admin tunggal di MVP).

## 5. Invariant per relasi

| Relasi / entitas | Invariant | Sumber |
|---|---|---|
| Token → Listing | satu token maksimum satu listing (PK `(contract,token)`) | INV-007 |
| Listing/Offer → Sale | listing/offer non-ACTIVE tidak bisa dibeli/di-accept | INV-008 |
| Offer (token, buyer) | ≤ 1 offer ACTIVE per buyer per token (`uq_offers_active`) | INV-024 |
| Offer.amount | ≥ 0.01 Ⓝ saat dibuat | INV-030 |
| Listing.price | ≥ 0.01 Ⓝ (non-bundle) | INV-030 |
| Bundle → BundleItem | 1 ≤ N ≤ 10 item | INV-021 |
| Sale.payout | 1..10 penerima unik; setiap amount > 0; Σ ≤ price − fee | INV-002, INV-003 |
| Sale.payout royalty | royalti per token ≤ 10% harga wajar token | INV-027 |
| Token dalam Bundle AKTIF | tidak boleh di-list/di-offer terpisah | INV-028 |
| LaunchpadPhase | berurutan, tidak overlap (maks 1 aktif) | INV-029 |
| Sale | append-only; buyer ≠ seller | INV-023 |
| PayoutSplit | tujuan transfer selalu turunan state tervalidasi | INV-014 |
| Report | 1 report per (reporter, target, hari kalender) | api-security §1 |
| AdminAudit | append-only (REVOKE UPDATE/DELETE) | SEC-DB-003 |

## 6. Writer ownership per entitas (siapa yang memutasi)

> Prinsip: **chain = otoritas**; hanya tabel app-mutable/transient/audit yang ditulis langsung oleh aplikasi. Tabel chain-derived ditulis **hanya** oleh indexer (fase 2), tidak pernah oleh API/FE.

| Entitas / tabel | Writer | Operasi | Fase |
|---|---|---|---|
| `profiles` | API `/api/accounts/me/profile` | INSERT/UPDATE (upsert) | MVP |
| `reports` | API `POST /api/reports` | INSERT | MVP |
| `reports` (status) | API `PATCH /api/admin/reports/:id` | UPDATE status/decided_* | MVP |
| `blocklist` | API `POST/DELETE /api/admin/blocklist` | INSERT/DELETE | MVP |
| `collections.verified` | API `PATCH /api/admin/collections/:id/verified` | UPDATE flag | MVP |
| `collections.blocklisted` | API admin (mirror blocklist) | UPDATE flag | MVP |
| `auth_nonce` | API auth | INSERT; UPDATE `used=true` (atomik) | MVP |
| `sessions` | API auth | INSERT; UPDATE `revoked_at`; DELETE expire | MVP |
| `admin_audit` | API admin (dan script manual break-glass) | INSERT saja | MVP |
| `accounts`, `collections`, `tokens`, `listings`, `offers`, `bundles`, `bundle_items`, `sales`, `launchpad_phases`, `launchpad_mints`, `events_raw` | **Indexer Neardata** (TASK-014) | upsert per event final | Fase 2 |
| `auctions` | Indexer | (belum dirilis) | Fase 2 |

**Larangan writer:**
- API/FE **tidak boleh** menulis tabel chain-derived (SEC-INDEX-001); pada MVP tabel itu kosong.
- Indexer **tidak boleh** menulis `profiles`/`reports`/`blocklist`/`admin_audit` (tidak rebuild-able; bukan miliknya).
- `sales`/`events_raw` tidak di-UPDATE setelah insert (append-only); koreksi via rebuild.

## 7. Kebenaran sumber (ringkas ulang)

Lihat tabel "Kebenaran sumber" di atas (bagian awal dokumen) — tidak diulang di sini (single source of truth). Untuk konflik state, chain menang; DB adalah proyeksi (SEC-INDEX-001).

## 8. Entitas PayoutSplit (formal)

`PayoutSplit` adalah komponen distribusi pembayaran hasil settlement — satu baris per penerima setelah merge (INV-003). Karena jumlahnya kecil dan selalu dibaca bersama `Sale`, ia disimpan sebagai **array di dalam `sales.payout` jsonb**, bukan tabel terpisah. Ini keputusan bentuk penyimpanan (bukan keputusan produk); bila analitik fase 2 butuh query per-penerima, opsi normalisasi dicatat sebagai ⏳ open-by-design.

**Definisi entitas:**

| Atribut | Tipe | Null | Keterangan |
|---|---|---|---|
| `sale_id` | BIGINT (FK implisit) | tidak | pemilik: baris `sales.id` |
| `receiver_id` | TEXT | tidak | akun penerima (NEAR account) |
| `amount` | string yoctoNEAR (u128) | tidak | nilai untuk penerima |
| `role` | enum | tidak | `seller` \| `royalty` \| `treasury` |

**Relasi:** `Sale 1 ──< N PayoutSplit`; `Account 1 ──< N PayoutSplit` (sebagai penerima).

**Bentuk jsonb (`sales.payout`):**

```json
{
  "price": "1000000000000000000000000",
  "fee_bps": 200,
  "fee": "20000000000000000000000",
  "distributions": [
    { "receiver_id": "seller.testnet",   "amount": "970000000000000000000000", "role": "seller" },
    { "receiver_id": "creator.testnet",  "amount": "10000000000000000000000",  "role": "royalty" },
    { "receiver_id": "treasury.testnet", "amount": "20000000000000000000000",  "role": "treasury" }
  ]
}
```

**Invariant PayoutSplit:**

- `1 ≤ len(distributions) ≤ 10` dan `receiver_id` unik (INV-003, INV-021).
- `fee + Σroyalti + seller == price` (internal, exact); `Σpayout ≤ price − fee` (INV-001/002). Residual `price − fee − Σpayout` → seller (tanpa batas atas — koreksi ronde 23).
- `fee = floor(price × fee_bps / 10_000)` dengan `fee_bps = 200` (2%), cap `MAX_FEE_BPS = 500` (INV-004).
- Semua `amount` = string desimal yoctoNEAR (tidak pernah JSON `number`).
- Pada bundle: `distributions` = hasil merge royalti **per token dijumlahkan** tanpa cap agregat (INV-027), penerima unik ≤ 10.
