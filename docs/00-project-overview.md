# 00 — Project Overview

> Ringkasan satu halaman. Detail panjang ada di 01-PRD dan 02-product-requirements.

## Apa ini

Marketplace NFT non-custodial di NEAR Protocol dengan fitur setara OpenSea. Codename: **NearSea**.

## Latar belakang & masalah

Ekosistem NEAR belum punya marketplace NFT kelas dunia setara OpenSea. NearSea mengisi celah itu:

- **Biaya & kecepatan** — trading NFT di Ethereum mahal dan lambat; NEAR menawarkan gas murah + finality cepat, memungkinkan orderbook 100% on-chain.
- **Kepercayaan royalti** — di banyak platform royalti kini opsional; NearSea memaksa royalti on-chain di settlement.
- **Aksesibilitas** — newcomer crypto butuh onboarding ramah (halaman guide + UX simpel); komunitas NEAR butuh venue native.
- **Kreator** — launchpad ala OpenSea Studio: phase mint fleksibel + royalti terjamin.

Target audiens (ronde 7): trader NFT berpengalaman, kreator/artist, newcomer crypto, komunitas NEAR.

## Komponen utama

1. **Kontrak NFT + Factory/Launchpad** (NEP-171/177/178/181/199/297 + royalti + phase mint) — buku besar kepemilikan token.
2. **Kontrak Market** (listing, offer, private/bundle, settlement) — orderbook on-chain.
3. **Report API + PostgreSQL kecil di VPS** — report, badge verified, profil custom.
4. **Frontend dApp** — discovery, halaman koleksi, profil, transaksi via wallet; data via RPC + NearBlocks.
5. **Indexer custom (fase 2)** — activity feed penuh, pencarian trait, statistik koleksi.

## Glosarium & Terminologi

> Daftar istilah yang dipakai lintas dokumen. Definisi di sini **ringkas**; kepemilikan fakta tetap pada dokumen sumber yang dirujuk.

| Istilah | Arti | Rujukan |
|---|---|---|
| **NEP-171** | Standar Core NFT NEAR: `nft_transfer`, `nft_token`, `nft_transfer_call`. | RESEARCH.md §3 |
| **NEP-177** | Standar metadata NFT (`nft_metadata`, `token_metadata`). | RESEARCH.md §3 |
| **NEP-178** | Approval management: `nft_approve`, `nft_revoke`, `nft_revoke_all`, `nft_is_approved` — dasar model listing non-custodial. | ADR-002, ADR-007 |
| **NEP-181** | Enumeration: `nft_tokens_for_owner`, `nft_supply_for_owner` — sumber tab Owned. | features/users.md |
| **NEP-199** | Royalti payout: `nft_transfer_payout` mengembalikan objek `Payout` yang divalidasi market (UNTRUSTED). | features/payments.md |
| **NEP-297** | Event standard (`EVENT_JSON:`) — semua event market `x-nearsea-market`. | api/webhooks.md |
| **NEP-330** | Contract metadata + reproducible build (mainnet gate). | decisions/ADR-013 |
| **NEP-413** | Sign-message standard (auth API utama); custom challenge = fallback. | security/wallet-authentication.md §3 |
| **NEP-145** | Storage management: user membayar storage sendiri (`storage_deposit`/`storage_withdraw`). | security/smart-contract-invariants.md INV-020 |
| **NEP-141** | Standar fungible token (FT) — pembayaran multi-currency **fase 2**. | features/payments.md |
| **yoctoNEAR** | Satuan terkecil NEAR: `1 Ⓝ = 1e24 yocto`. Semua nilai on-chain/JSON = **string** yoctoNEAR (u128). | features/payments.md § Presisi |
| **Tgas** | Teragas = `1e12` gas. Batas keras **300 Tgas/call**. | architecture/system-architecture.md § Anggaran gas |
| **near-connect** | Library wallet adapter (HOT/Meteor/Nightly/…); sumber daftar wallet resmi. | architecture/frontend-architecture.md |
| **Storage staking** | Ⓝ yang di-lock kontrak untuk menampung state (NEP-145); dibayar pemilik state, bukan kontrak. | INV-020 |
| **fee_bps** | Fee platform dalam basis poin; `fee_bps=200` (2%), cap immutable `MAX_FEE_BPS=500`. | ADR-005, tech-stack.md |
| **Escrow offer** | Ⓝ tawaran yang di-hold market selama offer ACTIVE; selalu user-recoverable. | order-protocol-security.md §1 |
| **Listing non-custodial** | NFT tetap di wallet seller selama listing (approval model); market hanya punya approval. | ADR-007 |
| **Stale listing** | Listing yang ownership-nya berubah di luar market → auto-invalid + disembunyikan. | INV-016 |
| **Launchpad phase** | Fase mint berurutan (harga/alokasi/allowlist/waktu); maksimum satu fase aktif. | ADR-008, INV-029 |
| **Allowlist on-chain** | Set akun yang boleh mint pada fase whitelist; disimpan on-chain, pre-deposit creator. | ADR-008 |
| **ACTIVE/SOLD/STALE/PARTIAL** | Status lifecycle order (listing/offer/bundle). | order-protocol-security.md §2 |

