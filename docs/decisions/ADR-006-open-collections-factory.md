# ADR-006: Factory Koleksi Open-Minting + Badge Verified + Report System

## Status

**Accepted** (ronde 1 + ronde 3, 2026-10-01).

## Problem

Bagaimana siapa pun bisa membuat koleksi NFT di NearSea, sekaligus menjaga kepercayaan (koleksi palsu/impersonasi) tanpa menyensor on-chain?

## Context

- Posisi produk: marketplace **terbuka** umum ("OpenSea-nya NEAR") — siapa pun boleh membuat koleksi.
- Chain bersifat immutable: konten tidak bisa dihapus on-chain (FACT) → moderasi hanya bisa di **layer tampilan**.
- Impersonasi (koleksi menyerupai brand resmi) adalah risiko nyata pada platform terbuka.
- Model NEAR: satu kontrak per koleksi, di-deploy lewat factory.

## Options

1. **Factory open + badge verified + report system** — siapa pun deploy koleksi; kurasi via badge; moderasi via report.
2. Koleksi hanya oleh allowlist kreator (kurasi penuh di muka).
3. Tanpa moderasi apa pun.

## Trade-offs

| | Opsi 1 | Opsi 2 | Opsi 3 |
|---|---|---|---|
| Keterbukaan | tinggi | rendah | tinggi |
| Beban kurasi | reaktif (report) | proaktif (berat) | nol |
| Risiko impersonasi | sedang (badge + report) | rendah | tinggi |
| Butuh backend | ya (ringan) | ya | tidak |
| Skalabilitas kreator | tak terbatas | terbatas | tak terbatas |

## Security implications

- **Tidak ada penyensoran on-chain** — moderasi = sembunyikan dari discovery (display-layer), bukan hapus state (FACT: chain immutable). Ditegaskan di [metadata-security.md](../security/metadata-security.md) §4.
- Report system butuh penyimpanan → inilah pengecualian "ada backend" (ADR-004/009). Wajib: rate limit, admin auth, audit trail (SEC-ADMIN-*).
- Badge verified = sinyal off-chain; **bukan** jaminan keamanan kontrak. Market tetap memvalidasi payout apa pun kontraknya (ADR-002 dual verification).
- Factory memakai Pausable (INV-022) untuk menghentikan deploy koleksi baru saat insiden.

## Scalability / Operational / Cost

- Satu kontrak factory; tiap koleksi = kontrak terpisah (storage dibayar pembuat).
- Beban backend ringan: antrean report + flag verified di tabel kecil.
- Moderasi manual oleh admin (1 orang) — perlu SLA & antrean (lihat [features/notifications.md](../features/notifications.md), TASK-018/019).

## Decision

**Opsi 1 — factory open-minting + badge verified + report system.**
- Siapa pun bisa deploy koleksi via factory (open, tanpa cap).
- **Badge verified** = kurasi admin (allowlist/flag di DB).
- **Report system**: user report → antrean admin → takedown **display-layer saja** (sembunyikan dari discovery/listing FE).
- Supply koleksi ditentukan kreator (fixed supply atau open edition) + mint price (ronde 6).

## Rejected

- **Opsi 2 (allowlist kreator)** — ditolak: bertentangan dengan positioning terbuka.
- **Opsi 3 (tanpa moderasi)** — ditolak: impersonasi & phishing tidak terkendali; badge/report adalah mitigasi minimum.

## Consequences

- Muncul kebutuhan backend ringan (PostgreSQL) — memperkuat ADR-004 Opsi 3 & ADR-009.
- Admin panel + antrean report jadi pekerjaan MVP (TASK-018/019/023).
- Kontrak NFT pihak ketiga harus diperlakukan **untrusted** (G1) → verifikasi ganda di market (ADR-002).
- Terkait: [ADR-008](./ADR-008-launchpad-phases.md), [security/fraud-and-abuse.md](../security/fraud-and-abuse.md), [security/permissions.md](../security/permissions.md).
