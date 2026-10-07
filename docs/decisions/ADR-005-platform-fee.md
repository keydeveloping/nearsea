# ADR-005: Fee Platform 2% Dipotong On-Chain di Payout

## Status

**Accepted** (ronde 1, 2026-10-01).

## Problem

Bagaimana platform mengambil fee dari setiap penjualan, dengan jumlah, jalur, dan tujuan yang tidak bisa dimanipulasi?

## Context

- Fee harus berlaku **konsisten di semua jalur penjualan**: buy-now, accept-offer, bundle (ronde 9).
- Market non-custodial: dana buyer mengalir lewat market hanya dalam satu transaksi settlement (escrow offer sementara, ADR-007).
- Nilai fee harus bisa diaudit dan tidak bisa dinaikkan sepihak oleh owner key (bahaya CS-1).
- Treasury address belum ditentukan (open-by-design, ditetapkan saat deploy testnet).

## Options

1. **Fee 2% dipotong on-chain** saat settlement, langsung ke treasury.
2. Fee dipungut off-chain (invoice terpisah).
3. Fee 0% (tanpa monetisasi di MVP).

## Trade-offs

| | Opsi 1 | Opsi 2 | Opsi 3 |
|---|---|---|---|
| Tidak bisa dihindari | ya (atomik di settlement) | tidak | — |
| Konsisten semua jalur | ya | sulit | — |
| Kompleksitas kontrak | sedang | rendah | nol |
| Auditabilitas | tinggi (on-chain) | rendah | — |

## Security implications

- Fee dipotong **sebelum** distribusi seller+royalti; validasi: `fee + Σroyalti + seller ≤ harga` (INV-001), sisa ≤ 1 yocto (INV-003).
- **`MAX_FEE_BPS` immutable cap** (≤500 = 5%) membatasi kerusakan bila owner key disusupi; perubahan `fee_bps` hanya lewat governance ADR-013 (SEC-CONTRACT-004).
- **Nilai default `fee_bps = 200` (2%)** — cap 500 bukan berarti fee 5% (koreksi kontradiksi: lihat [tech-stack.md](../architecture/tech-stack.md)).
- Payout ke treasury divalidasi seperti receiver lain (masuk hitungan ≤10 penerima).

## Scalability / Operational / Cost

- Fee on-chain = nol biaya operasional tambahan; tidak ada penagihan manual.
- Penarikan treasury via `withdraw_fees` (owner-only pada MVP default).
- Gas: satu transfer tambahan per settlement — dapat diabaikan.

## Decision

**Opsi 1 — fee platform 2%** (`fee_bps = 200`) dipotong on-chain di payout, dikirim ke treasury; berlaku di **semua** jalur penjualan (buy now, accept offer, bundle — dipotong sekali per penjualan). Cap immutable `MAX_FEE_BPS = 500`. Alamat treasury **open-by-design** (ditetapkan saat deploy testnet).

## Rejected

- **Opsi 2 (off-chain)** — ditolak: tidak konsisten, mudah dihindari, tidak auditabel.
- **Opsi 3 (0%)** — ditolak: model bisnis perlu fee; royalti on-chain adalah value prop, fee 2% menopang operasi.

## Consequences

- Setiap settlement wajib menghitung & memvalidasi fee; error case: payout invalid → refund penuh (SEC-ORDER-001).
- Treasury address harus diisi sebelum deploy; sampai itu, `withdraw_fees` tidak bisa diuji end-to-end.
- Terkait: [features/payments.md](../features/payments.md), [01-PRD.md](../01-PRD.md) §13, SEC-CONTRACT-004, INV-001/003.
