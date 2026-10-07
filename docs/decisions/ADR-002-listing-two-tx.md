# ADR-002: Pola Listing 2-Transaksi + Dual Verification

## Status

**Accepted** (ronde 3, 2026-10-01) — framing disempurnakan oleh [ADR-007](./ADR-007-approval-listing-model.md) (model approval non-custodial). Tidak superseded.

## Problem

Bagaimana seller menaruh NFT untuk dijual di market contract tanpa memindahkan kepemilikan NFT ke market?

Dua hal harus benar sebelum listing dianggap sah:
1. Market tahu **harga** dan siapa **seller**-nya.
2. Market punya **izin** (approval NEP-178) untuk memindahkan NFT saat terjual.

Jika listing hanya satu transaksi (`nft_approve` dengan payload yang membuat market mencatat sale di callback `nft_on_approve`), market bergantung pada satu jalur: data sale lahir dari callback NFT contract — yang berarti **kontrak NFT pihak ketiga** (market terbuka, ADR-006) ikut menentukan apakah listing tercatat benar. Kontrak NFT jahat/buggy bisa memalsukan atau menghilangkan callback.

## Context

- Market menerima koleksi dari kontrak NFT mana pun (open marketplace, ADR-006) — termasuk kontrak yang tidak kita audit.
- NEP-178: approval adalah state di NFT contract, dengan `approval_id` yang **invalid setelah transfer** (FACT).
- Pola tutorial resmi (`near-examples/nft-tutorial`, RESEARCH.md §10) memakai dua langkah: `nft_approve` lalu `list_nft_for_sale`, dengan market memverifikasi ulang lewat dua view call paralel.
- Non-custodial (ADR-007): NFT tetap di wallet seller selama listing.

## Options

1. **Satu transaksi** — `nft_approve` dengan payload; market mencatat sale di `nft_on_approve`.
2. **Dua transaksi** — seller `nft_approve` (memberi izin), lalu seller memanggil `list_nft_for_sale` di market; market **memverifikasi ulang** lewat dua view call (`nft_token` untuk ownership + `nft_is_approved` untuk approval) sebelum mencatat sale.

## Trade-offs

| | Opsi 1 (1-tx) | Opsi 2 (2-tx) |
|---|---|---|
| UX | 1 signing | 2 signing (UI harus menjelaskan langkah 1/2) |
| Kepercayaan pada kontrak NFT | tinggi (callback = sumber kebenaran) | rendah (market verifikasi sendiri) |
| Ketahanan kontrak jahat/buggy | lemah | kuat (dual verification) |
| Gas | sedikit lebih murah | satu transaksi tambahan |
| Cocok untuk market terbuka | tidak | ya |

## Security implications

- **Dual verification** adalah inti: ownership (`nft_token().owner_id == signer`) **dan** approval (`nft_is_approved(signer, market, approval_id)`) dicek market sendiri, tidak dari callback. Ini memenuhi SEC-ORDER-004.
- Verifikasi ulang saat **settle** juga wajib (INV-016): listing bisa jadi stale antara list dan buy.
- Callback `nft_on_approve` (bila tetap dipakai) harus `#[private]` + validasi payload (SEC-CONTRACT-003).

## Scalability / Operational / Cost

- Biaya: +1 transaksi per listing (gas < 0.01 Ⓝ) — dapat diterima, gas NEAR murah.
- Operasional: tidak ada beban server tambahan; verifikasi dilakukan on-chain.
- UX: butuh penjelasan dua langkah di UI (dicatat di 03-user-flows + 04-ux-ui-spec).

## Decision

**Opsi 2 — 2 transaksi + dual verification.** Seller: `nft_approve(market, approval_id)` → `list_nft_for_sale(..., msg)`; market memverifikasi ownership + approval via dua view call sebelum mencatat `Sale`. `approval_id` dicatat per listing; invalid setelah transfer (dicek ulang tiap settle).

## Rejected

- **Opsi 1 (1-tx, callback-only)** — ditolak karena market terbuka tidak boleh mempercayai callback dari kontrak NFT arbitrary (G1: kontrak jahat tidak boleh bisa memaksa listing/payout palsu).

## Consequences

- Listing = 2 signing; UI wajib menjelaskan langkah dan menangani kegagalan di antara langkah (approve sukses tapi list gagal → approval menggantung, bisa dipakai ulang / di-revoke).
- `approval_id` menjadi bagian state yang harus di-sync; invalid setelah transfer → settle selalu cek ulang (INV-016).
- Terkait: [ADR-007](./ADR-007-approval-listing-model.md) (model non-custodial), [features/marketplace.md](../features/marketplace.md), [security/order-protocol-security.md](../security/order-protocol-security.md), SEC-ORDER-004.
