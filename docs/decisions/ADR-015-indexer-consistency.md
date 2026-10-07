# ADR-015: Indexer Consistency Model — Proyeksi Read-only, Chain Otoritatif

## Status
Accepted (design untuk fase 2; tooling: **Neardata** — decided 2026-10-01)

## Problem
Indexer menyediakan pencarian/agregat, tapi jika keliru didesain bisa "menjadi" otoritas kepemilikan/settlement (state korup → UI salah → keputusan salah; atau lebih buruk, fitur yang memindahkan dana dari state indexer).

## Context
Fase 2: indexer (NEP-297 events → PostgreSQL). NEAR punya finality deterministik (FACT) — berbeda dari probabilistic finality EVM; reorg pasca-final tidak ada.

## Options
1. Indexer menyimpan state terkini dan API membaca darinya untuk semua kebutuhan (termasuk harga sebelum beli).
2. Indexer = proyeksi pencarian/agregat; data uang/ownership selalu re-verified via kontrak (view call); ingest hanya blok final; dedup by (receipt_id, event_index); reconciliation berkala.

## Trade-offs
- Opsi 1: satu sumber baca (cepat), tapi indexer bug/compromise = keputusan salah + risiko jika ada fitur berbayar yang percaya indexer.
- Opsi 2: dua jalur baca (RPC + API), tapi konsistensi aman dan rebuild-able dari events_raw.

## Security implications
Opsi 2 = SEC-INDEX-001 (indexer tidak pernah otoritas). Poisoning/poisoned-state hanya memengaruhi tampilan.

## Scalability / Operational / Cost
Opsi 2: reconciliation job ringan; view call on-demand murah.

## Decision
**Opsi 2.** Detail: indexer-security.md. Ingest final-only untuk data uang, optimistic opsional untuk UX dengan indikator pending. **Tooling sumber stream: Neardata** (drop-in replacement NEAR Lake yang deprecated).

## Rejected
Opsi 1.

## Kenapa Neardata (bukan NEAR Lake)

- **FACT (docs.near.org)**: **NEAR Lake + Lake Framework DEPRECATED (24 Maret 2026)** — S3 bucket berhenti indexing; JANGAN dipakai sebagai dasar.
- **Neardata (neardata.xyz)** = drop-in replacement Lake: stream blok murah untuk indexer self-host → cocok dengan ADR-009 (VPS sendiri).
- Alternatif dievaluasi dan tidak dipilih sekarang:

| Sumber | Kelebihan | Kekurangan | Status |
|---|---|---|---|
| **Neardata** | murah, drop-in Lake, cocok VPS self-host | — | **DIPILIH** |
| Nearcore Indexer Framework | tercepat (±3.8s end-to-end) | **$500+/mo** + maintenance nearcore (Rust only) | ditolak (biaya/ops) |
| Goldsky / The Graph / SubQuery / Indexer.xyz | hosted, tanpa infra | biaya per-request, vendor lock-in | fallback bila self-host terlalu berat |

- **RPC failover (urutan terkunci)**: FASTNEAR → official (`rpc.near.org`/`rpc.testnet.near.org`) → dRPC. Switch hanya di boundary blok final.

## Skema `events_raw` (pointer)

Sumber rebuild kanonik. **DDL lengkap + partisi dimiliki [database-schema.md](../database/database-schema.md)**; bentuk ringkas + aturan dedup ada di [indexer-security.md](../security/indexer-security.md) §8. Poin wajib:

- Kolom kunci: `block_height, receipt_id, event_index, tx_hash, event_json (jsonb mentah), ingested_at, finality`.
- **Dedup key kanonik = `(receipt_id, event_index)`** — **BUKAN `tx_hash`** (satu tx = banyak receipt/event). `tx_hash` disimpan hanya untuk korelasi.
- `PRIMARY KEY (block_height, receipt_id, event_index)`, `PARTITION BY RANGE (block_height)`; upsert idempoten `ON CONFLICT … DO NOTHING`.
- Ingest **hanya blok `final`**; kolom `finality` efektif selalu `final`.
- Ordering kanonik: `block_height` → urutan receipt → `event_index`.

## Algoritma rekonsiliasi (outline)

