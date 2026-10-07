# Testing Strategy

> Semua bukti kebenaran. Kontrak = lapisan paling kritis (dana user).

## Piramida testing proyek ini

```text
E2E (frontend + testnet)          — sedikit, jalur emas saja
Integration sandbox (kontrak)     — BANYAK: 2 kontrak bersama (pola tutorial auction)
Property/fuzz (invariants)        — INV-001..022+ sebagai target (quickcheck/proptest)
Unit (kontrak & util)             — matematika payout, validasi, konversi unit
API test (API activity/report)     — endpoint + verifikasi signature wallet (NEP-413) (NEP-413)
Security test                     — checklist NEAR + skenario threat-model
```

## Tooling

- Kontrak: `cargo test` + **near-workspaces** (sandbox, deploy NFT + market bersama).
- **Property/fuzz**: quickcheck/proptest di atas near-workspaces untuk invariant (referensi pola: near-prop); **cargo-fuzz** untuk fungsi murni non-chain (parsing payout, konversi u128, perhitungan fee/split) — keputusan 2026-10-01.
- Frontend: **Vitest + React Testing Library** (unit/hook) + **Playwright** (E2E jalur emas: connect → list → buy, offer → accept).
- API/report: integration test endpoint + verifikasi signature (NEP-413 + fallback).
- **Setiap endpoint API wajib punya test** — minimal 1 happy path + 1 jalur error (kode error
  mengikuti katalog [development/error-handling.md](../development/error-handling.md)). Endpoint
  tanpa test dianggap belum selesai (lihat DoD di bawah).

## Target cakupan (coverage targets)

> Target = kontrak kualitas minimum sebelum fitur dianggap selesai. Aturan yang sudah mengikat repo
> ditandai **WAJIB**; angka persen yang belum final ditandai PROPOSED (dikunci saat scaffold, TASK-001).

| Lapisan | Target | Status |
|---|---|---|
| Kontrak — invariant | **30/30 INV-001..INV-030** punya ≥1 test yang benar-benar mengeksekusinya | WAJIB |
| Kontrak — method mutasi | setiap method mutasi publik punya ≥1 sandbox test sukses + ≥1 jalur gagal | WAJIB |
| Kontrak — line/branch | ≥ 90% line pada kontrak `market`/`factory` (diukur `cargo llvm-cov`) | PROPOSED |
| API — endpoint | **100% endpoint MVP** punya ≥1 happy + ≥1 error test | WAJIB |
| API — kode error | setiap kode registry yang dipakai endpoint punya ≥1 test pemicunya ([error-handling.md](../development/error-handling.md) §3) | PROPOSED |
| FE — unit | ≥ 70% line pada `lib/` (format yocto, pemetaan error, breakdown fee/royalti) | PROPOSED |
| FE — komponen kritikal | Buy/Offer/List/Bundle modal + WalletButton punya RTL test (state loading/error/disabled) | WAJIB |
| E2E | 2 jalur emas + ≥1 jalur error | WAJIB |
| Fuzz | target `cargo-fuzz` fungsi murni lulus smoke + corpus tanpa crash | WAJIB |

- Persentase line **bukan** pengganti invariant coverage: 100% line tetap bisa melanggar INV.
- Baseline coverage disimpan per branch; penurunan > 2 poin pada PR → blok merge (usulan gate CI).

## Strategi fixture & test data

> Dataset kanonik = [database/seed-data.md](../database/seed-data.md) §1. Jangan membuat dataset paralel;
> setiap test merujuk ke sana.

| Lapisan | Fixture | Sumber |
|---|---|---|
| unit / fuzz | nilai murni (payout list, u128, fee) + generator deterministik | kode test; tidak menyentuh chain |
| sandbox (near-workspaces) | akun ephemeral + deploy NFT + market per test; state dibangun lewat transaksi kontrak | test code (bukan seed) |
| API integration | skema DB dari migration + baris app-mutable (profiles/reports/blocklist/collections) | seed app-mutable §1; isolasi per test (transaksi/rollback) |
| E2E (testnet) | dataset seed lengkap (4 akun, 2 koleksi, 20 token, listing/offer/bundle) | `scripts/seed.ts` — [seed-data.md](../database/seed-data.md) |

Aturan:

