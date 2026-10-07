# 01 — Product Requirements Document (PRD)

> Menjawab: "kita sebenarnya sedang membangun apa dan kenapa?" — level produk, bukan implementasi.
>
> **⚠️ Niat proyek (ADR-016, ronde 17):** ini proyek **pembelajaran/portofolio**, **bukan** usaha
> komersial. Tidak ada target pengguna, volume, atau pendapatan. Sukses = tesis inti terbukti jalan
> + penulisnya paham cara kerjanya. Klaim komersial di dokumen ini adalah **deskripsi teknis**,
> bukan klaim pasar. Lihat [decisions/ADR-016](./decisions/ADR-016-project-intent-and-scope-cut.md).

## 1. Product Overview

NearSea adalah marketplace NFT non-custodial terbuka di NEAR Protocol — fitur setara OpenSea: fixed-price listing, offers, private listing, bundle, launchpad koleksi berphase ala OpenSea Studio, royalti on-chain, activity feed, profil, dan notifikasi in-app. Orderbook 100% on-chain (gas NEAR murah), frontend Next.js, data via RPC + NearBlocks.

> **Cakupan yang benar-benar dibangun lebih dulu (M1 = vertical slice, ADR-016):** mint → list →
> buy + royalti + fee, di testnet, dengan satu koleksi yang di-deploy manual. Offers, bundle,
> launchpad, moderasi, dan infrastruktur = **M1+** (lanjutan eksplisit, bukan dibuang).

## 2. Problem Statement

> **⚠️ Premis diperbarui ronde 17.** Poin lama "ekosistem NEAR belum punya marketplace NFT kelas
> dunia" **secara teknis benar tapi menyesatkan** — bukan karena belum ada yang membangun, melainkan
> karena **beberapa tim berdana besar sudah membangun dan semuanya mati**. Bukti terverifikasi:
> [research/near-nft-market-2026.md](./research/near-nft-market-2026.md).
>
> Karena itu poin di bawah dibaca sebagai **daftar masalah teknis yang menarik untuk dipelajari**,
> **bukan** sebagai peluang pasar yang belum tergarap.

1. ~~Ekosistem NEAR belum punya marketplace NFT kelas dunia~~ → **DIKOREKSI**: NEAR **punya** riwayat
   marketplace NFT (Paras, MITTE, Mintbase, Few and Far), dan **semuanya sudah berhenti** — Paras pivot
   ke produk AI, MITTE (klaim ~91% pangsa) tutup April 2025. Yang tersisa: satu venue kecil.
   Masalah sebenarnya = **tidak ada permintaan**, bukan kekurangan marketplace. (FACT, riset 2026-10-07)
2. Trading NFT di chain besar mahal (gas) dan lambat — menghalangi trading aktif. *(masih relevan sebagai
   alasan teknis memilih NEAR: gas murah + finality ~1,3 dtk)*
3. Royalti kreator di banyak platform besar kini opsional — kreator kehilangan penghasilan.
   *(tetap fakta; tapi lihat catatan: memaksa royalti on-chain **kalah** di perang royalti 2022–2023 —
   ini pilihan **ideologis/teknis**, bukan keunggulan kompetitif.)*
4. Onboarding crypto masih menakutkan bagi newcomer. *(masih relevan sebagai masalah UX yang dipelajari)*
5. Proses launch koleksi (drop berphase: whitelist → public) butuh tooling khusus yang belum tersedia
   native di NEAR. *(masih relevan secara teknis — bagian dari M1+)*

## 3. Goals

- G1: MVP jual-beli (list/buy/offer/private/bundle) end-to-end di testnet, lolos sandbox tests.
- G2: Royalti kreator terbayar otomatis on-chain di 100% penjualan.
- G3: Launchpad berphase (allowlist on-chain) yang bisa dipakai kreator tanpa tulis kode.
- G4: UX yang bisa dipahami newcomer (onboarding guide + bahasa sederhana).
- G5: Infrastruktur self-hosted yang bisa dioperasikan satu orang (VPS + backup + alert Telegram).

### 3.1 Key Results (terukur) per Goal

> Setiap goal punya Key Result (KR) dengan **angka target + jendela pengukuran**. KR MVP diukur di **testnet** selama M1; KR bisnis dikunci di M4 (lihat §16.1). Nilai bertanda PROPOSED = usulan engineering, bukan janji bisnis.

