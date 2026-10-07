# ADR-012: Order & Settlement Authority — On-chain Orderbook, Tanpa Off-chain Order Signature

## Status
Accepted (2026-10-01)

## Problem
Marketplace NFT secara umum punya dua model order: (a) off-chain signed orders (Seaport/EIP-712: order = pesan tertanda tangan, settlement saat fill), atau (b) on-chain orders (order = state kontrak). Model menentukan seluruh arsitektur signature & replay protection.

## Context
NEAR: gas sangat murah (<0.01Ⓝ per tx — FACT), storage staking murah, finality cepat. Kita sudah memutuskan orderbook on-chain (ADR-003) dan model listing approval non-custodial (ADR-002, framing disempurnakan ADR-007).

## Options
1. Off-chain signed orders ala Seaport (order gasless, settlement on-chain saat fill).
2. On-chain orderbook penuh (setiap list/offer = tx).

## Trade-offs
- Opsi 1: hemat tx user, tapi butuh infrastruktur signature verification on-chain, orderDB otoritatif off-chain, cancellation on/off-chain, dan membuka kelas serangan signature (replay, domain binding, malleability).
- Opsi 2: setiap aksi = tx (murah di NEAR), replay protection = protokol (nonce+blockhash — FACT), state selalu konsisten, tidak ada orderDB yang bisa menyelisih dengan chain.

## Security implications
Opsi 2 mengeliminasi seluruh kelas signature-order attacks (replay, cross-contract replay, stale order execution via signature) karena tidak ada signature order — SEC-SIGN-001.

## Scalability / Operational / Cost
Opsi 2: beban pindah ke chain (murah); tanpa kebutuhan signer infra.

## Perbandingan biaya (gas NEAR)

> Angka Tgas dari [order-protocol-security.md](../security/order-protocol-security.md) §11 (komponen FACT, total PROPOSED diukur di sandbox). Anchor: **<0.01 Ⓝ per tx = FACT**; NEAR tidak punya priority fee.

| Item | Opsi 2 — on-chain orderbook | Opsi 1 — off-chain signed order |
|---|---|---|
| List/offer | 1 tx: `list_nft_for_sale` ~10–20 Tgas; `make_offer` ~5 Tgas | sign off-chain: 0 gas user |
| Fill (buy) | `buy` <150 Tgas | 1 tx settlement + **verifikasi signature on-chain** (lebih mahal dari `buy` murni) |
| Cancel | 1 tx ~5–10 Tgas | off-chain (0 gas) atau on-chain (2 jalur state) |
| Biaya per aksi user | **<0.01 Ⓝ** (FACT) | ~0 untuk user |
| Biaya server | 0 (tidak ada signer/orderDB otoritatif) | orderDB otoritatif + verifier + (opsional) relayer |
| Risiko state divergen | tidak ada (chain = state order) | orderDB vs chain bisa menyelisih |

- **Kesimpulan biaya**: keuntungan gasless Opsi 1 hanya menghapus **satu tx kecil (~10–20 Tgas)** yang di NEAR sudah **<0.01 Ⓝ**; sebagai gantinya muncul biaya infrastruktur (orderDB otoritatif + verifikasi signature) dan kelas serangan signature. Untuk NEAR, keuntungan gasless **tidak sepadan**.
- `buy_bundle` ≤10 token tetap jalur termahal (<300 Tgas batas keras — INV-021); Opsi 1 justru menambah biaya verifikasi signature di atasnya.

## Attack list & mitigasi

> Daftar serangan yang **dieliminasi** Opsi 2 (tidak ada signature order) dan yang **tetap ada** (serangan order umum) dengan kontrolnya. Detail tree: order-protocol-security.md §12.