- **Determinisme** ([seed-data.md](../database/seed-data.md) §2): akun/token/harga/trait deterministik; tidak ada RNG untuk data yang di-assert.
- Sandbox **tidak** memakai seed testnet — state dibangun di dalam test agar cepat & terisolasi; seed hanya untuk E2E/manual.
- Nilai uang di fixture selalu **string yoctoNEAR** (1 Ⓝ = `1000000000000000000000000`; 0.01 Ⓝ = `10000000000000000000000`).
- Fixture E2E wajib lolos checklist post-seed [seed-data.md](../database/seed-data.md) §5 sebelum dipakai.
- Reset E2E: `tsx scripts/seed.ts --reset` (idempoten; dilarang menyentuh data non-seed).

## Strategi mocking

> Prinsip: **jangan pernah mem-mock jalur uang**. Settlement, payout, dan otorisasi diuji dengan kontrak nyata.

| Diuji | Real vs mock | Catatan |
|---|---|---|
| Kontrak market + NFT | **real** (near-workspaces, 2 kontrak di-deploy bersama) | cross-contract nyata; tidak ada mock contract |
| Waktu/expiry | durasi pendek + `produce_blocks`; inject `expires_at` bila memungkinkan | near-workspaces tidak punya time warp (lihat TC-004) |
| FE — RPC/view call | mock client (`near-api-js`/fetch) dengan fixture respons | komponen diuji tanpa network |
| FE — wallet (near-connect) | mock signer/`signMessage` | signature nyata diuji di sandbox/API, bukan unit FE |
| API — DB | **real** Postgres (testcontainer / DB test terpisah) | migration dijalankan nyata; DB sendiri tidak di-mock |
| API — NearBlocks / RPC eksternal | mock / recorded fixture | data pihak ketiga = proyeksi, bukan otoritas |
| Signature NEP-413 | **real** — test vector ([api/authentication.md](../api/authentication.md) § NEP-413 test vectors) | jangan mock verifikasi signature |
| Rate limit | fake clock / kontrol jendela | diuji di API integration |

- Dilarang mem-mock `assert_one_yocto`, cek ownership, atau perhitungan payout/fee.
- Mock hanya menggantikan **I/O eksternal**, bukan logika domain.

## Konvensi penamaan test

| Artefak | Pola | Contoh |
|---|---|---|
| Rust unit/sandbox | `test_<method>_<kondisi>_<harapan>` | `test_buy_when_sale_removed_refunds_buyer` |
| Rust invariant | modul `inv_<nnn>_<topik>` | `inv_025_bundle_prevalidation_aborts` |
| TS unit (Vitest) | `<unit>.test.ts`; `describe`=unit, `it`=perilaku | `it('formats 1e24 yocto as 1 NEAR')` |
| TS komponen | `<Component>.test.tsx` | `BuyButton.test.tsx` |
| API integration | `<method>_<path>.spec.ts` | `patch_profile.spec.ts` |
| E2E (Playwright) | `<flow>.spec.ts` | `buy-happy-path.spec.ts` |

- Setiap test mencantumkan ID yang dirujuk di komentar header: `// TC-002 · INV-001, INV-011`.
- Satu test = satu asersi utama (boleh beberapa asersi pendukung dari hasil yang sama).
- Dilarang nama generik (`test_works`, `test_1`).

## Anggaran waktu CI

> Angka = usulan; **timeout job kontrak (12 menit) & FE (6 menit) sudah dipasang di `ci.yml` (TASK-001)**.
> Job yang melewati timeout dibatalkan.

| Job | Isi | Anggaran | Timeout | Jalan |
|---|---|---|---|---|
| `contract-test` | fmt + clippy + unit + sandbox | ≤ 8 menit | 12 menit | setiap PR |
| `fe-test` | lint + typecheck + Vitest | ≤ 3 menit | 6 menit | setiap PR |
| `api-test` | integrasi endpoint + DB | ≤ 2 menit | 5 menit | setiap PR |
| `fuzz-smoke` | corpus + `-max_total_time=60` per target | ≤ 2 menit | 4 menit | setiap PR |
| `e2e-golden` | Playwright jalur emas (testnet) | ≤ 10 menit | 15 menit | nightly / pra-rilis |
| `fuzz-deep` | property/fuzz durasi panjang | ≤ 30 menit | 45 menit | nightly |