| Goal | KR | Target | Jendela ukur | Cara ukur |
|---|---|---|---|---|
| **G1** | KR-G1-1: jalur emas list→buy end-to-end lolos di UI testnet | 100% TC-040 lulus | per rilis M1 | TC-040 (Playwright) |
| **G1** | KR-G1-2: offer→accept end-to-end lolos | 100% TC-041 lulus | per rilis M1 | TC-041 |
| **G1** | KR-G1-3: race 20 pembeli 1 NFT → tepat 1 pemenang | 20/20 run (0 double-pay) | per rilis M1 | TC-016, TC-042 |
| **G2** | KR-G2-1: setiap penjualan mendistribusikan royalti on-chain | 100% settlement (0 kasus fee/royalti hilang) | setiap tx M1 | event `market_sale`/`market_offer_accept` |
| **G2** | KR-G2-2: cap royalti per token ditegakkan | 0 penerimaan >10% | setiap tx M1 | TC-015 |
| **G3** | KR-G3-1: koleksi + phase berhasil di-deploy via factory tanpa tulis kode | ≥1 koleksi 2 phase di testnet | M1 | AC-COLL-1, TC-007 |
| **G3** | KR-G3-2: phase overlap ditolak | 100% kasus | M1 | TC-021, INV-029 |
| **G4** | KR-G4-1: newcomer menyelesaikan alur onboarding | halaman `/onboarding` publik + 4 langkah tampil | M3 (selaras TASK-024/P2 — koreksi ronde 16, sebelumnya M1) | AC-ONBOARD-1 |
| **G4** | KR-G4-2: pesan error dipetakan ke kode user (bukan teks mentah) | 100% panic terpetakan | setiap tx M1 | error-handling §4 |
| **G5** | KR-G5-1: deploy ulang FE/API | < 15 menit | per deploy | catatan rilis |
| **G5** | KR-G5-2: backup DB harian + alert Telegram aktif | 7 hari berturut tanpa gap | mingguan M1 | log cron + monitoring |

## 4. Non-Goals (v1 — hasil ronde 1 tanya-jawab, 2026-10-01)

- **Auction/lelang** — fase 2.
- **Pembayaran FT/multi-currency** — fase 2; v1 NEAR saja.
- **Lazy minting** — fase 3.
- **Cross-chain & fiat on-ramp** — fase 4.
- **Trait-search via custom indexer** — fase 3 / M3 (data layer MVP = RPC + NearBlocks, ronde 2; label "fase N" di dokumen ini = milestone M sama nomornya).

## 5. Target Users

1. **Trader NFT berpengalaman** — butuh data akurat, fee rendah, eksekusi cepat.
2. **Kreator / artist** — butuh launch & drop mudah, royalti terjamin, kontrol phase mint.
3. **Newcomer crypto** — butuh onboarding ramah, bahasa sederhana (EN, siap i18n).
4. **Komunitas NEAR** — wallet sudah ada, mencari venue native ekosistem.

## 6. User Personas

- **Rian — Trader** (25–40, aktif harian): browses floor/harga, offer cepat, sensitif fee & kecepatan. Kebutuhan: grid informatif, harga real-time, notifikasi offer.
- **Sari — Artist** (20–35): ingin drop koleksi 50 karya dengan whitelist collectors. Kebutuhan: launchpad phase + allowlist upload + royalti 10%.
- **Budi — Newcomer** (18–30): baru punya wallet pertama. Kebutuhan: onboarding guide, copy sederhana, konfirmasi transaksi yang menjelaskan.
- **Dewa — Admin platform** (tim inti): memproses report & verifikasi koleksi. Kebutuhan: panel admin login wallet + antrean review.

## 7. Core Use Cases

1. Browse & discover (search + grid listing, leaderboard koleksi)
2. Connect wallet (near-connect; onboarding guide untuk newcomer)
3. Create collection via factory (+ set launchpad phases)
4. Mint (sesuai phase aktif: whitelist/public; storage dibayar pemicu mint)
5. List for sale (approval model — NFT tetap di wallet) / cancel / update price
6. Buy now (settlement + royalti + fee 2% on-chain)
7. Make / accept offer (semua token, escrow Ⓝ, expire 7 hari default)
8. Private listing & bundle (satu harga per bundle)
9. Kelola profil & lihat activity
10. Report konten → admin review (blocklist tampilan, badge verified)

## 8. Features

**Ronde-1 keputusan (2026-10-01)** — MVP memuat: fixed-price list/buy, Offers, private listing, bundle, koleksi open + badge verified, pembayaran NEAR, fee 2%. Auction & multi-currency = fase 2.

> **Ringkas saja.** Behavior detail tiap fitur dimiliki **[02-product-requirements.md](./02-product-requirements.md)** (spec per kemampuan) dan **[features/](./features/marketplace.md)** (spec per fitur, termasuk skema method kontrak). Bagian ini hanya daftar scope — jangan menyalin behavior di sini agar tidak terjadi drift (aturan single-source-of-truth, [DOCUMENTATION-MAP.md](./DOCUMENTATION-MAP.md)).

| Fitur MVP | Spec detail |
|---|---|
| Listing (2-tx, approval non-custodial) | [features/marketplace.md](./features/marketplace.md) |
| Buy now (settlement + royalti + fee 2%) | [features/marketplace.md](./features/marketplace.md), [features/payments.md](./features/payments.md) |
| Make / accept offer (escrow) | [features/marketplace.md](./features/marketplace.md) |
| Private listing | [features/marketplace.md](./features/marketplace.md) |
| Bundle (satu harga, maks 10 token) | [features/marketplace.md](./features/marketplace.md) |
| Create collection + launchpad berphase | [02-product-requirements.md](./02-product-requirements.md) |
| Profil custom + activity | [features/users.md](./features/users.md) |
| Notifikasi in-app | [features/notifications.md](./features/notifications.md) |
| Report + admin panel + badge verified | [02-product-requirements.md](./02-product-requirements.md), [security/permissions.md](./security/permissions.md) |