## Diagram Konteks Sistem

> Ringkas C4 Level 1. Detail kontainer & sequence: [architecture/system-architecture.md](./architecture/system-architecture.md).

```text
   ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────────┐
   │  Trader   │  │  Kreator  │  │   Admin   │  │  Auditor/DAO  │
   │ (buy/sell)│  │ (mint/list)│ │ (moderasi)│  │ (owner mainnet)│
   └─────┬─────┘  └─────┬─────┘  └─────┬─────┘  └───────┬───────┘
         │              │              │                │
         ▼              ▼              ▼                ▼
   ┌─────────────────────────────────────────────────────────────┐
   │                        NearSea (sistem)                     │
   │  Frontend dApp · Kontrak NFT/Market/Factory · Report API/DB │
   └──────┬──────────────────────┬───────────────────┬───────────┘
          │                      │                   │
          ▼                      ▼                   ▼
   ┌─────────────┐       ┌───────────────┐    ┌───────────────────┐
   │ NEAR Chain  │       │ NearBlocks API│    │ IPFS Gateway      │
   │ (RPC node)  │       │ (riwayat tx)  │    │ (media/metadata)  │
   │ = otoritas  │       │ = proyeksi    │    │ = untrusted       │
   └─────────────┘       └───────────────┘    └───────────────────┘
```

| Pihak | Peran terhadap NearSea | Batas kepercayaan |
|---|---|---|
| Trader / Kreator | Baca discovery, sign tx via wallet | Memegang kunci sendiri; FE tidak pernah sign atas namanya |
| Admin | Moderasi report, verified badge, blocklist | Allowlist DB + scope `admin` + step-up |
| Auditor / DAO council | Owner actions mainnet (upgrade/fee/pause) | Sputnik DAO 2-of-3 + timelock (ADR-013) |
| NEAR Chain + RPC | Otoritas tunggal dana/ownership | **Dipercaya sebagai otoritas** (ADR-010); RPC hanya pintu masuk |
| NearBlocks API | Proyeksi riwayat/aktivitas | **Tidak dipercaya** — tidak pernah memicu tx |
| IPFS gateway | Hosting media/metadata | Konten untrusted (ADR-014) |

## Status saat ini

