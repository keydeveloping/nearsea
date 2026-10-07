# Implementation Plan

> Jembatan dokumen â†’ kode. Urutan kerja konkret per fase, dengan dependensi.
> **Pemetaan Fase <-> Milestone**: Fase 1-2 = M1 (vertical slice); Fase 2b = M1+ (MVP completion); Fase 3 = M2; Fase 4 = M3; Fase 5 = M4 (hanya jalur produk).
> Proses kerja (ronde 12): timeline fleksibel dipandu *done when*; agent bekerja per milestone â†’ demo di akhir milestone; git branch per fitur + conventional commits (AGENTS.md).

## Fase 0 â€” Keputusan (ronde 1â€“13 SELESAI 2026-10-01)

Seluruh keputusan produk, arsitektur, business rules, UX struktur, API, security, dan ops sudah diambil.
Detail: [decisions/ADR-001.md](../docs/decisions/ADR-001.md) (log ronde 1â€“13) + [AUDIT-TEMUAN.md](../AUDIT-TEMUAN.md) (perbaikan hasil review).
Sisa (non-blokir Fase 1): alamat treasury (saat deploy), provider IPFS (saat fitur mint), domain (saat rilis). **TASK-008b (branding): M1+ — TIDAK memblokir M1.**

## Fase 1 â€” Kontrak inti (M1 = vertical slice) â­

> **Rombak ronde 17 (ADR-016):** Fase 1 + Fase 2 sekarang menargetkan **M1 = vertical slice**
> (8 task). Factory, launchpad, offers, bundle, dan infra **tidak** di jalur M1 â€” mereka di **M1+**.

Urutan kerja (dependency-ordered). Semua yang di bawah ini **wajib** untuk M1:

```text
TASK-001 scaffold repo (contract/market/frontend, CI gates)
         + TASK-031 git/repo hygiene (3 branch, .gitignore, secret scan) + TASK-032 versioning/CHANGELOG
TASK-002 NFT core: mint, metadata, NEP-178 approvals, NEP-199 payout, NEP-297 events
   â†’ TASK-003 royalty (cap 10%)
   â†’ TASK-004 market: storage NEP-145 + listing 2-tx + dual verification
   â†’ TASK-005 market: buy + settle + resolve/refund
   â†’ TASK-006 sandbox tests 2-kontrak (mintâ†’listâ†’buyâ†’refund + race TC-016..018)
```

**Koleksi di-deploy manual** (`cargo near deploy`) â€” **TASK-012 factory tidak dipakai di M1.**
Market hanya menerima koleksi NearSea di slice (menutup temuan H1 review desain).

Aturan: tidak lanjut ke task berikut sebelum tests hijau. Semua method wajib memenuhi SEC-* P0 relevan
(security-requirements.md) + INV-001..016 (subset yang berlaku di slice).

## Fase 2 â€” Frontend slice (M1)

```text
TASK-007 wallet connect (near-connect)
   â†’ TASK-008 browse / list / buy UI
   â†’ DEMO M1: dua akun testnet listâ†’buy end-to-end via UI, royalti + fee 2% terlihat di breakdown,
              NFT tetap di wallet seller selama listing
```

**Tidak ada prasyarat branding** untuk M1 â€” pakai default Tailwind. Sesi desain (TASK-008b) pindah ke M1+.
**Tidak ada VPS/infra** di M1 â€” FE jalan lokal, kontrak di testnet.

## Fase 2b â€” M1+ (MVP completion, lanjutan eksplisit)

> Dikerjakan **setelah** M1 lulus. Urutan bebas (tidak memblokir M1).

```text
TASK-008b branding â†’ TASK-009 offers â†’ TASK-010 private listing + bundle âš ï¸ â†’ TASK-012 factory
   â†’ TASK-020 launchpad kontrak â†’ TASK-021 launchpad UI â†’ TASK-022 stale cleanup
   â†’ TASK-015 notifikasi â†’ TASK-018/019 report + badge â†’ TASK-023 admin panel â†’ TASK-033 error module
   â†’ TASK-026 SEC-* P0 lengkap â†’ TASK-028/029/030 infra (VPS, CI/CD, backup)
```

