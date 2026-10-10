# Database Schema (Indexer & Report API)

> Database off-chain TIDAK menyimpan dana/kepemilikan — sumber kebenarannya kontrak on-chain.
> DB = proyeksi pencarian + data aplikasi. **Kelas tabel dipisahkan** (rebuild-able ≠ backup-wajib).

## Kelas tabel

| Kelas | Tabel | Sifat | Boleh hilang? |
|---|---|---|---|
| Chain-derived (proyeksi) | collections, tokens, listings, sales, offers, bundles, bundle_items, launchpad_phases, launchpad_mints, accounts, events_raw, auctions (fase 2) | Append/update dari event | Ya — rebuild dari chain |
| App-mutable | profiles, reports, blocklist, collections.verified, collections.blocklisted | Tidak rebuild-able | TIDAK — backup wajib |
| Auth-transient | auth_nonce, sessions | TTL singkat | Ya (expire sendiri) |
| Audit append-only | admin_audit | Immutable (REVOKE UPDATE/DELETE) | TIDAK — ikut backup |

## Tabel (final ronde 13 + audit)

```text
-- Chain-derived  (★ = baru terisi penuh saat indexer fase 2; MVP report API punya profiles/reports/blocklist/admin_audit/auth_nonce)
collections    (contract_account_id PK, name, symbol, base_uri, owner, verified, blocklisted, created_at_block)
tokens         (contract_account_id, token_id, owner_id, creator_id?, metadata jsonb, minted_at_block, burned_at)
listings       (contract_account_id, token_id, seller, price_yocto, approval_id, allowed_buyer NULL,
                bundle_id NULL, is_stale, status, listed_at, removed_at)
               -- bila bundle_id terisi, harga bundle = bundles.price_yocto (sumber kebenaran tunggal); listings.price_yocto tidak dipakai
sales          (id, contract_account_id, token_id, bundle_id NULL, buyer, seller, price_yocto,
                payout jsonb, block_height)
offers         (id, contract_account_id, token_id, buyer, amount_yocto, expires_at, status,
                created_at)   -- UNIQUE partial: (contract,token,buyer) WHERE status='active'
bundles        (id PK, seller, price_yocto, status, created_at, removed_at)
bundle_items   (bundle_id, contract_account_id, token_id)
launchpad_phases (id PK, collection, phase_index, name, price_yocto, allocation, max_per_wallet,
                  starts_at, ends_at, allowlist_required)   -- phase berurutan, tidak overlap (INV-029)
launchpad_mints  (phase_id, account_id, minted_count)  -- UNIQUE (phase_id, account_id)
accounts       (account_id PK, joined_at_block)
events_raw ★   (block_height, receipt_id, event_index, tx_hash, event_json, ingested_at, finality)
               -- dedup key = (receipt_id, event_index); kolom finality disiapkan 3 nilai, nilai efektif selalu `final` (ingest uang = final-only)
auctions ★     (… fase 2 — belum dirilis)

-- App-mutable
profiles       (account_id PK, alias, bio, avatar_url, updated_at)
reports        (id, reporter, contract_account_id, token_id NULL, reason, status, decided_by, decided_at)
blocklist      (target_type, target_id, reason, decided_by, decided_at)

-- Auth-transient
auth_nonce     (nonce PK, account_id, expires_at, used)
sessions       (token_id PK, account_id, scope, expires_at, rotated_from NULL)   -- refresh rotasi SEC-AUTH-006

-- Audit
admin_audit    (id, actor_account, action, target_type, target_id, reason, decision, created_at)
```

## Index pencarian (fase 2 — indexer)

- Trait: `jsonb` + GIN; nama koleksi/token: trigram/full-text; sort harga & waktu: btree.
- Trait filter & rarity baru aktif saat indexer fase 2 jalan.

## Rebuild & lag (fase 2)

- Backfill: replay dari `events_raw` sejak block kontrak pertama — **hanya kelas chain-derived**; tabel app-mutable/audit TIDAK ikut rebuild dan wajib backup (database-security.md).
- Indikator lag (chain final height vs DB height) di status page.

---

## Konvensi penamaan (naming conventions)

Berlaku untuk semua objek schema; dipakai konsisten oleh migration Prisma dan raw SQL.

| Objek | Pola | Contoh |
|---|---|---|
| Tabel | `snake_case`, plural | `launchpad_phases`, `bundle_items` |
| Kolom | `snake_case` | `price_yocto`, `created_at_block` |
| Primary key | `id` (identity) untuk tabel ber-event; pasangan natural key untuk tabel proyeksi | `sales.id`; `listings(contract_account_id, token_id)` |
| Foreign key | `<tabel_singular>_id` atau `<ref>_account_id` | `bundle_id`, `contract_account_id` |
| Enum type | `<domain>_status` / `<domain>_reason` | `listing_status`, `report_reason` |
| Index | `idx_<tabel>_<kolom>` | `idx_tokens_owner` |
| Unique index | `uq_<tabel>_<kolom>` | `uq_offers_active` |
| Check constraint | `chk_<tabel>_<rule>` | `chk_launchpad_phases_window` |
| FK constraint | `fk_<tabel>_<ref>` | `fk_listings_bundle` |

Aturan tambahan:

