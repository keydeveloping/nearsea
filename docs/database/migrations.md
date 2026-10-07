# Migrations

## Aturan

- Semua perubahan schema lewat file migration berurutan, tidak pernah edit manual.
- Migration wajib idempoten dan aman untuk indexer yang sedang berjalan (backward-compatible: tambah kolom, jangan rename langsung).
- Setelah schema berubah → jalankan rebuild dari `events_raw` bila perlu.

## Tooling (default ronde 13)

- **Prisma migrate** (TypeScript-native, cocok dengan Next.js API routes di VPS) — keputusan final.

## Log

> Setiap migration dicantumkan di sini:

| # | File | Tujuan | Tanggal |
|---|---|---|---|
| 0001 | `20261002000000_baseline_mvp` | Baseline schema MVP: `profiles`, `reports`, `blocklist`, `admin_audit`, `auth_nonce`, `sessions` + REVOKE `admin_audit` | 2026-10-02 |
| — | *(berikutnya ditambahkan per PR migration)* | | |

> **Baseline (0001)** = titik nol schema MVP. Semua migration setelahnya bernomor lebih besar dan tidak pernah mengubah/menghapus baris 0001. Baris di tabel ini ditambah bersamaan dengan file migration pada PR yang sama (lihat §Workflow).

---

## 1. Konvensi penamaan & versi

- Tool: **Prisma migrate** (keputusan final ronde 13; [../architecture/tech-stack.md](../architecture/tech-stack.md)).
- Lokasi: `prisma/migrations/<timestamp>_<snake_case_nama>/migration.sql`.
- Format timestamp: `YYYYMMDDHHMMSS` (UTC), 14 digit — urutan leksikografis = urutan aplikasi.
- Nama migration: `snake_case`, deskriptif, kata kerja + objek: `add_blocklist_reason_index`, `create_events_raw_partitions`.
- Satu migration = satu perubahan logis. Dilarang mencampur DDL tak berhubungan dalam satu file.
- **Immutable**: file migration yang sudah di-merge ke `main` TIDAK pernah diedit. Perbaikan = migration baru.
- Nomor urut di tabel Log = urutan 4 digit yang dipetakan ke timestamp; dipakai untuk rujukan di PR/issue.

## 2. Struktur file & contoh up/down

Prisma menyimpan `migration.sql` (arah **up**). Prisma tidak otomatis membuat `down`; karena itu **setiap migration wajib menyertakan `down.sql`** di folder yang sama untuk rollback manual (konvensi proyek, bukan fitur Prisma). Runner rollback (`scripts/migrate-down.ts`) membaca `down.sql`.

```text
prisma/migrations/
  20261002000000_baseline_mvp/
    migration.sql        # up (dijalankan Prisma)
    down.sql             # down (dijalankan scripts/migrate-down.ts)
    README.md            # opsional: catatan data/backfill
  20261005093000_add_reports_status_index/
    migration.sql
    down.sql
```

Contoh **up** — `20261005093000_add_reports_status_index/migration.sql`:

```sql
-- Tujuan: index antrean report untuk panel admin (status + urutan waktu).
-- Aman untuk indexer berjalan: hanya CREATE INDEX (tanpa lock tulis lama pada tabel kecil).
CREATE INDEX IF NOT EXISTS idx_reports_status_created
  ON reports (status, created_at DESC);
```

Contoh **down** — `.../down.sql`:

```sql
DROP INDEX IF EXISTS idx_reports_status_created;
```

Contoh migration tabel baru + constraint (up):

```sql
-- 20261010080000_create_events_raw_partitions/migration.sql
CREATE TABLE IF NOT EXISTS events_raw (
  block_height bigint          NOT NULL,
  receipt_id   TEXT            NOT NULL,
  event_index  integer         NOT NULL CHECK (event_index >= 0),
  tx_hash      TEXT            NOT NULL,
  event_json   jsonb           NOT NULL,
  ingested_at  timestamptz     NOT NULL DEFAULT now(),
  finality     event_finality  NOT NULL DEFAULT 'final',
  PRIMARY KEY (block_height, receipt_id, event_index)
) PARTITION BY RANGE (block_height);

CREATE TABLE IF NOT EXISTS events_raw_p0
  PARTITION OF events_raw FOR VALUES FROM (0) TO (1000000);
```

Down yang setara (destruktif — lihat §Rollback):

```sql
DROP TABLE IF EXISTS events_raw;   -- CASCADE tidak dipakai; tidak ada FK yang menunjuk events_raw
```

## 3. Strategi rollback

