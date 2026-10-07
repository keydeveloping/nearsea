# ADR-007: Model Listing Approval Non-Custodial (ala Seaport) + Auto-Stale

## Status

**Accepted** (ronde 9, 2026-10-01 — hasil riset OpenSea/Seaport). **Menyempurnakan framing [ADR-002](./ADR-002-listing-two-tx.md).**

## Problem

Saat NFT di-list, apakah NFT **dipindah ke custody market** (escrow) atau **tetap di wallet seller** hanya dengan approval?

Pertanyaan ini muncul dari user: *"bukankah NFT harusnya keluar dari wallet owner saat di-list?"* Jawabannya menentukan seluruh model keamanan listing.

## Context

- Riset OpenSea: **Seaport bersifat approval-based non-custodial** — NFT tetap di wallet seller; hanya approval yang berpindah; listing yang sudah tidak valid (token dipindah) otomatis dibatalkan (sumber: opensea.io/blog pengumuman Seaport, docs.opensea.io/docs/seaport, github.com/ProjectOpenSea/seaport).
- NEP-178: approval adalah izin, bukan transfer; `approval_id` invalid setelah transfer (FACT).
- Model escrow (NFT masuk market) menambah risiko: market memegang aset user → kontrak compromise = aset hilang.
- Posisi produk: **setara OpenSea** → perilaku harus mirip.

## Options

1. **Approval non-custodial** — NFT tetap di wallet seller; market hanya punya approval; listing permanen; auto-stale bila ownership berubah.
2. **Escrow** — NFT dipindah ke market saat listing; dikembalikan saat cancel/expire.
3. Escrow hanya untuk offer (dana Ⓝ), bukan NFT.

## Trade-offs

| | Opsi 1 | Opsi 2 |
|---|---|---|
| Risiko market compromise | rendah (tidak pegang NFT) | tinggi (pegang NFT) |
| Seller bisa pakai NFT selama listing | ya (bisa di-transfer → stale) | tidak |
| Perlu deteksi stale | ya (wajib) | tidak |
| Sinkron dengan OpenSea | ya | tidak |
| Kompleksitas | sedang (stale handling) | sedang (escrow lifecycle) |

## Security implications

- **Market tidak pernah memegang NFT user** (non-custodial) — mengurangi dampak kompromi kontrak (selaras ADR-010 chain-centric boundary).
- Konsekuensi wajib: **deteksi stale** — bila `nft_token().owner_id != sale.owner_id`, listing disembunyikan dan tidak bisa dibeli; `remove_stale_listing` membersihkan state (INV-016, SEC-ORDER-004).
- Dual verification (ADR-002) menjadi penjaga utama: settle selalu cek ulang ownership + approval.
- Dana: escrow Ⓝ **hanya** untuk offer aktif, selalu user-recoverable via cancel/expire (01-PRD §10).

## Scalability / Operational / Cost

- Tidak ada biaya kustodi; tidak ada transfer bolak-balik (hemat gas).
- Stale check = view call saat discovery/beli → beban read ke RPC (dimitigasi failover).
- Listing permanen → tidak butuh cron expire untuk listing (offer tetap punya expiry, lazy-evaluated).

## Decision

**Opsi 1 — approval non-custodial ala Seaport.**
- NFT **tetap di wallet seller** selama listing; market hanya memegang approval (NEP-178).
- **Listing permanen** (tanpa expiry).
- **Auto-stale**: ownership berubah → listing otomatis tidak valid + disembunyikan dari discovery.
- **Offer boleh pada token apa pun** (listed atau tidak).
- **Fee 2% konsisten** di buy-now maupun accept-offer (ADR-005).

## Rejected

- **Opsi 2 (escrow NFT)** — ditolak: market memegang aset user = risiko besar & tidak sesuai perilaku OpenSea yang jadi acuan produk.

## Consequences

- Wajib ada mekanisme stale (TASK-022) + `remove_stale_listing`; test TC-006.
- Seller yang memindahkan NFT di luar market otomatis membatalkan listingnya (perilaku harus dijelaskan di UI).
- Terkait: [ADR-002](./ADR-002-listing-two-tx.md), [security/order-protocol-security.md](../security/order-protocol-security.md), INV-016, [03-user-flows.md](../03-user-flows.md).