- Akun NEAR disimpan sebagai `TEXT` (nama named/sub/implicit `0x`/`0s`), tidak pernah `citext`; perbandingan case-sensitive sesuai protokol NEAR.
- Nilai uang: kolom `*_yocto` bertipe `NUMERIC(39,0)` (u128 max ≈ 3,4×10³⁸ → 39 digit cukup). **Dilarang** `numeric` tanpa presisi, `float`, `double`, atau `bigint` untuk uang.
- `event_json`/`metadata`/`payout` = `jsonb` (bukan `json`).
- Tidak ada `SERIAL`; gunakan `GENERATED ALWAYS AS IDENTITY` (standar modern, aman sequence).

## Konvensi tipe & waktu

| Konsep | Tipe | Alasan |
|---|---|---|
| Waktu aplikasi (`created_at`, `expires_at`, `decided_at`, `listed_at`) | `timestamptz` | UTC instan; `now()` default; perbandingan expiry benar lintas timezone |
| Tinggi blok (`block_height`, `minted_at_block`, `created_at_block`, `joined_at_block`) | `bigint` | Monotonik, sumber ordering chain (bukan waktu) |
| Waktu turunan chain (`burned_at`, `removed_at`) | `timestamptz` bila berasal dari `block_timestamp`; jika hanya blok → simpan `*_block bigint` | Hindari asumsi timestamp palsu |
| Uang (`price_yocto`, `amount_yocto`) | `NUMERIC(39,0)` | u128 yoctoNEAR exact; tidak ada pembulatan float |
| Boolean flag | `boolean NOT NULL DEFAULT false` | Hindari tiga-nilai (NULL = tidak diketahui) untuk `verified`/`is_stale`/`blocklisted` |
| Hash/ID chain (`receipt_id`, `tx_hash`) | `TEXT` | Base58; jangan dipotong |
| `event_index` | `integer` | Urutan log EVENT_JSON dalam receipt (≥0) |
| `token_id` | `TEXT` | NEP-171 token_id bisa string arbitrer |

**Aturan ordering**: urutan kanonik = `block_height` → urutan receipt dalam blok → `event_index`. Jangan gunakan `ingested_at`/`created_at` untuk ordering chain (itu waktu server, bisa beda).

## Enumerasi (enum value lists)

Semua status memakai `CREATE TYPE ... AS ENUM` (bukan `TEXT` bebas) agar invalid state tertolak di DB. Nilai diambil dari lifecycle kanonik ([../security/order-protocol-security.md](../security/order-protocol-security.md) §2); dokumen itu pemilik lifecycle, tabel di sini hanya menyimpan representasinya.

```sql
-- Listing (order-protocol §2 LISTING)
CREATE TYPE listing_status AS ENUM ('ACTIVE', 'SOLD', 'CANCELLED', 'STALE');

-- Offer (order-protocol §2 OFFER)
CREATE TYPE offer_status AS ENUM ('ACTIVE', 'ACCEPTED', 'CANCELLED', 'EXPIRED', 'SUPERSEDED');

-- Bundle (order-protocol §2 BUNDLE; INV-025)
CREATE TYPE bundle_status AS ENUM ('ACTIVE', 'SOLD', 'CANCELLED', 'PRE_VALIDATE_FAILED', 'PARTIAL');

-- Finality event (indexer-security §1; nilai efektif ingest uang selalu 'final')
CREATE TYPE event_finality AS ENUM ('optimistic', 'near-final', 'final');

-- Scope sesi (wallet-authentication §4; admin-security §2)
CREATE TYPE session_scope AS ENUM ('report', 'profile', 'admin');

-- Blocklist display-layer (api/endpoints.md §Moderasi)
CREATE TYPE blocklist_target_type AS ENUM ('collection', 'token');
```

| Domain | Nilai | Status |
|---|---|---|
| `listing_status` | `ACTIVE`, `SOLD`, `CANCELLED`, `STALE` | ✅ decided (order-protocol §2) |
| `offer_status` | `ACTIVE`, `ACCEPTED`, `CANCELLED`, `EXPIRED`, `SUPERSEDED` | ✅ decided (order-protocol §2) |
| `bundle_status` | `ACTIVE`, `SOLD`, `CANCELLED`, `PRE_VALIDATE_FAILED`, `PARTIAL` | ✅ decided (order-protocol §2, INV-025) |
| `event_finality` | `optimistic`, `near-final`, `final` | ✅ decided (indexer-security §1) |
| `session_scope` | `report`, `profile`, `admin` | ✅ decided (api-security §1); `support`/`finance` = ⏳ open-by-design (access-control-matrix: PROPOSED fase 2/mainnet) |
| `blocklist_target_type` | `collection`, `token` | ✅ decided (endpoints: report koleksi/token); `account` = ⏳ open-by-design |
| `report_status` | `pending`, `hidden`, `ignored` | ✅ diturunkan dari keputusan admin `hide_target`/`ignore` (endpoints §Moderasi) |
| `report_reason` | kandidat: `spam`, `scam`, `copyright`, `explicit`, `impersonation`, `other` | ⏳ open-by-design — 04-ux-ui-spec menyatakan "alasan (enum)" tetapi daftar final belum ditetapkan; jangan dipakai sebagai keputusan produk sebelum disahkan |
| `admin_audit.action` | mengikuti action step-up kanonik — **9 nilai** (SSOT: [../security/admin-security.md](../security/admin-security.md) §11): `report_decide`, `verified_set`, `blocklist_add`, `blocklist_remove`, `allowlist_add`, `allowlist_revoke`, `appeal_decide`, `approval_execute`, `break_glass` | ✅ DECIDED (ronde 16); template pesan step-up: [../security/wallet-authentication.md](../security/wallet-authentication.md) §3 |

