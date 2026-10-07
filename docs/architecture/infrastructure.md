# Infrastructure

> Tempat semuanya berjalan: chain network, hosting, storage pihak ketiga.

## Jaringan NEAR (ronde 3+12)

- Alur: localnet/sandbox → **testnet (seluruh MVP)** → mainnet di M4 setelah audit.
- Akun: `nearsea.testnet` (draft naming: factory `nearsea.testnet`, koleksi `<nama>.nearsea.testnet`) — ⏳ open-by-design (final saat deploy).

## RPC provider (daftar resmi docs api/rpc/providers — riset 2026-10-01)

- **Testnet**: `rpc.testnet.near.org` (official, rate-limited), `test.rpc.fastnear.com` (FASTNEAR, publik), `near-testnet.drpc.org` (dRPC), `testnet-rpc.intea.rs` (Intear), QuickNode/Tatum/ZAN (berkunci).
- **Mainnet** (nanti): `rpc.mainnet.near.org`, `free.rpc.fastnear.com`, dRPC, QuickNode, dst.
- Fallback order (DECIDED): FASTNEAR primary → official → dRPC; health-check bandingkan block hash.
- **Perilaku saat rate-limited/sibuk**: yang gagal adalah *pengiriman* tx (429/timeout), bukan kebenaran settlement; FE menonaktifkan aksi tulis sementara + retry backoff, read dialihkan ke provider berikutnya. RPC **bukan** penentu pemenang saat rebutan beli (lihat [../development/concurrency-and-races.md](../development/concurrency-and-races.md)).

## Hosting (DIPUTUSKAN ronde 5: VPS sendiri)