- ✅ Riset teknis selesai → [RESEARCH.md](../RESEARCH.md)
- ✅ Kerangka dokumentasi lengkap (struktur app kompleks: docs/ + tasks/ + AGENTS.md)
- ✅ **Ronde 1 tanya-jawab**: MVP = fixed-price + offers + private listing + bundle; NEAR saja; koleksi open + badge verified; fee platform 2%
- ✅ **Ronde 2 tanya-jawab**: Next.js + TS + Tailwind; data via RPC + NearBlocks (tanpa indexer sendiri di MVP); UI English siap i18n; tema custom branding (sesi desain terpisah)
- ✅ **Ronde 3 tanya-jawab**: pola listing 2-tx dual-verification; codename "NearSea"; moderasi via report system (butuh penyimpanan ringan — pengecualian "tanpa backend"); target rilis testnet dulu
- ✅ **Ronde 4–6 tanya-jawab**: royalti maks 10%; offer 7 hari, 1/buyer/token, harga min 0.01 Ⓝ; bundle satu harga (gaya OpenSea); counter-offer tidak di MVP; notifikasi in-app masuk MVP (polling); supply koleksi creator-pilih; infra self-host VPS (FE + report API + PostgreSQL); TanStack Query + Zustand; IPFS ditunda
- ✅ **Ronde 7–12 tanya-jawab**: positioning terbuka umum; top-nav + halaman inti/leaderboard/settings/onboarding; responsive setara penuh; **model listing approval ala OpenSea (hasil riset) + auto-stale**; listing permanen; launchpad berphase fleksibel ala OpenSea Studio (allowlist on-chain); REST + cursor + auth signature wallet; admin panel allowlist; ops fleksibel done-when, demo per milestone, git branch per fitur, alert Telegram
- ✅ **Review subagent 71 file (2026-10-01) → AUDIT-TEMUAN.md → semua perbaikan dieksekusi** (6 P0 + gap cakupan + koreksi faktual + sistemik)
- ✅ **Crosscheck ronde-3 per file (2026-10-02)**: indeks dokumen bertaut lengkap (0 orphan), 0 tautan rusak, 0 heading duplikat, 123/123 ID terdefinisi, nilai bisnis (fee 2%, royalti 10%) konsisten, 0 istilah usang
- ✅ **Ronde 14 (2026-10-02)**: proses kerja dimatangkan — **3 branch** (`dev`/`testnet`/`mainnet`) + aturan wajib tanya user sebelum merge ke testnet/mainnet; **SemVer per-artefak** + CHANGELOG; **CI/CD** (workflow skeleton + secret scanning); **`.gitignore` ketat** + kebijakan file/secret; **kebijakan komentar** (hanya baris penting); **error/notifikasi terpusat**; **analisis race** (20 pembeli 1 NFT → 1 menang, sisanya refund) + test TC-016..018; **rencana scaling** horizontal/vertical. Lihat `docs/development/` + `docs/architecture/scaling.md`.
- ✅ **Ronde 15 (2026-10-02)**: audit kedalaman 3 subagent atas 81 file → **8 ADR baru (ADR-002..009)** dibuat (ADR-001 jadi indeks murni) + **~15 kontradiksi diperbaiki** (template login, domain, cakupan API MVP, idempotency, fee cap vs nilai, min-price, media MVP, bundle vs receiver, duplikasi PRD, §3a rotasi council, Astro deprecated, G1, format 9-field CS, readiness RESEARCH) + register SEC dinormalisasi (enum-only + kolom Fase). **Lalu seluruh dokumentasi diperluas ke level engineering spec** (Fase 2–5): DDL database nyata + enum + ERD, skema per-endpoint + contoh JSON, algoritma payout dengan contoh angka, attack tree + peta INV→test→SEC, runbook deploy/restore/insiden, wireframe 9 halaman + aksesibilitas WCAG, kolom task (Status/Estimate/Done-when/Milestone), ADR-010..015 diperluas setara.
- ✅ **Ronde 16 (2026-10-02)**: full crosscheck kesiapan bangun → **~20 inkonsistensi diperbaiki** (DDL sessions/admin_audit ganda, registry error, lockout auth, kosakata aksi admin, CORS, retensi/RTO, payload event, gas listing, MVP search, RESEARCH.md usang, TV duplikat, milestone drift) + **11 keputusan penutup A1–A11** (ADR-001 ronde 16 — bisa di-override user) + **file baru**: `docs/contracts/{nft-collection,market,factory}.md` (reference implementasi kontrak — signature/init/layout/konstanta/view shapes), `docs/features/launchpad.md`, `docs/features/report.md`, `.gitleaks.toml`, `rust-toolchain.toml`, `.nvmrc` + **TC-043..052** + prosedur tambah SEC-requirement & pembaruan threat-model. Sisi factory/launchpad/NFT-contract kini bisa diimplement dari docs.
- ✅ **Ronde 17 (2026-10-07)**: sesi **mempertajam ide** (`/grill-with-docs`) + **riset pasar** + **pemotongan scope**. Temuan pasar (terverifikasi): **pasar NFT NEAR sudah praktis mati** — Paras pivot ke produk AI, MITTE (klaim ~91% pangsa) tutup April 2025, hanya HotCraft tersisa (~$742/minggu); perang royalti 2022–2023 sudah dimenangkan pihak yang membuat royalti opsional. **Keputusan user: proyek ini pembelajaran/portofolio** ("mau ada trader atau engga juga saya aman") → **ADR-016**: niat proyek ditetapkan, **M1 dipotong jadi vertical slice (8 task)**, 16 task pindah ke **M1+**, M4/audit/DAO jadi **opsional** (hanya bila beralih ke jalur produk). Juga: premis PRD §2 dikoreksi dengan bukti; 7 temuan kritis review desain dicatat; katalog pesan error dilengkapi 10→~60 kunci; **CONTEXT.md** (glosarium) dibuat.
- ✅ **Ronde 17e (2026-10-07)**: slice M1 dipecah jadi **10 tiket siap-kerjakan** di `.scratch/m1-slice/` (frontier = tiket 01 = TASK-001).
- ✅ **Ronde 18 — TASK-001 scaffold (2026-10-07)**: **kode pertama masuk, dan TASK-001 `done`.** Cargo workspace (`contract/`/`market/`/`factory/`) + frontend Next.js 16 (TS strict, Tailwind 4, ESLint, Prettier, Vitest, i18n) + `Cargo.lock`/`pnpm-lock.yaml` ter-commit. **Gate hijau lokal**: `cargo fmt`/`clippy -D warnings`/`test --workspace` (4 test), build wasm ketiga kontrak, `pnpm lint`/`format:check`/`typecheck`/`test` (3 test)/`build`. CI + Security berisi perintah nyata dengan **semua action di-pin SHA** + `cargo-near` 0.22.0 di-pin sha256; `deploy-*.yml` jadi manual-only sampai TASK-028/029; `CODEOWNERS` + `dependabot.yml` dibuat. **Dua versi naik dari rujukan riset** (dengan alasan tercatat): Rust 1.77.1 → **1.93.1** dan `near-sdk` 4.x → **5.29.1** (dependency tree 5.x butuh Cargo dengan `edition2024`; dokumen kontrak memang sudah bersintaks 5.x); Node 20 → **24 LTS** (20 EOL April 2026). Rincian pin: [tech-stack.md](./architecture/tech-stack.md) §Version pins.
- ✅ **Repo GitHub + bukti gate (2026-10-07)**: remote `github.com/keydeveloping/nearsea` (private) dibuat user; 3 branch permanen (`mainnet`/`testnet`/`dev`) di-push; default branch di-set `mainnet` sesuai [git-workflow.md](./development/git-workflow.md) §1. PR #1 squash-merge ke `dev` (commit `223015b`). **CI + Security hijau di `dev`** — bukti: run [CI 37627825466](https://github.com/keydeveloping/nearsea/actions/runs/37627825466) + [Security 37627825490](https://github.com/keydeveloping/nearsea/actions/runs/37627825490), keduanya `success`. Itu memenuhi Done-when TASK-001 (M0 selesai). Tiga bug nyata ditemukan run CI pertama dan sudah diperbaiki (tipe Next.js belum di-generate; `tinypool` critical lewat vitest 3 → vitest 4; gitleaks butuh scope `pull-requests: read`).
- ⚠️ **TASK-031 selesai setelah repo dijadikan publik (ronde 18c):** saat repo masih **private**, branch protection butuh **GitHub Pro** (API 403 "Upgrade to GitHub Pro or make this repository public") meski token punya `admin`. User mengubah repo menjadi **publik**, sehingga blokir hilang. **Proteksi kini aktif** di `dev`/`testnet`/`mainnet`: PR wajib, force-push & delete diblokir **termasuk admin**, 5 required status checks, conversation resolution; `strict` di testnet/mainnet. Tag protection via ruleset (`contract-v*`/`web-v*`/`indexer-v*`). Bukti: push langsung ke `dev` ditolak GitHub. **Catatan terbuka:** required approval ditunda (0) karena repo hanya punya satu akun dan GitHub melarang self-approve — dinaikkan ke testnet=1/mainnet=2 saat maintainer kedua ada. Fitur GitHub-native lain (secret scanning, Dependabot security updates) kini juga tersedia untuk diaktifkan.
- ✅ **TASK-002 kontrak NFT core (2026-10-07, ronde 19)**: **kode kontrak pertama masuk.** `contract/src/lib.rs` kini mengimplementasikan NEP-171/177/178/181 (via derive `NonFungibleToken`), NEP-145 storage, event NEP-297, plus ekstensi NearSea `nft_mint` (launchpad-aware, deposit exact-match) + `set_phases`/`allowlist_add`/`get_launchpad`/`allowlist_contains`/`royalty_config`. **Gate hijau lokal**: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (0 warning), `cargo test --workspace` (**34 test**), `cargo near build non-reproducible-wasm` (265 KB) dengan ABI wasm diverifikasi memuat seluruh method NEP + ekstensi. TC-001 (mint → transfer → events) lolos di level **unit**; versi sandbox milik TASK-006. **Belum di-deploy** ke testnet (wajib tanya user dulu). Sisa: `nft_transfer_payout` (TASK-003), fase bebas penuh (TASK-020).
- ✅ **TASK-032 versioning & rilis (2026-10-07, ronde 20)**: skema versi per-artefak ditegakkan mesin. `[package.metadata.near.reproducible_build]` (image Docker ter-pin **by digest**) + `repository` ditambahkan ke ketiga crate; **workflow baru `.github/workflows/release.yml`** dipicu tag `contract-v*`/`web-v*`/`indexer-v*` — memeriksa **versi manifest == tag**, membangun wasm reproducible, **membuktikan metadata NEP-330 tertanam == tag**, lalu melampirkan artifact + `code-hash.txt` ke GitHub Release (bisa diuji-kering via `workflow_dispatch`); `ci.yml` kini membangun **ABI** + memverifikasi metadata NEP-330 tiap PR. **Bukti lokal**: metadata ketiga wasm memuat `version=0.1.0` + `link=https://github.com/keydeveloping/nearsea` (sebelumnya `link=null`); `fmt`/`clippy -D warnings`/`test --workspace` (34 test) hijau. **Belum:** tag pertama **belum dibuat** — tag sah hanya di `testnet`/`mainnet` setelah PR promosi, dan itu **wajib persetujuan user**. Rilis pertama disiapkan: `contract-v0.1.0`.
- ⏳ Sisa keputusan (non-blokir): alamat treasury, provider IPFS, domain, sesi custom branding, G8/G14 (bisnis — kini opsional karena bukan produk). A1–A11 (ronde 16) bisa di-override.
- ✅ **TASK-003 royalti NEP-199 (2026-10-08, ronde 21)**: paruh kedua tesis slice terpasang — koleksi kini memberi tahu market cara membagi uangnya. `nft_transfer_payout` memindahkan token (otorisasi approval NEP-178, `assert_one_yocto`, `max_len_payout` dihormati) **dan** mengembalikan payout royalti dalam panggilan yang sama; payout diturunkan dari konfigurasi royalti **level kontrak** (bukan metadata per-token) dan selalu ≤10% harga (INV-027) lewat helper murni `royalty_amount = floor(balance × bps / 10_000)` dengan `checked_mul`. **Gate hijau lokal**: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (0 warning), `cargo test --workspace` (**45 test**, 11 di antaranya baru — termasuk dust `19` → `0` / `20` → `1` yocto dan tepat 10% di cap), build wasm 272 KB dengan `nft_transfer_payout` terverifikasi ada di ABI. **Belum diklaim**: angka gas 15 Tgas dan TC-003 versi sandbox (termasuk validasi payout di sisi market + refund) — keduanya butuh suite dua-kontrak TASK-006.
- ✅ **TASK-004 kontrak market — listing 2-tx (2026-10-08, ronde 22)**: jalur listing market di `market/src/lib.rs` — `list_nft_for_sale` (2-tx, non-custodial: NFT tetap di wallet seller) + callback `process_listing` `#[private]` yang **memverifikasi sendiri** lewat dua view XCC (`nft_token` kepemilikan, `nft_is_approved` approval — ADR-002; mekanisme sama dipakai lagi saat settle, SEC-ORDER-004), `remove_sale`, `update_price`, view `get_sale`/`get_sales`/`get_supply_sales`, storage NEP-145 (bounds `min = storage_per_sale()`), Pausable (INV-022), dan event `market_list`/`market_delist`/`market_update_price`. **Gate hijau lokal**: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (0 warning), `cargo test --workspace` (**75 test**, 32 baru), build wasm 197 KB. Non-custodial dibuktikan dengan **membaca receipt** jalur listing (hanya 2 view + callback; tidak ada `nft_transfer*`). **Koreksi dokumen**: `nft_revoke_token` tidak ada di NEP-178 (hanya `nft_revoke`/`nft_revoke_all`, owner-only) → `remove_sale` **tidak** mencabut approval ([contracts/market.md](./contracts/market.md) §2a). **Belum diklaim**: paruh buy TC-002, race TC-016/017, TC-022, TC-048, angka gas penuh — butuh dua kontrak nyata (TASK-005/006).
- ✅ **TASK-007 wallet connect (2026-10-08, ronde 25)**: **jalur FE slice M1 mulai jalan.** `frontend/features/auth/` menyediakan state wallet app-wide (`WalletContext` + `useWallet()`, satu tipe `WalletApi`) di atas `@hot-labs/near-connect` 0.11.4 + `near-connect-hooks` 1.1.6 (di-pin exact), header dengan indikator jaringan permanen + kontrol wallet, banner peringatan jaringan + flag gerbang `transactionsDisabled`, dan mode baca tanpa wallet (halaman tetap ter-prerender statis). Modul pendukung: `lib/near/network.ts` (jaringan dari env — nilai tak dikenal gagal saat start), `lib/near/wallet-connector.ts` (tanpa daftar wallet hardcoded), `lib/near/wallet-errors.ts`, `lib/format/money.ts` (`YoctoNear` + BigInt, tanpa float). **Gate FE hijau**: `lint`, `format:check`, `typecheck`, `test` (**52 test**), `build` (4/4 halaman statis); bundle bersih dari secret dan dari kode penandatanganan lokal. **Belum diklaim**: connect dengan wallet testnet nyata + persistensi setelah reload (jalur emas Playwright = TASK-008), dan penonaktifan tombol aksi berbasis `transactionsDisabled` (tombol aksi baru ada di TASK-008). Review kode menemukan dan menutup 7 cacat nyata (disconnect tampil sebagai "Connecting…", Retry selalu connect, error disconnect tak terlihat, placeholder `{network}` tak tersubstitusi, efek probe berjalan tanpa henti, dua komponen dalam satu file, dependency tidak di-pin). Temuan terbuka: dependency transitif `function-call-key-plugin` menyimpan private key function-call di `localStorage` — jalurnya tidak aktif di NearSea, ditinjau sebagai **TASK-037**.
- ✅ **TASK-008 UI browse/list/buy (2026-10-09, ronde 26)**: **demo M1 lengkap di level UI.** `frontend/features/marketplace/` menyediakan grid Explore (baca `get_sales` langsung lewat view call RPC), halaman `/token/[contract]/[tokenId]` (metadata, harga, status listing, aksi sesuai peran), modal listing **dua langkah tanda tangan** (`nft_approve` "Step 1 of 2" → `list_nft_for_sale` "Step 2 of 2" dengan `approval_id` konkret dari `nft_token` + deposit storage NEP-145 yang ditampilkan sebelum signing), dan modal beli dengan **breakdown fee + royalti** sebelum konfirmasi serta **re-verify sebelum signing** (SEC-ORDER-003). Modul pendukung: `lib/format/fees.ts` (breakdown BigInt), `lib/errors/market-errors.ts` (panic → kode registry), `lib/near/contracts.ts`, `components/ui/Modal.tsx`, namespace i18n `marketplace`/`errors`. Wallet disuntikkan lewat interface sempit `MarketChain` (`app/MarketChainProvider.tsx`) supaya fitur tidak saling impor. **Gate FE hijau**: `lint`, `format:check`, `typecheck`, `test` (**127 test**, 20 di antaranya jalur emas browse→list→buy), `build` (`/` statis + `/token/...` partial prerender). **Satu AC tersisa**: jalur emas Playwright dengan dua akun testnet — terblokir deploy testnet yang **wajib persetujuan user**. **Koreksi**: klaim tiket 09 bahwa kode `function-call-key-plugin` ter-tree-shake dari bundle tidak benar (kode penandatanganan lokal tetap ada di chunk klien) → TASK-037 naik prioritas.
- ✅ **TASK-038 lint tertunda diaktifkan (2026-10-09, ronde 27)**: tiga aturan yang tertunda sejak TASK-001 kini aktif di `frontend/eslint.config.mjs` — `import/order` (grup §5: `type` di akhir, satu baris kosong antar grup, alfabetis), `react/jsx-no-useless-fragment`, dan `no-restricted-imports` lewat override per-`files` yang menegakkan tabel batas dependensi [frontend-architecture.md](./architecture/frontend-architecture.md) §1. **Tanpa dependency baru**: dugaan lama "butuh plugin tambahan" tidak benar — `eslint-config-next` 16.4.0 sudah membawa `eslint-plugin-import` + `eslint-plugin-react`. Aktivasi memunculkan **64 pelanggaran `import/order`** (semua auto-fixable, murni urutan import) dan **nol** pelanggaran dua aturan lain. **Gate FE hijau**: `lint`, `format:check`, `typecheck`, `test` (**127 test**), `build`.
- ✅ **Ronde 28 — slice M1 mendarat di `dev` (2026-10-09)**: seluruh branch bertumpuk yang sebelumnya hanya hidup lokal kini di-push dan di-merge ke `dev` lewat PR berurutan — #11 (tiket 03) → #12 (05) → #13 (06) → #15 (09) → #16 (10 + TASK-038) — dengan CI hijau di tiap langkah. **Tiket 07/08 (#14) belum mendarat**: check wajib `Dependency audit` gagal karena advisory **RUSTSEC-2026-0285** (`rustls 0.23.43`) terbit di tengah proses; pelacaknya **TASK-039**. **Bukti CI yang selama ini hilang**: suite sandbox tiket 08 benar-benar berjalan di CI Linux ([run 37895683629](https://github.com/keydeveloping/nearsea/actions/runs/37895683629), `18 passed; 0 failed`, 221s). **Koreksi proses**: file tiket `.scratch/` di-track git sehingga statusnya bisa berbeda antar branch (tiket 05–08 `done` di branch kontrak, `ready-for-agent` di `dev`) — aturan anti-divergensi ditambahkan di [docs/agents/issue-tracker.md](./agents/issue-tracker.md).
- ⏳ **Berikutnya:** yang tersisa di slice M1 bukan kode, melainkan **keputusan user**: (a) promosi `dev → testnet → mainnet` + tag `contract-v0.1.0` (TASK-032) — butuh persetujuan eksplisit ([git-workflow.md](./development/git-workflow.md) §3); (b) deploy testnet untuk jalur emas Playwright (AC terakhir tiket 10); (c) **TASK-039** — advisory RUSTSEC-2026-0285 memblokir PR #14 (tiket 07/08); pilihannya memecahkan konflik bump `rustls`/`aws-lc-rs` atau menerimanya dengan alasan tertulis. Satu advisory `braces` tanpa patch upstream dilacak di TASK-035; `function-call-key-plugin` di TASK-037.