## DDL — tabel app-mutable (MVP)

Tabel yang **tidak** rebuild-able; wajib backup (database-security §4). Hanya tabel-tabel ini (plus auth/audit) yang ditulis pada MVP (ADR-004).

```sql
-- Profil custom per akun (alias/bio/avatar). Batas panjang = validasi API (users.md).
CREATE TABLE profiles (
  account_id  TEXT        PRIMARY KEY,
  alias       TEXT        CHECK (alias IS NULL OR char_length(alias) <= 32),
  bio         TEXT        CHECK (bio IS NULL OR char_length(bio) <= 280),
  avatar_url  TEXT        CHECK (avatar_url IS NULL OR avatar_url ~ '^https://'),
  updated_at  timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT chk_profiles_account_id CHECK (char_length(account_id) BETWEEN 2 AND 64)
);
-- Catatan: allowlist gateway avatar ditegakkan di API (api-security §1), bukan di CHECK.
-- CHECK '^https://' hanya pertahanan berlapis, bukan pengganti allowlist.

-- Report moderasi. Idempotency: 1 report per (reporter, target, hari kalender) — lihat unique index di §Index.
CREATE TABLE reports (
  id                  BIGINT      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  reporter            TEXT        NOT NULL,
  contract_account_id TEXT        NOT NULL,
  token_id            TEXT,       -- NULL = report level koleksi
  reason              TEXT        NOT NULL,  -- nilai enum report_reason (⏳ open-by-design) — lihat §Enumerasi
  details             TEXT,
  status              TEXT        NOT NULL DEFAULT 'pending'
                                  CHECK (status IN ('pending', 'hidden', 'ignored')),
  decided_by          TEXT,
  decided_at          timestamptz,
  created_at          timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT chk_reports_decision_consistency
    CHECK ((status = 'pending') = (decided_by IS NULL AND decided_at IS NULL))
);

-- Blocklist display-layer (chain TIDAK tersentuh). Untuk target_type='collection',
-- collections.blocklisted adalah mirror denormalisasi dari tabel ini (writer: admin API).
CREATE TABLE blocklist (
  id          BIGINT               GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  target_type blocklist_target_type NOT NULL,
  target_id   TEXT                 NOT NULL,
  reason      TEXT                 NOT NULL,
  decided_by  TEXT                 NOT NULL,
  decided_at  timestamptz          NOT NULL DEFAULT now(),
  CONSTRAINT uq_blocklist_target UNIQUE (target_type, target_id)
);
```

## DDL — tabel auth-transient

TTL singkat; boleh hilang (expire sendiri) — tidak masuk backup wajib.

```sql
-- Nonce challenge auth (SEC-AUTH-001). Konsumsi: UPDATE ... WHERE used = false (atomik).
CREATE TABLE auth_nonce (
  nonce      TEXT        PRIMARY KEY,          -- 32-byte random hex
  account_id TEXT        NOT NULL,
  scope      TEXT        NOT NULL DEFAULT 'profile'
                         CHECK (scope IN ('report', 'profile', 'admin')),
  expires_at timestamptz NOT NULL,             -- now() + 5 menit
  used       boolean     NOT NULL DEFAULT false,
  created_at timestamptz NOT NULL DEFAULT now()
);

-- Sesi JWT (SEC-AUTH-003/006). token_id = klaim jti. Rotasi refresh menunjuk sesi lama.
-- SSOT skema sessions = dokumen ini (wallet-authentication.md §9 merujuk ke sini).
CREATE TABLE sessions (
  token_id           TEXT          PRIMARY KEY,
  account_id         TEXT          NOT NULL,
  scope              session_scope NOT NULL,
  refresh_token_hash TEXT          UNIQUE,   -- SHA-256 dari refresh opaque; NULL untuk scope admin (tanpa refresh)
  family_id          TEXT          NOT NULL, -- rantai rotasi; reuse → cabut seluruh family
  rotated_from       TEXT          REFERENCES sessions(token_id) ON DELETE SET NULL,
  expires_at         timestamptz   NOT NULL, -- refresh ≤12 jam; access 15 mnt dicek di JWT exp; admin tanpa silent refresh
  revoked_at         timestamptz,            -- denylist server-side (reuse → cabut satu family)
  created_at         timestamptz   NOT NULL DEFAULT now()
);
CREATE INDEX idx_sessions_family   ON sessions (family_id);
CREATE INDEX idx_sessions_account  ON sessions (account_id, created_at);
```

> **Deteksi reuse (SEC-AUTH-006)**: refresh yang sudah dirotasi (baris lamanya punya `rotated_from`
> dan dipakai lagi) → `UPDATE sessions SET revoked_at = now() WHERE family_id = <family>` →
> semua token dalam rantai ditolak → 401 `AUTH_REFRESH_REUSED`.

## DDL — tabel audit (append-only)

