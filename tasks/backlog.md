# Backlog

> Semua pekerjaan yang diketahui, diurutkan by ID. Prioritas: **P0 = memblokir M1 atau gate milestone lain (mis. M4)**, P1 = MVP lanjutan/fase 2, P2 = fase lanjut.
> Area boleh gabungan (mis. `contract/frontend`) bila satu task menyentuh beberapa lapisan.
>
> **Tiket kerja M1:** slice M1 sudah dipecah jadi tiket siap-kerjakan di
> [`.scratch/m1-slice/`](../.scratch/m1-slice/spec.md) (10 tiket, `TASK-001..008` + `031`/`032`).
> Tabel di bawah tetap **SSOT prioritas & status**; tiket hanya state kerja turunan.

## Format task

```text
TASK-xxx — <judul>
Area: contract | indexer | frontend | backend | infra | design | docs | security
Priority: P0 | P1 | P2
Depends: TASK-xxx
Spec: docs/… (file spesifikasi)
Status: todo | in-progress | done | blocked
Estimate: <hari-dev> (PROPOSED — dikalibrasi saat eksekusi)
Done-when: <kriteria tunggal yang bisa diverifikasi>
Milestone: M0 | M1 | M2 | M3 | M4
```

- **Status**: `todo` = belum mulai · `in-progress` = dikerjakan · `done` = AC + Done-when terpenuhi · `blocked` = butuh prasyarat (sebut di kolom Done-when).
- **Estimate** dalam **hari-dev** (satu orang fokus); angka **PROPOSED** — kalibrasi ulang setelah task pertama selesai. Nilai = pekerjaan bersih, belum termasuk review.
- **Done-when** = satu kalimat yang bisa diuji; bukan daftar AC (AC tetap di [docs/testing](../docs/testing/acceptance-criteria.md)).
- **Milestone** = milestone penutup (SSOT: [milestones.md](./milestones.md)); task lintas-milestone memakai milestone gate terakhir.

## Backlog (final — ronde 1–13 + audit)