âš ï¸ **Prasyarat TASK-010 (bundle):** temuan **C1/C2/C3** dari review desain ronde 17 wajib diselesaikan
lebih dulu â€” `PARTIAL` tanpa alokasi kerugian; gas `buy_bundle` >300 Tgas; dana buyer bisa nyangkut bila
callback gagal. Lihat [docs/contracts/market.md](../docs/contracts/market.md) Â§Catatan review desain.

## Fase 3 â€” Trading lanjutan (M2)

```text
TASK-017 auctions â†’ TASK-011 multi-currency FT â†’ TASK-014 custom indexer (Neardata, trait/floor/volume dasar)
```

## Fase 4 â€” Ekosistem (M3)

```text
TASK-013 lazy minting + TASK-014 lanjutan (trait filter, rarity, search) + notifikasi real-time dari indexer
```

## Fase 5 â€” Rilis (M4)

```text
Audit eksternal lulus (TASK-027) â†’ transfer owner ke Sputnik DAO V2 council 2-of-3 â†’ reproducible build terverifikasi (NEP-330) â†’ deploy mainnet
```

## Definisi "selesai" per fase

- Kode + tests + docs/features ter-update + AC terkait lolos + requirement SEC-* P0 terpenuhi.

Pembagian kerja: **agent mengerjakan per milestone â†’ demo di akhir milestone**; user review/keputusan di titik demo.

## Verifikasi, branch, & estimasi per task

> Kolom **Branch** = nama branch yang dipakai (pola [git-workflow.md](../docs/development/git-workflow.md) Â§4). Kolom **Verifikasi** = bukti minimum sebelum task dianggap selesai. **Est** = hari-dev (PROPOSED, selaras [backlog.md](./backlog.md)).