```sql
CREATE TABLE admin_audit (
  id            BIGINT      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  actor_account TEXT        NOT NULL,
  action        TEXT        NOT NULL,          -- action step-up kanonik (9 aksi — admin-security §11)
  target_type   TEXT,                          -- 'collection' | 'token' | 'report' | 'admin_allowlist' | …
  target_id     TEXT,
  reason        TEXT        NOT NULL,          -- wajib bebas-text (admin-security §5)
  decision      TEXT,                          -- mis. 'hide_target' | 'ignore'
  created_at    timestamptz NOT NULL DEFAULT now()
);

-- Permintaan approval 2-admin (admin-security §10). Dipakai untuk aksi berdampak luas (mainnet PROPOSED).
CREATE TABLE approval_request (
  id           BIGINT      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  action       TEXT        NOT NULL,             -- salah satu aksi step-up kanonik
  target_type  TEXT        NOT NULL,
  target_id    TEXT        NOT NULL,
  payload      jsonb       NOT NULL DEFAULT '{}',
  requested_by TEXT        NOT NULL,             -- admin pertama
  approved_by  TEXT,                             -- admin kedua (WAJIB ≠ requested_by)
  status       TEXT        NOT NULL DEFAULT 'pending'  -- 'pending' | 'approved' | 'rejected' | 'expired'
               CHECK (status IN ('pending','approved','rejected','expired')),
  expires_at   timestamptz NOT NULL,             -- usulan: now() + interval '24 hours'
  created_at   timestamptz NOT NULL DEFAULT now(),
  decided_at   timestamptz
);
```

**REVOKE append-only (SEC-DB-003)** — dijalankan di migration, per-role, bukan per-tabel-owner. Role aplikasi (`nearsea_app`) hanya boleh INSERT + SELECT; owner/migration role tetap bisa DDL.

```sql
-- Prasyarat: role aplikasi non-superuser (SEC-INFRA-002) sudah ada.
REVOKE UPDATE, DELETE, TRUNCATE ON admin_audit FROM nearsea_app;
GRANT  INSERT, SELECT            ON admin_audit TO   nearsea_app;

-- Defense-in-depth: tolak juga lewat default privilege untuk tabel audit baru di masa depan.
ALTER DEFAULT PRIVILEGES IN SCHEMA public
  GRANT INSERT, SELECT ON TABLES TO nearsea_app;

-- Verifikasi (CI/ops): harus gagal untuk nearsea_app
--   UPDATE admin_audit SET reason = 'x' WHERE id = 1;   -- ERROR: permission denied
--   DELETE FROM admin_audit WHERE id = 1;               -- ERROR: permission denied
```

> Catatan: REVOKE/GRANT di atas adalah **kontrol DB**, bukan pengganti audit aplikasi. `admin_audit` ikut backup (database-security §4). Migration yang menjalankan REVOKE harus memakai role owner/migrator, bukan `nearsea_app`.

## DDL — tabel chain-derived ★ (fase 2)

> **★ = baru terisi penuh saat indexer fase 2 jalan** (ADR-004, ADR-015). DDL disiapkan sekarang; pada MVP tabel ini ada tetapi kosong (atau tidak dibuat sampai migration fase 2 — lihat [migrations.md](./migrations.md)). Chain = otoritas; tabel ini proyeksi (SEC-INDEX-001).
>
> **Pengecualian MVP**: `collections` perlu ada sejak MVP karena membawa flag app-mutable `verified`/`blocklisted` (database-security §1). Pada MVP baris `collections` boleh dibuat seadanya — `contract_account_id` + `verified`/`blocklisted` — saat admin menandai badge/blocklist; kolom chain-derived (`owner`, `created_at_block`, name/symbol) dibiarkan `NULL` sampai diisi indexer fase 2. Karena itu `owner`/`created_at_block` **NULL-able** (lihat DDL di bawah), bukan `NOT NULL`.