## Prinsip yang sudah diputuskan (dari riset)

- Orderbook 100% on-chain (gas NEAR murah) — kebalikan arsitektur off-chain OpenSea.
- Royalti dipaksa on-chain saat settlement — **maks 10%**, payout maks 10 akun penerima.
- NFT contract dan market contract terpisah, dihubungkan NEP-178 (approval).

## Milestone

> SSOT milestone: [tasks/milestones.md](../tasks/milestones.md). Tabel ini ringkasan navigasi.

| ID | Nama | Isi utama | Done when (ringkas) |
|---|---|---|---|
| **M0** | Fondasi dokumen & scaffold | Dokumen inti terisi, stack final, repo scaffold | Agent bisa mulai koding tanpa bertanya arah |
| **M1** | MVP (testnet) | Fixed-price + offers + private/bundle + launchpad; fee 2%; report + badge verified; FE end-to-end | Dua akun testnet list→buy & offer→accept via UI; mint launchpad; report masuk antrean admin |
| **M2** | Trading lanjutan (fase 2) | Auction, multi-currency FT, indexer fase awal (Neardata + floor/volume) | Auction end-to-end lolos AC + stats dasar tampil |
| **M3** | Ekosistem | Lazy minting + indexer penuh (trait filter, rarity, search) + notifikasi real-time | Trait search dari indexer + notifikasi real-time |
| **M4** | Rilis mainnet | Audit eksternal lulus, transfer ownership ke Sputnik DAO, reproducible build NEP-330, drill pause & restore | Semua gate lulus → deploy mainnet (keputusan eksplisit user) |

