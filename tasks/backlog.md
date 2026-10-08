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
| TASK-003 | Kontrak NFT: royalty NEP-199 (cap 10%) | contract | P0 | 002 | features/marketplace | done | 1 | Payout royalti ≤10% teruji (INV-003/027) | M1 |
| TASK-004 | Kontrak market: storage NEP-145 + listing 2-tx + dual verification | contract | P0 | 002 | features/marketplace | done | 4 | List/cancel + storage deposit lolos (INV-020, TC-002) | M1 |
| TASK-005 | Kontrak market: buy + nft_transfer_payout + resolve/refund (+ private listing & bundle — dgn TASK-010) | contract | P0 | 004 | features/marketplace.md + payments.md | done | 5 | Buy sukses + refund + race 20 pembeli lolos (INV-001/016, TC-003/016) | M1 |
| TASK-006 | Sandbox tests 2-kontrak **subset slice**: mint->list->buy->refund + race (TC-001/002/013/016/017/020/022/044/047/048) + INV slice (001..016, 023, 030, 031) | contract | P0 | 002-005 | testing/test-cases.md (Slice M1) | todo | 4 | Suite sandbox hijau; semua INV **slice** punya test (INV M1+ di-defer eksplisit) | M1 |
| TASK-007 | Frontend: connect wallet (near-connect) | frontend | P0 | 001 | features/auth | todo | 2 | Connect/disconnect + banner network jalan di testnet | M1 |
| TASK-008 | Frontend: browse/listings/buy UI (Next.js + Tailwind, EN + i18n) — **tanpa prasyarat branding** (default Tailwind) | frontend | P0 | 007 | features/marketplace | todo | 5 | Browse->list->buy end-to-end via UI (jalur emas Playwright) | M1 |
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
| TASK-032 | Versioning & rilis: SemVer per-artefak + tag + CHANGELOG + versi kontrak NEP-330 (SEC-CONTRACT-006) | infra/docs | P0 (M1) | 001 | development/versioning-and-release | done | 1 | Tag pertama + CHANGELOG terisi + versi NEP-330 terverifikasi | M0 |
| TASK-033 | Error & notifikasi terpusat: registry kode error + pemetaan panic kontrak + kebijakan notifikasi (FE + API) | frontend/backend | P1 | 008 | development/error-handling | todo | 2 | Modul error terpusat dipakai; tidak ada pesan ad-hoc | M1+ |
| TASK-034 | Prep scaling: app stateless + pooling + rencana read replica/LB (aktif saat trafik naik) | infra | P1 (fase 2) | 028 | architecture/scaling | todo | 3 | App stateless terverifikasi + rencana replica/LB tertulis | M2 |
| TASK-035 | Audit dependensi: tinjau ulang advisory tanpa patch yang di-*ignore* (`--ignore-unfixable`) | infra | P2 | 001 | development/ci-cd.md §3 | todo | 0.5 | Advisory yang di-ignore punya keputusan tercatat: diperbaiki, diganti, atau diterima + alasan | M1+ |

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

> **TASK-032 `done` (ronde 20):** skema versi per-artefak ditegakkan mesin, bukan sekadar dokumen.
> **Yang dibangun**: `[package.metadata.near.reproducible_build]` di ketiga crate (image Docker
> `sourcescan/cargo-near:0.21.1-rust-1.96.0` **ter-pin by digest**) + `repository` di `[package]`;
> workflow baru `.github/workflows/release.yml` (dipicu tag `contract-v*`/`web-v*`/`indexer-v*`, bisa
> diuji-kering lewat `workflow_dispatch`) yang memeriksa **versi manifest == tag**, membangun wasm
> reproducible di container, **membuktikan metadata NEP-330 tertanam == tag**, lalu melampirkan
> artifact + `code-hash.txt` ke GitHub Release; `ci.yml` kini membangun **ABI** (bukan `--no-abi`) dan
> memverifikasi metadata NEP-330 tiap PR. **Bukti lokal**: `cargo fmt --check` + `clippy -D warnings`
> (0 warning) + `cargo test --workspace` (34 test) hijau; metadata NEP-330 ketiga crate dibaca langsung
> dari wasm (`version=0.1.0`, `link=https://github.com/keydeveloping/nearsea`) — sebelumnya `link` = `null`.
> **Belum tertutup di tiket ini (sengaja, butuh aksi user):** tag pertama **belum dibuat** — tag hanya
> sah di `testnet`/`mainnet` setelah PR promosi, dan merge ke sana **wajib tanya user** (`git-workflow.md`
> §3/§12). Verifikasi reproducible + ABI juga butuh Docker/Linux → dijalankan CI, bukan lokal Windows
> (`ci-cd.md` §14). Rilis pertama yang disiapkan: `contract-v0.1.0` (prosedur + rollback di `CHANGELOG.md`).

> **TASK-003 `done` (ronde 21):** royalti NEP-199 di `contract/src/lib.rs` — `nft_transfer_payout`
> (transfer + payout dalam satu panggilan, `assert_one_yocto`, `max_len_payout` dihormati) + helper murni
> `royalty_amount(balance, bps) = floor(balance × bps / 10_000)` dengan `checked_mul`. Payout diturunkan
> dari konfigurasi royalti **level kontrak** (bukan `TokenMetadata.extra`), satu penerima (`creator_id`).
> **Bukti**: 11 test baru (45 test workspace) — perpindahan kepemilikan, ≤10% untuk 4 rate × 6 basis,
> dust (`19` → `0`, `20` → `1` yocto di 500 bps), tepat 10% di cap, wajib 1 yocto, penolakan pengirim
> tanpa approval, token tidak ada, `max_len_payout` terlalu kecil, approval invalid setelah transfer
> (INV-011). Gate lokal hijau: `fmt --check`, `clippy -D warnings` (0 warning), `test --workspace`,
> build wasm 272 KB (ABI memuat `nft_transfer_payout`). **Belum diklaim**: angka gas 15 Tgas dan
> TC-003 versi sandbox — keduanya butuh suite dua-kontrak milik TASK-006; sisi **market** validasi
> payout (≥1 penerima, Σ ≤ harga−fee, sisa ≤1 yocto, refund) juga di sana.