```sql
-- Koleksi = satu kontrak NFT (owner = owner kontrak).
-- owner/created_at_block NULL-able: pada MVP baris dibuat lebih dulu oleh admin (verified/blocklisted),
-- kolom chain-derived diisi menyusul oleh indexer fase 2 (lihat catatan di atas).
CREATE TABLE collections (
  contract_account_id TEXT        PRIMARY KEY,
  name                TEXT,
  symbol              TEXT,
  base_uri            TEXT,
  owner               TEXT,                                 -- NULL sampai indexer fase 2
  verified            boolean     NOT NULL DEFAULT false,   -- app-mutable (admin)
  blocklisted         boolean     NOT NULL DEFAULT false,   -- app-mutable (mirror blocklist)
  created_at_block    bigint                                -- NULL sampai indexer fase 2
);

-- Token NEP-171. metadata = NEP-177 (lihat §Skema JSONB).
CREATE TABLE tokens (
  contract_account_id TEXT   NOT NULL
                        REFERENCES collections(contract_account_id) ON DELETE CASCADE,
  token_id            TEXT   NOT NULL,
  owner_id            TEXT   NOT NULL,
  creator_id          TEXT,                       -- ekstensi custom kita (users.md); NULL untuk koleksi pihak ketiga
  metadata            jsonb,
  minted_at_block     bigint NOT NULL,
  burned_at           timestamptz,
  PRIMARY KEY (contract_account_id, token_id)
);

-- Bundle dibuat sebelum listing bundle (FK listings.bundle_id → bundles.id).
CREATE TABLE bundles (
  id          BIGINT        GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  seller      TEXT          NOT NULL,
  price_yocto NUMERIC(39,0) NOT NULL CHECK (price_yocto > 0),
  status      bundle_status NOT NULL DEFAULT 'ACTIVE',
  created_at  timestamptz   NOT NULL DEFAULT now(),
  removed_at  timestamptz
);

-- Listing. Bila bundle_id terisi → harga bundle = bundles.price_yocto (sumber kebenaran tunggal).
CREATE TABLE listings (
  contract_account_id TEXT           NOT NULL,
  token_id            TEXT           NOT NULL,
  seller              TEXT           NOT NULL,
  price_yocto         NUMERIC(39,0)  NOT NULL CHECK (price_yocto >= 0),
  approval_id         BIGINT,
  allowed_buyer       TEXT,                                  -- private listing (INV-026)
  bundle_id           BIGINT         REFERENCES bundles(id) ON DELETE SET NULL,
  is_stale            boolean        NOT NULL DEFAULT false,
  status              listing_status NOT NULL DEFAULT 'ACTIVE',
  listed_at           timestamptz    NOT NULL,
  removed_at          timestamptz,
  PRIMARY KEY (contract_account_id, token_id),
  CONSTRAINT fk_listings_token
    FOREIGN KEY (contract_account_id, token_id)
    REFERENCES tokens(contract_account_id, token_id) ON DELETE CASCADE
);
-- Catatan: min harga 0.01 Ⓝ (INV-030) ditegakkan on-chain, BUKAN sebagai CHECK DB,
-- karena baris bundle memakai price_yocto yang tidak dipakai (harga = bundles.price_yocto).

-- Item bundle (maks 10 — INV-021; ditegakkan kontrak, bukan CHECK DB).
CREATE TABLE bundle_items (
  bundle_id           BIGINT NOT NULL REFERENCES bundles(id) ON DELETE CASCADE,
  contract_account_id TEXT   NOT NULL,
  token_id            TEXT   NOT NULL,
  PRIMARY KEY (bundle_id, contract_account_id, token_id)
);

-- Penjualan (riwayat permanen). payout = distribusi tervalidasi (lihat §Skema JSONB).
CREATE TABLE sales (
  id                  BIGINT        GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  contract_account_id TEXT          NOT NULL,
  token_id            TEXT,                                   -- NULL bila penjualan bundle
  bundle_id           BIGINT        REFERENCES bundles(id) ON DELETE SET NULL,
  buyer               TEXT          NOT NULL,
  seller              TEXT          NOT NULL,
  price_yocto         NUMERIC(39,0) NOT NULL CHECK (price_yocto >= 0),
  payout              jsonb         NOT NULL,
  block_height        bigint        NOT NULL,
  CONSTRAINT chk_sales_token_or_bundle
    CHECK (token_id IS NOT NULL OR bundle_id IS NOT NULL)
);

-- Offer. UNIQUE partial (contract, token, buyer) WHERE status='ACTIVE' → INV-024.
CREATE TABLE offers (
  id                  BIGINT        GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  contract_account_id TEXT          NOT NULL,
  token_id            TEXT          NOT NULL,
  buyer               TEXT          NOT NULL,
  amount_yocto        NUMERIC(39,0) NOT NULL CHECK (amount_yocto >= 0),
  expires_at          timestamptz   NOT NULL,
  status              offer_status  NOT NULL DEFAULT 'ACTIVE',
  created_at          timestamptz   NOT NULL DEFAULT now()
);

-- Launchpad: phase berurutan, tidak overlap (INV-029). phase_index = urutan.
CREATE TABLE launchpad_phases (
  id                 BIGINT        GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  collection         TEXT          NOT NULL
                       REFERENCES collections(contract_account_id) ON DELETE CASCADE,
  phase_index        integer       NOT NULL CHECK (phase_index >= 0),
  name               TEXT,
  price_yocto        NUMERIC(39,0) NOT NULL CHECK (price_yocto > 0),
  allocation         integer       NOT NULL CHECK (allocation >= 0),
  max_per_wallet     integer       NOT NULL CHECK (max_per_wallet >= 1),
  starts_at          timestamptz   NOT NULL,
  ends_at            timestamptz   NOT NULL,
  allowlist_required boolean       NOT NULL DEFAULT false,
  CONSTRAINT uq_launchpad_phases_index UNIQUE (collection, phase_index),
  CONSTRAINT chk_launchpad_phases_window CHECK (ends_at > starts_at)
);

-- Progres mint per wallet per phase. UNIQUE (phase_id, account_id).
CREATE TABLE launchpad_mints (
  phase_id     BIGINT  NOT NULL REFERENCES launchpad_phases(id) ON DELETE CASCADE,
  account_id   TEXT    NOT NULL,
  minted_count integer NOT NULL DEFAULT 0 CHECK (minted_count >= 0),
  PRIMARY KEY (phase_id, account_id)
);

-- Akun yang pernah terlihat on-chain (untuk discovery/profil ringan).
CREATE TABLE accounts (
  account_id      TEXT   PRIMARY KEY,
  joined_at_block bigint NOT NULL
);

-- auctions ★: FASE 2, belum dirilis — DDL ditunda (⏳ open-by-design, TASK-017).
```

## Skema JSONB (metadata, payout, event_json)

`jsonb` dipakai untuk payload yang bentuknya milik standar/kontrak; **kolom uang di dalam JSON tetap string yoctoNEAR** (aturan repo: tidak pernah `number` untuk nilai).

### `tokens.metadata` — NEP-177

```json
{
  "title": "NearSea Genesis #1",
  "description": "First item of the genesis collection.",
  "media": "ipfs://bafybeigdyrzt.../1.png",
  "media_hash": "base64:...",
  "copies": 1,
  "issued_at": "1700000000000",
  "expires_at": null,
  "starts_at": null,
  "updated_at": null,
  "extra": "{\"attributes\":[{\"trait_type\":\"Background\",\"value\":\"Blue\"}]}",
  "reference": "ipfs://bafybeigdyrzt.../1.json",
  "reference_hash": "base64:..."
}
```

