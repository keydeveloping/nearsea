# ADR-009: Infra Self-Hosted VPS (FE + Report API + PostgreSQL)

## Status

**Accepted** (ronde 5, 2026-10-01). **Menggantikan** asumsi awal ronde 3 (serverless Vercel + Supabase).

## Problem

Di mana menjalankan frontend, API ringan (profil/report/badge/auth), dan PostgreSQL?

## Context

- Kebutuhan backend muncul dari ADR-006 (report/badge) + ADR-004 Opsi 3 (profil custom/auth).
- Awalnya diasumsikan serverless (Vercel) + DB terkelola (Supabase) — ronde 3.
- Tim 1 orang; ingin kendali penuh atas data & biaya; tidak ingin ketergantungan vendor terkelola.
- Indexer fase 2 (ADR-015) juga butuh PostgreSQL — engine yang sama akan dipakai ulang.

## Options

1. **VPS self-hosted** — Next.js (FE + API routes) + PostgreSQL di satu VPS; Docker Compose + Caddy/nginx + Let's Encrypt.
2. Serverless terkelola (Vercel) + DB terkelola (Supabase).
3. Hybrid — FE di edge, API + DB di VPS.

## Trade-offs

| | Opsi 1 (VPS) | Opsi 2 (serverless) |
|---|---|---|
| Kendali data | penuh | terbatas (vendor) |
| Biaya | tetap & rendah ($5–20/bln) | variabel per-request |
| Beban ops | tim kelola SSL/backup/monitoring | vendor kelola |
| Vendor lock-in | rendah | sedang-tinggi |
| Kesulitan scaling | manual (lihat scaling.md) | otomatis |

## Security implications

- VPS = tanggung jawab keamanan pindah ke tim: hardening SSH, firewall, patch, `.env` chmod 600, owner key terpisah dari app user (SEC-INFRA-001, [infrastructure-security.md](../security/infrastructure-security.md)).
- DB tidak boleh terekspos publik; app role non-superuser (SEC-INFRA-002).
- Backup harian pg_dump → object storage + restore drill (SEC-DB-001/004).
- Wajib monitoring + alert Telegram (TASK-030) — tidak ada vendor yang memantau.

## Scalability / Operational / Cost

- Scale manual: naik tier VPS (vertical) lalu tambah instance + LB + read replica (horizontal) — lihat [scaling.md](../architecture/scaling.md).
- Biaya terprediksi: VPS $5–20/bln, domain ~$10–15/thn, object storage backup kecil.
- Risiko: satu VPS = satu titik kegagalan → DR plan ([disaster-recovery.md](../deployment/disaster-recovery.md)).

## Decision

**Opsi 1 — self-hosted VPS.** Frontend Next.js + API ringan + PostgreSQL berjalan di **VPS milik sendiri** (satu VPS untuk MVP), dengan:
- reverse proxy + SSL otomatis (Caddy, atau nginx + certbot),
- proses manager Docker Compose,
- backup pg_dump harian → object storage,
- monitoring server + alert Telegram.
- Staging = subdomain terpisah di VPS yang sama, Docker Compose + DB terpisah.

## Rejected

- **Opsi 2 (serverless terkelola)** — ditolak: user memilih kendali penuh + biaya tetap; menghindari ketergantungan vendor terkelola.
- **Opsi 3 (hybrid)** — ditolak: menambah kompleksitas tanpa manfaat jelas untuk skala MVP.

## Consequences

- Tim menanggung ops: SSL renewal, backup, monitoring, hardening, patching.
- Environment: `local` / `dev` (staging) / `testnet` / `mainnet` ([environments.md](../deployment/environments.md)).
- Menutup ADR-004 Opsi 3 & ADR-006 (backend ringan); menjadi prasyarat TASK-028/029/030.
- Terkait: [ADR-015](./ADR-015-indexer-consistency.md), [architecture/infrastructure.md](../architecture/infrastructure.md).