## Traceability Keputusan

> Setiap keputusan besar (ronde tanya-jawab) → ADR pemilik + dokumen kanonik. Ini peta cepat; log lengkap di [decisions/ADR-001.md](./decisions/ADR-001.md).

| Keputusan (ronde) | ADR | Dokumen kanonik | Ringkasan |
|---|---|---|---|
| Scope MVP, fee 2%, NEAR-only (ronde 1) | ADR-005, ADR-006 | [01-PRD.md](./01-PRD.md) §8, §13 | Fixed-price + offers + private + bundle; koleksi open + badge; fee 2% |
| Frontend & data layer MVP (ronde 2) | ADR-004, ADR-009 | [architecture/tech-stack.md](./architecture/tech-stack.md) | Next.js/TS/Tailwind; RPC + NearBlocks; VPS self-host |
| Listing 2-tx + moderasi report (ronde 3) | ADR-002, ADR-006 | [decisions/ADR-002-listing-two-tx.md](./decisions/ADR-002-listing-two-tx.md) | `nft_approve` → `list_nft_for_sale`; dual verification; report system |
| Business rules offer/royalti/bundle (ronde 4–6) | ADR-003 | [01-PRD.md](./01-PRD.md) §13, [features/payments.md](./features/payments.md) | Offer 7 hari, 1/buyer/token, min 0.01 Ⓝ; royalti ≤10%; bundle satu harga |
| Approval model + auto-stale (ronde 9) | ADR-007 | [decisions/ADR-007-approval-listing-model.md](./decisions/ADR-007-approval-listing-model.md) | NFT tetap di wallet seller; listing permanen; stale disembunyikan |
| Launchpad berphase + allowlist on-chain (ronde 10) | ADR-008 | [decisions/ADR-008-launchpad-phases.md](./decisions/ADR-008-launchpad-phases.md) | Phase berurutan; allowlist on-chain; storage dibayar pemicu mint |
| API auth NEP-413 (security research) | ADR-011 | [security/wallet-authentication.md](./security/wallet-authentication.md) §3 | NEP-413 utama + custom fallback |
| Orderbook on-chain (security research) | ADR-012 | [security/order-protocol-security.md](./security/order-protocol-security.md) | Order = state on-chain; tanpa off-chain signature |
| Key management Sputnik DAO (security research) | ADR-013 | [security/key-management.md](./security/key-management.md) §3 | Single-key testnet → DAO 2-of-3 mainnet |
| Metadata isolation (security research) | ADR-014 | [security/metadata-security.md](./security/metadata-security.md) | Browser-only MVP; fetcher terisolasi fase 2 |
| Indexer Neardata (security research) | ADR-015 | [security/indexer-security.md](./security/indexer-security.md) | Proyeksi read-only; chain otoritatif |