> Status stale (listing auto-invalid saat ownership berubah) adalah perilaku lintas-fitur — spec: [features/marketplace.md](./features/marketplace.md) + [security/order-protocol-security.md](./security/order-protocol-security.md).

## 9. Functional Requirements

Lengkap di [02-product-requirements.md](./02-product-requirements.md). Ringkas: wallet connect; create collection + launchpad phases; mint per phase; list/buy/cancel/update-price (approval model); make/accept offer; private listing; bundle; search & filter (nama + harga); profil + activity; notifikasi in-app; report + admin panel.

### 9.1 Requirement ID & Prioritas

> ID `FR-<AREA>-<n>` stabil — dipakai di traceability (§18), test case, dan PR. Prioritas: **P0** = wajib untuk M1 (tanpa ini MVP tidak "selesai"); **P1** = wajib sebelum mainnet gate; **P2** = nice-to-have M1, boleh menyusul. Behavior detail dimiliki [02-product-requirements.md](./02-product-requirements.md).

| ID | Requirement | Area | Prioritas | Milestone | Test |
|---|---|---|---|---|---|
| FR-WALLET-1 | Connect/disconnect wallet via near-connect; reconnect otomatis | Wallet | P0 | M1 | TC-040, AC-WALLET-1..4 |
| FR-WALLET-2 | Deteksi network mismatch → banner + disable tx | Wallet | P0 | M1 | AC-NETWORK-1/2 |
| FR-COLL-1 | Create collection via factory (metadata, supply, royalti ≤10%) | Launchpad | P0 | M1 | TC-007, AC-COLL-1/2 |
| FR-COLL-2 | Definisikan phase mint (harga/alokasi/allowlist/waktu), non-overlap | Launchpad | P0 | M1 | TC-021, AC-COLL-3 |
| FR-MINT-1 | Mint sesuai phase aktif (allowlist, exact deposit, max/wallet) | Launchpad | P0 | M1 | TC-007, AC-MINT-1..5 |
| FR-LIST-1 | List 2-tx (approve → list) + dual verification | Listing | P0 | M1 | TC-002, AC-LIST-1 |
| FR-LIST-2 | Cancel / update price (owner-only, 1 yocto) | Listing | P0 | M1 | AC-LIST-5 |
| FR-LIST-3 | Stale listing auto-invalid + disembunyikan | Listing | P0 | M1 | TC-006, AC-STALE-1 |
| FR-BUY-1 | Buy now (deposit = harga) + settlement royalti/fee | Buy | P0 | M1 | TC-002, AC-BUY-1 |
| FR-BUY-2 | Refund 100% otomatis saat settlement gagal | Buy | P0 | M1 | TC-003, AC-BUY-2 |
| FR-BUY-3 | Race: 1 pemenang, sisanya refund (tanpa double-pay) | Buy | P0 | M1 | TC-016/017/042, AC-BUY-6 |
| FR-OFFER-1 | Make/accept/cancel offer (escrow, 7 hari, 1/buyer/token, min 0.01 Ⓝ) | Offer | P0 | M1 | TC-004/005/008, AC-OFFER-1..8 |
| FR-OFFER-2 | Offer auto-cancel + refund saat offer lain di-accept | Offer | P0 | M1 | TC-019, AC-OFFER-4 |
| FR-PRIV-1 | Private listing (`allowed_buyer`) | Private | P0 | M1 | TC-011, AC-BUNDLE-1 |
| FR-BUNDLE-1 | Bundle satu harga, maks 10 token, pre-validasi all-or-nothing | Bundle | P0 | M1 | TC-009/010/014, AC-BUNDLE-2..4 |
| FR-DISC-1 | Browse/search/filter/sort client-side atas view (MVP) | Discovery | P0 | M1 | AC-PERF-5 (fase 2 endpoint) |
| FR-PROFILE-1 | Profil publik (Owned/Created/Offers/Activity) | Profile | P0 | M1 | TC-031, AC-PROFILE-1 |
| FR-PROFILE-2 | Profil custom (alias/bio/avatar) via signature | Profile | P0 | M1 | TC-032, AC-PROFILE-1..3 |
| FR-NOTIF-1 | Notifikasi in-app (polling 30–60 dtk, idempoten) | Notif | P0 | M1 | TC-023/024, AC-NOTIF-1..4 |
| FR-REPORT-1 | Report konten (rate-limited, idempoten per hari) | Report | P0 | M1 | TC-033, AC-REPORT-1..6 |
| FR-ADMIN-1 | Panel admin (allowlist + scope + step-up) | Admin | P0 | M1 | TC-025/026/034..039, AC-ADMIN-1..6 |
| FR-BADGE-1 | Verified badge (admin-controlled) | Badge | P0 | M1 | TC-036, AC-BADGE-1..3 |
| FR-LEADER-1 | Leaderboard koleksi | Leaderboard | P2 | M3 (selaras TASK-025 — koreksi ronde 16, sebelumnya M2) | AC-LEADER-1..3 |
| FR-PAUSE-1 | Pausable (blokir mutasi, izinkan refund/cancel/withdraw) | Safety | P0 | M1 | TC-012, AC-PAUSE-1/2 |

