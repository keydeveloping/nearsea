# Deployment

## Alur rilis

```text
local dev → sandbox test → testnet deploy → verifikasi manual → mainnet (keputusan eksplisit user)
```

## Kontrak

- Build: `cargo near build` → `out/*.wasm` (**workflow reproducible — NEP-330**; verifikasi via `contract_source_metadata`)
- Deploy testnet: `cargo near deploy` dengan init `new(owner_id)` — **CATATAN MAINNET: `owner_id` = akun Sputnik DAO V2 council 2-of-3 + timelock, BUKAN wallet pribadi** (ADR-013)
- Upgrade kontrak: redeploy ke akun sama (state persist); perubahan struktur state → `#[init(ignore_state)]` migrate (docs resmi); break-glass terakhir: State Cleaner
- Global contract (hemat storage) sebagai opsi — RESEARCH.md bagian 4

## Frontend / Backend (self-hosted VPS)

- Next.js self-hosted di VPS — Docker atau node + reverse proxy (Caddy/nginx) + SSL Let's Encrypt.
- API report + PostgreSQL di VPS yang sama; backup DB harian wajib (lihat disaster-recovery.md).
- CI/CD: **GitHub Actions** → build & test → deploy ke VPS via SSH (docker compose pull / rsync + restart). Preview: branch deploy ke subdomain staging.

## Checklist rilis kontrak

- [ ] Semua test sandbox hijau (unit + cross-contract, lihat testing-strategy)
- [ ] Checklist keamanan NEAR ([RESEARCH.md](../../RESEARCH.md) bag. 3) + threat-model direview ulang
- [ ] `cargo clippy` tanpa warning; overflow-checks aktif di release profile
- [ ] Argumen method & event schema terdokumentasi di docs/features + api/webhooks
- [ ] Alamat kontrak tercatat di environments.md; init `new(owner_id)` benar (mainnet: owner = DAO)
- [ ] Pausable teruji (pause → mutasi ditolak; withdraw escrow tetap bisa)
- [ ] **Reproducible build terverifikasi (NEP-330 — SEC-CONTRACT-006)**
- [ ] **Audit eksternal lulus (TASK-027) — gate khusus mainnet** ⚠️ *hanya bila proyek beralih ke jalur produk (ADR-016). Proyek ini saat ini pembelajaran/portofolio → gate ini **tidak wajib**.*

## Prasyarat deploy

| Kebutuhan | local | dev/staging | testnet | mainnet |
|---|---|---|---|---|
| `near-cli` + `cargo-near` | ✓ | ✓ | ✓ | ✓ |
| Akses SSH VPS (deploy key) | — | ✓ | ✓ | ✓ (approval) |
| Akses tulis branch | lokal | `dev` | `testnet` | `mainnet` (manual) |
| Secret server (`.env.server`) | `.env` lokal | secret dev | secret testnet | secret produksi |
| Approval | tidak | tidak | 1 (environment) | reviewer + persetujuan user |
| Gate tambahan | — | — | reproducible hash | audit TASK-027 + owner DAO |

> Status tooling & akun: TASK-028/029. Nilai konkret per env: [environments.md](./environments.md).
> Versi `cargo-near`/`near-cli` di-pin di scaffold (TASK-001); flag di bawah mengikuti versi yang di-pin.

## Runbook: deploy kontrak

### 1. Build reproducible (NEP-330)

```bash
# Dari root repo, pada commit/tag yang akan dirilis.
git fetch --tags
git checkout contract-v0.1.0        # tag rilis (SemVer per-artefak)

# Build reproducible — output di contract/out/
cd contract
cargo near build

# Catat hash artifact (dibandingkan dengan artifact CI & metadata on-chain).
sha256sum out/*.wasm | tee /tmp/nearsea-contract-hashes.txt
```

### 2. Verifikasi hash lokal == artifact CI

```bash
# Unduh artifact wasm dari run CI yang lulus (tag sama), lalu bandingkan.
sha256sum -c /tmp/nearsea-contract-hashes.txt
```

- Hash berbeda = build tidak reproducible → **jangan deploy**; investigasi toolchain (SEC-CONTRACT-006).

### 3. Deploy (testnet)

```bash
# Login akun deploy (key tidak pernah dipakai app — key-management.md).
near login

# Deploy kontrak + init. Ganti <account> dan nilai init sesuai lingkungan.
cargo near deploy <market-account>.testnet \
  with-init-call new \
  '{"owner_id":"<owner-account>.testnet"}'

cargo near deploy <nft-account>.testnet \
  with-init-call new \
  '{"owner_id":"<owner-account>.testnet","metadata":{}}'
```

> **Mainnet**: `owner_id` = akun Sputnik DAO V2 council 2-of-3 (BUKAN wallet pribadi) + timelock
> (ADR-013); deploy hanya via `deploy-mainnet.yml` (manual) setelah gate TASK-027 + approval user.
> Jika owner = akun DAO, `pause` ditangani `pause_callers` (guardian) — lihat key-management.md §3a.

### 4. Verifikasi pasca-deploy (kontrak)

```bash
# 1) Baca code hash on-chain dan bandingkan dengan artifact CI.
near state <market-account>.testnet --networkId testnet    # kolom code_hash

# 2) Baca metadata NEP-330 (versi + sumber + hash build).
near view <market-account>.testnet contract_source_metadata '{}' --networkId testnet

# 3) Sanity call method view.
near view <market-account>.testnet get_fee_bps '{}' --networkId testnet
```

- Hash cocok + metadata versi benar = deploy sah (SEC-CONTRACT-006).
- Hash **tidak** cocok → deploy dianggap gagal → rollback ke versi sebelumnya (lihat Rollback).

### 5. Upgrade / migrate state

