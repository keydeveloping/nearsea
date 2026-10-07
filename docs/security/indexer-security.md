# Indexer Security

> Indexer custom = FASE 2. Didesain sekarang agar implementasi nanti tidak jadi tambal sulam. Prinsip mutlak: **indexer = proyeksi pencarian; TIDAK PERNAH otoritas ownership/settlement** (SEC-INDEX-001).

## 1. Model konsistensi (ADR-015)

| Konsep | Keputusan |
|---|---|
| Finality | **FACT (docs.near.org)**: NEAR punya 3 tingkat finality — `optimistic`, `near-final` (DoomSlug; irreversible kecuali producer slashed), `final` (mutlak irreversible; ~1.3 detik, tanpa reorg). Ingest event HANYA dari blok `final` untuk data uang; `optimistic` boleh untuk UX read dengan indikator "pending" |
| Reorg | TIDAK ADA reorg di belakang final (FACT) → rollback path hanya untuk bug ingestion, bukan reorg |
| Event identity | **(receipt_id, event_index)** — dedup key unik; `event_index` = urutan log EVENT_JSON dalam receipt. tx_hash saja TIDAK unik per event (banyak receipt per tx) |
| Ordering | urut block height → receipt order dalam block (deterministik) |
| Duplicate events | idempotent upsert by dedup key |
| Stale data | kolom `ingested_height` vs `chain final height` = lag; > N blok → indikator "data mungkin basi" di UI + alert |
| Reconciliation | job berkala: sample acak (listing aktif) → bandingkan DB vs view-call kontrak; mismatch → rebuild entitas dari chain + alert |

## 2. Tooling & RPC failover (DIPUTUSKAN — riset docs 2026-10-01)

- **FACT (docs.near.org)**: **NEAR Lake + Lake Framework DEPRECATED (24 Maret 2026)** — S3 buckets berhenti indexing. JANGAN pakai.
- **Pilihan sumber stream (urutan rekomendasi docs)**:
  1. **Neardata** (neardata.xyz) — drop-in replacement Lake, stream blok murah untuk indexer self-host → **PILIHAN KITA** (cocok VPS self-host).
  2. **Nearcore Indexer Framework** — node sendiri, tercepat (±3.8s end-to-end) tapi $500+/mo + maintenance nearcore (Rust only).
  3. **Goldsky / The Graph / SubQuery / Indexer.xyz** — hosted/jasa (fallback kalau self-host terlalu berat).
- **RPC failover (urutan prioritas terkunci)**: FASTNEAR → official (`rpc.near.org`/`rpc.testnet.near.org`) → dRPC. Daftar provider (docs api/rpc/providers):
  - Mainnet: `free.rpc.fastnear.com` (FASTNEAR, publik — endpoint resmi di tabel docs), `rpc.mainnet.near.org` (official), dRPC, Intear, QuickNode, Lava, ZAN, dst.
  - Testnet: `test.rpc.fastnear.com` (FASTNEAR, publik), `rpc.testnet.near.org` (official), `near-testnet.drpc.org`, `testnet-rpc.intea.rs`.
  - Deteksi provider tidak sehat: lag, error rate, perbandingan block hash antar provider; switch hanya di boundary blok final.

## 3. Batas otoritas (aturan arsitektur)

```text
BLOCKCHAIN TRUTH      : sales/offers/approval/ownership/mint — dibaca via kontrak (view call)
INDEXED STATE         : pencarian, sort, agregat (floor, volume), riwayat, trait

DILARANG: "indexer bilang token X milik Y" sebagai dasar transaksi apa pun.
DILARANG: fitur yang memindahkan dana berdasarkan state indexer.
```

## 4. Ancaman khusus indexer

| Ancaman | Kontrol |
|---|---|
| Event palsu dari kontrak NFT jahat (emit format kita) | Filter by `standard: "x-nearsea-market"` + kontrak market kita sebagai emitter/receiver — event dari kontrak lain diabaikan |
| Poisoning via metadata | metadata-security.md fetcher design |
| Ingestion bug → state korup | reconciliation + rebuild-from-events_raw (schema dapat direbuild penuh) |
| DoS ingestion (flood tx spam ke market) | batching, backpressure, dedup; spam tx = mahal bagi penyerang (gas) — FACT |
| DB compromise | database-security.md (read-mostly, rebuild-able) |

## 5. Status

- Desain — PROPOSED (jadi prasyarat TASK-014 fase 2).
- Tooling — **DECIDED (2026-10-01)**: Neardata (stream) — NEAR Lake DEPRECATED 24 Maret 2026; alternatif Nearcore Indexer Framework / Goldsky (lihat §2).

