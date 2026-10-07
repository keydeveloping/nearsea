# Backend Architecture

> Lapisan off-chain di luar kontrak: API report, (fase 2) indexer, job. Backend TIDAK berada di jalur settlement — semua nilai lewat kontrak.
> Backend hanya: layanan aplikasi (report/profil/admin/auth), (fase 2) indexing event, agregasi (floor/volume/rarity), pencarian trait. Notifikasi in-app MVP murni client-side (polling FE).

## Indexer

> **DIPUTUSKAN (ronde 2 + riset)**: MVP memakai opsi A — tanpa indexer sendiri (view call RPC + NearBlocks API).
> Opsi B (custom indexer → Postgres) ditunda ke fase 2 — **ingestion via Neardata** (NEAR Lake DEPRECATED Maret 2026 — dilarang). Detail keamanan: [../security/indexer-security.md](../security/indexer-security.md).

## API service

> **Pengecualian "tanpa indexer" (ronde 3, revised)**: MVP punya API report + PostgreSQL di VPS untuk: reports, verified flag, blocklist, **profil custom (alias/bio/avatar)**, dan **auth (NEP-413 utama + custom fallback — api/authentication.md)**.
> Konsekuensi: tim mengelola SSL (Let's Encrypt), backup DB terjadwal, dan monitoring VPS — lihat [deployment/](../deployment/deployment.md) dan [disaster-recovery.md](../deployment/disaster-recovery.md).

## Job / cron

- **MVP: tidak ada cron kritis.** Validasi phase launchpad, expire offer, dan stale check semuanya **on-chain saat dipicu** (saat mint/offer/buy) — bukan pekerjaan background.
- Opsional belakangan: refresh cache metadata IPFS, pre-warm cache NearBlocks.

## Deployment backend

- VPS sendiri (ronde 5): Next.js (frontend + API routes) + PostgreSQL di VPS yang sama → detail [deployment/deployment.md](../deployment/deployment.md).

---

## 1. Modul & layering API

> Backend MVP = Next.js API routes (Route Handlers). Prinsip: **route tipis, logika di service, akses data hanya via Prisma**. API tidak pernah menyentuh dana (ADR-010).

```text
app/api/**/route.ts        ← HTTP layer: parse request, panggil service, bungkus response
   │
   ▼
lib/api/middleware/*       ← auth, rate limit, validasi, error envelope
   │
   ▼
features/<fitur>/service/* ← logika aplikasi (profil, report, admin, auth)
   │
   ▼
features/<fitur>/repo/*    ← akses data (Prisma) — satu-satunya lapisan yang menyentuh DB
   │
   ▼
prisma/schema.prisma       ← model & migrasi (SSOT schema: database/database-schema.md)
```

| Lapisan | Tanggung jawab | Dilarang | Contoh file (final saat scaffold) |
|---|---|---|---|
| Route handler | Parsing/validasi bentuk HTTP, status code, envelope | Query DB langsung, logika bisnis | `app/api/reports/route.ts` |
| Middleware | Auth session, rate limit, body size, request id | Akses data selain verifikasi | `lib/api/middleware/auth.ts` |
| Service | Aturan bisnis (mis. idempotensi report, step-up admin) | SQL mentah | `features/reports/service.ts` |
| Repo/data-access | Query Prisma, transaksi DB | Panggilan HTTP eksternal | `features/reports/repo.ts` |
| Error module | Definisi kode & pesan error | Pesan ad-hoc | `lib/errors.ts` ([error-handling.md](../development/error-handling.md) §8) |

> **Aturan**: kode error & pesan hanya dari satu modul terpusat ([error-handling.md](../development/error-handling.md) §8); semua nilai chain dalam string yoctoNEAR (api-overview.md).

## 2. Request lifecycle & rantai middleware

> Envelope error: `{ "error": { "code", "message", "requestId" } }` ([error-handling.md](../development/error-handling.md) §2).

```text
Request
  │
  ▼
[1] Request ID        → hasilkan requestId, pasang di context & log
  │
  ▼
[2] Body/size guard   → cap body (mis. 16KB, SEC-API-001); tolak 413 bila lebih
  │
  ▼
[3] Auth              → verifikasi session JWT (scope) / signature (NEP-413) untuk write
  │                      gagal → 401 AUTH_*  (SEC-AUTH-003/006)
  ▼
[4] Rate limit        → per IP / per akun / per scope (api-security-architecture.md §1)
  │                      gagal → 429 RATE_LIMITED
  ▼
[5] Validasi schema   → whitelist field, tipe, panjang, enum (strict)
  │                      gagal → 400 INVALID_*
  ▼
[6] Handler/service   → logika bisnis + akses data (Prisma)
  │                      error tak terduga → 5xx SERVER_ERROR (detail hanya di log)
  ▼
[7] Error envelope    → normalisasi SEMUA error ke envelope konsisten
  │
  ▼
Response (+ security headers dari Caddy)
```

| Middleware | Urutan | Gagal → | Requirement |
|---|---|---|---|
| Request ID | 1 | — | Korelasi log ([monitoring.md](../deployment/monitoring.md)) |
| Body size cap | 2 | 413 | SEC-API-001 |
| Auth (session/signature) | 3 | 401 | SEC-AUTH-001..006 |
| Rate limit | 4 | 429 | SEC-API-001 |
| Validasi schema | 5 | 400 | api-security-architecture.md §1 |
| Error envelope | 7 | — | error-handling.md §2 |

- **Auth write**: `POST/PATCH` butuh session scope (`report`/`profile`/`admin`); admin destruktif butuh **step-up signature** (SEC-ADMIN-003).
- **AuthZ per resource**: session `sub` = pemilik; tidak ada endpoint "ambil milik orang lain" (api-security-architecture.md §2, SEC-API-002).

## 3. Lapisan akses data & pemetaan Prisma

> Schema kanonik dimiliki [database/database-schema.md](../database/database-schema.md); migrasi lewat Prisma ([database/migrations.md](../database/migrations.md)). Backend **tidak pernah** memetakan tabel chain-derived ★ sebagai otoritas.

| Tabel | Kelas | Dipakai API MVP | Pemetaan Prisma |
|---|---|---|---|
| `profiles` | App-mutable | ya (GET/PATCH profil) | model `Profile` (account_id PK) |
| `reports` | App-mutable | ya (POST report + admin antrean) | model `Report` + enum status |
| `blocklist` | App-mutable | ya (admin) | model `BlocklistEntry` |
| `admin_audit` | Audit append-only | ya (tulis dari admin) | model `AdminAudit` (REVOKE UPDATE/DELETE di DB) |
| `auth_nonce` | Auth-transient | ya (auth) | model `AuthNonce` (UNIQUE nonce, TTL) |
| `sessions` | Auth-transient | ya (session/refresh) | model `Session` (rotated_from) |
| `collections`/`tokens`/`listings`/… ★ | Chain-derived | **tidak** (MVP) | model disiapkan, terisi fase 2 |

- **Konvensi pemetaan**: nama tabel `snake_case` di DB, nama model `PascalCase` di Prisma; kolom waktu `created_at/updated_at` default `now()`; FK + NOT NULL di tempat relevan ([database-security.md](../security/database-security.md) §3).
- **Nilai uang** disimpan sebagai **string/NUMERIC yoctoNEAR** bila ada (bukan `float`); konversi Ⓝ hanya di layer tampilan (AGENTS.md Blockchain Rules).
- **Transaksi**: operasi multi-tabel (mis. konsumsi nonce + terbit session) memakai `prisma.$transaction` agar atomik (SEC-AUTH-001).
- **Tidak ada secret di DB** (SEC-DB-002); tidak ada PII selain konten yang user tulis (alias/bio).

## 4. Strategi caching MVP

> Prinsip: cache **hanya** untuk read; transaksi uang selalu re-verify on-chain (SEC-ORDER-003). MVP sengaja tanpa cache aplikasi (tech-stack.md).

| Data | Cache MVP | TTL | Invalidasi | Catatan |
|---|---|---|---|---|
| Discovery (listing/koleksi) | **Tanpa cache server** | — | — | Dibaca FE langsung dari RPC (ADR-004) |
| Riwayat NearBlocks | Cache pendek di FE (TanStack Query) | 30–60 dtk | refetch saat tab aktif | features/notifications.md |
| Profil publik | Tanpa cache server (query DB langsung) | — | write langsung | Volume kecil di MVP |
| Session/nonce | DB (bukan cache) | 15 menit / 5 menit | atomik consume | SEC-AUTH-001/006 |
| Agregat (floor/volume) | **Fase 2** (Redis) | event-driven | saat event masuk | scaling.md §5 |

- **HTTP cache**: response API profil/report `Cache-Control: no-store` (data per-akun); aset statis Next.js di-cache Caddy/CDN (fase 2).
- **Alasan tanpa cache MVP**: menghindari kontradiksi data; beban baca ada di RPC dengan failover, bukan di API.

## 5. Manajemen koneksi DB

| Aspek | MVP | Fase 2 (instance > 1) |
|---|---|---|
| Pool driver | Prisma connection pool (default) | PgBouncer di depan Postgres |
| Mode pooling | session (langsung ke Postgres) | `transaction` mode PgBouncer |
| Batas koneksi | `connection_limit` kecil (mis. 5–10) | total pool ≤ `max_connections` Postgres |
| Statement/query timeout | query timeout pendek (mis. 5 dtk) | sama |
| Health | query ringan saat startup | health check terpisah |

- **Aturan**: `DATABASE_URL` server-side (SECRET) — tidak pernah di frontend (secrets-and-gitignore.md).
- **Scaling**: begitu instance API > 1, pooling **wajib** (scaling.md §3); sizing detail di [scaling.md](./scaling.md).

## 6. Observability plan

> Ringkas; dokumen operasional: [deployment/monitoring.md](../deployment/monitoring.md).

| Sinyal | Sumber | Ambang / aksi | Alert |
|---|---|---|---|
| Latency p95 API | log request (requestId) | > ambang → investigasi | Telegram bila konsisten |
| Error rate 5xx | log + error envelope | > ambang → alert | Telegram (error-handling.md §7) |
| Percobaan auth gagal | log auth | lonjakan → alert | Telegram (SEC-AUTH-005) |
| Rate limit ditolak | log middleware | lonjakan → tinjau kapasitas | — |
| Health `/api/health` | endpoint | gagal → alert | Telegram / uptime-kuma |
| DB connections | pool metrics | mendekati batas → naikkan pool/PgBouncer | — |
| (Fase 2) lag indexer | `ingested_height` vs chain final | > N blok → alert | Telegram |

- **Logging**: detail teknis (stack, requestId) **server-side saja**; tidak ada stack trace di response (SEC-API).
- **Alert Telegram**: 5xx, API down, report baru, event kontrak kritis (pause/owner-tx) — [monitoring.md](../deployment/monitoring.md).

## 7. Topologi Docker Compose

> MVP 1 VPS (ADR-009); staging = Compose + DB terpisah di VPS yang sama. Nama service & port bertanda PROPOSED (dikunci saat scaffold TASK-001).

```text
docker-compose.yml (VPS)
┌───────────────────────────────────────────────────────────────────────┐
│  service: caddy          ports: 80:80, 443:443                          │
│    volumes: ./caddy/Caddyfile, caddy_data, caddy_config                 │
│  service: web (next)     expose: 3000    depends_on: db                 │
│    env_file: .env.server  (SECRET)                                      │
│    volumes: (none — stateless)                                          │
│  service: db (postgres)  expose: 5432    volumes: pgdata                │
│    env_file: .env.server                                                │
│  (fase 2) service: indexer    depends_on: db   egress: RPC/Neardata     │
│  (fase 2) service: pgbouncer  expose: 6432    depends_on: db            │
└───────────────────────────────────────────────────────────────────────┘
Volumes: pgdata (persisten), caddy_data/caddy_config (sertifikat), backup (opsional lokal)
```

| Service | Image | Port | Volume | Jaringan | Catatan |
|---|---|---|---|---|---|
| `caddy` | caddy | 80, 443 | caddy_data, caddy_config | publik | TLS + security headers + rate limit |
| `web` | node/next build | 3000 (internal) | — | internal | Stateless (siap horizontal) |
| `db` | postgres | 5432 (localhost) | pgdata | internal | **Tidak** expose publik (SEC-INFRA-002) |
| `pgbouncer` (fase 2) | pgbouncer | 6432 | — | internal | Wajib saat instance > 1 |
| `indexer` (fase 2) | worker | — | checkpoint | internal + egress | Container terpisah, DB role read-only |
| `backup` (cron) | postgres client | — | — | internal | `pg_dump` harian → object storage (SEC-DB-001) |

- **Immutable deploy**: konfigurasi produksi tidak diubah manual di server (infrastructure-security.md §6).
- **Secrets**: `.env.server` chmod 600 di server, bukan di image ([secrets-and-gitignore.md](../development/secrets-and-gitignore.md)).

## 8. Inventaris env var (pointer)

> **Daftar kanonik & klasifikasi PUBLIC/SECRET dimiliki** [deployment/environments.md](../deployment/environments.md); template aman [.env.example](../../.env.example). Dokumen ini tidak menyalin nilai.

| Kelompok | Contoh (nama saja) | Klasifikasi |
|---|---|---|
| Chain | `NEAR_NETWORK`, `NEAR_RPC_URL`, `NEAR_RPC_FALLBACKS`, `MARKET_CONTRACT_ID`, `FACTORY_CONTRACT_ID` | PUBLIC |
| API/DB | `DATABASE_URL`, `JWT_SECRET` | SECRET |
| Alert | `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHAT_ID` | SECRET |
| Backup | `BACKUP_S3_ENDPOINT`, `BACKUP_S3_KEY`, `BACKUP_S3_SECRET` | SECRET |
| IPFS (fase lanjut) | `IPFS_GATEWAY`, `IPFS_PINNING_KEY` | publik / SECRET |

- Aturan: prefix `NEXT_PUBLIC_` hanya untuk nilai PUBLIC; SECRET **dilarang** di bundle frontend ([frontend-security.md](../security/frontend-security.md) §6).

## 9. Pendekatan testing API

> Aturan AGENTS.md: **setiap endpoint API wajib punya test** (happy + error). Strategi: [testing-strategy.md](../testing/testing-strategy.md).

| Lapis | Alat | Cakupan |
|---|---|---|
| Unit | Vitest | util validasi, pemetaan error, idempotensi report |
| Integration (endpoint) | Vitest + test DB | Semua route: happy + error envelope + status code |
| Auth | test signature NEP-413 + fallback | Tolak mismatch domain/network/scope/nonce (SEC-AUTH-002) |
| AuthZ (BOLA) | matriks per endpoint | Akses lintas akun ditolak (SEC-API-002) |
| Rate limit | test beban ringan | 429 saat melewati batas (SEC-API-001) |
| E2E (FE) | Playwright | Alur profil write → tampil publik |

- **DoD**: endpoint tanpa test (happy + error) dianggap belum selesai (testing-strategy.md).

## 10. Outline indexer fase 2

> Desain lengkap: [indexer-security.md](../security/indexer-security.md), keputusan [ADR-015](../decisions/ADR-015-indexer-consistency.md). Sumber stream = **Neardata** (NEAR Lake DEPRECATED — dilarang).

```text
Neardata stream ──► [fetcher/consumer] ──► [validator final-only] ──► [dedup & upsert]
                         │                        │                        │
                    checkpoint             buang blok bukan final    key (receipt_id,event_index)
                         │                        │                        ▼
                         └────────────────────────┴──────────────► PostgreSQL (events_raw + proyeksi)
                                                                          │
                                                              [reconciliation job]
                                                                  sample vs view call
```

| Aspek | Keputusan | Rujukan |
|---|---|---|
| **Reorg/finality** | Ingest HANYA blok `final` untuk data uang; NEAR tidak ada reorg di belakang final (FACT) | indexer-security.md §1, SEC-INDEX-002 |
| **Dedup** | Key `(receipt_id, event_index)` — idempoten upsert | SEC-INDEX-002 |
| **Checkpoint** | Posisi blok terakhir tersimpan; restart melanjutkan dari checkpoint | ADR-015 |
| **Backfill** | Replay dari `events_raw` sejak blok kontrak pertama — **hanya kelas chain-derived**; app-mutable/audit TIDAK ikut rebuild | database-schema.md §Rebuild |
| **Partisi** | Partisi tabel besar (activity/event mentah) per waktu | scaling.md §4 |
| **Reconciliation** | Job berkala bandingkan sample listing vs view call; mismatch → rebuild + alert | indexer-security.md §1 |
| **Otoritas** | Indexer **tidak pernah** otoritas ownership/settlement | SEC-INDEX-001 |
| **Prasyarat** | Fetcher metadata terisolasi (SEC-META-002) sebelum fetch server-side | ADR-014 |

- **Lag indicator**: `ingested_height` vs chain final height; > N blok → indikator "data mungkin basi" di UI + alert (indexer-security.md §1).
- **Scale**: tambah worker consumer + partisi per rentang blok (scaling.md §8).
