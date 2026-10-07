# ADR-004: Data Layer MVP — RPC + NearBlocks (Tanpa Indexer Sendiri)

## Status

**Accepted** (ronde 2, 2026-10-01). Pengecualian API ringan ditambahkan ronde 3/5 (lihat Decision).

## Problem

Dari mana frontend membaca data (daftar koleksi, listing, offer, riwayat aktivitas, statistik) pada MVP — tanpa membangun indexer sendiri?

## Context

- Indexer custom (ingest event NEP-297 → Postgres) adalah pekerjaan besar: butuh infrastruktur, konsistensi finality, dedup, rekonsiliasi.
- Alternatif MVP: **view call RPC langsung** (state kontrak) + **NearBlocks API** (riwayat/aktivitas yang tidak ada di state kontrak).
- Namun ada kebutuhan yang tidak bisa dilayani chain: **profil custom** (alias/bio/avatar), **report system**, **verified badge**, **blocklist**, dan **auth** (NEP-413) — semua butuh penyimpanan off-chain ringan.

## Options

1. **Tanpa indexer sendiri** — view call RPC + NearBlocks API untuk read; tanpa backend.
2. **Custom indexer sejak MVP** — ingest event → Postgres; FE baca dari API.
3. **Tanpa indexer + API ringan** — read chain via RPC/NearBlocks, plus satu API kecil untuk data app-mutable (profil/report/badge/auth).

## Trade-offs

| | Opsi 1 | Opsi 2 | Opsi 3 |
|---|---|---|---|
| Waktu bangun | tercepat | terlama | sedang |
| Data app-mutable (profil/report) | tidak bisa | bisa | bisa |
| Beban RPC | tinggi (banyak view call) | rendah | tinggi (MVP) |
| Trait search / rarity / floor | tidak | bisa | tidak (fase 2) |
| Biaya infra | minimal | tinggi | sedang (1 VPS) |

## Security implications

- Read dari RPC/NearBlocks = **proyeksi**, bukan otoritas (selaras ADR-010). Transaksi uang wajib re-verify on-chain sebelum sign (SEC-ORDER-003).
- API ringan = permukaan serang baru → wajib auth NEP-413 (ADR-011), rate limit (SEC-API-001), dan tidak pernah menyimpan kunci/dana.
- NearBlocks = pihak ketiga: data bisa basi/salah → jangan pernah memicu transaksi dari data itu.

## Scalability / Operational / Cost

- Beban read terbesar di RPC → mitigasi failover FASTNEAR → official → dRPC + health check ([infrastructure.md](../architecture/infrastructure.md)).
- API ringan di VPS yang sama (ADR-009); PostgreSQL = engine yang sama dengan indexer fase 2 nanti.
- Fase 2: indexer Neardata menggantikan beban read + mengaktifkan trait search/floor/volume (ADR-015).

## Decision

**Opsi 3 — tanpa indexer sendiri, dengan API ringan.**
- **Read discovery** (koleksi, listing, offer, bundle, aktivitas): view call RPC + NearBlocks API, **langsung dari FE** pada MVP.
- **API ringan (VPS + PostgreSQL)** hanya untuk data app-mutable: profil custom, report, verified badge, blocklist, admin audit, dan auth (NEP-413).
- **Custom indexer ditunda ke fase 2** — ingest via **Neardata** (NEAR Lake DEPRECATED).

## Rejected

- **Opsi 1 (tanpa backend sama sekali)** — ditolak: profil custom, report, dan badge tidak mungkin tanpa penyimpanan.
- **Opsi 2 (indexer sejak MVP)** — ditolak: biaya/waktu tidak sepadan untuk MVP; trait search & rarity bukan kebutuhan M1.

## Consequences

- Beban read MVP ada di RPC (rate limit provider) — diterima, dengan failover.
- Tabel chain-derived di `database-schema.md` ditandai ★ (hanya terisi penuh di fase 2); MVP hanya mengisi tabel app-mutable. **Ini sumber kontradiksi yang dikoreksi** — lihat catatan di [api/api-overview.md](../api/api-overview.md).
- Terkait: [ADR-015](./ADR-015-indexer-consistency.md), [architecture/backend-architecture.md](../architecture/backend-architecture.md), [security/indexer-security.md](../security/indexer-security.md).