## 6. Algoritma job rekonsiliasi + frekuensi

> Tujuan: mendeteksi divergensi DB (proyeksi) vs chain (otoritas) — **bukan** memperbaiki dana (dana selalu di chain). Rekonsiliasi **read-only** terhadap chain; koreksi dilakukan dengan rebuild entitas dari `events_raw`.

```text
JOB: reconcile(sample_size = N, scope = "active_listings")

1. Ambil sampel acak N entitas AKTIF dari DB (mis. listings WHERE status='ACTIVE').
   - N = 100 (PROPOSED) per run; sertakan juga entitas yang paling lama tidak disentuh.
2. Untuk tiap entitas, bandingkan DB vs chain via VIEW CALL kontrak:
   - listing:   get_sale(contract, token)  → owner, price, approval_id, exists?
   - offer:     get_offer(contract, token, buyer) → amount, expires_at, exists?
   - bundle:    get_bundle(bundle_id) → seller, price, status
   - ownership: nft_token(contract, token).owner_id
3. Klasifikasi:
   - MATCH            → tidak ada aksi (hitung metrik ok).
   - MISMATCH_FIELD   → tandai entitas; rebuild entitas itu dari events_raw (replay sempit).
   - MISSING_IN_DB    → event belum ter-ingest (lag / bug) → rebuild + alert.
   - MISSING_ON_CHAIN → DB punya baris yang chain tidak punya (bug ingest) → hapus baris (rebuild).
4. Rebuild = replay event terkait dari events_raw (bukan tulis manual), idempoten.
5. Emit metrik: reconcile_checked, reconcile_mismatch{field}, reconcile_rebuilt.
6. Mismatch yang melibatkan nilai UANG (amount/payout) → alert Telegram SEGERA + tandai insiden
   (indexer bukan otoritas, tetapi divergensi uang = sinyal bug ingestion — CS-2).
```

| Parameter | Nilai | Jenis |
|---|---|---|
| Frekuensi rekonsiliasi sampling | **setiap 15 menit** | PROPOSED |
| Ukuran sampel per run | **100 entitas aktif** | PROPOSED |
| Rekonsiliasi penuh (full sweep) | **harian (mis. 03:00 UTC)** | PROPOSED |
| Frekuensi cek lag | **setiap blok (kontinu)** | PROPOSED |
| Ambang mismatch → alert | **≥1 mismatch bernilai uang** atau **>5 mismatch struktural/run** | PROPOSED |

- **Aturan**: job rekonsiliasi **tidak pernah** memicu transaksi on-chain (indexer read-only — SEC-INDEX-001).
- Rekonsiliasi penuh harian = sampel lebih besar / seluruh entitas aktif, dijalankan di luar jam sibuk.

## 7. Ambang lag (PROPOSED) + alert

> Lag = `chain_final_height − ingested_height`. Ingest uang **hanya blok final** (§1) → lag dihitung terhadap tinggi final.

| Ambang | Nilai (blok) | Aksi |
|---|---|---|
| Normal | **≤ 5 blok** | tidak ada aksi |
| Warning | **> 5 blok** (≈ > 6,5 detik @1,3s/blok) | tampilkan indikator "data mungkin basi" di UI + metrik |
| Alert | **> 50 blok** (≈ > 65 detik) | alert Telegram; halaman status menandai degraded |
| Kritis | **> 300 blok** (≈ > 6,5 menit) | alert S1; discovery diberi banner; investigasi ingestion |

- **PROPOSED**: nilai di atas adalah usulan konkret; finalisasi saat tuning fase 2. Yang **dikunci**: lag diukur terhadap **final height**, bukan optimistic.
- Transaksi uang **tidak** bergantung pada lag (FE selalu view-call on-chain sebelum sign — SEC-ORDER-003); lag hanya memengaruhi **discovery**, bukan keselamatan dana.
- Alert mengikuti kanal Telegram ([monitoring.md](../deployment/monitoring.md)).

## 8. Skema `events_raw` (SQL) + dedup

> Sumber rebuild kanonik. DDL lengkap + partisi dimiliki [database-schema.md](../database/database-schema.md); berikut bentuk ringkas + kolom wajib untuk dedup/replay.