> Tujuan: mendeteksi divergensi DB (proyeksi) vs chain (otoritas) — **bukan** memperbaiki dana (dana selalu di chain). Rekonsiliasi **read-only**; koreksi = rebuild dari `events_raw`. Detail: indexer-security.md §6.

```text
JOB reconcile(sample = 100 entitas aktif):
  1. Ambil sampel acak entitas AKTIF dari DB (+ entitas yang lama tak disentuh).
  2. Untuk tiap entitas, bandingkan DB vs chain via VIEW CALL:
       listing: get_sale(contract, token)      → owner, price, approval_id, exists?
       offer:   get_offer(contract, token, buyer) → amount, expires_at, exists?
       bundle:  get_bundle(bundle_id)          → seller, price, status
       owner:   nft_token(contract, token).owner_id
  3. Klasifikasi:
       MATCH            → tidak ada aksi
       MISMATCH_FIELD   → tandai; rebuild entitas dari events_raw (replay sempit)
       MISSING_IN_DB    → event belum ter-ingest → rebuild + alert
       MISSING_ON_CHAIN → baris DB tak ada di chain (bug ingest) → hapus + rebuild
  4. Emit metrik: reconcile_checked, reconcile_mismatch{field}, reconcile_rebuilt.
  5. Mismatch bernilai UANG → alert Telegram SEGERA (sinyal bug ingestion — CS-2).
```

- **Aturan**: job rekonsiliasi **tidak pernah** memicu transaksi on-chain (indexer read-only — SEC-INDEX-001).

## Ambang lag (PROPOSED, dalam blok)

> Lag = `chain_final_height − ingested_height`, diukur terhadap **final height** (bukan optimistic). Nilai **PROPOSED** — finalisasi saat tuning fase 2.

| Ambang | Nilai (blok) | Aksi |
|---|---|---|
| Normal | **≤ 5** | tidak ada aksi |
| Warning | **> 5** (≈ >6,5 detik @1,3s/blok) | indikator "data mungkin basi" di UI + metrik |
| Alert | **> 50** (≈ >65 detik) | alert Telegram; status page degraded |
| Kritis | **> 300** (≈ >6,5 menit) | alert S1; banner discovery; investigasi ingestion |

- Frekuensi cek lag: **setiap blok (kontinu)** (PROPOSED). Frekuensi rekonsiliasi sampling: **15 menit**; full sweep **harian 03:00 UTC** (PROPOSED).
- Transaksi uang **tidak** bergantung pada lag: FE selalu view-call on-chain sebelum sign (SEC-ORDER-003) — lag hanya memengaruhi **discovery**, bukan keselamatan dana.

## Prosedur rebuild (bug ingestion)

> NEAR tidak punya reorg di belakang final (FACT) → "rollback" = **rebuild proyeksi dari `events_raw`**, bukan restore backup. Detail: indexer-security.md §9.

```text
1. STOP ingest (hentikan worker).
2. SNAPSHOT tabel proyeksi terdampak (pg_dump, untuk forensik).
3. IDENTIFIKASI rentang blok/event terdampak (log + events_raw).
4. FIX kode ingest + tambah test regresi.
5. REBUILD: hapus baris proyeksi terdampak → replay dari events_raw sejak blok awal rentang
   (idempoten). Bila events_raw korup → backfill dari chain via Neardata/RPC.
6. VERIFIKASI: bandingkan hasil rebuild dengan sampel chain (rekonsiliasi) + hitungan baris.
7. RESUME ingest dari blok terverifikasi; pantau lag.
8. POSTMORTEM: root cause + update SEC-* + test agar tidak berulang.
```

- **Prinsip**: `events_raw` = sumber rebuild; tabel proyeksi (`listings`/`sales`/`offers`/`bundles`/…) boleh dihapus + dibangun ulang. Tabel **app-mutable** (`profiles`, `reports`, `blocklist`, `admin_audit`) **tidak** ikut rebuild.
- Rebuild **tidak** menyentuh dana/aset (indexer bukan otoritas — SEC-INDEX-001).

## Consequences
TASK-014 (indexer) wajib implement reconciliation + lag indicator + prasyarat SEC-META-002 (fetcher isolasi). Tooling & RPC failover: lihat [../security/indexer-security.md](../security/indexer-security.md) §2.