| Task | Branch | Est | Verifikasi (bukti selesai) |
|---|---|---|---|
| TASK-001 | `chore/scaffold-repo` | 3 | CI + Security hijau di `dev`; struktur workspace & manifest ada; `cargo test`/`pnpm build` placeholder jalan |
| TASK-002 | `feat/nft-core` | 4 | `cargo test` hijau; mint + `nft_transfer` + events NEP-297 lolos; TC mint |
| TASK-003 | `feat/nft-royalty` | 1 | Test payout royalti â‰¤10% (INV-003/027) |
| TASK-004 | `feat/market-listing` | 4 | List/cancel 2-tx + storage deposit lolos (INV-020, TC-002) |
| TASK-005 | `feat/market-buy-settle` | 5 | Buy sukses + refund + race 20 pembeli (INV-001/016, TC-003/016) |
| TASK-006 | `test/sandbox-two-contracts` | 5 | Seluruh suite sandbox hijau; semua INV P0 punya test |
| TASK-007 | `feat/wallet-connect` | 2 | Connect/disconnect + banner network di testnet |
| TASK-008 | `feat/marketplace-ui` | 5 | Playwright jalur emas browseâ†’listâ†’buy hijau |
| TASK-008b | `feat/branding-design-system` | 3 | Token warna/font + primitives dipakai TASK-008/021 |
| TASK-009 | `feat/offers` | 4 | Offer lifecycle + refund (INV-005/006/024, TC-004/018) |
| TASK-010 | `feat/private-bundle` | 3 | Private guard + bundle pre-validasi (INV-025/026/028, TC-009/011) |
| TASK-011 | `feat/ft-settlement` | 4 | Settlement FT lolos AC pembayaran |
| TASK-012 | `feat/collection-factory` | 3 | Deploy koleksi via factory + halaman create jalan |
| TASK-013 | `feat/lazy-minting` | 4 | Mint-on-buy + metadata benar; sandbox hijau |
| TASK-014 | `feat/indexer-neardata` | 8 | Lag=0; floor/volume tampil; dedup `(receipt_id,event_index)` lolos |
| TASK-015 | `feat/notifications-polling` | 3 | Bell + unread â‰¤60 dtk; dedup event id lolos |
| TASK-016 | `feat/meta-tx` | 3 | Relayer testnet sukses; tidak menahan kunci dana |
| TASK-017 | `feat/auctions` | 5 | Auction end-to-end lolos AC + sandbox |
| TASK-018 | `feat/report-system` | 3 | `POST /reports` + antrean admin; idempotensi harian lolos |
| TASK-019 | `feat/verified-badge` | 2 | Set verified (step-up) + badge tampil; audit tercatat |
| TASK-020 | `feat/launchpad-contract` | 4 | Mint per phase + allowlist + pause (INV-017/018/022/029) |
| TASK-021 | `feat/launchpad-ui` | 4 | Mint satu koleksi via wizard end-to-end testnet |
| TASK-022 | `feat/stale-listing` | 2 | Stale terdeteksi & disembunyikan (INV-016, TC-006/010) |
| TASK-023 | `feat/admin-panel` | 3 | Guard allowlist + step-up; 403 bila bukan admin |
| TASK-024 | `docs/onboarding-guide` | 2 | Halaman `/onboarding` tampil + langkah akurat |
| TASK-025 | `feat/leaderboard` | 2 | `/leaderboard` menampilkan agregat terbatas |
| TASK-026 | `security/sec-p0-implementation` | 5 | Semua SEC-* P0 relevan terverifikasi; gate CI aktif |
| TASK-027 | `security/external-audit` | 2â€“4 mgg | Laporan audit + temuan kritis ditutup (definisi [milestones.md](./milestones.md)) |
| TASK-028 | `chore/provision-vps` | 3 | Stack jalan di VPS + staging subdomain hidup |
| TASK-029 | `chore/cicd-deploy` | 3 | Deploy dev/testnet otomatis + mainnet manual ter-gate |
| TASK-030 | `chore/backup-monitoring` | 2 | Backup harian + restore drill lulus + alert 5xx sampai |
| TASK-031 | `chore/repo-hygiene` | 1 | Proteksi 3 branch + gitleaks hijau + CODEOWNERS aktif |
| TASK-032 | `chore/versioning-release` | 1 | Tag pertama + CHANGELOG terisi + versi NEP-330 terverifikasi |
| TASK-033 | `feat/centralized-errors` | 2 | Modul error terpusat dipakai; tidak ada pesan ad-hoc |
| TASK-034 | `chore/scaling-prep` | 3 | App stateless terverifikasi + rencana replica/LB tertulis |

- Branch dibuat dari `dev` (kecuali `hotfix/*` dari `mainnet`); PR selalu ke `dev` (promosi terpisah).
- Estimasi = pekerjaan bersih; tambahkan buffer review/promosi saat menjadwalkan.

## Checkpoint & rollback per fase

| Fase | Checkpoint (harus hijau sebelum lanjut) | Bila gagal â†’ rollback |
|---|---|---|
| Fase 1 | TASK-006 suite sandbox hijau + semua INV **slice** (001..016, 023, 030, 031) punya test | Perbaiki di branch; tidak promosi; kontrak belum ter-deploy |
| Fase 2 | Demo M1 lulus end-to-end di testnet (list->buy + royalti + fee) | Web: deploy tag sebelumnya; kontrak: redeploy wasm sebelumnya |
| Fase 3 | AC auction + FT lulus; indexer lag 0 | Hentikan indexer + rebuild dari `events_raw`; revert fitur |
| Fase 4 | Trait/rarity/search + notifikasi real-time lulus | Revert merge commit fitur; matikan fitur via flag bila ada |
| Fase 5 | Gate audit + DAO + reproducible build + drill lulus | Tunda deploy mainnet; perbaiki dan ulangi gate |