```sql
CREATE TYPE event_finality AS ENUM ('optimistic', 'near-final', 'final');

CREATE TABLE events_raw (
  block_height bigint          NOT NULL,
  receipt_id   TEXT            NOT NULL,
  event_index  integer         NOT NULL CHECK (event_index >= 0),
  tx_hash      TEXT            NOT NULL,
  event_json   jsonb           NOT NULL,   -- payload NEP-297 mentah (jangan normalisasi saat ingest)
  ingested_at  timestamptz     NOT NULL DEFAULT now(),
  finality     event_finality  NOT NULL DEFAULT 'final',
  PRIMARY KEY (block_height, receipt_id, event_index)   -- partisi harus ada di PK
) PARTITION BY RANGE (block_height);

-- Dedup & replay
CREATE INDEX idx_events_raw_block ON events_raw (block_height);
CREATE INDEX idx_events_raw_tx    ON events_raw (tx_hash);
CREATE INDEX idx_events_raw_dedup ON events_raw (receipt_id, event_index);
```

**Dedup (implementasi)**:

```sql
-- Opsi 1 (DIPILIH — rekomendasi database-schema §Partisi):
-- upsert dengan kunci lengkap; event identik selalu di blok yang sama (receipt_id unik per blok).
INSERT INTO events_raw (block_height, receipt_id, event_index, tx_hash, event_json, finality)
VALUES ($1, $2, $3, $4, $5, 'final')
ON CONFLICT (block_height, receipt_id, event_index) DO NOTHING;

-- Idempotent: replay event yang sama tidak menambah baris.
-- Opsi 2 (alternatif): tabel dedup global non-partisi.
--   events_seen(receipt_id TEXT, event_index integer, PRIMARY KEY (receipt_id, event_index))
-- ditulis dalam transaksi yang sama sebagai guard; pilih final saat TASK-014 (⏳).
```

- **Dedup key kanonik = `(receipt_id, event_index)`** — **bukan** `tx_hash` (satu tx = banyak receipt/event). `tx_hash` tetap disimpan untuk korelasi/observability.
- Ingest **hanya blok `final`** (kolom `finality` disiapkan 3 nilai, efektif selalu `final`); `optimistic` boleh untuk UX read dengan indikator "pending" tetapi **tidak** masuk data uang.
- Ordering kanonik = `block_height` → urutan receipt dalam blok → `event_index` (deterministik).

## 9. Prosedur rollback bug ingestion

> NEAR tidak punya reorg di belakang final (FACT) → rollback **hanya** untuk bug ingestion (kode kita salah), bukan chain. Karena chain = otoritas, "rollback" = **rebuild** proyeksi dari `events_raw`/chain, bukan restore backup.

```text
TRIGGER: bug ingest terdeteksi (event salah diparse, entitas korup, dedup gagal, mapping salah).

1. STOP ingest (hentikan worker) — cegah kerusakan bertambah.
2. SNAPSHOT: pg_dump tabel proyeksi terdampak (bukti pra-perbaikan; untuk forensik).
3. IDENTIFIKASI rentang blok/event terdampak (dari log + events_raw).
4. FIX kode ingest (patch) + tambah test regresi untuk kasus itu.
5. REBUILD: hapus baris proyeksi terdampak → replay dari events_raw sejak blok awal rentang
   (idempoten). Bila events_raw sendiri korup → backfill dari chain via Neardata/RPC.
6. VERIFIKASI: bandingkan hasil rebuild dengan sampel chain (rekonsiliasi §6); cek hitungan baris.
7. RESUME ingest dari blok terakhir yang ter-verifikasi; pantau lag.
8. POSTMORTEM: catat root cause, perbarui SEC-*, tambah test agar tidak berulang.
```

- **Prinsip**: `events_raw` = sumber rebuild; tabel proyeksi (`listings`/`sales`/`offers`/`bundles`/…) boleh dihapus + dibangun ulang. Tabel **app-mutable** (`profiles`, `reports`, `blocklist`, `admin_audit`) **tidak** ikut rebuild — jangan pernah terhapus oleh prosedur ini.
- Rebuild **tidak** menyentuh dana/aset (indexer bukan otoritas — SEC-INDEX-001).
- Jika bug menyentuh data yang dipakai FE untuk **preview**, FE tetap view-call on-chain sebelum sign → bug ingest **tidak** menyebabkan tx uang yang salah.
- Runbook restore DB (bukan rebuild) ada di [disaster-recovery.md](../deployment/disaster-recovery.md); rollback ingest = rebuild dari chain, **bukan** restore backup.

## 10. Read-only role grants (SQL)

> Indexer fase 2 menulis ke tabel proyeksi, tetapi **tidak** boleh menyentuh tabel app-mutable/audit/auth. Batasi dengan role terpisah (SEC-INFRA-002). `nearsea_app` = role API; `nearsea_indexer` = role worker indexer.

