# Database Security

> Cakupan: PostgreSQL di VPS (MVP: profiles, reports, blocklist, admin_audit, auth_nonce, sessions + proyeksi ringan; fase 2: tabel proyeksi penuh dari indexer Neardata). Prinsip desain: pisahkan **fakta turunan blockchain** (immutable, rebuild-able) dari **data aplikasi mutable**.

## 1. Klasifikasi data

| Kelas | Tabel | Sifat | Boleh hilang? |
|---|---|---|---|
| Chain-derived (proyeksi) | listings/sales/offers cache **(fase 2)**, events_raw **(fase 2)** | Immutable-append, rebuild dari chain | Ya (rebuild) |
| App mutable | profiles, reports, blocklist, verified flags | Tidak rebuild-able | TIDAK (backup wajib) |
| Auth/transient | auth_nonce, sessions | Sekali pakai/TTL | Ya (expire sendiri) |
| Audit | admin_audit | Append-only, TIDAK boleh edit/delete | TIDAK |
| Secret | — | **TIDAK ADA secret di DB** (FACT by design) | — |

- **Tidak ada PII**: hanya wallet address + konten yang user tulis sendiri (alias/bio). Email/password tidak ada (wallet auth).
- GDPR-style: data user minim by design; hapus profil = delete row (right-to-erasure sederhana).

## 2. Access boundaries

- App connect via user DB non-superuser, password di `.env` server-side (bukan git).
- Database bind localhost saja (tidak expose publik); akses admin via SSH.
- Tidak ada shared credential antara service; indexer fase 2 = user DB read-only terpisah.

## 3. Integrity constraints

- `auth_nonce`: UNIQUE(nonce), kolom `used`, TTL check di query (bukan hanya aplikasi).
- `reports`: UNIQUE(account_id, target, created_at::date — hari kalender; konsisten dengan api-security) untuk anti-spam idempotency.
- `admin_audit`: append-only — REVOKE UPDATE/DELETE dari app role (SEC-DB-003).
- `profiles`: UNIQUE(account_id); URL avatar divalidasi (allowlist) sebelum insert.
- Semua FK + NOT NULL di tempat relevan; timestamps default now().

## 4. Backup & restore

- Backup: `pg_dump` harian via cron → object storage terpisah dari VPS ([../deployment/disaster-recovery.md](../deployment/disaster-recovery.md)).
- **Aturan**: backup dianggap INVALID sampai restore didesain & diuji → SEC-DB-004: restore drill di staging sebelum mainnet (dokumen hasil drill).
- Retensi: **7 harian + 4 mingguan + 3 bulanan** (SSOT rotasi: [disaster-recovery.md](../deployment/disaster-recovery.md) §Jadwal); audit table di-backup bersama (append-only history).

## 5. Encryption

- At-rest: disk-level (LUKS/fscrypt) — **tergantung dukungan provider VPS** (cek saat provisioning); data kita tidak sensitif-murni (publik chain data + bio user) sehingga risk acceptance rendah.
- In-transit: TLS di edge (Caddy); DB-connection = localhost (tidak perlu TLS internal) — FACT.
- Standar enkripsi dijawab eksplisit: **apa** = disk backup + dump; **di mana** = object storage; **siapa pegang kunci** = kredensial object storage di VPS env; **jika kunci bocor** = data bocor adalah data publik/minim-PII → dampak rendah; **verifikasi** = restore drill.

## 6. Retensi

- auth_nonce/sessions: TTL ≤ 12 jam (hard delete).
- reports: permanen (riwayat moderasi); profil: sampai user hapus.
- events_raw (fase 2): permanen (sumber rebuild).

## 7. Klasifikasi data per kolom (column-level)

> Melengkapi §1 (per-tabel) dengan granularitas kolom: **sensitivitas**, **boleh hilang?**, **boleh terlihat di log/backup?**, dan **dasar hukum/hapus**. Klasifikasi ini menentukan GRANT (§8), masking, dan apa yang tidak boleh masuk log.