| Skenario | Tindakan |
|---|---|
| Migration belum deploy ke env mana pun | edit file (belum di-merge) atau `prisma migrate resolve --rolled-back` |
| Sudah di-merge tapi belum di-apply ke testnet/mainnet | buat migration koreksi baru; jangan edit |
| Sudah di-apply ke dev/testnet, perlu mundur | jalankan `down.sql` di env tersebut, lalu perbaiki dan buat migration baru |
| Sudah di-apply ke mainnet | **dilarang rollback destructif tanpa approval** (lihat §Workflow); gunakan pola expand-contract untuk mundur tanpa kehilangan data |

**Aturan rollback:**
- `down.sql` hanya boleh membalik perubahan schema; **tidak boleh** menghapus data app-mutable/audit (`profiles`, `reports`, `blocklist`, `admin_audit`). Untuk itu, `down` wajib gagal (guard) atau dilarang.
- Rollback migration yang menghapus kolom = **data hilang permanen**. Karena itu drop kolom hanya lewat expand-contract (§4), bukan drop langsung.
- Rollback harus diuji di CI (§6) — `up` lalu `down` lalu `up` lagi harus menghasilkan schema identik.

Guard contoh di `down.sql` yang menyentuh tabel backup-wajib:

```sql
-- Tolak rollback bila tabel berisi data (mencegah kehilangan tak sengaja).
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM reports LIMIT 1) THEN
    RAISE EXCEPTION 'rollback ditolak: reports berisi data — gunakan expand-contract';
  END IF;
END $$;
DROP TABLE IF EXISTS reports;
```

## 4. Expand-contract (zero-downtime)

Dipakai bila indexer/API sedang berjalan dan kolom/tabel harus berubah tanpa memutus pembacaan. Prinsip: **jangan pernah rename/drop kolom yang masih dibaca**; lakukan dalam 3 rilis.

Contoh: mengganti `reports.reason` dari `TEXT` bebas menjadi `report_reason` enum.

```text
Rilis 1 (EXPAND)   : tambah kolom baru nullable + backfill; tulis ke dua kolom; baca lama.
Rilis 2 (MIGRATE)  : semua pembaca pindah ke kolom baru; tulis hanya kolom baru.
Rilis 3 (CONTRACT) : drop kolom lama (setelah tidak ada pembaca).
```

**Rilis 1 — expand:**

```sql
-- 20261012_expand_reports_reason_enum/migration.sql
CREATE TYPE report_reason AS ENUM ('spam','scam','copyright','explicit','impersonation','other');
ALTER TABLE reports ADD COLUMN reason_enum report_reason;   -- nullable, belum dipakai
-- Backfill nilai yang bisa dipetakan (lihat §7); sisanya dibiarkan NULL untuk triase.
UPDATE reports SET reason_enum = 'other'
  WHERE reason_enum IS NULL;
```

**Rilis 3 — contract (setelah kode tidak membaca `reason`):**

```sql
-- 20261026_contract_drop_reports_reason/migration.sql
ALTER TABLE reports DROP COLUMN reason;
ALTER TABLE reports RENAME COLUMN reason_enum TO reason;
ALTER TABLE reports ALTER COLUMN reason SET NOT NULL;
```

Aturan expand-contract:
- Kolom baru **selalu nullable** saat ditambah (tidak ada default mahal yang mengunci tabel).
- `ALTER TABLE ... ADD COLUMN ... DEFAULT <konstan>` boleh (PG 11+ metadata-only), tapi hindari default volatil.
- Rename kolom hanya pada langkah CONTRACT, setelah kode lama tak membaca.
- Index baru dibuat `CREATE INDEX CONCURRENTLY` (tidak memblok tulis) — **harus di luar transaksi** Prisma; jalankan via `migration.sql` dengan `-- prisma:disable-transaction` bila didukung, atau script terpisah.

```sql
-- Contoh index concurrent (tidak boleh di dalam transaksi)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_listings_price
  ON listings (price_yocto) WHERE status = 'ACTIVE';
```

## 5. Koordinasi rebuild indexer

Schema chain-derived ★ dan `events_raw` terikat ke indexer fase 2. Urutan aman:

```text
1. STOP ingest (indexer berhenti menulis).
2. Backup: pg_dump tabel app-mutable/audit (wajib) + snapshot events_raw.
3. Jalankan migration schema.
4. REBUILD proyeksi: TRUNCATE tabel chain-derived (kecuali events_raw) → replay
   dari events_raw sejak block kontrak pertama (urutan block_height → receipt → event_index).
5. Verifikasi: lag indicator = 0; sample rekonsiliasi (indexer-security §1) lolos.
6. RESUME ingest dari blok final terakhir yang diproses.
```

