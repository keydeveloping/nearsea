# Monitoring & Observability

## Yang dipantau

### Kontrak / chain
- Tx gagal ke market contract (dari event/receipt)
- Gas usage per method (regresi gas = alarm)
- **Alert event kritis kontrak: `pause`/`unpause`, payout gagal (resolve refund), tx dari owner/DAO/`pause_callers`** — dasar deteksi drill SEC-IR-001 (parsing event via NearBlocks/API)

### Report API (MVP)
- Latency p95, error rate 5xx, request/menit
- Percobaan auth gagal (login/step-up) — alert Telegram

### Indexer (fase 2)
- Lag ingestion (chain final height vs DB height), error ingestion, reconciliation mismatch

### Frontend
- Error tracking MVP: console + error boundary (Sentry self-host/GlitchTip opsional — ditunda)

## Health checks

- `/api/health` API report (**kanonik** — dipakai workflow deploy & uptime check; `/healthz` lama ditinggalkan)
- Infra minimal: **disk usage, memori, expiry sertifikat Let's Encrypt** (cron check → Telegram)
- Uptime-kuma (opsional) untuk uptime FE/API
- (Fase 2) lag indexer + konektivitas RPC

## Alerting (Telegram)

- Bot Telegram ke admin: error kritis, API down, report baru masuk, **event kontrak kritis (pause/owner-tx)**
- Setelah indexer fase 2: alert lag ingestion.

## Audit log

- Sejak testnet: tx owner/DAO dicatat manual dari explorer; setelah mainnet: otomatis (owner actions = perubahan param kontrak, terbaca dari tx history).

## SLO & ambang metrik

> Angka **PROPOSED** (kalibrasi setelah baseline MVP; bukan jaminan SLA). Ambang = dasar alert.

| Metrik | Sumber | Target (SLO) | Warning | Critical |
|---|---|---|---|---|
| Availability FE/API | synthetic + uptime | ≥ 99.5%/bulan (MVP) | 1 gagal berturut | 3 gagal berturut / 5 menit |
| Latency p95 API | log request | < 500 ms | > 750 ms 10 mnt | > 1.5 s 10 mnt |
| Error rate 5xx | log + error envelope | < 1% | > 2% selama 5 mnt | > 5% selama 5 mnt |
| Auth gagal (login/step-up) | log auth | lonjakan anomali | > 20/menit | > 100/menit |
| DB connections | pool metrics | < 80% `max_connections` | > 80% | > 95% |
| Disk VPS | node exporter / cron | < 80% | > 80% | > 90% |
| Memori VPS | node exporter / cron | < 85% | > 85% | > 95% |
| Sertifikat TLS | cron `openssl` | > 21 hari tersisa | ≤ 21 hari | ≤ 7 hari |
| Kontrak: tx gagal | event/receipt | 0 anomali | pola naik | gagal massal |
| Kontrak: gas per method | event/receipt | tanpa regresi | +20% vs baseline | +50% |
| Kontrak: event kritis (pause/owner-tx) | NearBlocks poll | 0 tak terjadwal | — | setiap kejadian |
| (fase 2) lag indexer | tinggi DB vs chain final | 0 blok final | > 5 blok | > 50 blok |

## Definisi alert-rule

| Nama alert | Kondisi | Severity | Aksi / runbook |
|---|---|---|---|
| `api_health_down` | `/api/health` gagal 3× berturut | S2 | restart `web`; cek log |
| `api_5xx_spike` | error rate > 5% selama 5 mnt | S2 | cek log; rollback bila regresi |
| `api_latency_p95` | p95 > 1.5 s selama 10 mnt | S3 | cek DB/pool; naikkan tier |
| `auth_bruteforce` | auth gagal > 100/menit | S2 | blokir IP; cek SEC-AUTH |
| `db_disk_high` | disk > 90% | S3 | bersihkan log/WAL; resize |
| `db_connections` | > 95% `max_connections` | S2 | naikkan pool/PgBouncer |
| `tls_expiry` | ≤ 7 hari | S2 | cek Caddy/ACME |
| `contract_pause_event` | event `pause`/`unpause` | S1 (tak terjadwal) | [incident-response](../security/incident-response.md) |
| `contract_owner_tx` | tx dari owner/DAO/`pause_callers` | S1/S2 | verifikasi manual |
| `contract_payout_fail` | payout gagal (resolve refund) | S1 | pause + investigasi |
| `rpc_failover` | provider utama gagal, fallback aktif | S3 | pantau; lapor provider |
| `backup_failed` | job `pg_dump` gagal | S2 | jalankan manual; cek storage |
| `indexer_lag` (fase 2) | lag > 50 blok | S3 | cek ingest; rebuild bila korup |
| `dns_change` | `dig` ≠ record tercatat (mingguan) | S2 | cek registrar (CS-8) |

## Dashboard

| Dashboard | Isi | Sumber |
|---|---|---|
| App (FE/API) | uptime, p95, 5xx, request/menit, auth gagal | log + uptime |
| Infra | CPU/RAM/disk, DB connections, volume | node exporter / cron |
| Chain/kontrak | tx gagal, gas per method, event kritis, saldo escrow | NearBlocks/API |
| Backup/DR | status job, ukuran dump, restore terakhir | cron log |
| (fase 2) Indexer | lag, error ingest, reconciliation | metrics indexer |
| Biaya | estimasi bulanan VPS/storage/RPC | tagihan/usage |

