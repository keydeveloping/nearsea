# NearSea — NFT Marketplace di NEAR

> Codename **"NearSea"** (diputuskan di ronde tanya-jawab ke-3 — log lengkap: [docs/decisions/ADR-001.md](./docs/decisions/ADR-001.md)) — nama final bisa diganti sebelum rilis.

`status: pre-release` · `milestone: M0 (fondasi & scaffold)` · `network: testnet` · `build: ⏳` `ci: ⏳`

> Baris status di atas = **badge placeholder** (teks). Badge gambar (shields.io) ditambahkan saat repo publik/CI menyala (TASK-001/029); nilai mengikuti [docs/00-project-overview.md](./docs/00-project-overview.md) § Status.

## Description

Marketplace NFT di NEAR Protocol dengan fitur setara OpenSea: fixed-price listing, offers, private listing, bundle, collections (launchpad berphase), royalti on-chain, activity feed, profil custom, dan notifikasi in-app. Auctions menyusul di fase 2. Non-custodial — user memegang wallet sendiri.

## Core Purpose

Marketplace NFT **terbuka** untuk ekosistem NEAR — setara OpenSea. Value proposition: gas murah & transaksi cepat; **fee platform 2% dengan royalti kreator dipaksa on-chain** (fee OpenSea kini ~1% setelah perubahan 2025 — keunggulan kita bukan fee, tapi royalti terjamin + orderbook on-chain + launchpad); UX simpel untuk semua level; **launchpad koleksi ala OpenSea Studio** (phase mint bebas: allowlist/public, allowlist upload). Target audiens: trader NFT berpengalaman, kreator/artist, newcomer crypto, dan komunitas NEAR.

## Tech Stack

- Smart contract: Rust (`near-sdk`, `near-sdk-contract-tools`), standar NEP-171/177/178/181/199/297
- Frontend: Next.js + TypeScript + Tailwind, wallet via near-connect, UI English siap i18n
- Data: view call RPC langsung + NearBlocks API (MVP); custom indexer fase 2
- Deploy: testnet dulu (ronde 3); **mainnet M4 setelah audit eksternal lulus (TASK-027)**

Detail: [docs/architecture/tech-stack.md](./docs/architecture/tech-stack.md)

## Repository Structure

- `RESEARCH.md` — hasil riset NEAR (standar, tutorial, market-contract, peta fitur OpenSea)
- `CONTEXT.md` — glosarium domain: istilah kanonik + istilah yang dilarang dipakai (baca sebelum menulis dokumen/kode)
- `arsitektur-dokumentasi-app-kompleks.md` — panduan struktur dokumentasi ini
- `docs/` — source of truth: produk → arsitektur → data/API → fitur → security → development → testing → deployment
- `tasks/` — backlog, milestones, implementation plan
- `CHANGELOG.md` — catatan perubahan (SemVer per-artefak)
- `.github/workflows/` — pipeline CI/CD (CI, security scan, deploy per-branch)
- `.gitignore` / `.env.example` — higienitas file & template environment
- `LICENSE` — MIT
- (kode akan ditambah: `contract/`, `market/`, `factory/`, `frontend/`, `indexer/` — final via tanya-jawab)

### Pohon repo (aktual saat ini)

```text
Near Marketplace/
├── README.md                  # pintu masuk (file ini)
├── AGENTS.md                  # aturan kerja AI agent (+ konvensi repo)
├── CONTEXT.md                 # glosarium domain (istilah kanonik)
├── RESEARCH.md                # riset teknis NEAR
├── LICENSE                    # MIT (diputuskan ronde 17)
├── CHANGELOG.md               # catatan perubahan per-artefak
├── arsitektur-dokumentasi-app-kompleks.md
├── AUDIT-TEMUAN.md            # hasil audit dokumen
├── .env.example               # template env (placeholder saja)
├── .gitignore                 # daftar ignore ketat
├── .gitattributes             # normalisasi line ending
├── .editorconfig              # konsistensi indentasi/encoding
├── .gitleaks.toml             # konfigurasi secret scan
├── rust-toolchain.toml        # pin toolchain Rust (1.77.1) — SSOT versi
├── .nvmrc                     # pin Node (20) — SSOT versi
├── .github/
│   ├── workflows/
│   │   ├── ci.yml             # fmt/clippy/test + lint/build + artifact
│   │   ├── security.yml       # gitleaks + audit + dependency review
│   │   ├── deploy-dev.yml     # auto-deploy staging
│   │   ├── deploy-testnet.yml # auto-deploy testnet (gate CI)
│   │   └── deploy-mainnet.yml # manual + approval + gate
│   └── (CODEOWNERS, dependabot.yml — PROPOSED, ditambah saat scaffold)
├── docs/                      # source of truth
│   ├── 00-project-overview.md … 04-ux-ui-spec.md
│   ├── DOCUMENTATION-MAP.md   # peta sinkronisasi dokumen
│   ├── agents/                # konfigurasi konsumsi skill engineering (domain, tracker, label)
│   ├── api/                   # kontrak API
│   ├── architecture/          # sistem, FE, backend, infra, scaling, stack
│   ├── contracts/             # reference implementasi kontrak (nft-collection, market, factory)
│   ├── database/              # schema, data model, migration, seed
│   ├── decisions/             # ADR-001..016
│   ├── deployment/            # env, deploy, monitoring, DR
│   ├── development/           # git, versioning, CI/CD, kode, error, concurrency, secret
│   ├── features/              # auth, users, marketplace, payments, notifications, launchpad, report
│   ├── research/              # riset pasar/ekosistem (bertanggal, berlabel FACT/INFERENCE)
│   ├── security/              # threat model, SEC-*, protokol, key management
│   └── testing/               # strategi, test case, acceptance criteria
└── tasks/
    ├── backlog.md             # TASK-001..034
    ├── milestones.md          # M0..M4
    └── implementation-plan.md # urutan kerja per fase
```