| Kelas | Kolom (contoh) | Sensitivitas | Boleh hilang? | Boleh di log? |
|---|---|---|---|---|
| **C0 — publik chain** | `listings.*`, `tokens.owner_id`, `sales.payout`, `events_raw.event_json`, semua `*_yocto` | publik (sudah on-chain) | Ya (rebuild) | Ya |
| **C1 — konten user** | `profiles.alias`, `profiles.bio`, `profiles.avatar_url` | rendah (user-published) | TIDAK | Terbatas (jangan dump penuh) |
| **C2 — moderasi** | `reports.reason`, `reports.details`, `blocklist.reason`, `admin_audit.reason`, `decided_by` | sedang (bisa memuat klaim sensitif) | TIDAK | TIDAK (hanya id/aksi) |
| **C3 — auth/sesi** | `auth_nonce.nonce`, `sessions.token_id`, `sessions.account_id`, `scope`, `expires_at`, `revoked_at` | tinggi (dapat menyamar) | Ya (expire) | **TIDAK** (nonce/token_id dilarang di log) |
| **C4 — rahasia** | — (tidak ada) | — | — | — |

- **Tidak ada PII**: hanya wallet address + konten yang user tulis sendiri (alias/bio) — tidak ada email/password (wallet auth).
- **Dilarang di log**: `nonce`, `token_id` (jti), refresh token, signature, JWT, dan body `auth_nonce`/`sessions`. Kode C3 = kelas paling sensitif di DB.
- **Right-to-erasure**: hapus `profiles` row (C1) = cukup; C2/C3 tidak memuat PII pribadi (alamat wallet = identitas publik chain, bukan PII dalam konteks ini).
- **Masking**: bila perlu menampilkan C2/C3 di tooling ops, tampilkan id/prefix saja, bukan nilai penuh.

## 8. GRANT / REVOKE (SQL eksak)

> Model least-privilege per-service: **`nearsea_app`** (API), **`nearsea_indexer`** (worker indexer fase 2), **`nearsea_migrator`** (migration/owner DDL). Tidak ada shared credential antar-service (§2). Semua dijalankan oleh role owner/migrator, **bukan** oleh app.

```sql
-- ============ Peran (non-superuser; SEC-INFRA-002) ============
-- Dibuat di luar migration aplikasi (bootstrap), login + password di secret store.

-- ============ nearsea_app (API) ============
-- Baca proyeksi + tulis data app-mutable.
GRANT SELECT ON collections, tokens, listings, sales, offers, bundles, bundle_items,
     launchpad_phases, launchpad_mints, accounts, events_raw
  TO nearsea_app;
GRANT SELECT, INSERT, UPDATE, DELETE
  ON profiles, reports, blocklist, auth_nonce, sessions
  TO nearsea_app;
-- Audit append-only (SEC-DB-003)
GRANT  INSERT, SELECT ON admin_audit TO nearsea_app;
REVOKE UPDATE, DELETE, TRUNCATE ON admin_audit FROM nearsea_app;
-- App TIDAK menulis events_raw (hanya indexer).
REVOKE INSERT, UPDATE, DELETE ON events_raw FROM nearsea_app;

-- ============ nearsea_indexer (worker fase 2) ============
GRANT SELECT, INSERT, UPDATE, DELETE
  ON collections, tokens, listings, sales, offers, bundles, bundle_items,
     launchpad_phases, launchpad_mints, accounts, events_raw, auctions
  TO nearsea_indexer;
-- Indexer DILARANG menyentuh data app-mutable/audit/auth.
REVOKE ALL ON profiles, reports, blocklist, admin_audit, auth_nonce, sessions
  FROM nearsea_indexer;
-- Flag app-mutable di collections (verified/blocklisted) tidak ditulis indexer:
GRANT UPDATE (name, symbol, base_uri, owner, created_at_block)
  ON collections TO nearsea_indexer;

-- ============ nearsea_migrator (DDL) ============
-- Hanya role ini yang boleh DDL + menjalankan REVOKE/GRANT + migration.
-- Tidak dipakai oleh service runtime (tidak ada kredensial migrator di container app).

-- ============ Verifikasi (CI) ============
-- Harus GAGAL untuk nearsea_app:
--   UPDATE admin_audit SET reason='x' WHERE id=1;   -- permission denied
--   DELETE FROM events_raw WHERE block_height=1;     -- permission denied
-- Harus GAGAL untuk nearsea_indexer:
--   UPDATE profiles SET alias='x' WHERE account_id='a.testnet';  -- permission denied
```