| # | Serangan | Berlaku di Opsi 1? | Berlaku di Opsi 2? | Mitigasi (Opsi 2) |
|---|---|---|---|---|
| A1 | **Replay order signature** (order sah dipakai 2×) | ya | **tidak** (tidak ada signature order) | order = state; sekali dihapus saat settle (optimistic removal) |
| A2 | **Cross-contract replay** (signature order dipakai di kontrak/market lain) | ya | **tidak** | tidak ada signature untuk di-replay; state per kontrak |
| A3 | **Stale order execution** (order lama dieksekusi setelah harga/ownership berubah) | ya | tidak (cek ulang on-chain) | dual verification ownership + approval tiap settle (INV-016) |
| A4 | **Signature malleability / domain binding salah** | ya | **tidak** | tidak ada domain-binding order; replay = nonce+blockhash protokol (FACT) |
| B1 | Payout > harga−fee (kontrak NFT jahat) | ya | ya | validasi payout INV-002 → tolak + refund (SEC-ORDER-001) |
| B2 | Royalti per token > 10% | ya | ya | INV-027 → tolak + refund (TC-015) |
| B3 | Receiver duplikat / >10 unik | ya | ya | merge per receiver; >10 ditolak saat `create_bundle` (INV-003/021) |
| B4 | Fee dinaikkan > cap | ya | ya | `MAX_FEE_BPS=500` (INV-004) → revert |
| B5 | Forge `nft_on_approve` | ya | ya | validasi payload NEP-178 + predecessor = NFT sah (INV-013) |
| B6 | Self-buy / bypass private listing | ya | ya | INV-023 / INV-026 |
| C1 | Double-buy listing sama | ya | ya | optimistic removal + unique key (INV-007/008, SEC-ORDER-005/006) |
| C2 | Double-accept offer | ya | ya | entry dihapus atomik (INV-009) |
| C3 | Offer ganda buyer/token | ya | ya | unique state, maks 1 offer aktif/buyer/token (INV-024) |
| D1 | Refund ke alamat arbitrer | ya | ya | tujuan = `offer.buyer_id` hardcoded (INV-005/014) |
| D2 | Storage-DoS refund massal | ya | ya | storage tidak auto-refund; tarik via `storage_withdraw` |

- Baris A1–A4 adalah **alasan inti Opsi 2**: seluruh kelas serangan signature order hilang karena order bukan pesan tertanda tangan. Baris B–D adalah kontrol order umum yang tetap wajib (dan tetap diuji) di kedua model.

## Sequence diagram (ASCII) — lifecycle order

```text
LISTING (2 tx, ADR-002)
 Seller        NFT Contract        Market Contract
   │  nft_approve(market, id) ──►│
   │                             │
   │  list_nft_for_sale(msg) ────┼──────────────────►│
   │                             │  view nft_token ─►│  (dual verify)
   │                             │  view nft_is_approved ─►│
   │                             │                   │  simpan Sale (state on-chain)
   │◄────────────────────────────┴───────────────────│  event market_list

BUY (1 tx)
 Buyer         Market Contract        NFT Contract
   │  buy(token) + deposit = harga ─►│
   │                                 │  view nft_token / nft_is_approved (re-verify)
   │                                 │  optimistic: hapus Sale
   │                                 │  nft_transfer_payout ─►│
   │                                 │◄── payout (untrusted) ─│
   │                                 │  validasi payout (INV-002/003)
   │                                 │  distribusi fee 2% + royalti + seller
   │                                 │  resolve_purchase (#[private]) ── gagal? pulihkan Sale + refund
   │◄────────────────────────────────│  event market_sale

OFFER → ACCEPT
 Buyer         Market Contract              Seller
   │  make_offer(token) + escrow ─►│
   │                                │  Offer + escrow (state on-chain)
   │                                │◄── accept_offer (owner saat itu, sebelum expiry)
   │                                │  nft_transfer_payout ─► distribusi escrow
   │◄───────────────────────────────│  event market_offer_accept
   │  cancel_offer / EXPIRED (lazy) │  refund penuh ke buyer_id (hardcoded)
```

- Tidak ada panah "order signature" — order hanya hidup sebagai state kontrak; semua transisi = tx protokol (replay protection = nonce+blockhash, FACT).

## Decision
**Opsi 2 — on-chain orderbook; TIDAK ada off-chain order signature.** Order lifecycle & invariants: order-protocol-security.md, smart-contract-invariants.md. Kompromi diterima: tidak ada listing gasless (tidak relevan di NEAR), tidak ada partial-fill ala Seaport (NFT unit).

## Rejected
Opsi 1 — kompleksitas signature tak perlu untuk keuntungan gasless yang sudah "gratis" di NEAR.

## Consequences
signature-architecture.md S1 adalah satu-satunya signature dana; backend tidak pernah sign (non-custodial murni).