- `media` hanya URL/hash on-chain; gambar TIDAK disimpan di DB (AGENTS Blockchain Rules).
- `extra` sering berupa string JSON berisi `attributes` (trait) → diekstrak saat fase 2 untuk rarity.
- Validasi bentuk/ukuran di metadata-security.md; DB hanya menyimpan.

### `sales.payout` — distribusi tervalidasi

Mencerminkan Payout NEP-199 yang sudah di-merge per receiver (INV-003, INV-021). Semua `amount` **string yoctoNEAR**.

```json
{
  "price": "1000000000000000000000000",
  "fee_bps": 200,
  "fee": "20000000000000000000000",
  "distributions": [
    { "receiver_id": "seller.testnet",  "amount": "970000000000000000000000", "role": "seller" },
    { "receiver_id": "creator.testnet", "amount": "10000000000000000000000",  "role": "royalty" },
    { "receiver_id": "treasury.testnet","amount": "20000000000000000000000",  "role": "treasury" }
  ]
}
```

- `role` ∈ `seller` | `royalty` | `treasury` (penerima unik setelah merge ≤ 10 — INV-021).
- `fee + Σroyalti + seller == price` (internal, exact); `Σpayout ≤ price − fee` (INV-001/002). Ini dihitung on-chain; DB hanya menyalin event.

### `events_raw.event_json` — payload NEP-297

```json
{
  "standard": "x-nearsea-market",
  "version": "1.0.0",
  "event": "market_sale",
  "data": [
    {
      "contract_id": "nft.creator.testnet",
      "token_id": "1",
      "buyer_id": "buyer.testnet",
      "seller_id": "seller.testnet",
      "price": "1000000000000000000000000",
      "payout": {
        "seller.testnet":  "970000000000000000000000",
        "creator.testnet": "10000000000000000000000",
        "treasury.testnet":"20000000000000000000000"
      }
    }
  ]
}
```

- `event_json` menyimpan **payload mentah** apa adanya (sumber rebuild); jangan normalisasi saat ingest.
- Event identity/dedup = `(receipt_id, event_index)`, bukan `tx_hash` (indexer-security §1). Filter emitter/standard ada di indexer-security §4.

## Peta relasi FK & aturan integritas referensial

```text
collections (PK contract_account_id)
  ├─< tokens            (contract_account_id)              ON DELETE CASCADE
  └─< launchpad_phases  (collection)                       ON DELETE CASCADE
                              └─< launchpad_mints (phase_id) ON DELETE CASCADE

bundles (PK id)
  ├─< bundle_items      (bundle_id)                        ON DELETE CASCADE
  ├─< listings.bundle_id                                   ON DELETE SET NULL
  └─< sales.bundle_id                                      ON DELETE SET NULL

tokens (PK contract_account_id, token_id)
  └─< listings          (contract_account_id, token_id)    ON DELETE CASCADE

sessions (PK token_id)
  └─ sessions.rotated_from → sessions.token_id             ON DELETE SET NULL

tabel tanpa FK (akun/waktu chain tidak dijamin ada barisnya di DB):
  profiles, reports, blocklist, admin_audit, auth_nonce,
  offers, sales(contract/token/buyer/seller), accounts, events_raw
```

| Relasi | Kardinalitas | ON DELETE | Aturan integritas |
|---|---|---|---|
| collections → tokens | 1 : N | CASCADE | token tidak bisa ada tanpa koleksi (kontrak NFT adalah koleksinya) |
| collections → launchpad_phases | 1 : N | CASCADE | phase milik tepat satu koleksi |
| launchpad_phases → launchpad_mints | 1 : N | CASCADE | progres mint mengikuti phase |
| bundles → bundle_items | 1 : N | CASCADE | item bundle hilang bersama bundle |
| bundles → listings.bundle_id | 1 : N | SET NULL | listing bundle tidak dihapus bila bundle dihapus; jadi listing biasa |
| tokens → listings | 1 : N (efektif 1:0..1 aktif) | CASCADE | satu token maksimum satu listing (INV-007 ditegakkan kontrak + PK `(contract,token)`) |
| sessions → sessions.rotated_from | 1 : N | SET NULL | jejak rotasi; sesi lama boleh hilang (transient) |

**Aturan referensial eksplisit:**

- **Tidak ada FK dari kolom akun** (`owner_id`, `seller`, `buyer`, `reporter`, `actor_account`, `account_id`) ke `accounts`. Alasan: akun adalah identitas chain yang selalu valid tanpa perlu baris di DB; FK akan memaksa ingest akun lebih dulu dan menambah failure mode. Validasi format akun dilakukan di API/kontrak, bukan FK.
- **Tidak ada FK ke `events_raw`** dari tabel proyeksi. Alasan: `events_raw` sumber rebuild; arah ketergantungan adalah rebuild membaca `events_raw`, bukan baris proyeksi menunjuk event.
- **`sales` adalah riwayat append-only** (proyeksi). Jangan UPDATE baris `sales`; koreksi dilakukan dengan rebuild dari `events_raw`.
- **`offers`/`listings` status terminal** tidak boleh diubah kembali ke `ACTIVE` (lihat [data-model.md](./data-model.md) §Lifecycle).
- **Order insert**: `collections` → `tokens` → `bundles` → `listings` (karena `listings.bundle_id` FK ke `bundles`). Ingest harus mengurutkan per blok/receipt (indexer-security §1).

