# Tech Stack

> Keputusan teknologi FINAL per kategori. Agent wajib mengikuti ini, tidak boleh memilih sendiri.

## Smart Contract

| Kategori | Pilihan | Status |
|---|---|---|
| Bahasa | Rust | ✅ Accepted (riset) |
| Framework | near-sdk + near-sdk-contract-tools | ✅ Accepted (riset) |
| Standar | NEP-171/177/178/181/199/297 | ✅ fix (riset + verifikasi ulang) |
| Safety & fee | **Pausable** (blokir mutasi, izinkan cancel/withdraw); fee default **`fee_bps = 200` (2%)** dengan **cap immutable `MAX_FEE_BPS = 500` (5%)** — cap ≠ nilai fee | ✅ Accepted (ronde 13 + audit) |
| Verifikasi build | Reproducible build **NEP-330** (contract_source_metadata) | ✅ Accepted (audit) |
| Auth API | **NEP-413 utama** + custom challenge fallback | ✅ Accepted (ADR-011) |
| Testing | near-workspaces (sandbox, deploy NFT+market bersama) + cargo test | ✅ default riset |
| Build tool | cargo-near | ✅ draft |

## Frontend — DIPUTUSKAN (ronde 2, 2026-10-01)

| Kategori | Pilihan | Status |
|---|---|---|
| Framework | **Next.js (App Router)** | ✅ Accepted |
| Language | **TypeScript** (strict, tanpa `any` tanpa alasan) | ✅ Accepted |
| Styling | **Tailwind CSS** | ✅ Accepted |
| UI language | English copy terpusat, siap i18n | ✅ Accepted |
| Tema visual | Custom branding — sesi desain terpisah (TASK-008b) → Tailwind theme | ⏳ open-by-design (sebelum Fase 2) |
| Server state | **TanStack Query** | ✅ Accepted (ronde 5) |
| Global client state | **Zustand** | ✅ Accepted (ronde 5) |
| Wallet | near-connect (+ hooks resminya) | ✅ Accepted (ronde 5) |
| RPC client | near-api-js / @near-js/* | ✅ default riset |

## Backend / Indexer — DIPUTUSKAN (ronde 2 + audit)

| Kategori | Pilihan | Status |
|---|---|---|
| Backend MVP | **Next.js API routes + PostgreSQL di VPS** (report/profil/auth/admin) | ✅ Accepted (ronde 5) |
| Sumber data read MVP | View call RPC langsung + NearBlocks API untuk riwayat | ✅ Accepted |
| Custom indexer (trait search, floor, volume) | Fase 2 — **ingestion via Neardata** (NEAR Lake DEPRECATED — dilarang) + PostgreSQL | ✅ Accepted (ditunda) |
| Migration | Prisma migrate | ✅ Accepted (ronde 13) |
| Cache | Menunggu fase 2 (indexer) | — |

## Infra — DIPUTUSKAN (ronde 5): self-hosted VPS

| Kategori | Pilihan | Status |
|---|---|---|
| Hosting FE | **VPS sendiri** — Next.js self-hosted, reverse proxy + SSL | ✅ Accepted (ronde 5) |
| Hosting BE/report API | **VPS yang sama** (bukan serverless — API route Next.js atau service kecil) | ✅ Accepted (ronde 5) |
| DB report/verified | **PostgreSQL di VPS** — engine sama dengan indexer fase 2 | ✅ Accepted (ronde 5) |
| IPFS pinning | ⏳ open-by-design — diputuskan saat mulai fitur mint (ronde 5) | ⏳ open-by-design |
| SSL/domain | Let's Encrypt via Caddy/nginx — detail di [deployment.md](../deployment/deployment.md) | ✅ Accepted |

---

## Version pins

> **Aturan**: pin dikunci saat TASK-001 (scaffold) dan dicatat di manifest/lockfile
> (`Cargo.lock`, `frontend/pnpm-lock.yaml`). Sumber kebenaran angka = manifest tersebut;
> tabel ini ringkasannya. Dua nilai **naik dari rujukan riset** (Rust & `near-sdk`) — lihat catatan di bawah.

| Komponen | Rujukan riset | **Pin final (TASK-001)** | Catatan |
|---|---|---|---|
| Node.js | LTS terbaru saat scaffold | **24 (LTS)** | `.nvmrc`; Node 20 sudah EOL (April 2026) dan `jsdom` 30 (test FE) mensyaratkan ≥22.22.2 |
| Next.js | App Router (versi terbaru saat scaffold) | **16.4.0** | App Router + Turbopack; `cacheComponents` aktif |
| React | — | **19.3.0** | Peer Next.js 16 |
| TypeScript | strict mode | **5.9.x** (`^5`) | Tanpa `any` tanpa alasan |
| Tailwind CSS | v3/v4 sesuai scaffold | **4.3.x** (`^4`) | Theme dari sesi branding (TASK-008b) |
| ESLint | — | **9.x** | `eslint-config-next` 16 + aturan proyek (code-standards §9) |
| pnpm | — | **10.29.3** | `packageManager` di `frontend/package.json` (SSOT versi pnpm); CI membacanya dari manifest |
| Vitest + jsdom | — | **4.x** + **30.x** | Unit/komponen FE; vitest 4 dipilih karena vitest 3 menarik `tinypool` yang punya advisory critical tanpa patch di jalur 3.x |
| Rust edition | 2021 (minimum) | **2021** | — |
| Rust toolchain | rustc 1.77.1 (rujukan riset) | **1.93.1** | `rust-toolchain.toml`; **naik** — dependency tree `near-sdk` 5.x butuh Cargo dengan dukungan `edition2024` (stabil sejak 1.85) |
| `near-sdk` | 4.x (rujukan riset) | **`5.18`** (ter-lock 5.29.1) | **naik** — sintaks `#[near(contract_state)]` di [docs/contracts/](../contracts/nft-collection.md) §1 adalah near-sdk 5.x |
| `near-sdk-contract-tools` | versi terbaru kompatibel | **4.0.0** | Derive NEP + Owner/Pause; mensyaratkan `near-sdk ^5.18`. **Nama crate = `near_sdk_contract_tools`** (bukan `near_contract_tools`) |
| `cargo-near` | 0.6.1 (rujukan riset) | **0.22.0** | Di-pin + verifikasi sha256 di CI (`.github/workflows/ci.yml`) |
| `near-cli-rs` | 0.17.0 (rujukan riset) | belum di-pin | Hanya tooling interaktif deploy; dipin saat deploy testnet pertama |
| `near-api-js` | versi terbaru saat scaffold | **7.3.1** (transitif) | RPC client — **tidak** dideklarasikan langsung oleh NearSea; ditarik `near-connect-hooks` (aturan "Simplicity First": tidak memasang dependensi sebelum dipakai) |
| `near-connect` | versi terbaru | **0.11.4** | Wallet adapter (ronde 5) — ditambah di TASK-007 |
| `near-connect-hooks` | — | **1.1.6** | `NearProvider` + `useNearWallet` (ronde 5 "hooks resminya") — ditambah di TASK-007 |
| Prisma | versi terbaru saat scaffold | belum ditambah | Migrate + client — masuk bersama API/report (TASK-018) |
| PostgreSQL | 15+ (rujukan; final saat provision) | belum di-pin | Lihat ekstensi di bawah |
| TanStack Query | v5 | belum ditambah | Server state — masuk bersama TASK-008 |
| Zustand | v4/v5 sesuai scaffold | belum ditambah | Client state — masuk bersama TASK-008 |
| gitleaks | terbaru | **v2** (action ter-pin SHA) | SEC-CICD-002; gate CI + `.gitleaks.toml` |

- **Kenapa dua versi naik:** `rust-toolchain.toml` dan `near-sdk` saling terikat — `near-sdk` 5.x (satu-satunya jalur untuk `near-sdk-contract-tools` 4.x) mensyaratkan toolchain ≥1.85. Alternatif "tetap 1.77.1 + near-sdk 4.1 + contract-tools 2.1" ditolak karena dokumen kontrak sudah dikunci ke sintaks 5.x (dan 4.x tidak lagi menerima perbaikan).
- **Belum ditambah** = sengaja; paket masuk saat task pemiliknya (aturan "Simplicity First": tidak memasang dependensi sebelum dipakai).


## Rasional & alternatif yang ditolak

> Setiap pilihan harus punya alasan; alternatif yang ditolak dicatat agar tidak ditinjau ulang tanpa alasan baru. Keputusan lengkap ada di ADR.

| Pilihan | Rasional | Alternatif ditolak | Rujukan |
|---|---|---|---|
| Rust + near-sdk | Standar produksi NFT di NEAR; keamanan tipe & WASM | AssemblyScript (ekosistem produksi kurang matang) | RESEARCH.md §3 |
| near-sdk-contract-tools | Derive NEP + Owner/Pause/Escrow — kurangi boilerplate | Implementasi manual standar (rawan bug) | RESEARCH.md §3 |
| Market contract terpisah dari NFT | Pola NEAR: satu market melayani banyak koleksi | Satu kontrak gabungan (tidak bisa open market) | ADR-003 |
| Listing 2-tx + dual verification | Tahan kontrak NFT jahat/buggy (open market) | 1-tx callback-only (percaya callback pihak ketiga) | ADR-002 |
| Approval non-custodial | NFT tetap di wallet seller; setara OpenSea | Escrow NFT (market pegang aset user) | ADR-007 |
| On-chain orderbook | Eliminasi serangan signature-order | Off-chain signed order ala Seaport | ADR-012 |
| Fee 2% on-chain | Tidak bisa dihindari, auditabel, konsisten semua jalur | Fee off-chain / 0% | ADR-005 |
| Next.js App Router | SSR + API routes dalam satu runtime | Astro (deprecated untuk UI) / SPA murni | ADR-009 |
| TanStack Query + Zustand | Pemisahan server state vs client state | Redux (overhead) / Context-only (cache manual) | frontend-architecture.md |
| VPS self-hosted | Kontrol penuh, konsistensi engine, biaya | Vercel + Supabase (managed) | ADR-009 |
| PostgreSQL (bukan NoSQL) | Data relasional (profil/report/audit) + jsonb untuk trait | Document DB (kurang cocok relasi & audit) | ADR-004 |
| Prisma migrate | TypeScript-native, cocok Next.js | Raw SQL migration (kurang terintegrasi) | ADR-009/migrations.md |
| Neardata (bukan NEAR Lake) | Lake DEPRECATED 2026-03 (FACT) | NEAR Lake (dilarang) | ADR-015 |
| NEP-413 utama | Standar Final, dukungan wallet luas | Custom challenge saja (fallback tetap ada) | ADR-011 |
| Sputnik DAO V2 (mainnet) | Multisig native NEAR + timelock | Single key (MVP saja) / wallet-2FA legacy (deprecated) | ADR-013 |

## Lisensi & kepatuhan

> Catatan lisensi (diverifikasi saat scaffold; bukan nasihat hukum).

| Komponen | Lisensi umum | Catatan |
|---|---|---|
| Rust / near-sdk | MIT / Apache-2.0 | Permisif |
| near-sdk-contract-tools | MIT | Permisif |
| Next.js / React | MIT | Permisif |
| Tailwind CSS | MIT | Permisif |
| TanStack Query | MIT | Permisif |
| Zustand | MIT | Permisif |
| Prisma | Apache-2.0 | Permisif |
| PostgreSQL | PostgreSQL License | Permisif |
| near-api-js / near-connect / near-connect-hooks | MIT | Permisif (terverifikasi saat TASK-007) |
| NEP standar | spesifikasi publik | Implementasi bebas |

- **Aturan**: sebelum menambah dependensi, verifikasi lisensi kompatibel (tidak ada copyleft kuat yang membatasi distribusi) dan catat di PR.

## Kebijakan dependensi (tambah/update)

| Aturan | Detail |
|---|---|
| Tambah dependensi | Wajib alasan + cek lisensi + cek ukuran bundle (FE) + tidak duplikasi fungsi yang ada |
| Update | Dependabot/renovate + `npm audit`/`cargo audit` di CI (cicd-security.md) |
| Lockfile | **Wajib di-commit** (`package-lock.json`/`yarn.lock`, `Cargo.lock`) — SEC-CICD-001 |
| Kerentanan critical/high | Blok merge sampai diperbaiki (ci-cd.md §3) |
| Pin versi | Runtime/toolchain di-pin; lib aplikasi boleh range dengan lockfile |
| Library baru vs tulis sendiri | Utamakan yang terpelihara & auditabel; hindari dependensi dengan transitive tree besar |

## Dev tooling

| Kategori | Alat | Catatan |
|---|---|---|
| Format Rust | `cargo fmt` | Wajib tanpa perubahan |
| Lint Rust | `cargo clippy` | Wajib tanpa warning |
| Test kontrak | `cargo test` + near-workspaces | Sandbox 2 kontrak |
| Fuzz/property | cargo-fuzz, proptest/quickcheck | INV-001..030 |
| Build kontrak | cargo-near | NEP-330 |
| Lint/format FE | ESLint + Prettier | TypeScript strict |
| Test FE | Vitest + RTL, Playwright | Unit + E2E |
| Bundle analysis | Next.js bundle analyzer | Performance budget |
| Audit | cargo audit, npm audit | CI gate |
| Secret scan | gitleaks | SEC-CICD-002 |
| CLI NEAR | near-cli-rs | Deploy/interaksi |

## Runtime target

| Artefak | Target | Catatan |
|---|---|---|
| Kontrak | WASM `wasm32-unknown-unknown` | Reproducible build (NEP-330) |
| Frontend/API | Node.js LTS (server) | Self-hosted di VPS (ADR-009) |
| Browser | evergreen modern (Chrome/Firefox/Safari/Edge terbaru) | Wallet extension modern |
| Database | PostgreSQL 15+ di VPS | Engine sama MVP & fase 2 |

## PostgreSQL — versi & ekstensi

> Extensions disiapkan untuk fase 2 (indexer); MVP tidak bergantung padanya.

| Extension | Fungsi | Kapan | Catatan |
|---|---|---|---|
| `pg_trgm` | pencarian nama koleksi/token (trigram) | fase 2 | Index GIN/trigram |
| GIN (built-in) | index `jsonb` untuk trait/filter | fase 2 | Tanpa extension tambahan |
| btree (built-in) | sort harga/waktu | MVP/fase 2 | Index biasa |

- **Versi**: PostgreSQL 15+ (final saat provision — [infrastructure.md](./infrastructure.md)); ekstensi di-`CREATE EXTENSION` lewat migration ([database/migrations.md](../database/migrations.md)).
- **Catatan**: index pencarian & trait baru aktif saat indexer fase 2 jalan ([database/database-schema.md](../database/database-schema.md)).