```sql
-- Prasyarat: role non-superuser dibuat (SEC-INFRA-002).
-- 1) Indexer: boleh tulis HANYA tabel chain-derived (proyeksi).
GRANT SELECT, INSERT, UPDATE, DELETE
  ON collections, tokens, listings, sales, offers, bundles, bundle_items,
     launchpad_phases, launchpad_mints, accounts, events_raw, auctions
  TO nearsea_indexer;

-- 2) Indexer: DILARANG menyentuh data app-mutable/audit/auth.
REVOKE ALL ON profiles, reports, blocklist, admin_audit, auth_nonce, sessions
  FROM nearsea_indexer;

-- 3) Indexer: collections.verified/blocklisted adalah app-mutable (admin) —
--    indexer hanya boleh UPDATE kolom chain-derived, bukan flag app.
--    (Implementasi: view khusus atau column-level GRANT bila didukung versi PG;
--     alternatif: pisahkan flag app ke tabel/kolom yang tidak di-GRANT ke indexer.)
GRANT UPDATE (name, symbol, base_uri, owner, created_at_block)
  ON collections TO nearsea_indexer;

-- 4) Role API: baca proyeksi + tulis app-mutable; TIDAK menulis events_raw.
GRANT SELECT ON collections, tokens, listings, sales, offers, bundles, bundle_items,
     launchpad_phases, launchpad_mints, accounts, events_raw
  TO nearsea_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON profiles, reports, blocklist, auth_nonce, sessions
  TO nearsea_app;
GRANT INSERT, SELECT ON admin_audit TO nearsea_app;      -- append-only (SEC-DB-003)
REVOKE UPDATE, DELETE, TRUNCATE ON admin_audit FROM nearsea_app;
REVOKE INSERT, UPDATE, DELETE ON events_raw FROM nearsea_app;   -- events_raw hanya indexer

-- 5) Verifikasi (CI): harus GAGAL untuk nearsea_indexer
--   UPDATE profiles SET alias='x' WHERE account_id='a.testnet';   -- permission denied
--   DELETE FROM admin_audit WHERE id=1;                           -- permission denied
```

- **Aturan**: role indexer **tanpa** hak pada tabel app-mutable/audit/auth; role API **tanpa** hak tulis `events_raw`. Ini mempersempit dampak bila salah satu service dikompromikan.
- Grants dijalankan lewat migration (role migrator/owner), bukan oleh `nearsea_app`/`nearsea_indexer`.
- Detail peran & rotasi kredensial DB: [database-security.md](./database-security.md) §2/§10.

## 11. Peta test (indexer)

| Kontrol | Test | Layer | SEC |
|---|---|---|---|
| Ingest hanya blok final | unit (fixture blok optimistic/near-final/final) | unit | SEC-INDEX-002 |
| Dedup `(receipt_id, event_index)` — replay tidak menambah baris | unit + integration (upsert idempoten) | unit | SEC-INDEX-002 |
| Event dari kontrak non-market diabaikan (filter emitter/standard) | unit | unit | SEC-INDEX-001 |
| Ordering deterministik (block → receipt → event_index) | unit | unit | SEC-INDEX-002 |
| Rekonsiliasi mendeteksi mismatch field/missing | integration (fixture chain vs DB) | integration | SEC-INDEX-001 |
| Lag alert pada ambang §7 | unit (fake clock) | unit | SEC-INDEX-002 |
| Rollback/rebuild idempoten | integration (rebuild lalu banding) | integration | SEC-INDEX-001 |
| Role grants: indexer ditolak pada app-mutable/audit | test DB perms | integration | SEC-INFRA-002 |
| Indexer tidak memicu tx on-chain | review arsitektur + grep (tidak ada signer di indexer) | review | SEC-INDEX-001 |

- **Aturan**: indexer **tidak boleh** punya akses kunci/tx; bukti = review dependensi (tidak ada `signer`/`Account` di kode indexer).
- Discovery ★ fase 2 diuji end-to-end saat TC discovery ditambahkan (lihat catatan akhir [test-cases.md](../testing/test-cases.md)).

## 12. Ringkasan status nilai

| Nilai | Jenis | Catatan |
|---|---|---|
| Frekuensi rekonsiliasi 15 menit / full sweep harian | PROPOSED | tuning saat fase 2 |
| Sampel 100 entitas/run | PROPOSED | tuning saat fase 2 |
| Lag warning >5 / alert >50 / kritis >300 blok | **PROPOSED** | finalisasi saat tuning; yang dikunci = diukur terhadap final height |
| Dedup opsi 1 (`ON CONFLICT` kunci lengkap) | DIPILIH (rekomendasi) | alternatif tabel `events_seen` ⏳ TASK-014 |
| Stream = Neardata | DECIDED | NEAR Lake DEPRECATED 2026-03 |