- Frontend Next.js + API report + PostgreSQL berjalan di **VPS milik sendiri**.
- Kebutuhan turunan: reverse proxy + SSL (Caddy/nginx + Let's Encrypt), proses manager (Docker/systemd), **backup DB pg_dump harian → object storage**, **monitoring server + alert Telegram** (lihat deployment/monitoring.md).
- **Kontrol mainnet**: ownership kontrak berpindah ke Sputnik DAO V2 council 2-of-3 + timelock (ADR-013 — DECIDED) sebelum deploy mainnet.
- Staging (default ronde 13): subdomain terpisah di VPS yang sama, Docker Compose + DB terpisah.
- **Scaling (horizontal/vertical)**: rencana lengkap di [scaling.md](./scaling.md) — app stateless di belakang LB, read replica PostgreSQL, cache fase 2, CDN/WAF saat trafik naik.

## IPFS pinning

> **DIPUTUSKAN ronde 5: ditunda** — provider (Pinata / NFT.Storage / lainnya) diputuskan saat mulai bangun fitur mint.
> Sementara dev memakai gateway publik untuk aset contoh.

## Domain & SSL

- SSL: Let's Encrypt otomatis via **Caddy** (default) atau nginx+certbot — di VPS (ronde 5).
- Domain: ⏳ **open-by-design** — nama domain produksi belum ditentukan (pakai subdomain sementara saat dev).

## Biaya estimasi (kasar, bukan angka final)

- VPS: $5–20/bulan (2 GB RAM cukup untuk MVP).
- Domain: ~$10–15/tahun (⏳ belum dipilih).
- Storage kontrak: testnet gratis (faucet); mainnet ±1 Ⓝ per 100 KB state.
- Gas per user tx: < 0.01 Ⓝ (settlement bundle lebih besar — masih sangat murah).
- IPFS pinning: $0–20/bulan tergantung volume (ditunda — ronde 5).

---

## Environment matrix

> Mapping branch → environment: [../deployment/environments.md](../deployment/environments.md); pipeline: [../development/ci-cd.md](../development/ci-cd.md). Kolom bertanda ⏳ = nilai final saat deploy.

| Aspek | local | dev/staging | testnet | mainnet |
|---|---|---|---|---|
| Chain | localnet/sandbox | testnet | testnet | mainnet |
| RPC | node lokal / sandbox | `test.rpc.fastnear.com` → `rpc.testnet.near.org` → dRPC | sama (testnet) | `free.rpc.fastnear.com` → `rpc.mainnet.near.org` → dRPC |
| Kontrak | deploy per test | `dev.*` terpisah | `<nama>.testnet` | akun final |
| DB | opsional (Docker lokal) | PostgreSQL staging (terpisah) | PostgreSQL testnet | PostgreSQL produksi |
| Domain | `localhost` | subdomain staging ⏳ | subdomain testnet ⏳ | domain produksi ⏳ |
| TLS | self-signed/http | Let's Encrypt | Let's Encrypt | Let's Encrypt |
| Owner key | dev key | dev key | single owner key (interim, ADR-013) | **Sputnik DAO V2 2-of-3 + timelock** |
| Treasury | dev | dev | ⏳ ditetapkan saat deploy | treasury final |
| Secrets | `.env` lokal | secret dev (server) | secret testnet | secret produksi |
| Backup | tidak | opsional | harian → object storage | harian → object storage (SEC-DB-001) |
| Alert | tidak | opsional | Telegram (admin) | Telegram + uptime-kuma |

## Topologi jaringan & firewall

> Detail hardening: [../security/infrastructure-security.md](../security/infrastructure-security.md) §1–2. Diagram ini = MVP 1 VPS (ADR-009).

```text
                     Internet
                        │
             ┌──────────┴───────────┐
             │   ufw/nftables       │   80/443  ALLOW (in)
             │   (VPS firewall)     │   SSH 22  ALLOW (key-only, rate-limited)
             └──────────┬───────────┘   5432    DENY dari publik (localhost only)
                        │               3000/6432 DENY dari publik (internal)
                        ▼
        ┌───────────────────────────────────────────────┐
        │                   VPS                         │
        │  ┌──────────┐   :3000   ┌──────────────────┐  │
        │  │  Caddy   │──────────►│ Next.js (web)    │  │
        │  │ :80/:443 │           │ FE + API routes  │  │
        │  └──────────┘           └────────┬─────────┘  │
        │                                  │ :5432      │
        │                         ┌────────▼─────────┐  │
        │                         │ PostgreSQL (db)  │  │
        │                         │ localhost only   │  │
        │                         └──────────────────┘  │
        │  owner key dir (chmod 600, bukan web-root)    │
        │  .env.server (chmod 600)                      │
        └───────────────────────────────────────────────┘
                        │ egress (outbound 443)
                        ▼
        RPC provider · NearBlocks · IPFS gateway · object storage (backup)
```

| Port | Layanan | Akses | Aturan |
|---|---|---|---|
| 22 | SSH | admin saja | key-only, no root, fail2ban |
| 80 | HTTP | publik | redirect → 443 (ACME challenge) |
| 443 | HTTPS (Caddy) | publik | TLS + security headers |
| 3000 | Next.js | **internal** | hanya dari Caddy (PROPOSED) |
| 5432 | PostgreSQL | **localhost** | DENY publik (SEC-INFRA-002) |
| 6432 | PgBouncer (fase 2) | internal | hanya dari app |

- **Egress**: hanya ke RPC/NearBlocks/IPFS/object storage; indexer fase 2 = egress terbatas ke RPC/Neardata (infrastructure-security.md §3).

## DNS & sertifikat

> Domain produksi = ⏳ open-by-design (placeholder kanonik `nearsea.example`). DNS dicatat saat TASK-029.

| Record | Nama (placeholder) | Tipe | Nilai | TTL | Catatan |
|---|---|---|---|---|---|
| A | `nearsea.example` | A | IP VPS | 300 | origin (MVP tanpa CDN — G12 terbuka) |
| A | `www.nearsea.example` | A/CNAME | → apex | 300 | redirect ke apex |
| A | `staging.nearsea.example` | A | IP VPS | 300 | staging (DB terpisah) |
| A | `testnet.nearsea.example` | A | IP VPS | 300 | testnet publik |
| CAA | `nearsea.example` | CAA | `letsencrypt.org` | 3600 | batasi penerbit sertifikat |
| TXT (opsional) | `_dmarc` / SPF | TXT | kebijakan email | 3600 | hanya bila email ditambah (fase lanjut) |

**Renewal sertifikat (Let's Encrypt via Caddy)**:

```text
Caddy  → ACME challenge (HTTP-01 atau TLS-ALPN) → Let's Encrypt
       → sertifikat disimpan di volume caddy_data (persisten)
       → renewal otomatis ~30 hari sebelum expiry (Caddy)
       → monitoring: cron cek expiry → alert Telegram (monitoring.md)
```

- **Syarat**: port 80/443 terbuka; A record mengarah ke VPS; `CAA` mengizinkan Let's Encrypt.
- **Kegagalan renewal** = insiden (bukan hanya warning) → alert Telegram (monitoring.md Health checks).

## Sizing resource per layanan

> Angka **PROPOSED** (estimasi; dikalibrasi setelah beban nyata — scaling.md §10). Tier T0 = MVP 2 vCPU/2 GB.

| Layanan | RAM (T0) | CPU (T0) | Disk | Catatan |
|---|---|---|---|---|
| Caddy | ~50 MB | minimal | kecil | TLS + proxy |
| Next.js (web) | ~300–500 MB | 1 vCPU | — | SSR + API routes |
| PostgreSQL | ~500 MB–1 GB | 1 vCPU | 20–40 GB SSD | shared_buffers disesuaikan |
| PgBouncer (fase 2) | ~20 MB | minimal | — | pooling |
| Indexer (fase 2) | ~300–500 MB | 1 vCPU | checkpoint kecil | worker consumer |
| OS + overhead | ~300 MB | — | 5–10 GB | hardening |

- **Bottleneck pertama** biasanya PostgreSQL → naikkan tier sebelum menambah instance bila single-thread (scaling.md §2/§4).

## Provider RPC — SLA & latensi (di balik urutan fallback)

> Urutan fallback terkunci: **FASTNEAR → official → dRPC** (DECIDED). Daftar provider & endpoint: [indexer-security.md](../security/indexer-security.md) §2.

| Provider | Peran | SLA/karakter (catatan, bukan jaminan) | Rate limit | Kunci |
|---|---|---|---|---|
| FASTNEAR | primary | publik, latensi rendah | rate-limited (publik) | tidak (publik); berbayar untuk throughput |
| NEAR official | fallback 1 | `rpc.near.org`/`rpc.testnet.near.org`, rate-limited | ketat | tidak |
| dRPC | fallback 2 | `near-testnet.drpc.org` | sesuai paket | opsional |
| QuickNode/Tatum/ZAN | opsi throughput | berbayar, SLA komersial | sesuai paket | ya |

- **Health check**: bandingkan **block hash** antar provider; switch hanya di boundary blok final (indexer-security.md §2).
- **Perilaku rate limit**: yang gagal adalah *pengiriman* tx (429/timeout), bukan kebenaran settlement; FE menonaktifkan aksi tulis sementara + retry backoff, read dialihkan (concurrency-and-races.md §4).
- **SLA bukan jaminan MVP**: karena chain tetap otoritatif, kegagalan provider tidak mengubah state uang.

## Container & orkestrasi

| Aspek | MVP | Fase 2 |
|---|---|---|
| Orkestrasi | **Docker Compose** (1 VPS) | Compose + LB; opsi k8s bila instance banyak |
| Build | image Next.js (multi-stage) | sama |
| Deploy | CI SSH → `docker compose pull/up` | rolling / blue-green (PROPOSED) |
| State | volume `pgdata`, `caddy_data` | + volume checkpoint indexer |
| Immutability | konfigurasi via deploy, bukan edit manual | sama |
| Rollback | deploy ulang tag/commit sebelumnya | revert merge + deploy |

- **Service/volume/port detail**: [backend-architecture.md](./backend-architecture.md) §7.
- **Alternatif ditolak**: serverless (Vercel) & managed DB (Supabase) — ditolak di ADR-009 (kontrol + konsistensi engine + biaya).

## Model biaya

> Estimasi **kasar, bukan angka final**; mata uang USD/bulan. Angka di selaras dengan scaling.md §10.

| Pos | MVP (T0) | T1 | T2 | Catatan |
|---|---|---|---|---|
| VPS | $5–20 | $20–40 | $40–80 | 2/4/8 vCPU |
| Domain | ~$1/bulan ($10–15/th) | sama | sama | ⏳ belum dipilih |
| Storage kontrak | testnet gratis (faucet) | gratis | mainnet ±1 Ⓝ/100 KB state | mainnet gate |
| Gas user tx | < 0.01 Ⓝ/tx | sama | sama | settlement bundle lebih besar |
| IPFS pinning | $0–20 | $0–20 | $20–50 | ditunda (ronde 5) |
| Object storage backup | ~$1–5 | ~$1–5 | ~$5–10 | pg_dump harian |
| RPC berbayar (opsional) | $0 | $0–50 | $50–200 | bila rate limit mengganggu |
| CDN/WAF (opsional) | $0 | $0 | $0–20 | menutup G12 (scaling.md §6) |
| **Total kasar** | **~$7–45** | **~$25–115** | **~$120–365** | indikatif |

- **Driver biaya**: RPC throughput & storage (bila mainnet), bukan compute MVP. Detail tier & pemicu: [scaling.md](./scaling.md) §10/§9.