- Gate PR total (paralel) target **≤ 15 menit**; bila terlampaui → pindahkan suite berat ke nightly, bukan memperpanjang PR.
- Bila `contract-test` > 8 menit → shard sandbox test (usulan).

## Penyediaan environment E2E

> E2E berjalan di **testnet** dengan dataset seed kanonik ([seed-data.md](../database/seed-data.md)).
> Sandbox near-workspaces tidak dipakai untuk E2E browser.

```text
1. Siapkan akun & dana (faucet testnet) — seed-data.md §9.
2. Deploy market + factory + 2 koleksi (alamat dari env; §7).
3. Jalankan `tsx scripts/seed.ts` (idempoten) → dataset §1.
4. Verifikasi checklist post-seed (§5) — gagal → hentikan E2E.
5. Set env E2E: NEAR_NETWORK=testnet, MARKET_CONTRACT_ID, NFT_CONTRACT_ID_C1/C2, base URL API.
6. Playwright: baseURL web + storageState wallet akun seed (buyer/seller).
7. Setelah run: `tsx scripts/seed.ts --reset` (opsional; jangan sentuh data non-seed).
```

- Akun E2E tidak boleh berbagi dengan akun manusia; semua dari `SEED_NAMESPACE`.
- Uang E2E = testnet saja (dilarang mainnet).

## Kebijakan flaky test

| Aturan | Detail |
|---|---|
| Retry | **Dilarang** retry untuk menutupi flaky di PR. Maks 1 retry hanya untuk E2E terikat network, dianotasi `flaky:network`. |
| Quarantine | Gagal pada 2 run berturut tanpa perubahan kode → tandai `@quarantine`, keluarkan dari gate, **wajib** buat task di [backlog.md](../../tasks/backlog.md). |
| Larangan `sleep` | Gunakan `waitFor`/`wait_for_selector`/`produce_blocks`, bukan `sleep` arbitrer. |
| Waktu | Test yang bergantung waktu memakai clock injection / durasi pendek deterministik. |
| Isolasi | Test tidak boleh bergantung urutan test lain; setiap test membangun state sendiri. |
| SLA | Test `@quarantine` > 2 minggu tanpa perbaikan → blocker rilis (usulan). |

## Desain harness & corpus fuzz

> Fuzz fungsi **murni** (non-chain) dengan `cargo-fuzz` (+ `arbitrary`); invariant lintas-chain dengan
> quickcheck/proptest di atas near-workspaces.

| Target | Input | Properti yang diasersi |
|---|---|---|
| `payout_parse` | byte arbitrer → daftar (receiver, amount) | tidak panic; duplikat di-merge; penerima 1..10 (INV-003) |
| `fee_split` | `(price, fee_bps, royalti[])` | `fee_bps ≤ MAX_FEE_BPS`; `sum(payout) ≤ price − fee`; sisa ∈ {0,1} yocto (INV-002/004) |
| `u128_arith` | pasangan u128 + operasi | tidak overflow/panic; string↔u128 round-trip |

```text
fuzz/
  corpus/<target>/          # corpus dikomit (minimal, deterministik)
  fuzz_targets/<target>.rs
```

- Temuan unik/crash disimpan sebagai **regression test** (unit), bukan hanya corpus.
- Smoke CI: `cargo fuzz run <target> -- -max_total_time=60`; nightly: durasi lebih panjang.
- Proptest sandbox: generator urutan aksi (`mint→list→offer→cancel/accept/stale→pause`) → asersi seluruh INV setelah tiap langkah.

## Performance test

| Aspek | Batas kanonik | Cara uji |
|---|---|---|
| Gas per call | ≤ 300 Tgas ([01-PRD.md](../01-PRD.md) §14) | sandbox: ukur gas `buy`, `accept_offer`, `buy_bundle`, `nft_mint` |
| Gas resolve payout | budget 115 Tgas ([features/payments.md](../features/payments.md)) | ukur jalur `resolve_purchase` sukses & gagal |
| Bundle worst-case | 10 token (INV-021) | `buy_bundle` 10 token royalti maks → gas ≤ budget |
| Latensi API | p95 endpoint MVP (usulan ≤ 300 ms, di luar RPC) | k6/autocannon pada API integration |
| Rate limit | discovery 60/IP/menit; report 5/akun/hari; dst. ([api-security-architecture.md](../security/api-security-architecture.md) §1) | uji beban: request ke-(limit+1) → 429 + `Retry-After` |
| FE | LCP/CLS — ⏳ open-by-design (dikunci saat scaffold FE) | Lighthouse CI (usulan) |