## Index DDL

Index dirancang untuk read-path discovery fase 2 + query moderasi MVP. Nama mengikuti §Konvensi penamaan.

```sql
-- ---- MVP (app-mutable / auth) ----
CREATE UNIQUE INDEX uq_reports_daily
  ON reports (reporter, contract_account_id, COALESCE(token_id, ''),
              ((created_at AT TIME ZONE 'UTC')::date));
CREATE INDEX idx_reports_status_created ON reports (status, created_at DESC);
CREATE INDEX idx_blocklist_target       ON blocklist (target_type, target_id);

CREATE INDEX idx_auth_nonce_account   ON auth_nonce (account_id, expires_at);
CREATE INDEX idx_auth_nonce_expires   ON auth_nonce (expires_at) WHERE used = false;
CREATE INDEX idx_sessions_account     ON sessions (account_id, expires_at);
CREATE INDEX idx_sessions_expires     ON sessions (expires_at);

CREATE INDEX idx_admin_audit_actor    ON admin_audit (actor_account, created_at DESC);
CREATE INDEX idx_admin_audit_target   ON admin_audit (target_type, target_id, created_at DESC);

-- ---- Chain-derived (fase 2) ----
CREATE INDEX idx_tokens_owner        ON tokens (owner_id);
CREATE INDEX idx_tokens_creator      ON tokens (creator_id) WHERE creator_id IS NOT NULL;
CREATE INDEX idx_tokens_metadata_gin ON tokens USING gin (metadata jsonb_path_ops);
-- Nama token/atribut: trigram (butuh ekstensi pg_trgm)
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX idx_tokens_title_trgm
  ON tokens USING gin ((metadata ->> 'title') gin_trgm_ops);
CREATE INDEX idx_collections_name_trgm
  ON collections USING gin (name gin_trgm_ops);

CREATE INDEX idx_listings_status     ON listings (status) WHERE status = 'ACTIVE';
CREATE INDEX idx_listings_seller     ON listings (seller, status);
CREATE INDEX idx_listings_price      ON listings (price_yocto) WHERE status = 'ACTIVE' AND bundle_id IS NULL;
CREATE INDEX idx_listings_bundle     ON listings (bundle_id) WHERE bundle_id IS NOT NULL;

CREATE INDEX idx_offers_token        ON offers (contract_account_id, token_id, status);
CREATE INDEX idx_offers_buyer        ON offers (buyer, status);
CREATE UNIQUE INDEX uq_offers_active
  ON offers (contract_account_id, token_id, buyer) WHERE status = 'ACTIVE';

CREATE INDEX idx_bundle_items_token  ON bundle_items (contract_account_id, token_id);
CREATE INDEX idx_bundles_seller      ON bundles (seller, status);

CREATE INDEX idx_sales_block         ON sales (block_height DESC);
CREATE INDEX idx_sales_token         ON sales (contract_account_id, token_id);
CREATE INDEX idx_sales_buyer         ON sales (buyer, block_height DESC);
CREATE INDEX idx_sales_seller        ON sales (seller, block_height DESC);

CREATE INDEX idx_launchpad_phases_collection ON launchpad_phases (collection, phase_index);
CREATE INDEX idx_launchpad_phases_window     ON launchpad_phases (starts_at, ends_at);

-- events_raw: dedup + replay
CREATE INDEX idx_events_raw_block ON events_raw (block_height);
CREATE INDEX idx_events_raw_tx    ON events_raw (tx_hash);
CREATE INDEX idx_events_raw_dedup ON events_raw (receipt_id, event_index);
```

| Index | Melayani |
|---|---|
| `uq_reports_daily` | idempotency anti-spam 1 report/target/hari (api-security §1) |
| `uq_offers_active` | INV-024 (1 offer aktif per buyer per token) |
| `idx_tokens_metadata_gin` | filter trait (fase 2) |
| `idx_tokens_title_trgm` / `idx_collections_name_trgm` | search nama (fase 2) |
| `idx_listings_price` | sort harga discovery |
| `idx_sales_block` | riwayat terbaru + agregat volume |
| `idx_events_raw_dedup` | dedup & replay indexer |

> **Partial index** (`WHERE status='ACTIVE'`, `WHERE bundle_id IS NOT NULL`) dipakai agar index aktif kecil dan cepat; query discovery selalu memfilter status.

## Partisi tabel besar

Tabel yang tumbuh monotonik per blok dipartisi `RANGE (block_height)` agar:
- detach/attach cepat (retensi & replay),
- index per partisi lebih kecil,
- VACUUM/ANALYZE terkelola.

**Batasan PostgreSQL yang harus diingat**: pada tabel terpartisi, **primary key/unique harus memuat kolom partisi**. Karena itu PK `events_raw` memuat `block_height`, sedangkan dedup kanonik tetap `(receipt_id, event_index)` (lihat catatan).

