# ADR-003: Satu Kontrak Market Gabungan (List + Offer + Private/Bundle)

## Status

**Accepted** (ronde 3, 2026-10-01).

## Problem

Berapa banyak kontrak market yang dibangun untuk MVP? Satu kontrak yang menangani semua jenis order, atau beberapa kontrak terpisah per jenis order (listing, offer, bundle, auction)?

## Context

- Jenis order MVP: listing (jual), offer (beli), private listing, bundle. Auction = fase 2.
- Semua jenis order **menyentuh state yang sama**: token bisa di-list sekaligus di-offer, dan ada aturan silang (INV-028: token dalam bundle aktif tidak boleh di-list terpisah; INV-023: self-buy ditolak).
- Aturan keamanan inti (payout validation, dual verification, optimistic removal) identik untuk listing dan offer-accept.
- Biaya deploy kontrak di NEAR dibayar storage (1 Ⓝ/100 KB) — memecah ke banyak kontrak menambah biaya dan titik sinkronisasi.

## Options

1. **Satu kontrak market gabungan** — `sales`, `offers`, `bundles` dalam satu kontrak.
2. **Kontrak terpisah per jenis order** — market-listing, market-offer, market-bundle.
3. **Satu kontrak per koleksi** — deploy market per koleksi NFT.

## Trade-offs

| | Opsi 1 | Opsi 2 | Opsi 3 |
|---|---|---|---|
| Aturan silang antar order | mudah (state satu tempat) | sulit (lintas kontrak) | sangat sulit |
| Jumlah deploy | 1 (+factory) | 3+ | N per koleksi |
| Storage/biaya | rendah | lebih tinggi | tinggi |
| Batas throughput | satu shard | tersebar (bila akun beda) | tersebar |
| Kompleksitas kode | satu kontrak besar | modular | berulang |

## Security implications

- Aturan silang (INV-023 self-buy, INV-028 bundle vs listing, INV-009 accept-offer sekali) hanya bisa ditegakkan andal bila state berada di satu kontrak. Memecah kontrak = perlu koordinasi lintas kontrak yang rawan race.
- Satu kontrak = satu otoritas settlement (selaras ADR-012 orderbook on-chain) → lebih mudah diaudit (satu permukaan).
- Risiko: satu kontrak = satu titik kegagalan. Dimediasi Pausable (INV-022) + escrow user-recoverable.

## Scalability / Operational / Cost

- Batas throughput = kapasitas satu shard (semua listing di satu akun kontrak). Untuk MVP jauh di atas kebutuhan (lihat [scaling.md](../architecture/scaling.md)).
- Bila kontensi jadi bottleneck nyata → opsi fase lanjut: **sharded market contracts** (routing per koleksi) — butuh ADR baru.
- Biaya: satu deploy kontrak, bukan N.

## Decision

**Opsi 1 — satu kontrak market gabungan** yang menangani listing + offer + private listing + bundle. Auction ditambahkan **di kontrak yang sama** pada fase 2 (bukan kontrak baru).

## Rejected

- **Opsi 2 (kontrak terpisah)** — ditolak: aturan silang antar order tidak bisa ditegakkan bersih lintas kontrak; menambah race & biaya.
- **Opsi 3 (per koleksi)** — ditolak: biaya deploy berulang, aturan silang mustahil, tidak sesuai model market terbuka.

## Consequences

- Kontrak market jadi kontrak terbesar → wajib batas loop statis (SEC-CONTRACT-010), gas budget per method, dan audit menyeluruh (TASK-027).
- Method terkanonisasi dalam satu kontrak (`buy`, `make_offer`, `accept_offer`, `buy_bundle`, `list_nft_for_sale`, `remove_sale`, `update_price`).
- Terkait: [ADR-012](./ADR-012-order-settlement-authority.md), [features/marketplace.md](../features/marketplace.md), [security/smart-contract-invariants.md](../security/smart-contract-invariants.md).