| ID | Judul | Area | Priority | Depends | Spec | Status | Estimate | Done-when | Milestone |
|---|---|---|---|---|---|---|---|---|---|
| TASK-001 | Scaffold workspace repo (contract/market/factory/frontend) + CI gates + SEC-* P0 | infra | P0 | — | implementation-plan | done | 3 | CI + Security hijau di `dev`; semua manifest & workflow ada | M0 |
| TASK-002 | Kontrak NFT: NEP-171/177/178/181/297 + mint + events (near-sdk-contract-tools) + **`set_phases` minimal (satu fase publik)** — wajib agar `nft_mint` bisa dipanggil (mint = launchpad-aware, INV-017) | contract | P0 | 001 | features/marketplace + contracts/nft-collection.md | done | 4 | `cargo test` hijau; mint + transfer + events lolos TC-001 | M1 |
| TASK-003 | Kontrak NFT: royalty NEP-199 (cap 10%) | contract | P0 | 002 | features/marketplace | todo | 1 | Payout royalti ≤10% teruji (INV-003/027) | M1 |
| TASK-004 | Kontrak market: storage NEP-145 + listing 2-tx + dual verification | contract | P0 | 002 | features/marketplace | todo | 4 | List/cancel + storage deposit lolos (INV-020, TC-002) | M1 |
| TASK-005 | Kontrak market: buy + nft_transfer_payout + resolve/refund (+ private listing & bundle — dgn TASK-010) | contract | P0 | 004 | features/marketplace.md + payments.md | todo | 5 | Buy sukses + refund + race 20 pembeli lolos (INV-001/016, TC-003/016) | M1 |
| TASK-006 | Sandbox tests 2-kontrak **subset slice**: mint->list->buy->refund + race (TC-001/002/013/016/017/020/022/044/047/048) + INV slice (001..016, 023, 030, 031) | contract | P0 | 002-005 | testing/test-cases.md (Slice M1) | todo | 4 | Suite sandbox hijau; semua INV **slice** punya test (INV M1+ di-defer eksplisit) | M1 |
| TASK-007 | Frontend: connect wallet (near-connect) | frontend | P0 | 001 | features/auth | done | 2 | Connect/disconnect + banner network jalan di testnet | M1 |
| TASK-008 | Frontend: browse/listings/buy UI (Next.js + Tailwind, EN + i18n) — **tanpa prasyarat branding** (default Tailwind) | frontend | P0 | 007 | features/marketplace | in-progress | 5 | Browse->list->buy end-to-end via UI (jalur emas Playwright) | M1 |
| TASK-008b | Sesi desain custom branding (warna, font, tokens) → design system | design | P0 (sebelum UI) | — | 04-ux-ui-spec.md | todo | 3 | Token warna/font + primitives `components/ui/` dipakai TASK-008/021 | M1+ |
| TASK-009 | Offers (escrow, accept, cancel, expire) — **MVP** | contract | P0 | 004 | features/marketplace | todo | 4 | Offer lifecycle + refund lolos (INV-005/006/024, TC-004/018) | M1+ |
| TASK-010 | Private listing + bundle — **MVP** | contract | P0 | 009 | features/marketplace | todo | 3 | Private guard + bundle pre-validasi lolos (INV-025/026/028, TC-009/011) | M1+ |
| TASK-011 | Multi-currency FT settlement — fase 2 | contract | P1 | 005 | features/payments | todo | 4 | Settlement FT (non-NEAR) lolos AC pembayaran | M2 |
| TASK-012 | Factory koleksi + UI create collection (supply creator-pilih — ronde 6) | contract | P0 (M1) | 002 | 02-product-requirements.md | todo | 3 | Deploy koleksi via factory + halaman create jalan | M1+ |
| TASK-013 | Lazy minting | contract | P2 (M3) | 012 | RESEARCH.md §11 | todo | 4 | Mint-on-buy + metadata benar; test sandbox hijau | M3 |
| TASK-014 | Custom indexer (ingestion **Neardata** → Postgres) + floor/volume dasar — M2; trait search/rarity penuh = M3 | indexer | P1 (M2) | 005 | indexer-security.md | todo | 8 | Lag=0; floor/volume tampil; dedup `(receipt_id,event_index)` lolos (SEC-INDEX-002) | M2 |
| TASK-015 | Notifications in-app minimal (polling NearBlocks + view call) — **MVP** | frontend | P1 | 008 | features/notifications | todo | 3 | Bell + unread ≤60 dtk; dedup event id lolos | M1+ |
| TASK-016 | Meta-transactions (gasless) | infra | P2 | 008 | RESEARCH.md §11 | todo | 3 | Relayer testnet sukses; tidak menahan kunci dana | M3 |
| TASK-017 | Auctions (bid escrow, end, claim) — fase 2 | contract | P1 | 005 | RESEARCH.md §10–11 | todo | 5 | Auction end-to-end lolos AC + sandbox | M2 |
| TASK-018 | Report system: UI report + endpoint ringan + antrean admin — **MVP** | backend | P1 | 005 | 02-product-requirements.md (ADMIN/MODERATION) + api/endpoints.md + features/report.md | todo | 3 | `POST /reports` + antrean admin jalan; idempotensi harian lolos | M1+ |
| TASK-019 | Badge verified: kurasi admin + tampilan badge — **MVP** | backend/frontend | P1 | 018 | security/permissions.md + features/report.md | todo | 2 | Set verified (step-up) + badge tampil; audit tercatat | M1+ |
| TASK-020 | **Launchpad kontrak**: config phase bebas (harga/alokasi/allowlist/waktu) + allowlist on-chain set + **Pausable + MAX_FEE_BPS** | contract | P0 (M1) | 002 | 02-product-requirements.md + features/launchpad.md + contracts/nft-collection.md | todo | 4 | Mint per phase + allowlist + pause lolos (INV-017/018/022/029) | M1+ |
| TASK-021 | **Launchpad UI**: create-collection wizard + upload allowlist + halaman mint per phase | frontend | P0 (M1) | 008b, 020 | 02-product-requirements.md | todo | 4 | Mint satu koleksi via wizard end-to-end di testnet | M1+ |
| TASK-022 | Stale listing: deteksi ownership mismatch + remove_stale_listing + sembunyi dari discovery | contract/frontend | P0 (M1) | 004 | features/marketplace | todo | 2 | Stale terdeteksi & disembunyikan (INV-016, TC-006/010) | M1+ |
| TASK-023 | Admin panel /admin: login wallet + antrean report + set verified/blocklist | frontend/backend | P1 | 018 | admin-security | todo | 3 | Guard allowlist + aksi destruktif step-up; 403 bila bukan admin | M1+ |
| TASK-024 | Onboarding guide page (buat wallet, transaksi pertama) | frontend | P2 | 008 | 03-user-flows | todo | 2 | Halaman `/onboarding` tampil + langkah akurat | M3 |
| TASK-025 | Leaderboard koleksi (agregasi NearBlocks terbatas) | frontend | P2 | 008 | 04-ux-ui-spec.md | todo | 2 | `/leaderboard` menampilkan agregat terbatas | M3 |
| TASK-026 | Implement semua requirement P0 `SEC-*` (security-requirements.md) — paralel sejak scaffold; gates CI | security | P0 | 001 | security-requirements | todo | 5 | Semua SEC-* P0 relevan berstatus terverifikasi; gate CI aktif | M1+ |
| TASK-027 | Audit eksternal market + launchpad sebelum mainnet (kandidat: OtterSec/Halborn/Block Security — G8) | security | P0 (gate M4) | 005, 020 | security-roadmap Phase E | todo | 2–4 minggu (eksternal) | Laporan audit + temuan kritis ditutup (gate M4) | M4 |
| TASK-028 | Infra: provisioning VPS + Docker Compose (FE + API + PostgreSQL) + staging subdomain | infra | P0 (M1) | 001 | infrastructure-security | todo | 3 | Stack jalan di VPS + staging subdomain hidup | M1+ |
| TASK-029 | Infra: CI/CD deploy (GitHub Actions → SSH) + domain + SSL Caddy | infra | P0 (M1) | 028 | cicd-security | todo | 3 | Deploy dev/testnet otomatis + mainnet manual ter-gate | M1+ |
| TASK-030 | Infra: backup pg_dump harian → object storage + restore drill (SEC-DB-004) + alert Telegram + monitoring | infra | P0 (M1) | 028 | disaster-recovery, monitoring | todo | 2 | Backup harian + restore drill lulus + alert 5xx sampai | M1+ |
| TASK-031 | Git & repo hygiene: 3 branch (`dev`/`testnet`/`mainnet`) + proteksi branch + `.gitignore` ketat + secret scanning (SEC-CICD-002/003) | infra | P0 | 001 | development/git-workflow, development/secrets-and-gitignore | done | 1 | Proteksi 3 branch + gitleaks hijau + CODEOWNERS aktif | M0 |
| TASK-032 | Versioning & rilis: SemVer per-artefak + tag + CHANGELOG + versi kontrak NEP-330 (SEC-CONTRACT-006) | infra/docs | P0 (M1) | 001 | development/versioning-and-release | todo | 1 | Tag pertama + CHANGELOG terisi + versi NEP-330 terverifikasi | M0 |
| TASK-033 | Error & notifikasi terpusat: registry kode error + pemetaan panic kontrak + kebijakan notifikasi (FE + API) | frontend/backend | P1 | 008 | development/error-handling | todo | 2 | Modul error terpusat dipakai; tidak ada pesan ad-hoc | M1+ |
| TASK-034 | Prep scaling: app stateless + pooling + rencana read replica/LB (aktif saat trafik naik) | infra | P1 (fase 2) | 028 | architecture/scaling | todo | 3 | App stateless terverifikasi + rencana replica/LB tertulis | M2 |
| TASK-035 | Audit dependensi: tinjau ulang advisory tanpa patch yang di-*ignore* (`--ignore-unfixable`) | infra | P2 | 001 | development/ci-cd.md §3 | todo | 0.5 | Advisory yang di-ignore punya keputusan tercatat: diperbaiki, diganti, atau diterima + alasan | M1+ |
| TASK-037 | Tinjau `function-call-key-plugin` (dependency transitif `near-connect-hooks`): menyimpan private key function-call di `localStorage` | security | P0 | 007 | security/key-management.md + features/auth.md | todo | 1 | Keputusan tercatat: jalur dinonaktifkan permanen (tanpa `addFunctionCallKey`), atau dependency diganti, atau diterima + alasan tertulis. **Kode plugin terbukti ADA di bundle klien (ronde 26) — bukan alasan tree-shaking** | M1+ |
| TASK-038 | Aktifkan lint yang tertunda setelah struktur `features/` ada: `import/order`, `no-restricted-imports` (larangan impor lintas-fitur), `react/jsx-no-useless-fragment` | frontend | P2 | 007 | development/code-standards.md §9 | todo | 0.5 | Ketiga aturan aktif di `eslint.config.mjs` dan gate FE tetap hijau | M1+ |