- MVP: uptime-kuma (opsional) + script cron + Telegram. Dashboard penuh (Grafana/Prometheus) = fase lanjut.

## Retensi log

| Jenis log | Retensi | Tempat |
|---|---|---|
| Access log Caddy | 14 hari | VPS (rotasi logrotate) |
| App log (Next.js) | 14 hari | stdout → file/rotasi |
| Alert & insiden Telegram | permanen (kanal) | Telegram |
| Audit admin (`admin_audit`) | permanen (DB) | PostgreSQL |
| Backup `pg_dump` | 7 harian + 4 mingguan + 3 bulanan | object storage (lihat disaster-recovery.md) |
| Log CI | default GitHub (90 hari) | GitHub Actions |

- Log **tidak** memuat secret/stack trace di response (backend-architecture §6).
- Retensi **PROPOSED** — final saat TASK-030.

## On-call & runbook

| Situasi | Runbook |
|---|---|
| API/FE down | [deployment.md](./deployment.md) §Rollback + §Verifikasi pasca-deploy |
| DB hilang/korup | [disaster-recovery.md](./disaster-recovery.md) §Runbook restore |
| Insiden keamanan (S1–S3) | [../security/incident-response.md](../security/incident-response.md) |
| Kontrak kritis (pause/exploit) | [../security/catastrophic-failure-scenarios.md](../security/catastrophic-failure-scenarios.md) |
| Semua RPC gagal | [disaster-recovery.md](./disaster-recovery.md) §Skenario RPC |
| Sertifikat / DNS | [../architecture/infrastructure.md](../architecture/infrastructure.md) §DNS & sertifikat |

- MVP: on-call = PLATFORM_OWNER tunggal (1 orang + agent); jalur eskalasi di incident-response.md.
- Mainnet: pemisahan peran FINANCE/SECURITY/DEVELOPER (access-control-matrix.md).

## Setup bot Telegram

```text
1. Di Telegram: chat @BotFather → /newbot → simpan token (TELEGRAM_BOT_TOKEN).
2. Buat grup/kanal admin → tambahkan bot sebagai member.
3. Dapatkan TELEGRAM_CHAT_ID:
   - kirim pesan ke grup, buka https://api.telegram.org/bot<TOKEN>/getUpdates
   - salin chat.id (grup = angka negatif).
4. Simpan TELEGRAM_BOT_TOKEN + TELEGRAM_CHAT_ID di .env.server (chmod 600) — SECRET.
5. Uji kirim:
   curl -fsS "https://api.telegram.org/bot$TELEGRAM_BOT_TOKEN/sendMessage" \
     -d chat_id="$TELEGRAM_CHAT_ID" -d text="NearSea alert test"
6. Hubungkan sumber alert (cron/uptime-kuma/script event kontrak) ke endpoint sendMessage.
```

- Token bocor = rotasi via BotFather (`/revoke`) + perbarui `.env.server` (secrets-and-gitignore §6).

## Matriks severity & paging

| Severity | Definisi | Contoh alert | Notifikasi | Target respons |
|---|---|---|---|---|
| S1 | Dana/aset berisiko aktif | `contract_pause_event`, `contract_owner_tx`, `contract_payout_fail` | Telegram segera (bunyi) | menit |
| S2 | Layanan kritis mati / kontrol rusak | `api_health_down`, `api_5xx_spike`, `db_connections`, `backup_failed`, `tls_expiry` | Telegram | jam |
| S3 | Degraded / non-kritis | `api_latency_p95`, `db_disk_high`, `rpc_failover`, `indexer_lag` | Telegram (bundle) | hari |

> Severity selaras [incident-response.md](../security/incident-response.md) §1. MVP tanpa paging otomatis (telepon) — eskalasi manual.

## Synthetic checks

| Cek | Frekuensi | Target |
|---|---|---|
| `GET /api/health` | 1 menit | 200 + status DB ok |
| `GET /` (FE) | 5 menit | 200 + konten render |
| Alur baca (RPC view) | 15 menit | `get_fee_bps` mengembalikan nilai |
| TLS expiry | harian | > 21 hari |
| DNS record | mingguan | cocok dengan catatan (G9) |
| Restore drill DB | per kuartal | sukses (SEC-DB-004) |

- MVP: uptime-kuma + cron. Synthetic transaksi **tulis** tidak dijalankan otomatis di mainnet.

## Monitoring biaya

| Pos | Sumber | Ambang alert |
|---|---|---|
| VPS | tagihan provider | > budget bulanan |
| Object storage backup | usage provider | pertumbuhan tak wajar |
| RPC berbayar (opsional) | usage provider | mendekati kuota |
| Domain/SSL | tagihan | renewal gagal |
| IPFS pinning (fase lanjut) | usage provider | > budget |

- Estimasi tier: infrastructure.md §Model biaya + scaling.md §10.
- Lonjakan biaya RPC = sinyal trafik/abuse → tinjau rate limit FE.

## Status

- Ambang/SLO, alert-rule, dashboard, retensi — **PROPOSED** (final saat TASK-030).
- Endpoint health kanonik: **`/api/health`** (DECIDED).
- Telegram alert + monitoring dasar — MVP; dashboard penuh + paging otomatis = fase lanjut.
