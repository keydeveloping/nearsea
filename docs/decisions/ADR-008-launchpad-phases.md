# ADR-008: Launchpad Berphase Fleksibel + Allowlist On-Chain

## Status

**Accepted** (ronde 10, 2026-10-01).

## Problem

Bagaimana kreator meluncurkan koleksi dengan **beberapa fase mint** (mis. allowlist murah dulu, lalu publik), dengan aturan yang tidak bisa dimanipulasi?

## Context

- Acuan produk: **OpenSea Studio** — kreator mendefinisikan fase mint sendiri.
- Allowlist off-chain (server) = titik kepercayaan & tidak bisa diverifikasi publik.
- NEAR: storage dibayar pemilik data (NEP-145); harga & alokasi per fase harus dicek saat mint.
- Koleksi baru harus langsung tampil di discovery (tanpa kurasi di muka — ADR-006).

## Options

1. **Phase bebas, allowlist on-chain** — kreator definisikan berapa pun fase; tiap fase: harga, alokasi, allowlist opsional, waktu; allowlist = set on-chain.
2. Satu fase publik saja (tanpa allowlist).
3. Allowlist off-chain (server) diverifikasi saat mint.

## Trade-offs

| | Opsi 1 | Opsi 2 | Opsi 3 |
|---|---|---|---|
| Fleksibilitas kreator | tinggi | rendah | tinggi |
| Verifikasi allowlist | publik (on-chain) | — | butuh server (titik percaya) |
| Biaya storage allowlist | kreator pre-deposit | — | nol |
| Kompleksitas kontrak | sedang-tinggi | rendah | sedang |

## Security implications

- **Allowlist on-chain** = dapat diverifikasi siapa pun, tidak bisa diubah diam-diam oleh server (menghilangkan titik percaya off-chain).
- Validasi saat `nft_mint`: fase aktif, alamat di allowlist, alokasi tersisa, **deposit exact-match** (INV-017/018, SEC-CONTRACT-009).
- **Phase berurutan, tidak overlap** (INV-029) — maksimum satu fase aktif; mencegah double-mint lintas fase.
- Storage allowlist dibayar kreator (pre-deposit) — kontrak tidak menanggung storage user (INV-019).
- Storage mint dibayar **pemicu mint** (minter).

## Scalability / Operational / Cost

- Setiap fase = entry kecil di kontrak koleksi; jumlah fase bebas → batas praktis = storage kreator.
- Tidak ada cron: validasi fase **on-chain saat mint** (bukan pekerjaan background) — lihat [backend-architecture.md](../architecture/backend-architecture.md).
- Mint bertekanan tinggi (fase publik) = banyak tx ke satu kontrak koleksi; serialisasi shard menjamin urutan (lihat [concurrency-and-races.md](../development/concurrency-and-races.md)).

## Decision

**Opsi 1 — phase bebas + allowlist on-chain.**
- Kreator mendefinisikan **berapa pun** fase; tiap fase: harga, alokasi, allowlist opsional, waktu mulai/selesai.
- Allowlist disimpan sebagai **set on-chain** (kreator pre-deposit storage).
- Koleksi baru **langsung tampil** di discovery; supply bebas tanpa cap platform.
- Storage mint dibayar **pemicu mint**; deposit mint **exact-match** harga fase.

## Rejected

- **Opsi 2 (satu fase publik)** — ditolak: tidak memenuhi kebutuhan launchpad ala OpenSea Studio.
- **Opsi 3 (allowlist off-chain)** — ditolak: titik percaya server; allowlist tidak bisa diverifikasi publik.

## Consequences

- Kontrak factory + launchpad jadi bagian Fase 1 (TASK-012/020) dan wajib diaudit (TASK-027).
- UI create-collection wizard + upload allowlist (TASK-021).
- Terkait: [ADR-006](./ADR-006-open-collections-factory.md), [features/marketplace.md](../features/marketplace.md), INV-017/018/019/029.