- Rebuild **hanya** kelas chain-derived; `profiles`/`reports`/`blocklist`/`admin_audit`/`auth_nonce`/`sessions` TIDAK ikut (database-security §1).
- Migration yang mengubah bentuk proyeksi (mis. kolom baru hasil ekstraksi event) **wajib** diikuti rebuild dari `events_raw` — jangan backfill dari state lama yang mungkin sudah dikoreksi.
- Bila `events_raw` belum ada (MVP), langkah ini tidak berlaku: MVP hanya migration tabel app-mutable.
- Rebuild dijalankan dengan ingest berhenti agar tidak ada race tulis; `events_raw` sendiri append-only, aman dibaca.
- Partisi baru `events_raw` dibuat sebelum ingest mencapai rentangnya (auto-create job).

## 6. Testing & langkah CI

Setiap PR yang mengubah schema menjalankan gate berikut (lihat [../development/ci-cd.md](../development/ci-cd.md)):

```text
[ ] 1. prisma migrate deploy  → apply semua migration ke DB test kosong (fresh)
[ ] 2. up → down → up          → schema akhir identik dengan baseline (bandingkan pg_dump --schema-only)
[ ] 3. prisma migrate status   → tidak ada migration pending / drift
[ ] 4. lint SQL: nomor file unik, migration.sql + down.sql ada, tidak ada edit file lama (git diff)
[ ] 5. test integritas constraint: uq_offers_active, uq_reports_daily, REVOKE admin_audit (gagal UPDATE/DELETE)
[ ] 6. (fase 2) rebuild smoke: replay events_raw sampel → jumlah baris proyeksi cocok
```

- CI memakai **DB ephemeral** (container PostgreSQL) — migration harus jalan dari nol, bukan hanya dari schema yang sudah ada.
- Test `REVOKE` dijalankan dengan role `nearsea_app` (koneksi kedua) untuk membuktikan UPDATE/DELETE `admin_audit` gagal.
- Migration yang gagal apply = PR merah, tidak boleh merge (AGENTS Testing Rules).

## 7. Pola data-backfill

Backfill = mengisi kolom/tabel baru dari data yang sudah ada, terpisah dari DDL agar tidak mengunci tabel lama.

```sql
-- Backfill batch, idempoten, bisa dijalankan ulang (resume by cursor).
-- Contoh: isi reports.reason_enum dari reports.reason teks lama.
UPDATE reports
SET    reason_enum = CASE
         WHEN reason ILIKE '%spam%'    THEN 'spam'::report_reason
         WHEN reason ILIKE '%scam%'    THEN 'scam'::report_reason
         WHEN reason ILIKE '%copy%'    THEN 'copyright'::report_reason
         ELSE 'other'::report_reason
       END
WHERE  reason_enum IS NULL
  AND  id > :last_id
ORDER BY id
LIMIT  5000;
```

Aturan backfill:
- **Batch kecil** (mis. 5000 baris) + `ORDER BY` key, agar tidak ada transaksi panjang / lock lama.
- **Idempoten**: filter `WHERE <kolom_baru> IS NULL` sehingga re-run tidak menimpa nilai yang sudah diisi.
- **Resumable**: simpan cursor terakhir (tabel job atau argumen script) supaya bisa lanjut setelah gagal.
- Backfill proyeksi chain **selalu** lewat replay `events_raw`, bukan UPDATE massal dari state lama (§5).
- Backfill tidak boleh mengubah fakta bisnis (fee/royalty) — hanya memindahkan/menormalkan representasi.
- Backfill dijalankan setelah langkah EXPAND dan sebelum langkah CONTRACT.

## 8. Workflow kepemilikan & approval

| Peran | Tanggung jawab |
|---|---|
| Penulis PR | menulis `migration.sql` + `down.sql`, menambah baris di §Log, menjelaskan dampak data |
| Reviewer (≥1) | cek idempotensi, lock, dampak indexer, `down.sql` benar, tidak menyentuh file migration lama |
| DBA/owner (mainnet) | approval wajib untuk migration destruktif / yang menyentuh tabel backup-wajib |
| CI | gate otomatis §6 |

Aturan approval:
- Migration **additive** (tambah tabel/kolom/index) → 1 reviewer.
- Migration **destruktif** (drop/rename kolom, ubah tipe, hapus tabel) → approval DBA/owner + wajib expand-contract + rencana rollback tertulis.
- Migration pada `mainnet` → approval eksplisit user (AGENTS Git Rules: dilarang deploy/ubah mainnet tanpa persetujuan) + backup terverifikasi sebelum apply.
- Semua migration dijalankan lewat pipeline (bukan manual di server), kecuali break-glass terdokumentasi (admin-security §6).

## 9. Status

- Tooling — **DECIDED**: Prisma migrate.
- Baseline MVP — **DECIDED** (0001, tabel app-mutable + auth + audit).
- Migration chain-derived/partisi — fase 2 (menunggu TASK-014 indexer).