> Struktur kode (`contract/`, `market/`, `factory/`, `frontend/`, `indexer/`) dibuat pada TASK-001 (Fase 1) — lihat [tasks/implementation-plan.md](./tasks/implementation-plan.md).

## Prerequisites

> Versi mengikuti file pin ([`rust-toolchain.toml`](./rust-toolchain.toml), [`.nvmrc`](./.nvmrc)) dan yang dipakai CI ([.github/workflows/ci.yml](./.github/workflows/ci.yml)) — samakan agar build lokal = CI.

| Alat | Versi | Catatan |
|---|---|---|
| **Rust** | **1.77.1** — SSOT: [`rust-toolchain.toml`](./rust-toolchain.toml) | `rustup` otomatis memakai versi ter-pin; target `wasm32-unknown-unknown` sudah didaftarkan di file itu |
| `cargo-near` | terbaru | `cargo install cargo-near` (build/deploy wasm) |
| **Node.js** | **20 LTS** — SSOT: [`.nvmrc`](./.nvmrc) | CI memakai Node 20 |
| **pnpm** | **9.x** | package manager FE (CI memakai pnpm 9) |
| **PostgreSQL** | 15+ (Docker) | untuk API/report & migration |
| Docker + Compose | terbaru | DB lokal & stack VPS |
| `near-cli-rs` | terbaru | interaksi akun/kontrak (opsional untuk dev) |
| `gitleaks` | terbaru | secret scan lokal (opsional; CI wajib) |
| `pre-commit` | terbaru | hook lokal (PROPOSED — [code-standards.md](./docs/development/code-standards.md) §14) |

- Akun testnet + faucet NEAR diperlukan untuk uji end-to-end ([docs/testing/testing-strategy.md](./docs/testing/testing-strategy.md) § Penyediaan environment E2E).
- **Jangan** pernah memakai akun/dana mainnet untuk development.

## Quickstart (lokal)

```bash
# 1. Clone + siapkan env (JANGAN commit .env).
git clone <repo-url> nearsea && cd nearsea
cp .env.example .env            # isi nilai lokal; .env TIDAK di-commit
chmod 600 .env

# 2. Kontrak — build + test (sandbox/localnet).
cd contract
cargo near build                # build wasm
cargo test                      # unit + sandbox (near-workspaces)
cd ..

# 3. Database lokal (Docker).
docker compose -f docker-compose.dev.yml up -d db
docker compose -f docker-compose.dev.yml run --rm web pnpm prisma migrate deploy

# 4. Frontend + API (dev).
cd frontend
pnpm install --frozen-lockfile
pnpm dev                        # http://localhost:3000

# 5. Pemeriksaan wajib sebelum PR (sama dengan gate CI).
cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings
pnpm --dir frontend lint && pnpm --dir frontend typecheck && pnpm --dir frontend test
gitleaks detect --config .gitleaks.toml --redact   # secret scan
```

- Perintah di atas = **target**; sebagian baru jalan setelah scaffold TASK-001 (manifest belum ada). Sketsa awal sebelumnya digantikan langkah ini.
- Bootstrap lengkap per environment: [docs/deployment/environments.md](./docs/deployment/environments.md) § Bootstrap lokal.
- Alur kerja branch & PR: [docs/development/git-workflow.md](./docs/development/git-workflow.md). Rilis & versi: [docs/development/versioning-and-release.md](./docs/development/versioning-and-release.md).

## Contributing

