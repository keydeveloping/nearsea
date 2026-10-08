# Environments

> Peta branch → environment: [development/git-workflow.md](../development/git-workflow.md).
> Pipeline deploy: [development/ci-cd.md](../development/ci-cd.md).

## Daftar environment

| Env | Jaringan | Kontrak | Data | Kepemilikan | Branch |
|---|---|---|---|---|---|
| local | localnet/sandbox | deploy per test | seed | dev | (lokal, tanpa branch) |
| dev | NEAR testnet (RPC testnet) | akun dev terpisah (`dev.*`) | seed/dev | tim dev | `dev` |
| testnet | NEAR testnet | akun `<nama>.testnet` | publik/faucet | tim | `testnet` |
| mainnet | NEAR mainnet | akun final | asli | **Sputnik DAO V2 council 2-of-3 + timelock** (ADR-013 — DECIDED) | `mainnet` |

> **Catatan devnet**: NEAR tidak menyediakan jaringan "devnet" terpisah. Branch `dev` memakai
> **RPC testnet** dengan **kontrak & subdomain staging terpisah** (namespace `dev.*`) agar
> developer tidak menimpa rilis testnet publik.

## Mapping branch → environment → deploy

| Branch | Environment | Trigger | Approval |
|---|---|---|---|
| `dev` | dev/staging | push otomatis **(aktif saat TASK-029)** | tidak |
| `testnet` | testnet | push (gate CI) **(aktif saat TASK-029)** | 1 (environment) |
| `mainnet` | mainnet | manual (`workflow_dispatch`) | wajib reviewer |

> **Status TASK-001:** workflow `deploy-*.yml` sudah ada tapi **manual-only** (`workflow_dispatch`) dan gagal cepat bila variabel environment belum diisi — environment (VPS + subdomain) baru di-provision di TASK-028 dan wiring SSH di TASK-029 ([ci-cd.md](../development/ci-cd.md) §7).

## Variabel environment (final — nilai diisi saat deploy/TASK-001)

- `NEAR_NETWORK` (testnet|mainnet) — PUBLIC
- `NEAR_RPC_URL` + `NEAR_RPC_FALLBACKS` (FASTNEAR → official → dRPC) — PUBLIC
- `MARKET_CONTRACT_ID`, `FACTORY_CONTRACT_ID` — PUBLIC
- `DATABASE_URL` (PostgreSQL report API) — SECRET
- `JWT_SECRET` (session API) — SECRET
- `TELEGRAM_BOT_TOKEN` + `TELEGRAM_CHAT_ID` (alert admin) — SECRET
- `DEPLOY_SSH_KEY` (GitHub Secret, bukan env file) — SECRET
- `BACKUP_S3_ENDPOINT` + `BACKUP_S3_KEY` + `BACKUP_S3_SECRET` (object storage backup pg_dump) — SECRET
- `IPFS_GATEWAY`, `IPFS_PINNING_KEY` — **fase lanjut** (saat fitur mint — ronde 5)

> **Frontend membaca versi ber-prefix** (TASK-007 + TASK-008): Next.js hanya mengekspos variabel
> `NEXT_PUBLIC_*` ke bundle browser, jadi aplikasi membaca `NEXT_PUBLIC_NEAR_NETWORK`,
> `NEXT_PUBLIC_NEAR_RPC_URL`, `NEXT_PUBLIC_NEAR_RPC_FALLBACKS` (modul `lib/near/network.ts`) dan
> `NEXT_PUBLIC_MARKET_CONTRACT_ID` (modul `lib/near/contracts.ts`). `.env.example` memuat keduanya
> agar tidak ada nilai yang berbeda antara proses server dan browser. Nilai jaringan selain
> `testnet`/`mainnet` = gagal saat start; alamat market yang kosong hanya menampilkan keadaan
> "belum dikonfigurasi" (build & halaman baca tetap jalan sebelum deploy).

Template aman (placeholder saja): [.env.example](../../.env.example).

## Klasifikasi