## 10. Non-Functional Requirements

- Non-custodial untuk NFT: kontrak TIDAK pernah memegang NFT user (approval model). Escrow Ⓝ **sementara hanya untuk offer aktif** — selalu user-recoverable via cancel/expire (koreksi ronde 13, gap-analysis §1).
- Settlement final di chain — frontend hanya re-verifikasi, bukan sumber kebenaran.
- UI responsive setara penuh (desktop = mobile, ronde 8); English siap i18n.
- Read data: RPC view call < 1 detik; polling notifikasi 30–60 dtk saat tab aktif.
- Infra 1-orang-able: deploy ulang < 15 menit, backup DB harian otomatis, alert Telegram.

### 10.1 Lokalisasi (Detail)

> Bahasa UI MVP = **English**; arsitektur **siap i18n** (ronde 2). Tidak ada string hardcode di komponen (AGENTS.md).

| Aspek | Keputusan | Catatan |
|---|---|---|
| Locale MVP | `en` saja | Struktur `i18n/en/*.json` ([frontend-architecture.md](./architecture/frontend-architecture.md) §9) |
| Locale tambahan | ⏳ open-by-design | Menambah locale = tambah folder `i18n/<locale>/`; tanpa ubah komponen |
| Format angka | Desimal NEAR via `lib/format`; pemisah ribuan mengikuti locale | Dilarang float pada yoctoNEAR |
| Format tanggal/waktu | ISO-8601 UTC di API/DB; tampilan mengikuti locale | `Intl.DateTimeFormat` |
| Mata uang | Ⓝ (NEAR) — simbol tetap lintas locale | v1 NEAR-only |
| String error | Lewat kunci i18n dari registry kode ([error-handling.md](./development/error-handling.md) §3) | mis. `errors.CONFLICT_SOLD` |
| Placeholder | `{count}`, `{price}`, `{collection}` — ICU-style | Bukan konkatenasi string |
| Pluralisasi | Aturan ICU plural | mis. `{count, plural, one {# item} other {# items}}` |
| Ekspansi teks | Desain harus toleran +30–40% panjang string | Lihat [04-ux-ui-spec.md](./04-ux-ui-spec.md) § i18n |
| RTL | ⏳ open-by-design | Belum ada locale RTL; struktur komponen tidak diasumsikan LTR keras |
| Bahasa konten user | Bebas (tidak diterjemahkan) | Alias/bio/nama koleksi = data, bukan UI copy |

## 11. User Stories

- Sebagai **trader**, saya bisa melihat listing termurah per koleksi agar cepat ambil keputusan.
- Sebagai **trader**, saya bisa membuat offer pada token yang belum di-list agar dapat harga di bawah list.
- Sebagai **kreator**, saya bisa membuat koleksi dengan phase whitelist (allowlist upload) dan phase public dengan harga berbeda.
- Sebagai **kreator**, saya menerima royalti otomatis setiap kali karya saya terjual ulang.
- Sebagai **newcomer**, saya bisa mengikuti guide untuk membuat wallet dan transaksi pertama.
- Sebagai **buyer**, saya di-refund penuh otomatis jika settlement gagal.
- Sebagai **user**, saya bisa melaporkan koleksi penipuan dan melihatnya hilang dari discovery setelah direview admin.

## 12. Acceptance Criteria

AC lengkap per fitur: [testing/acceptance-criteria.md](./testing/acceptance-criteria.md). Prinsip inti:
- Setiap penjualan (buy now / accept offer / bundle) → royalti + fee 2% terdistribusi on-chain.
- Setiap kegagalan settlement → refund 100% otomatis.
- NFT selalu tetap di wallet seller selama listing (approval model).

## 13. Business Rules (ronde 1–13 + pasca-audit)

