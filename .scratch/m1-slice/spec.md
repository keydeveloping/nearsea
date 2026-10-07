# M1 Vertical Slice — peta tiket

> Tracker: markdown lokal (lihat [docs/agents/issue-tracker.md](../../docs/agents/issue-tracker.md)).
> Dibuat ronde 17e dari `/to-tickets`. **Tiket = state kerja turunan**; SSOT tetap
> [tasks/backlog.md](../../tasks/backlog.md) + [tasks/milestones.md](../../tasks/milestones.md) +
> [docs/](../../docs/).

## Tesis yang dibuktikan slice ini

**mint → list (2-tx, non-custodial) → orang lain beli → royalti + fee terbayar → NFT tidak pernah dipegang kontrak.**

Sumber keputusan: [ADR-016](../../docs/decisions/ADR-016-project-intent-and-scope-cut.md). Proyek ini
**pembelajaran/portofolio** — sukses = tesis terbukti jalan + penulisnya paham cara kerjanya.

## Cakupan

| | Isi |
|---|---|
| **Termasuk** | M0 (fondasi & gate CI) + M1 (slice). Koleksi di-deploy **manual**, testnet saja, tanpa VPS/domain/IPFS/audit/DAO. |
| **Tidak termasuk** | M1+ (offers, bundle, factory, launchpad penuh, moderasi, admin, branding, infra) dan M2–M4. Ditangani tiket terpisah nanti. |
| **Jumlah** | **10 tiket**: 3 untuk M0, 7 untuk M1. |

## Peta ketergantungan

```text
01 scaffold + CI gate ─┬─→ 02 branch protection + secret scan
                       ├─→ 03 versioning & rilis
                       ├─→ 04 NFT core (mint/approval/set_phases) ─┬─→ 05 royalti NEP-199
                       │                                            └─→ 06 market listing 2-tx
                       │                                                    └─→ 07 market buy + refund
                       │                                                          └─→ 08 suite sandbox slice
                       └─→ 09 FE wallet connect ──────────────┐
                                                              └─→ 10 FE browse/list/buy
                                                                     (07 juga gate 10: jalur emas butuh kontrak nyata)
```

**Frontier** (bisa mulai sekarang): **01**. Setelah 01 selesai, empat jalur bebas paralel:
{02, 03}, {04}, {09}. Jalur kontrak (04→05/06→07→08) serial; jalur FE (09→10) paralel dengan itu,
dan tiket 10 menunggu **kedua** jalur (09 untuk wallet, 07 untuk kontrak).

## Daftar tiket

| # | Judul | Task | Blocked by |
|---|---|---|---|
| 01 | Scaffold workspace + gate CI | TASK-001 | — (mulai sekarang) |
| 02 | Proteksi 3 branch + secret scan | TASK-031 | 01 |
| 03 | Baseline versioning & rilis | TASK-032 | 01 |
| 04 | NFT core: mint, metadata, approval, satu fase publik | TASK-002 | 01 |
| 05 | Royalti NEP-199 | TASK-003 | 04 |
| 06 | Market listing: storage + 2-tx + dual verification | TASK-004 | 04 |
| 07 | Market buy: settlement, split royalti+fee, refund | TASK-005 | 06 |
| 08 | Suite sandbox dua-kontrak (slice) | TASK-006 | 04, 05, 06, 07 |
| 09 | FE: connect wallet | TASK-007 | 01 |
| 10 | FE: browse / list / buy | TASK-008 | 09, 07 |

## Demo M1 (bukti *done when*)

Skrip lengkap: [tasks/milestones.md](../../tasks/milestones.md) §Demo script per milestone.

```text
1. Dua akun testnet (seller/buyer) connect wallet via UI.
2. Seller: mint 1 NFT (koleksi di-deploy manual, satu fase publik) → list (2 langkah: approve + list).
3. Tunjukkan NFT TETAP di wallet seller selama listing — bukti non-custodial.
4. Buyer: buy → NFT pindah, seller + royalti dibayar, fee 2% terlihat di breakdown.
5. Tunjukkan `cargo test` hijau (suite slice) + race 20 pembeli → 1 menang, 19 refund.
```