- PUBLIC → boleh di frontend (NEXT_PUBLIC_*).
- SECRET → hanya server/CI (dilarang di repo); kebijakan lengkap di [development/secrets-and-gitignore.md](../development/secrets-and-gitignore.md).

## Nilai konkret per environment (placeholder bila belum diputuskan)

> Angka/URL final diisi saat provisioning (TASK-028/029). Tanda ⏳ = belum diputuskan (**open-by-design**).
> Dokumen ini **tidak** memuat kredensial nyata — hanya placeholder.

| Aspek | local | dev/staging | testnet | mainnet |
|---|---|---|---|---|
| App URL | `http://localhost:3000` | `https://staging.nearsea.example` ⏳ | `https://testnet.nearsea.example` ⏳ | `https://nearsea.example` ⏳ |
| Subdomain | — | `staging.` | `testnet.` | apex + `www` |
| Jaringan NEAR | localnet/sandbox | testnet | testnet | mainnet |
| RPC primary | node lokal / sandbox | `https://test.rpc.fastnear.com` | `https://test.rpc.fastnear.com` | `https://free.rpc.fastnear.com` |
| RPC fallback | — | `https://rpc.testnet.near.org`, `https://near-testnet.drpc.org` | idem dev | `https://rpc.mainnet.near.org`, dRPC |
| DB host | `localhost:5432` (Docker) | `db` (Compose internal) | `db` | `db` |
| DB name | `nearsea_local` | `nearsea_dev` | `nearsea_testnet` | `nearsea_mainnet` |
| DB publik? | tidak | tidak | tidak | tidak (SEC-INFRA-002) |
| Factory contract | deploy per test | `dev.nearsea.testnet` | `nearsea.testnet` | ⏳ akun final |
| Market contract | deploy per test | `market.dev.nearsea.testnet` | `market.nearsea.testnet` | ⏳ akun final |
| NFT collection | deploy per test | `<slug>.dev.nearsea.testnet` | `<slug>.nearsea.testnet` | ⏳ |
| Owner | dev key | dev key | single owner key (interim) | Sputnik DAO V2 2-of-3 + timelock |
| Backup | tidak | opsional | harian → object storage | harian → object storage |
| Alert | tidak | opsional | Telegram (admin) | Telegram + uptime-kuma |
| GitHub environment | — | `dev` | `testnet` | `mainnet` (required reviewer) |

- Nama provider/RPC lengkap & urutan fallback: [../architecture/infrastructure.md](../architecture/infrastructure.md).
- **Jangan** menaruh IP VPS/hostname nyata di dokumen ini — gunakan placeholder sampai TASK-029 menetapkan DNS.

## Konvensi penamaan

> Placeholder kanonik domain: `nearsea.example` (lihat infrastructure.md §DNS). ⏳ domain produksi belum diputuskan.

| Objek | Pola | Contoh | Catatan |
|---|---|---|---|
| Subdomain dev | `staging.<domain>` | `staging.nearsea.example` | DB terpisah di VPS yang sama |
| Subdomain testnet | `testnet.<domain>` | `testnet.nearsea.example` | publik |
| Domain produksi | `<domain>` + `www` | `nearsea.example` | redirect `www` → apex |
| Factory (testnet) | `nearsea.testnet` | `nearsea.testnet` | akun induk |
| Market (testnet) | `market.nearsea.testnet` | `market.nearsea.testnet` | |
| Koleksi (testnet) | `<slug>.nearsea.testnet` | `punks.nearsea.testnet` | sub-akun dari factory |
| Namespace dev | `<peran>.dev.nearsea.testnet` | `market.dev.nearsea.testnet` | agar tidak menimpa testnet publik |
| Database | `nearsea_<env>` | `nearsea_testnet` | satu DB per env |
| Container/service | `caddy`, `web`, `db` | — | backend-architecture §7 |
| Volume | `pgdata`, `caddy_data`, `caddy_config` | — | persisten |
| Image tag | `<artefak>-<semver>` | `web-v0.1.0` | versioning-and-release.md |

## Bootstrap lokal (local)