```bash
# Upgrade = redeploy wasm baru ke AKUN YANG SAMA (state persist).
cargo near deploy <account>.testnet out/<market>.wasm

# Bila storage layout berubah → panggil migrate (#[init(ignore_state)]).
near call <account>.testnet migrate '{}' \
  --accountId <owner-account>.testnet --gas 300000000000000 --networkId testnet
```

- Storage layout didokumentasikan per rilis (SEC-CONTRACT-008) + dicatat di CHANGELOG bagian kontrak.
- Break-glass terakhir: State Cleaner (docs resmi) — hanya dengan backup state yang terverifikasi.

## Runbook: deploy FE/API (VPS)

### 1. Prasyarat di server (sekali, saat provisioning — TASK-028)

```bash
sudo install -d -m 750 /opt/nearsea
sudo install -d -m 700 /opt/nearsea/.env
# .env.server chmod 600, di luar web-root, terpisah dari app user.
sudo chmod 600 /opt/nearsea/.env.server
```

### 2. Deploy (dipicu CI atau manual)

```bash
# CI deploy-dev/testnet/mainnet menjalankan ini via SSH (lihat ci-cd.md).
cd /opt/nearsea
docker compose pull
docker compose up -d
docker compose ps
```

### 3. Migration database

```bash
# Jalankan migration Prisma (bukan edit manual — database/migrations.md).
docker compose run --rm web npx prisma migrate deploy

# Verifikasi tidak ada drift/pending.
docker compose run --rm web npx prisma migrate status
```

- Migration destruktif / menyentuh mainnet butuh approval + backup terverifikasi (migrations.md §8).
- Rollback migration: `docker compose run --rm web npx tsx scripts/migrate-down.ts <migration>` (membaca `down.sql`).

### 4. Strategi zero-downtime

```text
MVP (1 instance)   : `docker compose up -d` me-recreate container `web`; Caddy tetap melayani.
                     Jeda ~detik → mitigasi: health check + retry FE, atau image siap dulu lalu
                     `docker compose up -d --no-deps web`.
Fase 2 (2 instance) : rolling/blue-green di belakang Caddy — scaling.md §5.
Database            : pola expand-contract (migrations.md §4) — jangan rename/drop kolom yang
                      masih dibaca; index baru CREATE INDEX CONCURRENTLY (di luar transaksi).
```

- Urutan aman rilis: migrate DB (expand) → deploy app baru → hapus kolom lama (contract) di rilis berikutnya.
- Caddy tidak di-recreate saat deploy app → koneksi TLS tetap hidup.

## SSL / sertifikat

```bash
# Caddy: renewal otomatis (~30 hari sebelum expiry). Cek status.
docker compose logs caddy | grep -i "certificate"

# Cek expiry dari luar.
echo | openssl s_client -connect <domain>:443 -servername <domain> 2>/dev/null \
  | openssl x509 -noout -dates

# Reload konfigurasi Caddy tanpa downtime.
docker compose exec caddy caddy reload --config /etc/caddy/Caddyfile

# nginx+certbot (alternatif): uji renewal tanpa benar-benar menerbitkan.
sudo certbot renew --dry-run
```

- Kegagalan renewal = insiden → alert Telegram (monitoring.md).
- CAA record harus mengizinkan `letsencrypt.org` (infrastructure.md §DNS & sertifikat).

## Verifikasi pasca-deploy (aplikasi)

```bash
APP_URL="https://<domain>"
curl -fsS "$APP_URL/api/health"                                  # health endpoint kanonik
curl -fsS -o /dev/null -w '%{http_code}\n' "$APP_URL/"           # FE 200
curl -fsS -o /dev/null -w '%{http_code}\n' "$APP_URL/api/reports" # contoh endpoint API

# DB reachable dari dalam container.
docker compose exec web npx prisma migrate status

# Log terakhir tanpa error.
docker compose logs --tail=100 web
```

## Smoke test frontend (pasca-deploy)

| # | Cek | Harapan |
|---|---|---|
| 1 | `/api/health` | 200 + status DB ok |
| 2 | Halaman beranda | render listing/empty-state tanpa error console |
| 3 | Connect wallet | near-connect membuka wallet, sesi terbentuk |
| 4 | Browse + buka detail NFT | data dari RPC/DB tampil, gambar dari IPFS gateway |
| 5 | Aksi tulis (uji testnet) | tx terkirim; UI menunggu konfirmasi |
| 6 | API report | POST report → 2xx, tercatat di antrean admin |
| 7 | i18n & tema | string tidak hardcode, bahasa sesuai |
| 8 | CSP & header | header keamanan terpasang, tidak ada CSP violation |

## Rollback

| Artefak | Langkah |
|---|---|
| FE/API | `git checkout <tag-sebelumnya>` → `docker compose pull && docker compose up -d`; atau pin image tag rilis sebelumnya |
| Database | jalankan `down.sql` (migrate-down.ts) bila aman; jika tidak → expand-contract; **jangan** rollback destruktif mainnet tanpa approval |
| Kontrak | redeploy wasm versi sebelumnya ke akun sama; bila storage berubah → jalankan migrate yang sesuai |
| DNS/SSL | kembalikan record/tag; Caddy reload |

```bash
# Contoh rollback FE ke tag sebelumnya.
cd /opt/nearsea
git fetch --tags
git checkout web-v0.1.0
docker compose pull
docker compose up -d
docker compose exec web npx prisma migrate status   # pastikan schema cocok
```

- Setiap rollback dicatat di [CHANGELOG.md](../../CHANGELOG.md) (bagian artefak terkait).
- Rollback kontrak mainnet = aksi owner → lewat Sputnik DAO + timelock (ADR-013), bukan key tunggal.