- Fee platform **2%** per penjualan, dipotong on-chain di payout, ke treasury. *(ronde 1)*
- Batas royalti kreator **maksimum 10%** — divalidasi kontrak saat mint/approve. *(ronde 4)*
- Durasi default offer **7 hari**; buyer dapat memilih durasi custom. *(ronde 4)*
- Maksimum **1 offer aktif per buyer per token** (cancel + buat ulang diperbolehkan). *(ronde 4)*
- Harga minimum listing & offer: **0.01 Ⓝ**. *(ronde 4)*
- Seller **tidak bisa** membeli listing miliknya sendiri.
- Royalti wajib dibayar tiap penjualan (payout on-chain, maks 10 penerima; pada bundle = payout per token dijumlahkan).
- Satu NFT tidak bisa di-list dua kali selama listing aktif.
- Counter-offer **tidak** di MVP — nego dilakukan lewat private listing. *(ronde 6)*
- Bundle: **satu harga untuk seluruh bundle** (gaya OpenSea), bukan penjumlahan otomatis per item. *(ronde 6)*
- Listing **permanen** sampai dibatalkan seller / terjual — tanpa expiry. *(ronde 9)*
- Offer boleh pada **token apa pun** (di-list maupun tidak). *(ronde 9)*
- Fee 2% berlaku **konsisten di semua jalur penjualan** (buy now & accept offer). *(ronde 9)*
- **Model listing: approval, non-custodial ala OpenSea/Seaport** (hasil riset ronde 9, ADR-007) — NFT tetap di wallet seller; token yang dipindah di luar market → listing **auto-stale + sembunyi** saat terdeteksi.
- **Launchpad**: creator bebas mendefinisikan phase mint (jumlah berapa pun; tiap phase punya **waktu mulai–selesai**, harga, alokasi/max, allowlist opsional); allowlist disimpan **on-chain set**; **phase berjalan berurutan, tidak overlap** (satu fase aktif pada satu waktu). *(ronde 10 + pasca-audit)*
- Koleksi baru **langsung tampil** di discovery — kualitas dijaga report system. *(ronde 10)*
- Max supply per koleksi **bebas, tanpa cap platform**. *(ronde 10)*
- Biaya storage mint dibayar **pemicu mint** (siapa memicu, dia bayar). *(ronde 10)*
- **Profil custom** (alias/bio/avatar) disimpan di DB kecil (VPS), diverifikasi signature wallet. *(ronde 13)*
- Kontrak market & factory memiliki **Pausable** (kill switch: MVP owner-only; mainnet guardian `pause_callers` terpisah dari owner-DAO — ADR-013); saat pause, mutasi baru berhenti namun penarikan escrow/cancel/refund tetap terbuka. *(ronde 13 + pasca-audit)*
- Private listing **tanpa durasi maksimum** — sama seperti listing biasa. *(ronde 13)*

## 14. Constraints

> Draft dari riset: gas ≤ 300 Tgas/call, payout ≤ 10 akun, media via IPFS (4 MB/call, 1 Ⓝ/100 KB).

**Kebijakan media MVP (saat provider IPFS belum ditetapkan):**
- On-chain **hanya** menyimpan URL + hash (`media`, `media_hash`) — tidak pernah gambar (batas 4 MB/call, 1 Ⓝ/100 KB).
- Provider IPFS (Pinata / NFT.Storage / lainnya) = **open-by-design**, diputuskan saat fitur mint dibangun (ronde 5). Selama belum ditetapkan, dev memakai **gateway publik** untuk aset contoh; mint produksi menunggu provider.
- FE hanya **membaca** media via gateway allowlist (`<img>` saja) — tidak ada upload/pinning server-side di MVP (ADR-014).
- Implikasi: fitur mint launchpad (TASK-020/021) **tidak memblokir** pekerjaan kontrak, tapi rilis mint publik menunggu keputusan provider.

### 14.1 Tabel Constraint Terkuantifikasi

> Angka **FACT** = batas protokol/riset; **PROPOSED** = target engineering diukur saat implementasi. Sumber lintas-dokumen: [system-architecture.md](./architecture/system-architecture.md) § Anggaran gas, [features/payments.md](./features/payments.md).

| Constraint | Nilai | Jenis | Sumber / catatan |
|---|---|---|---|
| Gas per call (keras) | **≤ 300 Tgas** | FACT NEAR | Batas protokol; pelanggaran → tx gagal |
| Gas `buy` (worst case) | **< 150 Tgas** | PROPOSED | Komponen FACT: `nft_transfer_payout` 15 + `resolve_purchase` 115 |
| Gas `buy_bundle` (10 token) | **wajib diukur** (risiko utama) | PROPOSED | 10× `nft_transfer_payout` ≈ 150 + resolve; batas statis 10 (INV-021) |
| Gas `list_nft_for_sale` | **≈10–20 Tgas total** (tulis ~5 + 2 view call ~3–5 masing-masing) | PROPOSED | Dual verification; total selaras order-protocol-security §11 |
| Payout penerima | **≤ 10 akun** (setelah merge) | FACT/keputusan | INV-003/021; batas gas sehat |
| Item bundle | **≤ 10 token** | Keputusan | INV-021 (batas berbeda dari 10 penerima) |
| Royalti per token | **≤ 10%** (cap inklusif) | Keputusan | INV-027; agregat bundle tanpa cap |
| Sisa pembulatan payout | **≤ 1 yocto** | Keputusan | INV-002; di atas itu → tolak + refund |
| Harga min listing & offer | **0.01 Ⓝ** = `10000000000000000000000` yocto | Keputusan | INV-030 |
| Fee platform | **2%** (`fee_bps=200`), cap `MAX_FEE_BPS=500` | Keputusan | ADR-005 |
| Durasi offer default | **7 hari** (custom diperbolehkan) | Keputusan | INV-024 |
| Offer aktif per buyer/token | **1** | Keputusan | INV-024 |
| Ukuran media on-chain | **tidak ada** — hanya URL + hash | Keputusan | Batas 4 MB/call, 1 Ⓝ/100 KB (media di IPFS) |
| Ukuran payload event | string yoctoNEAR; field aditif tanpa naik versi | Keputusan | api/webhooks.md |
| Representasi nilai JSON | **string** yoctoNEAR (u128) — dilarang `number` | Keputusan | AGENTS.md |
| Timeout read RPC | view call < 1 detik (target UX) | PROPOSED | §10 NFR |
| Polling notifikasi | 30–60 detik saat tab aktif | Keputusan | features/notifications.md |
| Payload size API | body size cap (rate limit + cap) | PROPOSED | SEC-API-001 |