- **Baca dulu:** [AGENTS.md](./AGENTS.md) (aturan kerja) dan [docs/DOCUMENTATION-MAP.md](./docs/DOCUMENTATION-MAP.md) (aturan sinkronisasi dokumen) — keduanya mengikat.
- **Alur:** branch `feat/*` dari `dev` → PR ke `dev` (CI hijau + 1 approval) → promosi `dev → testnet → mainnet`.
  Merge/deploy ke `testnet`/`mainnet` **wajib persetujuan user** ([git-workflow.md](./docs/development/git-workflow.md) §3).
- **Commit:** conventional commits (`feat:`, `fix:`, `docs:`, `test:`, `chore:`, `security:`); sebut ID task bila menutup task.
- **Test:** setiap method mutasi kontrak & setiap endpoint API wajib punya test; PR tanpa hasil test jelas tidak di-merge ([testing-strategy.md](./docs/testing/testing-strategy.md)).
- **Secret:** dilarang menulis kredensial usable di source/contoh/test ([secrets-and-gitignore.md](./docs/development/secrets-and-gitignore.md)).
- Belum ada file `CONTRIBUTING.md` terpisah — bagian ini + AGENTS.md adalah panduan kontribusi sampai dibuat.

## License

**MIT** — diputuskan ronde 17 (2026-10-07). File [`LICENSE`](./LICENSE) ada di repo.

Alasan: proyek ini **pembelajaran/portofolio** ([ADR-016](./docs/decisions/ADR-016-project-intent-and-scope-cut.md)),
bukan produk komersial. MIT paling permisif dan paling umum untuk proyek terbuka — orang lain boleh
belajar dari kode ini. Sebelumnya berstatus "semua hak dilindungi" (tanpa lisensi = tanpa izin pakai
ulang), yang keliru untuk proyek yang tujuannya dipelajari.

## Branch & Versioning

- **Tiga branch permanen**: `mainnet` (produksi), `testnet` (rilis testnet), `dev` (integrasi developer).
  Alur: `feat/*` → `dev` → `testnet` → `mainnet`. **Merge/deploy ke `testnet`/`mainnet` wajib persetujuan user.**
  Detail: [docs/development/git-workflow.md](./docs/development/git-workflow.md).
- **SemVer per-artefak** (tag `contract-vX.Y.Z`, `web-vX.Y.Z`, `indexer-vX.Y.Z`).
  Detail: [docs/development/versioning-and-release.md](./docs/development/versioning-and-release.md).

## Development

> Perintah lengkap ada di [Quickstart (lokal)](#quickstart-lokal) di atas. Ringkas:

```bash
# Kontrak (Rust)
cargo near build                      # build wasm
cargo test                            # unit + sandbox (near-workspaces)

# Frontend (pnpm — package manager final)
cd frontend && pnpm install && pnpm dev

# Lengkap: docs/deployment/deployment.md
```

## Documentation

| Dokumen | Isi |
|---|---|
| [DOCUMENTATION-MAP](./docs/DOCUMENTATION-MAP.md) | **Peta sinkronisasi dokumen** — single source of truth, sync triggers, checklist crosscheck |
| [CONTEXT](./CONTEXT.md) | Glosarium domain — istilah kanonik + istilah terlarang |
| [00-project-overview](./docs/00-project-overview.md) | Ringkasan proyek |
| [01-PRD](./docs/01-PRD.md) | Product Requirements Document |
| [02-product-requirements](./docs/02-product-requirements.md) | Behavior specification detail |
| [03-user-flows](./docs/03-user-flows.md) | Perjalanan user |
| [04-ux-ui-spec](./docs/04-ux-ui-spec.md) | Spesifikasi UI/UX |
| [architecture/](./docs/architecture/tech-stack.md) | Sistem, frontend, backend, infra, stack |
| [contracts/](./docs/contracts/market.md) | Reference implementasi kontrak (nft-collection, market, factory) |
| [database/](./docs/database/database-schema.md) | Schema & data model (indexer) |
| [api/](./docs/api/api-overview.md) | Kontrak API (indexer service) |
| [features/](./docs/features/marketplace.md) | Spesifikasi per fitur |
| [security/](./docs/security/security.md) | Keamanan, roles, threat model |
| [development/](./docs/development/git-workflow.md) | Git workflow, versioning, CI/CD, standar kode, error handling, concurrency, secret |
| [testing/](./docs/testing/testing-strategy.md) | Strategi & kriteria terima |
| [deployment/](./docs/deployment/deployment.md) | Environment & rilis |
| [decisions/](./docs/decisions/ADR-001.md) | Architecture Decision Records (ADR-001..016) |
| [research/](./docs/research/near-nft-market-2026.md) | Riset pasar/ekosistem (bertanggal, berlabel) |
| [tasks/](./tasks/implementation-plan.md) | Roadmap eksekusi |
| [CHANGELOG](./CHANGELOG.md) | Catatan perubahan per-artefak |