> **TASK-004 `done` (ronde 22):** jalur listing market di `market/src/lib.rs` — `list_nft_for_sale`
> (2-tx, non-custodial) + callback `process_listing` `#[private]` yang **memverifikasi sendiri** lewat
> dua view XCC (`nft_token` → kepemilikan, `nft_is_approved` → approval), `remove_sale`, `update_price`,
> view `get_sale`/`get_sales`/`get_supply_sales`, storage NEP-145 (bounds `min = storage_per_sale()`,
> 500 byte PROPOSED), Pausable (INV-022: `remove_sale` tetap boleh saat paused), dan tiga event
> (`market_list`/`market_delist`/`market_update_price`) dengan envelope `SingleEvent` yang sama dengan
> koleksi. **Bukti**: 32 test baru (75 test workspace) — dual verification termasuk jalur token tidak
> ada/`None`/approval `false`, non-custodial dibuktikan dengan **membaca receipt** (hanya 2 view +
> callback; tidak ada `nft_transfer*`), harga min inklusif, duplikat listing, storage kurang, storage
> kembali saat cancel, `approval_id` di luar rentang `u32`, paginasi/clamp. Gate lokal hijau:
> `fmt --check`, `clippy -D warnings` (0 warning), `test --workspace`, build wasm 197 KB (semua method
> ada di ABI). **Koreksi dokumen (temuan saat implementasi):** `nft_revoke_token` yang disebut 5 dokumen
> **tidak ada di NEP-178** (hanya `nft_revoke`/`nft_revoke_all`, keduanya owner-only) — jadi `remove_sale`
> **tidak** mencabut approval; re-list butuh `nft_revoke` oleh seller lalu `nft_approve` baru. Didokumentasikan
> di [contracts/market.md](../docs/contracts/market.md) §2a + 5 dokumen lain disinkronkan. **Belum diklaim**:
> paruh **buy** TC-002, TC-016/017 (race), TC-022, TC-048 (callback palsu), dan angka gas penuh — butuh
> dua kontrak nyata (TASK-005/006). `fee_bps`/`treasury` di init juga ditunda ke TASK-005 (hanya dipakai settlement).

> **TASK-005 `done` (ronde 23):** jalur settlement market di `market/src/lib.rs` — `buy` (tulis
> `pending_purchases` **sebelum** optimistic removal, INV-031) → callback `process_purchase` `#[private]`
> (dual verification saat settle; **stale dua kasus** INV-016 → refund + `market_stale_detected`; verifikasi
> tak pasti → refund + restore `Sale`) → `nft_transfer_payout` (1 yocto, `max_len_payout = 10`) → callback
> `resolve_purchase` `#[private]` (validasi payout UNTRUSTED: `1..=10` penerima, `amount > 0`,
> `Σ ≤ harga−fee`; distribusi fee → treasury, royalti → receiver, residual → seller, kelebihan deposit →
> buyer; payout invalid/promise gagal → refund penuh + restore `Sale`), `recover_stuck_purchase` +
> `process_recovery` (permissionless, jeda `RECOVERY_DELAY_BLOCKS`), `update_fee_bps`/`update_treasury`
> (owner-only, 1 yocto, cap `MAX_FEE_BPS`), `fee_bps`/`treasury` di init, view `get_fee_bps`/`get_treasury`/
> `get_pending_purchase`, event `market_sale`/`market_stale_detected`/`market_purchase_recovered`/
> `fee_update`/`treasury_update`. **Bukti**: 41 test baru (116 test workspace) — happy path dengan angka
> fee/royalti/seller exact (Σ keluar == Σ masuk), kelebihan deposit refund, private listing, self-buy,
> deposit kurang, `CONFLICT_SOLD` (sale/pending), stale dua kasus, verifikasi tak pasti, 6 jalur payout
> invalid → refund, recovery permissionless (refund + restore / tanpa restore bila token pindah), paused,
> dan anggaran gas `resolve_purchase` worst case. Gate lokal hijau: `fmt --check`, `clippy -D warnings`
> (0 warning), `test --workspace`, build wasm 239 KB. **Dua koreksi dokumen (temuan saat implementasi):**
> (1) **`withdraw_fees` dihapus** — fee ditransfer langsung ke treasury saat settlement (kontrak tidak
> pernah memegang fee; menghapus temuan M7 + honeypot saldo) → didokumentasikan di
> [contracts/market.md](../docs/contracts/market.md) §4a + 8 dokumen lain disinkronkan, TC-047 dialihkan
> ke `update_fee_bps`/`update_treasury`; (2) **aturan `sisa ≤ 1 yocto` (INV-002) dibatalkan** — koleksi
> mengembalikan **hanya royalti**, market menambahkan seller sebagai residual, jadi `harga − fee − Σpayout`
> = proceeds seller (wajar besar), bukan dust → INV-002 dikoreksi + ~15 dokumen disinkronkan. **Belum
> diklaim**: race TC-016/017 versi sandbox, TC-022, TC-048, dan angka gas terukur — semuanya butuh dua
> kontrak nyata (TASK-006).

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
