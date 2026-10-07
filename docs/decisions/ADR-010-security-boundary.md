# ADR-010: Security Boundary & Authority Model

## Status
Accepted (2026-10-01 — pre-implementation research)

## Problem
Sistem terdistribusi wallet+chain+API tanpa definisi batas kepercayaan formal → risiko: API dianggap otoritas dana, metadata dipercaya, frontend jadi sumber kebenaran.

## Context
Non-custodial marketplace di NEAR; API off-chain hanya report/profil; settlement 100% kontrak. Batas kepercayaan harus eksplisit agar setiap komponen tahu apa yang ia percayai dan apa yang tidak — tanpa itu, "API yang terlihat aman" bisa diam-diam memegang jalur dana. Inventaris lengkap + attack tree + asersi testable ada di [security/trust-boundaries.md](../security/trust-boundaries.md).

## Inventory batas kepercayaan

| # | Boundary | Apa yang melintas | Penegak (enforcer) | Kontrol kunci |
|---|---|---|---|---|
| 1 | User → Wallet | intent → payload yang di-sign | user (kunci wallet) | tampilkan argumen final; wallet menampilkan receiver/deposit |
| 2 | Wallet → NEAR Chain | signed tx → validator → kontrak | validator + kontrak | nonce+blockhash protokol (FACT); kontrak = gatekeeper otorisasi |
| 3 | Frontend → Contract (view) | query read-only (`get_sale`, `nft_token`) | RPC pihak ketiga (tidak tepercaya) | re-verify harga/ownership sebelum sign — SEC-ORDER-003 |
| 4 | Browser → Frontend | metadata untrusted dari chain | browser + CSP | `<img>` only, gateway allowlist, tanpa `innerHTML` — SEC-META-001 |
| 5 | Browser → Report API | JSON write (report/profil/admin) | API (kita) | NEP-413/custom + nonce sekali pakai + session — SEC-AUTH-001/002 |
| 6 | Report API → PostgreSQL | SQL via ORM (Prisma) | kita | least privilege, constraint, migrasi versioned |
| 7 | Market ↔ NFT Contract | receipt lintas-kontrak | market (validasi) | dual verification + validasi payout — INV-002/003/013 |
| 8 | Developer → Git → CI/CD → VPS | commit → artifact → deploy | maintainer + GitHub | test gate, `cargo/npm audit`, hash NEP-330 — SEC-CICD/CONTRACT-006 |
| 9 | Admin → Panel → Privileged API | aksi moderasi (hide/verified) | API + allowlist | step-up signature + audit table — SEC-ADMIN-001/003 |
| 10 | Indexer → Chain (fase 2) | block/receipt/event (Neardata) | tim (container terpisah) | final-only + dedup `(receipt_id, event_index)` — SEC-INDEX-001/002 |

## Options
1. API-centric (backend memvalidasi & memandu transaksi) — menempatkan kepercayaan di server kita.
2. Chain-centric (kontrak = satu-satunya otoritas; API proyeksi) — kepercayaan minimal pada infrastruktur kita.

## Trade-offs
- Opsi 1: UX lebih mudah dikontrol, tapi menambah trust requirement pada server (server compromise = dana berisiko).
- Opsi 2: server compromise = dampak terbatas (profil/report); UX harus selalu re-verify via RPC.

## Security implications
Opsi 2 menghilangkan kelas serangan server-side terhadap dana sepenuhnya. Batas-batas #1/#2/#7 berada di luar jangkauan infrastruktur kita — penyerang yang menguasai API tidak otomatis mendapat otoritas kontrak.

## Scalability / Operational / Cost
Opsi 2 memindahkan beban ke RPC (rate limit provider) — murah dan sesuai skala MVP. Tidak ada signer/infra otoritas yang perlu dioperasikan (bandingkan [ADR-012](./ADR-012-order-settlement-authority.md)).

## Decision
**Chain-centric (Opsi 2)**. Kontrak = satu-satunya otoritas dana/ownership; API = proyeksi; FE selalu re-verify on-chain sebelum sign. Trust boundary formal: docs/security/trust-boundaries.md.

## Rejected
API-centric (alasan di atas).

## Consequences
- Server compromise = dampak terbatas (profil/report), bukan dana — semua nilai tetap di kontrak.
- FE wajib re-verify harga & ownership via view call sebelum sign (SEC-ORDER-003) — menambah 1 round-trip RPC per transaksi.
- API tidak bisa "menolong" user membatalkan tx yang sudah di-sign; keputusan akhir selalu di wallet user.

## Contoh serangan yang diblokir boundary

**Skenario**: penyerang menguasai VPS/Report API (boundary #5/#6) dan ingin memindahkan dana escrow buyer.

| Langkah penyerang | Kenapa gagal |
|---|---|
| Baca DB untuk menemukan offer aktif | Berhasil — tapi DB hanya **proyeksi**; angka di DB tidak otoritatif. |
| Panggil method kontrak untuk refund ke dirinya | API tidak punya kunci kontrak dan tidak pernah sign (non-custodial, ADR-007/ADR-012); boundary #2 tidak bisa dilewati dari server. |
| Manipulasi sesi admin untuk "menyetujui" refund | Escrow offer keluar hanya via `cancel_offer`/`accept_offer` yang menegakkan `predecessor_account_id` + tujuan hardcoded ke `offer.buyer_id` (INV-005/010) — bukan lewat API. |
| Ganti tampilan harga di FE | FE wajib view-call on-chain sebelum sign (SEC-ORDER-003); harga palsu tidak lolos ke payload tx. |

**Hasil**: penyerang maksimal mengubah profil/report (data non-uang). **Boundary yang memutus**: #2 (kontrak = gatekeeper), #7 (payout divalidasi ≤ harga−fee, ≤10 penerima, sisa ≤1 yocto), #5 (auth tidak memberi kunci). Varian kontrak NFT jahat (mengirim `nft_on_approve` palsu / payout bohong) diputus boundary #7 dengan kontrol yang sama (INV-013) — attack tree lengkap: trust-boundaries.md §12.

## Metrik & kriteria sukses

| Metrik | Target | Bukti |
|---|---|---|
| Jalur server-side yang bisa memindahkan dana | 0 | review arsitektur: API/indexer tanpa signer/kunci (indexer-security.md §11) |
| Settlement berasal dari tx wallet user | 100% (predecessor = user/owner) | analisis event `market_*` |
| Settlement berbasis cache/NearBlocks | 0 | review + TC-022 (SEC-ORDER-003) |
| Asersi testable per boundary (AS-1..AS-24) | 100% lolos | trust-boundaries.md §13 |
| Kunci kontrak dipakai aplikasi | 0 (hanya CLI interaktif) | key-management.md §4 |

## Open questions
Kebijakan market terhadap kontrak NFT arbitrary (G1) kini **DIPUTUSKAN: permissionless** + mitigasi (dual verification, validasi payout INV-002/003, gas cap INV-021, optimistic removal+revert, moderasi display-layer + report). Risiko residual = DoS/UX, bukan kehilangan dana — lihat security-gap-analysis.md. Sisa terbuka: G11 (wallet threat-intel) & G12 (CDN/WAF) — ⏳ open-by-design, bukan blocker MVP.