- Checkpoint = **titik keputusan user** (bukan sekadar internal); agent berhenti dan melaporkan sebelum melangkah.
- Rollback detail: [ci-cd.md](../docs/development/ci-cd.md) Â§16; risiko per milestone: [milestones.md](./milestones.md).

## Kriteria keluar fase (eksplisit)

- **Fase 0 (selesai):** semua keputusan tercatat di ADR; sisa non-blokir ditandai â³.
- **Fase 1:** (a) kontrak NFT+market ter-compile & sandbox hijau; (b) INV slice >=1 test; (c) SEC-* P0 relevan slice terpenuhi; (d) CHANGELOG & versi terisi (TASK-032).
- **Fase 2:** (a) demo M1 (lihat [milestones.md](./milestones.md)) lulus; (b) FE jalan lokal + kontrak di testnet; (c) smoke test hijau.
- **Fase 3:** (a) auction & FT lolos AC; (b) indexer lag 0 + stats dasar; (c) SEC-INDEX-* terpenuhi.
- **Fase 4:** (a) lazy mint, trait filter, rarity, search berfungsi; (b) notifikasi real-time dari indexer; (c) coverage kontrak target terpenuhi.
- **Fase 5:** (a) "audit lulus" per definisi [milestones.md](./milestones.md); (b) owner â†’ DAO 2-of-3; (c) reproducible build terverifikasi; (d) drill pause/restore & backup lulus; (e) deploy mainnet disetujui user.

- Fase tidak dianggap keluar bila ada kriteria belum terpenuhi â€” tidak ada "lulus bersyarat" untuk gate P0.

## Penjadwalan kerja paralel

> Batasan keras: **E1 (kontrak inti) berurutan** (dependensi kontrak). Setelah TASK-001, jalur lain boleh paralel.

```text
Gelombang 0 (M0, serial):  TASK-001 â†’ TASK-031 â†’ TASK-032
                                     (026 menyertai sejak awal)

Gelombang 1 (paralel setelah 001):
  Jalur A (kontrak, SERIAL): 002 â†’ 003/012 â†’ 020 â†’ 004 â†’ 009 â†’ 005 â†’ 010 â†’ 022 â†’ 006
  Jalur B (FE, setelah 008b): 007 â†’ 008 â†’ 015 â†’ 018/019 â†’ 021 â†’ 023
  Jalur C (infra, paralel):   028 â†’ 029 â†’ 030
  Jalur D (security, paralel): 026 (sepanjang fase) â†’ 027 (M4)

Sinkronisasi M1: Jalur A (006 hijau) + Jalur B (demo UI) + Jalur C (deploy) â†’ DEMO M1
Gelombang 2 (M2): 017 â†’ 011 â†’ 014 â†’ 034
Gelombang 3 (M3): 013 â†’ 014 lanjutan â†’ 016/024/025
Gelombang 4 (M4): 027 â†’ DAO â†’ reproducible build â†’ deploy
```

Aturan penjadwalan paralel:

- **Satu penulis per file/lapisan**: dua agent tidak boleh menyentuh file yang sama bersamaan (hindari konflik; lihat git-workflow Â§13).
- Jalur B (FE) menunggu **TASK-008b** (branding) sebelum UI; TASK-007 boleh mulai lebih awal.
- Jalur B boleh memakai mock kontrak (fixture) sebelum Jalur A selesai, tetapi demo M1 memakai kontrak nyata.
- Jalur C (infra) tidak menunggu kontrak; deploy menyusul saat artefak siap.
- TASK-026 (SEC-*) berjalan **sepanjang** fase 1â€“2, bukan task akhir.
- Bila satu jalur terhambat, jalur lain tetap jalan â€” **kecuali** menghambat demo milestone (prioritaskan jalur kritis: A â†’ B â†’ C).
- Review/merge ke `dev` boleh paralel; promosi ke `testnet`/`mainnet` dilakukan **serial** dan setelah tanya user.