> **TASK-031 `done` (ronde 18c):** repo dijadikan **publik** oleh user → branch protection tersedia (sebelumnya 403 "butuh GitHub Pro" saat private). Proteksi **aktif** di `dev`/`testnet`/`mainnet`: PR wajib, force-push & delete diblokir **termasuk admin** (`enforce_admins`), 5 required status checks, conversation resolution; `strict` (branch up-to-date) di testnet/mainnet. Tag protection via ruleset `protect-release-tags` (`contract-v*`/`web-v*`/`indexer-v*`: delete + update diblokir). **Bukti**: push langsung ke `dev` ditolak GitHub — `GH006 Protected branch update failed … Changes must be made through a pull request. 5 of 5 required status checks are expected.`
> **Required approval = 0 (ditunda, keputusan user ronde 18c):** repo hanya punya satu akun dan GitHub melarang self-approve, jadi approval 2/1 akan mengunci semua PR. Naikkan ke `testnet`=1 / `mainnet`=2 saat ada maintainer kedua ([git-workflow.md](../docs/development/git-workflow.md) §16). `require_code_owner_reviews` juga ditunda karena alasan yang sama.

> **TASK-002 `done` (ronde 19):** kontrak koleksi di `contract/src/lib.rs` — NEP-171/177/178/181 via
> derive `NonFungibleToken` (`near-sdk-contract-tools` 4.0), NEP-145 storage, event NEP-297, plus ekstensi
> NearSea `nft_mint` (launchpad-aware) + `set_phases`/`allowlist_add`/`get_launchpad`/`allowlist_contains`/
> `royalty_config`. **Gate hijau**: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`
> (0 warning), `cargo test --workspace` (**34 test**, 31 di antaranya kontrak), dan
> `cargo near build non-reproducible-wasm` (265 KB) — ABI wasm diverifikasi memuat seluruh method NEP +
> ekstensi. TC-001 dibuktikan di level **unit** (mint → transfer → events `nft_mint`/`nft_transfer`/
> `launchpad_mint`); versi **sandbox** (`near-workspaces`) milik TASK-006 (slice ticket `08`), bukan tiket ini.
> **Belum termasuk** (sengaja): `nft_transfer_payout` NEP-199 → TASK-003; fase bebas penuh + allowlist
> penuh + Pausable → TASK-020. **Belum di-deploy** ke testnet (butuh persetujuan user, `git-workflow.md` §3).

> **TASK-035 (dibuat ronde 18, TASK-001):** `braces <=3.0.3` (high, ReDoS) masuk lewat
> `eslint-config-next → @next/eslint-plugin-next → fast-glob → micromatch`. Versi terbaru `braces` = 3.0.3
> dan **belum ada patch upstream** (`GHSA-vfj7-8cw-p6xm` / `CVE-2026-93687`), jadi gate `pnpm audit` tidak
> bisa hijau tanpa pengecualian. Pengecualiannya ditulis **eksplisit** di `frontend/pnpm-workspace.yaml`
> (`auditConfig.ignoreCves`) — terlihat saat review, bukan disenyapkan flag CI. Dampak: dev-only (linter),
> bukan runtime produksi. Task ini memastikan keputusannya ditinjau ulang saat upstream merilis patch.

> **TASK-031 riwayat `blocked` → `done` (ronde 18b → 18c):** saat repo masih **private**, branch protection
> butuh GitHub Pro (API 403 "Upgrade to GitHub Pro or make this repository public") meski token punya `admin`;
> hal yang sama menonaktifkan secret scanning GitHub-native + Dependabot security updates. **User menjadikan
> repo publik**, sehingga blokir itu hilang dan proteksi dipasang (lihat catatan `done` di atas).

> **TASK-007 `done` (ronde 25):** wallet connect di `frontend/features/auth/` — `@hot-labs/near-connect`
> 0.11.4 + `near-connect-hooks` 1.1.6 (`NearProvider`/`useNearWallet`), keduanya di-pin **exact**
> (`frontend-security.md` §9). State wallet app-wide lewat `WalletContext` + `useWallet()` dengan satu
> tipe `WalletApi`; lapisan near-connect **hanya di-mount di browser** (constructor-nya memakai
> `window`/IndexedDB + fetch manifest), sehingga halaman tetap ter-prerender statis dan app jalan tanpa
> wallet. Pemilih wallet = popup bawaan near-connect (tanpa daftar hardcoded). **Gate hijau**: `pnpm lint`,
> `format:check`, `typecheck`, `test` (**52 test**), `build` (4/4 halaman statis); bundle bersih dari
> secret **dan** dari kode penandatanganan lokal; `pnpm audit` hanya menyisakan advisory TASK-035 yang
> sudah dikecualikan. **Belum diklaim**: connect di testnet dengan wallet nyata + persistensi setelah
> reload (jalur emas Playwright = TASK-008), dan penonaktifan tombol aksi berbasis `transactionsDisabled`
> (baru ada tombol aksi di TASK-008).
> **Catatan implementasi**: (1) `tsconfig.target` ES2017 → **ES2020** (literal `BigInt` wajib untuk
> aritmetika yoctoNEAR); (2) `i18n` diperluas ke kedalaman kunci bebas (`auth.error.rejected`) — konvensi
> `code-standards.md` §10 memakai tiga segmen; (3) `vitest.setup.ts` ditambahkan untuk `cleanup()` RTL
> (Vitest jalan tanpa globals); (4) env dibaca sebagai `NEXT_PUBLIC_NEAR_NETWORK`/`_RPC_URL`/`_RPC_FALLBACKS`.
> **Review kode menemukan 7 cacat nyata** (disconnect tampil sebagai "Connecting…", Retry selalu connect,
> error disconnect tak terlihat, placeholder `{network}` tak tersubstitusi, efek probe berjalan tanpa
> henti karena `getBalance` dipakai sebagai dependensi, dua komponen dalam satu file, dependency tidak
> di-pin) — semuanya diperbaiki + test regresi; rincian di `.scratch/m1-slice/issues/09-fe-connect-wallet.md`.
> **Temuan keamanan (dicatat, bukan diperbaiki di sini):** plugin `function-call-key-plugin`
> (dependency `near-connect-hooks`) menyimpan **private key** function-call di `localStorage`; proyek ini
> tidak pernah memanggil `signIn({ addFunctionCallKey })`, jadi jalurnya tidak aktif. **Koreksi ronde 26**:
> klaim sebelumnya bahwa kode plugin "ter-tree-shake dari bundle" **tidak benar** — `next build` produksi
> tetap memuat `createLocalKeyFor`/`access_key::plugin`/`ed25519:` di chunk klien (diverifikasi ulang pada
> build commit `c7e081b` maupun sesudahnya). Jalurnya tetap tidak aktif karena tidak ada pemanggil, tetapi
> kode penandatanganan lokal ada di bundle user → **TASK-037** naik prioritas dan tidak boleh ditutup dengan
> alasan tree-shaking.

> **TASK-008 `in-progress` (ronde 26):** UI browse/list/buy terimplementasi di `frontend/features/marketplace/`
> (`ExplorePage`, `TokenPage`, `ListingGrid`/`ListingCard`, `ListModal`, `BuyModal`, `TokenActions`,
> `TokenMedia`) + `lib/format/fees.ts` (breakdown fee/royalti, BigInt), `lib/errors/market-errors.ts`
> (pemetaan panic → kode registry), `lib/near/contracts.ts` (kontrak dari env), dan namespace i18n
> `marketplace`/`errors`. Alur listing **dua transaksi** dengan `approval_id` konkret dari `nft_token` +
> deposit storage NEP-145; buy **re-verify sebelum signing** (`CONFLICT_PRICE_CHANGED`/`CONFLICT_STALE`
> tanpa tanda tangan) dan deposit = harga hasil verifikasi. **Gate hijau**: `lint`, `format:check`,
> `typecheck`, `test` (**127 test**, 20 di antaranya jalur emas browse→list→buy), `build` (`/` statis,
> `/token/...` partial prerender = "SSR shell + CSR data"). Prasyarat baru: `@tanstack/react-query`
> (diwajibkan `frontend-architecture.md` §Data fetching) dan `NEXT_PUBLIC_MARKET_CONTRACT_ID`.
> **Belum selesai — satu-satunya AC yang tersisa:** jalur emas **Playwright** dengan dua akun testnet.
> Blocker-nya bukan kode: kontrak harus ter-deploy di testnet, dan deploy testnet **wajib persetujuan user**
> (`git-workflow.md` §3). Test jalur emas saat ini berjalan di Vitest+RTL dengan chain palsu — jalur
> komponen/hook yang sama, tanpa browser dan tanpa wallet nyata.

> Penomoran ID final (ronde 1–13 + audit). Perubahan scope → update lewat prosedur DOCUMENTATION-MAP.
> Kolom Status/Estimate/Done-when/Milestone ditambahkan ronde 15; status awal semua `todo`.

## Pengelompokan epic (untuk perencanaan)

> Epic = kumpulan task dengan pemilik/tujuan sama; berguna untuk menjadwalkan kerja paralel & menilai kemajuan per area.

| Epic | Task | Milestone dominan |
|---|---|---|
| **E1 — Kontrak inti** | 002, 003, 004, 005, 006, 009, 010 | M1 |
| **E2 — Launchpad & factory** | 012, 020, 021, 013 | M1 → M3 |
| **E3 — Frontend MVP** | 007, 008, 008b, 015, 022, 024 | M1 → M3 |
| **E4 — API & moderasi** | 018, 019, 023, 033 | M1 |
| **E5 — Infra & rilis** | 001, 028, 029, 030, 031, 032, 034 | M0 → M2 |
| **E6 — Keamanan** | 026, 027 (+ SEC-* di setiap task) | M1, M4 |
| **E7 — Trading lanjutan & data** | 011, 014, 017, 025 | M2 → M3 |
| **E8 — Eksperimental/pasca-MVP** | 016 | M3 |

- Jalur kritis M1: **E1 → E3 → E2 (launchpad) → E5 (deploy)**; E4 & E6 berjalan paralel.
- E1 tidak boleh paralel dengan dirinya sendiri (kontrak bergantung berurutan); E3/E4/E5 boleh paralel setelah TASK-001.