```sql
-- events_raw: partisi per rentang 1.000.000 blok (contoh; final saat implementasi indexer).
CREATE TABLE events_raw (
  block_height bigint          NOT NULL,
  receipt_id   TEXT            NOT NULL,
  event_index  integer         NOT NULL CHECK (event_index >= 0),
  tx_hash      TEXT            NOT NULL,
  event_json   jsonb           NOT NULL,
  ingested_at  timestamptz     NOT NULL DEFAULT now(),
  finality     event_finality  NOT NULL DEFAULT 'final',
  PRIMARY KEY (block_height, receipt_id, event_index)   -- partisi harus ada di PK
) PARTITION BY RANGE (block_height);

CREATE TABLE events_raw_p0  PARTITION OF events_raw FOR VALUES FROM (0)          TO (1000000);
CREATE TABLE events_raw_p1  PARTITION OF events_raw FOR VALUES FROM (1000000)    TO (2000000);
CREATE TABLE events_raw_p2  PARTITION OF events_raw FOR VALUES FROM (2000000)    TO (3000000);
-- Partisi berikutnya dibuat otomatis oleh job ingest (CREATE TABLE ... PARTITION OF ...).

-- sales juga dipartisi per block_height pada skala besar (opsional; aktifkan bila > puluhan juta baris).
-- CREATE TABLE sales (...) PARTITION BY RANGE (block_height);  -- ⏳ open-by-design (tuning fase 2)
```

**Catatan dedup vs partisi (penting).** Unique global `(receipt_id, event_index)` **tidak dapat** ditegakkan pada tabel terpartisi tanpa menyertakan `block_height`. Dua opsi implementasi, pilih saat TASK-014 (⏳ open-by-design):

1. **Upsert dengan kunci lengkap** `ON CONFLICT (block_height, receipt_id, event_index) DO NOTHING` — dedup efektif karena event yang sama selalu berada di blok yang sama (receipt_id unik per blok).
2. **Tabel dedup terpisah** `events_seen(receipt_id, event_index)` PK non-partisi, ditulis dalam transaksi yang sama sebagai guard global.

Opsi 1 direkomendasikan karena `receipt_id` deterministik terikat blok; catat pilihan final di migration/indexer.

## Retensi per tabel

| Tabel | Retensi | Mekanisme | Alasan |
|---|---|---|---|
| `auth_nonce` | ≤ 5 menit (aktif) / hapus ≤ 12 jam | job harian `DELETE WHERE expires_at < now() - interval '12 hours'` | sekali pakai; transient |
| `sessions` | ≤ 12 jam | hapus sesi `expires_at < now()` (atau `revoked_at` lewat 30 hari) | transient; token expired tak berguna |
| `profiles` | sampai user hapus | `DELETE` row (right-to-erasure, database-security §1) | data user |
| `reports` | permanen | — | riwayat moderasi (database-security §6) |
| `blocklist` | permanen | — | keputusan moderasi |
| `admin_audit` | permanen + backup | append-only (REVOKE) | akuntabilitas; SEC-DB-003 |
| `events_raw` | permanen | partisi; partisi lama bisa di-archive ke object storage bila perlu | sumber rebuild (database-security §6) |
| `collections`/`tokens`/`listings`/`offers`/`bundles`/`sales`/`launchpad_*` | permanen (rebuild-able) | boleh TRUNCATE + rebuild dari `events_raw` | proyeksi chain |
| `accounts` | permanen (rebuild-able) | rebuild | proyeksi chain |

**Aturan**: job retensi HANYA menyentuh `auth_nonce`/`sessions`. Tabel audit/riwayat tidak pernah dihapus otomatis. Setiap job retensi wajib punya test (bukan menyentuh baris non-target).

## Diagram ER (ASCII)

```text
                         ┌───────────────────────┐
                         │      collections      │
                         │ PK contract_account_id│
                         │    verified, blocklisted (app-mutable)
                         └───────┬───────────┬───┘
                                 │1          │1
                                 │           │
                                 │N          │N
                         ┌───────▼──────┐ ┌──▼──────────────────┐
                         │    tokens    │ │  launchpad_phases   │
                         │ PK(contract, │ │ PK id               │
                         │    token_id) │ │ UQ(collection,idx)  │
                         └───┬──────┬───┘ └──┬──────────────────┘
                             │1     │1       │1
                             │      │        │N
                             │N     │N   ┌───▼──────────────┐
                    ┌────────▼──┐ ┌─▼───┐│ launchpad_mints  │
                    │ listings  │ │bundle│ PK(phase,account)│
                    │ PK(cont,  │ │items │└──────────────────┘
                    │  token)   │ └──┬───┘
                    └────┬──────┘    │N
                         │N          │
                         │           │1
                    ┌────▼───────────▼──┐
                    │     bundles       │
                    │ PK id             │
                    └─────────┬─────────┘
                              │1
                              │N
                    ┌─────────▼─────────┐
                    │       sales       │  (riwayat, append-only)
                    │ PK id             │
                    │ payout jsonb      │
                    └───────────────────┘

   offers ── (contract_account_id, token_id, buyer) ──► tokens  [tanpa FK; unique partial ACTIVE]
   accounts ──(account_id)──► identitas chain [tanpa FK]

   ── Kelas MVP (tidak rebuild-able) ──────────────────────────────
   profiles(account_id PK)      reports(id PK)      blocklist(id PK)
   auth_nonce(nonce PK)         sessions(token_id PK)   admin_audit(id PK, append-only)
   ───────────────────────────────────────────────────────────────

   ── Fase 2 (rebuild-able dari chain) ────────────────────────────
   events_raw(block_height, receipt_id, event_index) PK  ← sumber replay
   ───────────────────────────────────────────────────────────────
```

> Relasi tanpa FK digambar sebagai garis putus-putus/komentar: `offers` dan kolom akun tidak memakai FK (lihat §Peta relasi FK).