- **Aturan**: setiap tabel baru wajib diberi GRANT eksplisit di migration (default-deny); tabel audit = append-only via REVOKE (lihat juga [database-schema.md](../database/database-schema.md) §DDL audit).
- **Tidak ada** `GRANT ALL` ke app/indexer; **tidak ada** role runtime dengan hak DDL.
- **Defense-in-depth**: `ALTER DEFAULT PRIVILEGES` hanya untuk tabel audit baru; tabel proyeksi tetap GRANT eksplisit.

## 9. Connection pool & TLS

> DB **bind localhost** (§2) → koneksi app↔DB tidak melewati jaringan publik; TLS internal tidak wajib (FACT) tetapi tetap dipertimbangkan bila service dipisah host.

```text
POOL (Prisma / pg):
  connection_limit        = 10 (per instance app; PROPOSED — tuning)
  pool_timeout            = 10 s
  statement_timeout       = 5 s   (cegah query nyangkut)
  idle_in_transaction_timeout = 10 s
  max_lifetime            = 30 menit (rotasi koneksi; dukung rotasi kredensial §10)

TLS:
  host lokal (loopback)   → sslmode=disable (tidak keluar host; FACT)
  host terpisah (fase scaling) → sslmode=require + verify-full (CA provider) + TLS ≥1.2
  DATABASE_URL            = postgresql://nearsea_app:***@localhost:5432/nearsea_testnet?sslmode=disable
```

- **Kredensial di env server-side** (`.env.server`, bukan git; [secrets-and-gitignore.md](../development/secrets-and-gitignore.md)); **tidak ada** kredensial DB di FE/log.
- Pool per-instance kecil (MVP 1 VPS 1 instance); saat scaling horizontal, jumlah total koneksi = instance × `connection_limit` → sesuaikan `max_connections` DB atau pakai PgBouncer (⏳ saat scaling).
- **Wajib**: `statement_timeout` untuk mencegah query lambat menahan koneksi (DoS); timeout koneksi eksplisit.
- Koneksi **read-only indexer** memakai role `nearsea_indexer` (hak terbatas §8), bukan role app.

## 10. Prosedur rotasi kredensial DB

> Rotasi terjadwal (tahunan) + insidental (anggota keluar / dugaan bocor — [disaster-recovery.md](../deployment/disaster-recovery.md) § Playbook kompromi secret). Prinsip: **rotasi dulu, baru bersihkan jejak**; tanpa downtime berarti (buat kredensial baru → pindah → hapus lama).

```text
1. BUAT role/password baru (atau ALTER ROLE ... PASSWORD) — kredensial baru siap.
2. UPDATE `.env.server` (server-side, bukan git) → simpan kredensial baru.
3. ROLLING RESTART app (web) — koneksi baru memakai kredensial baru; `max_lifetime` pool
   mempercepat perpindahan koneksi lama.
4. VERIFIKASI: /api/health 200; query app sukses; tidak ada error auth di log.
5. CABUT kredensial lama: ALTER ROLE ... PASSWORD '<acak>' / DROP ROLE (bila role diganti).
6. VERIFIKASI negatif: kredensial lama GAGAL login.
7. ROTASI kredensial service lain bila terkait (indexer, backup S3).
8. CATAT di log rotasi (tanggal, aktor, role) — TANPA password.
```

- **Bila bocor**: contain dulu (revoke/ganti password), lalu rotasi semua service terkait (JWT/SSH/Telegram/backup) — urutan lengkap di disaster-recovery.md.
- **Tidak ada** kredensial bersama antara app dan indexer; rotasi salah satu tidak memutus yang lain.
- **Backup**: kredensial `BACKUP_S3_*` dirotasi terpisah; cron backup diuji pasca-rotasi.

## 11. RPO / RTO (angka)

> Angka **DECIDED** (selaras [disaster-recovery.md](../deployment/disaster-recovery.md) §RPO/RTO). Chain = otoritas; dana/aset tidak bergantung pada DB.

| Kelas data | RPO | RTO | Sumber pemulihan |
|---|---|---|---|
| Chain (ownership/dana) | 0 (final on-chain) | tidak relevan | chain (tidak dipulihkan kita) |
| App-mutable (`profiles`, `reports`, `blocklist`, `admin_audit`) | **≤ 24 jam** (backup harian) | **< 4 jam** (restore) | backup `pg_dump` |
| Auth/session (`auth_nonce`, `sessions`) | ≤ 24 jam | < 4 jam | backup (sesi boleh hilang → login ulang) |
| Chain-derived (fase 2) | 0 (dari chain) | **< 1 hari** (rebuild) | rebuild dari `events_raw`/chain |
| Secrets | 0 (rotasi) | < 1 jam | secret store + rotasi |
| Owner key | 0 | jam (DAO recovery) | seed offline 2 lokasi / proposal DAO |