```bash
# 1. Clone + env lokal (jangan commit .env).
git clone <repo-url> nearsea && cd nearsea
cp .env.example .env            # isi nilai lokal; .env TIDAK di-commit
chmod 600 .env

# 2. Kontrak: build + test (workspace contract/market/factory).
cargo test --workspace          # unit (host)
cargo near build non-reproducible-wasm --no-abi --manifest-path contract/Cargo.toml
#   ulangi untuk market/ dan factory/ → target/near/<crate>/<crate>.wasm

# 3. Database lokal (Docker) — belum ada sampai TASK-018/028.
docker compose -f docker-compose.dev.yml up -d db
docker compose -f docker-compose.dev.yml run --rm web pnpm prisma migrate deploy
docker compose -f docker-compose.dev.yml run --rm web pnpm prisma db seed   # bila ada seed

# 4. FE/API dev.
cd frontend
pnpm install --frozen-lockfile
pnpm dev      # http://localhost:3000
```

- NEAR tidak menyediakan jaringan "devnet" terpisah: local memakai **localnet/sandbox** (`near-sandbox`)
  atau testnet dengan kontrak lokal.
- Deploy kontrak lokal: `near dev-deploy` atau `cargo near deploy` ke akun dev lokal (bukan akun bersama).

## Config drift

| Sumber drift | Deteksi | Tindakan |
|---|---|---|
| Env var berbeda antara repo & server | CI banding `.env.example` vs `docker compose config` | perbarui `.env.server`/GitHub vars |
| Image tag tidak sesuai rilis | `docker compose ps` vs tag target | deploy ulang tag benar |
| Schema DB tertinggal | `prisma migrate status` | jalankan `migrate deploy` |
| Konfigurasi Caddy diubah manual | diff `Caddyfile` vs repo | kembalikan ke repo (immutable deploy) |
| Kontrak on-chain ≠ metadata NEP-330 | `near view … contract_source_metadata` | redeploy artifact terverifikasi |

- Aturan: **konfigurasi produksi tidak diubah manual di server** (infrastructure-security §6); semua lewat repo + pipeline.
- Drift = insiden kecil → catat di [../security/incident-response.md](../security/incident-response.md).

## Klasifikasi PUBLIC vs SECRET per environment

| Variabel | Kelas | local | dev | testnet | mainnet |
|---|---|---|---|---|---|
| `NEAR_NETWORK` | PUBLIC | `testnet` | `testnet` | `testnet` | `mainnet` |
| `NEAR_RPC_URL` | PUBLIC | node lokal | FASTNEAR testnet | FASTNEAR testnet | FASTNEAR mainnet |
| `NEAR_RPC_FALLBACKS` | PUBLIC | — | official, dRPC | official, dRPC | official, dRPC |
| `MARKET_CONTRACT_ID` | PUBLIC | akun lokal | `market.dev.nearsea.testnet` | `market.nearsea.testnet` | ⏳ final |
| `FACTORY_CONTRACT_ID` | PUBLIC | akun lokal | `dev.nearsea.testnet` | `nearsea.testnet` | ⏳ final |
| `DATABASE_URL` | SECRET | lokal | server dev | server testnet | server produksi |
| `JWT_SECRET` | SECRET | lokal | server dev | server testnet | server produksi |
| `TELEGRAM_BOT_TOKEN` | SECRET | — | opsional | server | server |
| `TELEGRAM_CHAT_ID` | SECRET | — | opsional | server | server |
| `DEPLOY_SSH_KEY` | SECRET (KEY) | — | GitHub Secret | GitHub Secret | GitHub Secret |
| `BACKUP_S3_ENDPOINT/KEY/SECRET` | SECRET | — | opsional | server | server |
| `IPFS_GATEWAY` | PUBLIC | publik | publik | publik | publik |
| `IPFS_PINNING_KEY` | SECRET | — | fase lanjut | fase lanjut | fase lanjut |

- PUBLIC boleh di frontend (`NEXT_PUBLIC_*`); SECRET hanya server/CI (secrets-and-gitignore.md).
- Template aman: [.env.example](../../.env.example).