## 15. Dependencies

> Draft: wallet provider (near-connect), RPC provider, IPFS pinning, indexer infra, faucet testnet.

### 15.1 Dependency Version Pins

> **SSOT pin versi = [architecture/tech-stack.md](./architecture/tech-stack.md) § Version pins.** Tabel ini **ringkasan** — jangan menyatakan nilai berbeda dari SSOT; nilai yang belum dipakai task-nya ditandai "belum ditambah".

| Dependency | Kategori | Target/rujukan | Pin final (TASK-001) |
|---|---|---|---|
| Node.js | Runtime FE/API | LTS terbaru saat scaffold | **24 LTS** |
| Next.js | Framework FE | App Router | **16.4.0** |
| TypeScript | Bahasa FE | strict mode, tanpa `any` tanpa alasan | **5.9.x** (`^5`) |
| Tailwind CSS | Styling | v3/v4 sesuai scaffold | **4.3.x** (`^4`) |
| Vitest + jsdom | Test FE | — | **3.x** + **30.x** |
| TanStack Query | Server state | v5 | belum ditambah (TASK-007/008) |
| Zustand | Client state | v4/v5 sesuai scaffold | belum ditambah (TASK-007/008) |
| near-connect | Wallet adapter | versi terbaru (daftar wallet = daftar resmi) | belum ditambah (TASK-007) |
| near-api-js / @near-js/* | RPC client | versi terbaru saat scaffold | belum ditambah (TASK-007) |
| Prisma | ORM/migrate | versi terbaru saat scaffold | belum ditambah (TASK-018) |
| PostgreSQL | DB | 15+ (final saat provision) | belum di-pin (TASK-028) |
| Rust edition | Bahasa kontrak | 2021 (minimum) | **2021** |
| Rust toolchain | Toolchain | rustc 1.77.1 (rujukan riset) | **1.93.1** (naik — lihat tech-stack §Version pins) |
| near-sdk | SDK kontrak | 4.x (rujukan riset) | **5.29.1** (naik) |
| near-sdk-contract-tools | Derive NEP | versi terbaru kompatibel | **4.0.0** |
| cargo-near | Build kontrak | 0.6.1 (rujukan riset) | **0.22.0** (di-pin sha256 di CI) |
| near-cli-rs | CLI NEAR | 0.17.0 (rujukan riset) | belum di-pin (deploy testnet pertama) |

- **Aturan**: lockfile wajib di-commit (SEC-CICD-001); audit kerentanan di CI; pin runtime/toolchain, lib aplikasi boleh range dengan lockfile (tech-stack.md § Kebijakan dependensi).
- Dua kenaikan dari rujukan riset (Rust, near-sdk) punya alasan teknis tercatat — **bukan** nilai baru yang dikarang: dependency tree `near-sdk` 5.x butuh Cargo dengan dukungan `edition2024`.

## 16. Success Metrics

- **Sekarang → mainnet (ronde 7)**: sukses = rilis mulus + transaksi end-to-end nyata di testnet.
- **Mendekati/pasca mainnet**: tetapkan metric bisnis (kandidat: volume Ⓝ/minggu, koleksi aktif, wallet unik, retensi) — ⏳ M4.

### 16.1 Success Metrics Terkuantifikasi (Target + Jendela Ukur)

> **MVP (M1) = metric fungsional** (bukan bisnis) — diukur di testnet. **Bisnis (pasca-mainnet)** = kandidat yang **wajib dikunci di M4** (OQ-005); angka di bawah **PROPOSED** dan belum menjadi komitmen.

| # | Metric | Target | Jendela ukur | Fase | Sumber data |
|---|---|---|---|---|---|
| SM-1 | Jalur emas list→buy E2E di UI | 100% pass | per rilis | M1 | TC-040 |
| SM-2 | Jalur emas offer→accept E2E di UI | 100% pass | per rilis | M1 | TC-041 |
| SM-3 | Race 1 NFT / 20 pembeli → 1 pemenang | 20/20 run, 0 double-pay | per rilis | M1 | TC-016/042 |
| SM-4 | Refund otomatis saat settlement gagal | 100% (0 dana hangus) | per rilis | M1 | TC-003/022 |
| SM-5 | Gas `buy` worst case | < 150 Tgas | per rilis | M1 | sandbox |
| SM-6 | Gas `buy_bundle` 10 token | < 300 Tgas (terukur) | sebelum M1 gate | M1 | sandbox |
| SM-7 | Deploy ulang FE/API | < 15 menit | per deploy | M1 | catatan rilis |
| SM-8 | Backup DB harian sukses | 7 hari tanpa gap | mingguan | M1 | log cron |
| SM-9 | Uptime API (MVP) | ≥ 99% (PROPOSED) | bulanan | M1 | monitoring |
| SM-10 | Volume Ⓝ/minggu | ⏳ ditetapkan M4 | mingguan | pasca-mainnet | indexer fase 2 |
| SM-11 | Koleksi aktif / wallet unik | ⏳ ditetapkan M4 | bulanan | pasca-mainnet | indexer fase 2 |
| SM-12 | Retensi trader (D30) | ⏳ ditetapkan M4 | bulanan | pasca-mainnet | analytics |

## 17. Future Scope

> Draft (RESEARCH.md bag. 11): cross-chain payment (Chain Signatures), fiat on-ramp, lazy minting (M3), notifikasi real-time indexer. Bundle & private listing = fitur MVP (§8), bukan future scope.

## 18. Traceability Matrix (Goal → Requirement → Test)

> Menutup rantai goal → requirement → acceptance/test. AC ID dimiliki [testing/acceptance-criteria.md](./testing/acceptance-criteria.md); TC dimiliki [testing/test-cases.md](./testing/test-cases.md); INV dimiliki [security/smart-contract-invariants.md](./security/smart-contract-invariants.md).

| Goal | Requirement (FR) | AC | TC | INV terkait |
|---|---|---|---|---|
| G1 | FR-LIST-1, FR-BUY-1 | AC-LIST-1, AC-BUY-1 | TC-002, TC-040 | INV-001, INV-011 |
| G1 | FR-BUY-3 | AC-BUY-6 | TC-016, TC-017, TC-042 | INV-007/008/009 |
| G1 | FR-OFFER-1, FR-OFFER-2 | AC-OFFER-1..8 | TC-004, TC-005, TC-008, TC-019, TC-041 | INV-005, INV-010, INV-024 |
| G1 | FR-PRIV-1, FR-BUNDLE-1 | AC-BUNDLE-1..5 | TC-009, TC-010, TC-011, TC-014 | INV-021, INV-025..028 |
| G2 | FR-BUY-1 | AC-BUY-1 | TC-002, TC-005 | INV-002, INV-003 |
| G2 | FR-BUY-1 (cap royalti) | AC-BUNDLE-2 | TC-015 | INV-027 |
| G3 | FR-COLL-1, FR-COLL-2 | AC-COLL-1..4 | TC-007, TC-021 | INV-029 |
| G3 | FR-MINT-1 | AC-MINT-1..5 | TC-007 | INV-017/018/019 |
| G4 | FR-WALLET-1, FR-WALLET-2 | AC-WALLET-1..4, AC-NETWORK-1/2 | TC-040 | — |
| G4 | (onboarding) | AC-ONBOARD-1..3 | — | — |
| G5 | FR-PAUSE-1 | AC-PAUSE-1/2 | TC-012 | INV-022 |
| G5 | (ops/backup) | — | — | — |
| — | FR-PROFILE-1/2 | AC-PROFILE-1..3 | TC-031, TC-032 | — |
| — | FR-NOTIF-1 | AC-NOTIF-1..4 | TC-023, TC-024 | — |
| — | FR-REPORT-1 | AC-REPORT-1..6 | TC-033 | — |
| — | FR-ADMIN-1, FR-BADGE-1 | AC-ADMIN-1..6, AC-BADGE-1..3 | TC-025/026/034..039 | — |
| — | FR-LEADER-1 | AC-LEADER-1..3 | — | — |

- Baris tanpa AC/TC = requirement fungsional/ops yang diverifikasi lewat review/drill (bukan test otomatis) — lihat [security/security-requirements.md](./security/security-requirements.md) kolom Verifikasi.

## 19. Assumptions & Risks Register

> **Assumption** = hal yang dianggap benar tanpa bukti; bila salah → rencana berubah. **Risk** = kemungkinan buruk + mitigasi. Tidak ada keputusan produk baru di sini.

### 19.1 Assumptions

| ID | Asumsi | Dampak bila salah | Validasi | Status |
|---|---|---|---|---|
| A-1 | near-connect mendukung `signMessage` (NEP-413) di wallet utama | Auth API gagal → fallback custom | Uji wallet saat scaffold | ⏳ |
| A-2 | Gas NEAR tetap murah (orderbook on-chain viable) | Biaya tx naik → UX buruk | Pantau saat testnet | ⏳ |
| A-3 | NearBlocks API cukup untuk riwayat/notifikasi MVP | Notifikasi tak akurat → butuh indexer lebih awal | Uji rate limit + cakupan | ⏳ |
| A-4 | `nft_transfer_payout` (NEP-199) tersedia di kontrak kita + koleksi pihak ketiga | Royalti gagal → tolak + refund | Uji sandbox 2 kontrak | ⏳ |
| A-5 | Satu orang bisa operasikan VPS (deploy/backup/alert) | Beban ops → butuh otomasi tambahan | Drill M1 | ⏳ |
| A-6 | Batas 10 token bundle cukup untuk kebutuhan user | Bundle besar ditolak → keluhan | Data pasca-rilis | ⏳ |
| A-7 | Testnet faucet cukup untuk uji end-to-end | Blokir pengujian | Cek ketersediaan | ⏳ |

### 19.2 Risks

| ID | Risiko | Likelihood | Impact | Mitigasi | Owner |
|---|---|---|---|---|---|
| R-1 | Bug settlement → dana buyer hangus | Rendah | Tinggi | Atomic resolve + refund 100% (INV-001), TC-003/022 | contract |
| R-2 | `buy_bundle` 10 token mendekati 300 Tgas | Sedang | Tinggi | Batas statis 10 (INV-021); wajib diukur di sandbox (SM-6) | contract |
| R-3 | Listing stale disalahgunakan | Sedang | Sedang | Dual verification + re-verify FE (SEC-ORDER-003/004) | contract/FE |
| R-4 | Race dua pembeli | Sedang | Sedang | Optimistic removal + unique key (SEC-ORDER-006) | contract |
| R-5 | Panic kontrak tampil mentah ke user | Sedang | Sedang | Pemetaan panic → kode user (error-handling §4) | FE |
| R-6 | Provider IPFS belum diputuskan | Tinggi | Sedang | Gate mint produksi (OQ-002); media contoh via gateway publik | owner |
| R-7 | Owner key bocor (testnet interim) | Rendah | Tinggi | SEC-KEY-001; DAO 2-of-3 + timelock di mainnet (ADR-013) | security |
| R-8 | Rate limit NearBlocks → notifikasi berhenti | Sedang | Rendah | Backoff + UI tetap hidup (AC-NOTIF-3) | FE |
| R-9 | Reproducible build gagal (NEP-330) | Sedang | Tinggi | Verifikasi hash CI + SourceScan sebelum mainnet | infra |
| R-10 | Kontradiksi dokumen saat fitur berkembang | Sedang | Sedang | DOCUMENTATION-MAP crosscheck + SSOT | semua |

## 20. Contoh Terhitung End-to-End — Penjualan 10 Ⓝ

> Menyatukan jalur list → buy → payout dalam satu contoh angka. **Aturan bisnis terkunci**: fee 2%, royalti ≤10% per token, payout ≤10 penerima, sisa ≤1 yocto. Semua nilai = **string yoctoNEAR**; `1 Ⓝ = 1000000000000000000000000`.

**Skenario**: Alice (seller) menjual token `C2.101` ke Bob (buyer) seharga `10 Ⓝ`. Koleksi `C2` punya royalti 5% ke `creatorX` + 2.5% ke `creatorY`. Fee platform 2%.

| Langkah | Aksi | Nilai |
|---|---|---|
| 1 | Alice `nft_approve(market, msg=10 Ⓝ)` | deposit 1 yocto + storage approval |
| 2 | Alice `list_nft_for_sale{contract:C2, token:101, price:10 Ⓝ}` | storage deposit (NEP-145) + dual verification → `market_list` |
| 3 | Bob `buy{contract:C2, token:101}` attach Ⓝ = `10 Ⓝ` | `10000000000000000000000000` yocto |
| 4 | Market hapus Sale (optimistic) → `nft_transfer_payout` | 15 Tgas |
| 5 | NFT contract kembalikan Payout (royalti, UNTRUSTED) | 5% + 2.5% |
| 6 | `resolve_purchase` validasi + distribusi | 115 Tgas |

**Perhitungan payout** (fee dipotong lebih dulu, sisa pembulatan → seller):

| Komponen | Rate | Perhitungan | Nilai (yocto) |
|---|---|---|---|
| Harga | — | — | `10000000000000000000000000` |
| Fee platform | 2% (200 bps) | `floor(10 Ⓝ × 200 / 10000)` | `200000000000000000000000` |
| Plafon payout | — | `harga − fee` | `9800000000000000000000000` |
| Royalti `creatorX` | 5% (500 bps) | `floor(10 Ⓝ × 500 / 10000)` | `500000000000000000000000` |
| Royalti `creatorY` | 2.5% (250 bps) | `floor(10 Ⓝ × 250 / 10000)` | `250000000000000000000000` |
| Seller (Alice, residual) | — | `plafon − Σroyalti` | `9050000000000000000000000` |
| **Sisa pembulatan** | — | `plafon − Σpayout` | **0** (≤1 yocto ✓) |

- Verifikasi: `fee + royaltiX + royaltiY + seller = 0.2 + 0.5 + 0.25 + 9.05 = 10 Ⓝ` ✓.
- Penerima = 4 (Alice + creatorX + creatorY + treasury ≤ 10 ✓).
- Semua nilai dibayar on-chain; frontend hanya menampilkan breakdown dari view call (re-verify SEC-ORDER-003).
- Bila Bob ternyata kalah race (token sudah terjual lebih dulu) → `resolve_purchase` refund penuh `10 Ⓝ`; listing tetap ada bila kegagalan bukan stale (AC-BUY-6, TC-042).