- **Angka MVP**: **RPO ≤ 24 jam / RTO < 4 jam** (DECIDED). Backup dianggap **INVALID** sampai restore diuji (SEC-DB-004).
- **Restore runbook**: langkah `pg_restore` + verifikasi ada di [disaster-recovery.md](../deployment/disaster-recovery.md) §Runbook restore (database); **pointer** — jangan duplikasi langkah di sini.
- **Rebuild ≠ restore**: tabel chain-derived dipulihkan dengan **rebuild dari chain** ([indexer-security.md](./indexer-security.md) §9), bukan dari backup.

## 12. Row-Level Security (diskusi)

> Apakah perlu PostgreSQL **Row-Level Security (RLS)**? Posisi: **⏳ open-by-design — tidak dipakai di MVP**.

| Aspek | Analisis |
|---|---|
| Kebutuhan MVP | rendah: DB bind localhost, hanya 2 service (app/indexer) dengan role terpisah; isolasi baris per-user **tidak** diperlukan karena app memfilter di query (wallet-only, data publik) |
| Model auth | identitas = wallet, bukan tenant; tidak ada multi-tenant per-baris yang perlu RLS |
| Bila RLS dipakai | butuh `SET LOCAL app.current_account_id` per request + policy per tabel → kompleksitas + risiko policy salah justru membocorkan data |
| Alternatif yang dipakai | **GRANT per-tabel** (§8) + guard aplikasi (BOLA matrix, [api-security-architecture.md](./api-security-architecture.md) §8) |
| Kapan dipertimbangkan | bila kelak ada role support/finance yang butuh akses baca terbatas per-baris (fase 2/mainnet, access-control-matrix PROPOSED) |

- **Keputusan sementara**: kontrol akses di **layer role (GRANT)** + aplikasi, **bukan** RLS. Bila role `SUPPORT` (read-only) diaktifkan, tinjau ulang RLS untuk pembatasan baris (mis. hanya report yang ditugaskan).
- RLS **bukan** pengganti otorisasi aplikasi; RLS hanya defense-in-depth bila ada akses langsung DB oleh banyak role.

## 13. Migration security review (langkah wajib)

> Setiap migration schema = perubahan permukaan keamanan. **Wajib** review keamanan sebelum merge (AGENTS: perubahan schema lewat migration, tidak pernah edit tabel langsung).

| # | Cek review migration | Kenapa |
|---|---|---|
| 1 | GRANT/REVOKE untuk tabel/kolom baru sudah ditulis (default-deny) | tabel baru tanpa GRANT = app error; atau salah GRANT = over-privilege |
| 2 | Tabel audit tetap append-only (REVOKE UPDATE/DELETE) | SEC-DB-003 |
| 3 | Indexer tidak diberi hak tabel app-mutable; app tidak diberi hak tulis `events_raw` | pemisahan peran (§8) |
| 4 | Tidak ada secret/PII baru yang disimpan tanpa klasifikasi (§7) | kebocoran |
| 5 | Kolom uang = `NUMERIC(39,0)`; tidak ada `float`/`double`/`bigint` | presisi uang |
| 6 | Constraint/unique/partial index baru tidak membuka state invalid | integritas (mis. `uq_offers_active`) |
| 7 | Migration idempoten / reversible; ada rencana rollback | upgrade aman |
| 8 | Perubahan menyentuh tabel besar → cek lock/downtime (CREATE INDEX CONCURRENTLY) | ketersediaan |
| 9 | Review oleh orang selain penulis migration (mainnet gate) | four-eyes |
| 10 | Update [database-schema.md](../database/database-schema.md) + [data-model.md](../database/data-model.md) sinkron | SSOT |

- **Aturan**: migration yang mengubah hak akses (`GRANT`/`REVOKE`) **wajib** disertai test DB perms (CI) — verifikasi positif & negatif.
- Review ini bagian dari definition-of-done task yang menyentuh DB (AGENTS documentation-sync).