## Pertanyaan Terbuka

> Semua non-blokir untuk M1 kecuali dinyatakan lain. Nilai `⏳ open-by-design`; tidak ada yang boleh "ditebak" di kode tanpa keputusan.

| ID | Pertanyaan | Owner | Dampak | Blokir? | Kapan diputuskan |
|---|---|---|---|---|---|
| ~~OQ-001~~ | ~~Alamat treasury account~~ | — | ✅ **DITUTUP (ronde 17):** testnet = akun owner market sendiri; mainnet = ditetapkan saat deploy (hanya jalur produk) | — | Selesai |
| ~~OQ-002~~ | ~~Provider IPFS pinning~~ | — | ✅ **DITUTUP (ronde 17):** **Pinata** (free tier cukup untuk demo); gateway publik (`ipfs.io`, `ipfs.dweb.link`) untuk baca. Bukan keputusan vendor besar — bisa diganti tanpa mengubah kontrak (on-chain hanya URL + hash) | — | Selesai |
| ~~OQ-003~~ | ~~Domain final + `callbackUrl` NEP-413~~ | — | ✅ **DITUTUP (ronde 17):** testnet pakai `localhost` + placeholder `nearsea.example`; domain nyata hanya perlu saat deploy publik (jalur produk) | — | Selesai (testnet) |
| OQ-004 | Sesi custom branding (tema visual + design token) | Tim desain | Tailwind theme; komponen UI final | Tidak (draft token ada) | M1+ |
| OQ-005 | Metric bisnis (volume Ⓝ/minggu, koleksi aktif, wallet unik, retensi) — angka target | — | ✅ **TIDAK BERLAKU** (ADR-016: proyek bukan produk) | — | Dibatalkan |
| OQ-006 | G8/G14 (keputusan bisnis) | — | ✅ **TIDAK BERLAKU** (ADR-016: bukan produk) | — | Dibatalkan |
| OQ-007 | Jumlah storage deposit per-entry (Sale/Offer/Bundle) | Tim kontrak | UX deposit; angka pasti | Tidak (dihitung dari `storage_usage`) | Saat implementasi kontrak |
| OQ-008 | Nama event agregat bundle (create/buy/cancel) | — | ✅ **DITUTUP (ronde 16/A3):** `market_bundle_create`/`market_bundle_cancel`; `market_purchase_recovered` ditambah ronde 17b | — | Selesai |
| OQ-009 | Path spesifik endpoint NearBlocks | Tim FE | Notifikasi/riwayat | Tidak (pola + param terkunci) | Saat implementasi |

