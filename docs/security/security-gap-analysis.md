# Security Gap Analysis

> CURRENT RESEARCH STATE vs REQUIRED PRE-IMPLEMENTATION STATE. Fokus: keputusan arsitektur yang belum ada — BUKAN bug kode (belum ada kode produksi).
>
> **Update 2026-10-01 (ronde riset 2)**: G2, G3, G4, G9, G10, G16 SELESAI diriset & diputuskan; G1 diputuskan; G5 sebagian; G6 → task implementasi (playbook IR sudah memuat langkah mass-revoke); **G7 → DEFAULT diterapkan (insurance fund 0.1% fee, aktif mainnet, pencairan via DAO — bisa di-override user)**. Sisa terbuka: **G8 (pilihan final), G12, G13, G14** (keputusan bisnis/fase lanjut), **G11, G15** (riset fase lanjut).
> **Register tunggal**: owner, target, closure criteria, dan skor risiko untuk SEMUA gap terbuka ada di [§ Register gap terbuka](#register-gap-terbuka-owner-target-closure-risiko-pemetaan). Setiap gap didefinisikan **sekali** di sana (bagian di atas hanya status riset).

## CRITICAL UNKNOWN — RESOLVED (2026-10-01)

| # | Gap | Hasil riset & keputusan |
|---|---|---|
| ~~G1~~ | Kebijakan market terhadap kontrak NFT arbitrary | **DIPUTUSKAN: permissionless** (konsisten positioning "open marketplace" ronde 7) — market menerima koleksi apa pun dengan mitigasi: dual verification, payout validation ketat (INV-002/003), gas caps (INV-021), optimistic removal+revert, dan moderasi display-layer + report. Risiko residual = DoS/UX (didokumentasikan di threat-model), bukan kehilangan dana. Kontrak jahat tidak bisa memaksa payout palsu karena payout selalu divalidasi market. |
| ~~G2~~ | Standard sign-message untuk auth API | **NEP-413 berstatus Final** + didukung hampir semua wallet (Defuse/Intents, Privy 2025) → DIPUTUSKAN sebagai format utama; custom challenge = fallback. (wallet-authentication.md) |
| ~~G3~~ | Invariant/fuzz test tooling untuk near-sdk | **DIPUTUSKAN**: near-workspaces (unit+integration) + property testing quickcheck/proptest di atas workspaces (referensi: near-prop) + cargo-fuzz untuk fungsi murni (parsing payout, u128, fee). (smart-contract-invariants.md) |

## HIGH PRIORITY UNKNOWN — sebagian resolved

| # | Gap | Hasil |
|---|---|---|
| ~~G4~~ | Multisig mainnet | **DIPUTUSKAN: Sputnik DAO V2 council 2-of-3** (via app NEAR Treasury — Astro UI deprecated); legacy wallet-2FA terkonfirmasi DEPRECATED — ditolak. (ADR-013, key-management.md) |
| G5 | Upgrade strategy final mainnet | **Sebagian**: pola resmi terkonfirmasi — redeploy + `#[init(ignore_state)]` migrate + State Cleaner; reproducible build NEP-330 untuk verifikasi. Pilihan immutable-vs-redeploy tetap dibuka sampai M4 (keputusan final). |
| G6 | Mass-revoke approval saat owner key compromise | Bukan riset — **task implementasi**: playbook pakai kombinasi `nft_revoke_all` per token (user-side) + `remove_stale_listing` (market-side). Ditambahkan ke incident-response playbook. |
| ~~G7~~ | Kebijakan kompensasi user saat insiden | **DEFAULT diterapkan**: insurance fund 0.1% dari fee 2%, aktif mainnet, pencairan via proposal Sputnik DAO; pilihan final bisa di-override user (incident-response.md §4). |
| ~G8~ | Auditor eksternal — riset SELESAI; **keputusan final & budget masih terbuka (M4)** | Kandidat: OtterSec, Halborn, Block Security, Blocksec (katalog Veridise `near_audits`) |
| ~~G9~~ | DNS hijack monitoring | **DIPUTUSKAN**: registrar transfer-lock + 2FA + cron mingguan `dig` banding record → alert Telegram (cukup untuk skala kita; Cloudflare opsional saat naik). |

## MEDIUM PRIORITY UNKNOWN — sebagian resolved

| # | Gap | Hasil |
|---|---|---|
| ~~G10~~ | Indexer tooling | **DIPUTUSKAN: Neardata** (NEAR Lake DEPRECATED 24 Maret 2026 — FACT docs); alternatif: Nearcore Indexer Framework ($500+/mo) atau Goldsky hosted. RPC failover list resmi terdokumentasi. (indexer-security.md) |
| ~~G16~~ | Reproducible build wasm | **RESOLVED — fitur first-class NEAR**: NEP-330 source metadata + cargo-near Docker build + `contract_source_metadata` verifiable + SourceScan. DIPUTUSKAN dipakai sejak commit pertama (SEC-CONTRACT-006 diperkuat). |
| G11 | Wallet threat-intel integration | RESEARCH REQUIRED (fase 2+) — belum ada katalog jelas di ekosistem NEAR. |
| G12 | CDN/WAF | Tetap terbuka (saat trafik naik). |
| G13 | Contract-account signing | Tetap UNKNOWN (fase lanjut). |
| G14 | Bug bounty program | Keputusan bisnis (mainnet). |

## LOW PRIORITY UNKNOWN

- G15: Formal verification subset invariants (cost/benefit) — satu-satunya item level ini. **Owner/target/closure: lihat register di bawah (definisi tunggal — tidak diulang).**

## Register gap terbuka (owner, target, closure, risiko, pemetaan)

> **G15 hanya didefinisikan SEKALI di sini** (entri sebelumnya di bagian "LOW PRIORITY UNKNOWN" di atas adalah ringkasan; register ini yang menjadi rujukan owner/target/closure — tidak ada definisi ganda).
> **Skala risiko**: Likelihood L (Rendah=1/Sedang=2/Tinggi=3) × Impact I (Rendah=1/Sedang=2/Kritis=3) = Skor (1–2 Rendah, 3–4 Sedang, 6–9 Tinggi).
> **Target** = milestone (M1/M2/M3/M4) atau ⏳ open-by-design; tidak ada tanggal kalender (proyek dipandu milestone — [milestones.md](../../tasks/milestones.md)).

| # | Gap | L | I | Skor | Owner | Target | Closure criteria | Pemetaan (threat-model / SEC-ID) |
|---|---|---|---|---|---|---|---|---|
| G5 | Upgrade strategy final (immutable vs redeploy) | 1 | 3 | 3 (Sedang) | contract | M4 | Pilihan final ditulis di ADR + prosedur upgrade/migrate teruji (drill) | smart-contract-security-architecture §2; SEC-CONTRACT-008 |
| G8 | Auditor eksternal — pilihan & budget | 2 | 3 | 6 (Tinggi) | bisnis (security mendukung) | M4 (sebelum Phase E) | Auditor dikontrak; laporan diterima; **semua temuan kritis/tinggi ditutup + re-test**; TASK-027 selesai | threat-model (Smart contract); TASK-027; SEC-CONTRACT-006 |
| G11 | Wallet threat-intel integration | 2 | 2 | 4 (Sedang) | security | ⏳ open-by-design (M2+, fase 2) | Provider intel NEAR teridentifikasi & diintegrasikan **ATAU** keputusan "tidak pakai, andalkan report manual" tercatat di ADR | threat-model (Marketplace abuse); fraud-and-abuse H5 |
| G12 | CDN/WAF | 2 | 2 | 4 (Sedang) | infra | ⏳ open-by-design (saat trafik naik / M2) | Cloudflare (atau setara) aktif di depan origin **ATAU** risk-accept tertulis dengan ambang pemicu | threat-model (Infrastructure/DoS); infrastructure-security §4; SEC-INFRA-001 |
| G13 | Contract-account signing | 1 | 3 | 3 (Sedang) | security + contract | ⏳ open-by-design (M3+) | Riset selesai → keputusan adopsi/tolak + alasan di ADR | threat-model (Authentication); key-management; SEC-AUTH-004 |
| G14 | Bug bounty program | 1 | 2 | 2 (Rendah) | bisnis | pasca-mainnet (Phase F) | Program diputuskan: aktif (scope+reward) **ATAU** ditolak dengan alasan tertulis | threat-model (semua); incident-response |
| G15 | Formal verification subset invariants | 1 | 2 | 2 (Rendah) | contract | pasca-mainnet (Phase F) | Studi cost/benefit selesai → keputusan subset invariant yang diformalkan (atau ditolak) | smart-contract-invariants; SEC-CONTRACT-005 |

**Catatan risiko**:
- Skor tertinggi = **G8 (6/Tinggi)** — karena audit adalah gate mainnet; status "terbuka" = pilihan vendor, bukan kelalaian teknis.
- G11/G12 berskor Sedang dan **sengaja dibiarkan terbuka** (bukan blocker MVP): G11 bergantung ekosistem, G12 bergantung trafik.
- G13/G14/G15 berisiko Rendah–Sedang, semua pasca-MVP.
- Item ⏳ open-by-design **wajib ditinjau tiap Phase gate** (roadmap) — tetap terlihat, tidak dihapus.

## Kontradiksi yang SUDAH ditemukan & diperbaiki (dari §1 gap review)

1. NFR "platform tidak pernah memegang Ⓝ user" ↔ escrow offer — **diperbaiki**: PRD §10 kini "escrow Ⓝ sementara hanya untuk offer aktif, user-recoverable".
2. Istilah "escrow" longgar (listing vs offer) — diperjelas: escrow HANYA untuk offer Ⓝ; listing tidak mengescrow NFT (approval model).
3. "Admin" ambigu — dipisah: PLATFORM_OWNER (on-chain) vs ADMIN (off-chain moderasi) di access-control-matrix.md.
4. Asumsi tak terdokumentasi: "market menerima semua koleksi" → kini EKSPRESI DIPUTUSKAN (G1).
5. **(Riset 2)** atribusi `nft_transfer_payout` NEP-178 → dikoreksi ke **NEP-199**; daftar standar semua dokumen kini NEP-171/177/178/181/199/297.
6. **(Riset 2)** klaim finality "2 chain heights" → dikoreksi ke 3 tingkat resmi (optimistic/near-final/final).

## Undocumented assumptions yang kini eksplisit

- A1: user MVP = named/implicit account (0x Ethereum-like juga didukung validasi — docs account-validation).
- A2: single-box VPS = single point of failure diterima di MVP (infrastructure-security).
- A3: single owner key = accepted risk testnet, dikompensasi (key-management).
- A4: near-sdk callback `#[private]` benar-benar membatasi predecessor (diverifikasi saat implementasi, bukan diasumsikan di audit).