## Aturan kerja tiket

- **Satu tiket = satu window `/implement`.** `/clear` di antaranya; tiket self-contained.
- **Kerjakan frontier**, blockers dulu. Jangan mulai tiket yang blocker-nya belum `resolved`.
- **Bukti sebelum selesai**: `cargo fmt --check` + `cargo clippy -D warnings` + `cargo test` hijau
  (kontrak), `pnpm lint` + `typecheck` + `test` hijau (FE), sesuai [AGENTS.md](../../AGENTS.md).
- **Requirement `SEC-*` P0 relevan** yang berstatus READY-FOR-IMPLEMENTATION/DECIDED adalah bagian dari
  definisi selesai ([docs/security/security-requirements.md](../../docs/security/security-requirements.md)).
- **Jangan menambah scope.** Kalau menemukan pekerjaan di luar tiket → catat di `tasks/backlog.md`,
  jangan kerjakan di sini (aturan "Surgical Changes", AGENTS.md).
- **Kesimpulan yang mengikat** (keputusan arsitektur/produk, nilai bisnis baru) **wajib dipromosikan** ke
  `docs/decisions/ADR-*` / `docs/features/*` / `tasks/backlog.md` — `.scratch/` bukan SSOT.
- **Deploy ke `testnet`/`mainnet` wajib tanya user dulu** ([git-workflow.md](../../docs/development/git-workflow.md) §3).

## Catatan: penyimpangan sadar dari resep `/to-tickets`

Skill `/to-tickets` menuntut **tracer bullet** — satu irisan tipis menembus semua lapisan, demoable
sendiri. Tiket di sini **tidak** begitu: 04–08 menyentuh lapisan kontrak lebih dulu, 09–10 lapisan FE,
mengikuti graf `Depends` yang sudah jadi SSOT di [tasks/backlog.md](../../tasks/backlog.md).

Alasan memilih graf yang ada:
1. **Memotong ulang akan memutus jejak `TASK-xxx`** yang dirujuk 100 file dokumen — melanggar aturan
   SSOT repo ini.
2. **Graf kontrak memang serial secara nyata** — `nft_transfer_payout` tidak bisa diuji sebelum ada
   token untuk di-transfer.
3. Skill mengizinkan alternatif "**verifiable** on its own", bukan hanya "demoable": tiap tiket kontrak
   diverifikasi `cargo test` + TC yang disebut namanya.

Yang **hilang** karena pilihan ini: tidak ada demo end-to-end yang bisa ditunjukkan sebelum tiket 10
selesai. Demo M1 tetap satu kali di ujung, persis seperti skrip di `milestones.md`.

Kalau kamu lebih suka tracer bullet murni (mis. tiket "jalur tipis mint→list→buy dengan UI kasar"),
bilang saja — tiket bisa dipotong ulang tanpa menyentuh `docs/` maupun `tasks/`.

## Referensi spec (dokumen pemilik detail)

| Kebutuhan | Dokumen |
|---|---|
| Perilaku listing/buy/refund | [docs/features/marketplace.md](../../docs/features/marketplace.md) |
| Algoritma payout, fee 2%, refund | [docs/features/payments.md](../../docs/features/payments.md) |
| Signature, layout state, konstanta gas, view | [docs/contracts/nft-collection.md](../../docs/contracts/nft-collection.md) · [docs/contracts/market.md](../../docs/contracts/market.md) |
| Invariant | [docs/security/smart-contract-invariants.md](../../docs/security/smart-contract-invariants.md) |
| Test case | [docs/testing/test-cases.md](../../docs/testing/test-cases.md) §Cakupan slice M1 |
| Registry kode error | [docs/development/error-handling.md](../../docs/development/error-handling.md) §3/§4 |
| Gate CI lokal | [AGENTS.md](../../AGENTS.md) §Build & Test Commands |
| Aturan keamanan NEAR | [RESEARCH.md](../../RESEARCH.md) §3 · [AGENTS.md](../../AGENTS.md) §Blockchain Rules |