## Matriks Scope (MVP vs Fase 2/3/4)

> SSOT scope MVP/Non-Goals: [01-PRD.md](./01-PRD.md) §8 & §4. Tabel ini memetakan fitur ke fase; **bukan** menambah keputusan baru.

| Kapabilitas | MVP (M1) | Fase 2 (M2) | Fase 3 (M3) | Fase 4 (M4/mainnet) |
|---|---|---|---|---|
| Fixed-price list/buy | ✅ | — | — | — |
| Offers (escrow, 7 hari, 1/buyer/token) | ✅ | — | — | — |
| Private listing | ✅ | — | — | — |
| Bundle (satu harga, maks 10 token) | ✅ | — | — | — |
| Open collections + verified badge | ✅ | — | — | — |
| Launchpad berphase + allowlist on-chain | ✅ | — | — | — |
| Pembayaran NEAR saja | ✅ | — | — | — |
| Notifikasi in-app (polling) | ✅ | — | — | — |
| Report + admin panel | ✅ | — | — | — |
| Auction / lelang | — | ✅ | — | — |
| Multi-currency (FT/NEP-141) | — | ✅ | — | — |
| Indexer fase awal (floor/volume) | — | ✅ | — | — |
| Lazy minting | — | — | ✅ | — |
| Trait filter + rarity + search indexer | — | — | ✅ | — |
| Notifikasi real-time (dari indexer) | — | — | ✅ | — |
| Cross-chain payment (Chain Signatures) | — | — | — | ✅ |
| Fiat on-ramp | — | — | — | ✅ |
| Rilis mainnet (audit + DAO + NEP-330) | — | — | — | ✅ |