- Performance test **bukan** gate PR (kecuali gas budget kontrak); dijalankan nightly / pra-rilis.

## Siapa menjalankan apa

| Peran | Kapan | Suite |
|---|---|---|
| Developer | sebelum buka PR | unit + sandbox + FE unit + API integration lokal |
| CI | setiap PR | `contract-test`, `fe-test`, `api-test`, `fuzz-smoke` |
| CI | nightly | `fuzz-deep`, `e2e-golden`, performance |
| QA / reviewer | sebelum merge ke `testnet` | E2E jalur emas + AC manual yang belum terotomasi |
| Rilis (pra-mainnet) | gate M4 | E2E + audit + verifikasi hash ([ci-cd.md](../development/ci-cd.md)) |

## Skenario wajib (dipetakan ke invariants — docs/security/smart-contract-invariants.md)

> **SSOT peta INV→test = tabel di [smart-contract-invariants.md](../security/smart-contract-invariants.md)**
> (peta invariant → predikat formal → test → SEC → prioritas). Tabel di bawah adalah **tampilan ringkas
> untuk perencanaan** — saat menambah/mengubah TC, perbarui tabel di invariants.md dulu, lalu sinkronkan di sini.

| Skenario | Invariant |
|---|---|
| Mint → list → buy happy path | INV-001, INV-011 |
| Buy saat token sudah terjual / stale (refund) | INV-008, INV-016 |
| Payout gagal (kosong / > harga / > 10 penerima / sisa > 1 yocto) | INV-001..003 |
| Royalti di batas (0%, 10%, pembulatan tak rata) + self-buy ditolak | INV-002, INV-023 |
| Remove/update listing oleh bukan owner (revert) | INV-012 |
| Storage deposit kurang saat listing | INV-020 |
| NFT dipindah di luar market lalu ada yang offer (stale) | INV-016 |
| Offer expire → refund escrow; accept offer → fee 2% konsisten | INV-005, INV-010 |
| Offer kedua oleh buyer sama / < min 0.01Ⓝ → ditolak | INV-024 |
| **Bundle: settle sukses + gagal pre-validasi → abort + refund; gagal residual → PARTIAL + kompensasi G7** | INV-025 |
| **Private listing: buyer lain ditolak** | INV-026 |
| **Royalti per token > 10% ditolak; bundle agregat > 10% tetap sah (tanpa cap agregat)** | INV-027 |
| **Bundle > 10 token ditolak** | INV-021 |
| Launchpad: mint di luar allowlist / phase belum mulai / alokasi habis (ditolak) + **phase overlap ditolak** | INV-017, INV-018, INV-029 |
| Pause: mutasi baru ditolak, penarikan escrow/cancel tetap bisa | INV-022 |
| **Race: 20 pembeli rebutan 1 listing → 1 sukses, 19 refund** | INV-007, INV-008, INV-009 (TC-016) |
| **Double-submit pembeli sama / double-accept offer → tx kedua revert** | INV-007, INV-008, INV-009 (TC-017/018) |
| **Token dalam bundle aktif di-list/di-offer terpisah → ditolak** | INV-028 (TC-043) |
| Cancel listing/offer/bundle → revoke approval + refund exact | INV-012, INV-005, INV-028 (TC-044..046) |
| `withdraw_fees` owner-only (non-owner revert) | SEC-CONTRACT-012 (TC-047) |
| Callback `nft_on_approve` dipalsukan → revert (#[private]) | INV-013 (TC-048) |
| Verify custom-challenge fallback + body cap + CORS + health | SEC-AUTH-002, SEC-API-001, SEC-FE-001 (TC-049..052) |

## Definisi selesai (ronde 12: fleksibel, dipandu kualitas)

Fitur dianggap **selesai** jika: (1) kode + spesifikasi docs/features sinkron, (2) test sandbox/terkait hijau (termasuk test endpoint API — happy + error path — bila fitur menambah endpoint), (3) AC fitur lolos, (4) tidak ada TODO baru tanpa task.
