# ADR-013: Key Management — Interim MVP vs Target Mainnet

## Status
Accepted (interim MVP) — **updated 2026-10-01**: mainnet model DIPUTUSKAN = Sputnik DAO V2 council 2-of-3 (via app NEAR Treasury — Astro UI deprecated); legacy wallet-2FA multisig terkonfirmasi DEPRECATED dan ditolak. Implementasi tetap PROPOSED (gate M4).

## Problem
Owner key market/factory = otoritas penuh (pause, fee, treasury, upgrade). Model penyimpanan menentukan skenario bencana terburuk (CS-1). Prompt security melarang "private key in env var" sebagai model PRODUKSI.

## Context
Tim 1 orang, infrastruktur 1 VPS, fase testnet dulu. near-sdk-contract-tools memakai Owner pattern (satu akun).

## Options
1. Single owner key di VPS (interim) + seed offline.
2. Multisig — **Sputnik DAO V2** (aktif, maintained) / ~~near multisig~~ (wallet-2FA legacy — DEPRECATED, ditolak).
3. MPC/KMS/HSM provider.
4. Dedicated signer service.

## Trade-offs
- Opsi 1: sederhana, cepat; risiko: VPS compromise = total control. Diterima di testnet (tanpa dana nyata) + kompensasi (MAX_FEE_BPS immutable, escrow user-recoverable, NFT tidak pernah dipegang).
- Opsi 2: separation of duties native NEAR; setup + risiko konfigurasi salah; UX slow untuk upgrade.
- Opsi 3: hardware-grade tapi tidak native NEAR, cost/ops tinggi untuk 1-orang team.
- Opsi 4: backend tidak pernah sign apa pun (ADR-012) — signer service kontradiktif.

## Security implications
Opsi 1 di mainnet = unacceptable single point of failure (CS-1). Opsi 2 + timelock membatasi: upgrade/fee tidak instan, pause tetap cepat.

## Scalability / Operational / Cost
Opsi 2 butuh 3+ key holders — komitmen organisasi; biaya minimal (kontrak multisig).

## Decision
- **MVP (DECIDED)**: Opsi 1 + kontrol kompensasi + seed offline 2 lokasi. Diterima risiko eksplisit di testnet.
- **Mainnet (DECIDED 2026-10-01 — implementasi PROPOSED, gate M4)**: Opsi 2 — **Sputnik DAO V2 council 2-of-3** (via app NEAR Treasury — Astro UI deprecated) + timelock untuk aksi finansial/upgrade; pause tetap cepat via guardian `pause_callers` (terpisah dari owner-DAO); FINANCE ≠ SECURITY ≠ DEVELOPER (access-control-matrix).
- RESEARCH REQUIRED tersisa (kecil): integrasi Owner pattern near-sdk-contract-tools dengan owner = DAO account + desain guardian pause terpisah; prosedur rotasi council (key-management.md §3a).

## Rejected
- **near multisig (wallet-2FA legacy)** — DEPRECATED (FACT riset 2026-10-01): wallet lama deprecated, ekosistem migrasi ke Sputnik DAO/NEAR Treasury/smart accounts.
- **MPC/KMS/HSM (Opsi 3)** — overkill untuk tahap ini (tidak native NEAR, cost/ops tinggi); tetap kandidat alternatif jangka panjang.
- **Dedicated signer service (Opsi 4)** — kontradiktif: backend tidak pernah sign (ADR-012).
- **Single owner key di mainnet** — unacceptable single point of failure (CS-1).

## Klarifikasi dua tingkat status

> ADR ini mengikat **dua model berbeda** pada dua fase. "DECIDED" = keputusan model sudah final; "PROPOSED" = implementasi teknis belum ditulis/diuji. Keduanya **tidak** bertentangan.

| Tingkat | Fase | Model | Status | Gate / bukti |
|---|---|---|---|---|
| **Interim MVP** | testnet | Opsi 1: single owner key + seed offline 2 lokasi | **Accepted / DECIDED** (dengan risiko eksplisit) | M1–M2; SEC-KEY-001; drill rotasi manual |
| **Target Mainnet** | mainnet | Opsi 2: Sputnik DAO V2 council 2-of-3 + timelock 24 jam + guardian `pause_callers` | **Model DECIDED (2026-10-01) / implementasi PROPOSED** | M4; SEC-KEY-002 & SEC-CONTRACT-012 → READY-FOR-IMPLEMENTATION |

- **Tidak ada** rencana menjalankan single-key di mainnet. Interim MVP hanya berlaku di testnet (tanpa dana nyata), dengan kompensasi terdokumentasi (key-management.md §2).
- **Bukan** "mainnet ditunda" — yang tertunda hanya **implementasi** (kode + drill), bukan keputusan model.

## Rotasi council (pointer)

Prosedur rotasi council & guardian lengkap **sudah ada** di **[security/key-management.md §3a](../security/key-management.md)** — dirujuk, tidak diduplikasi di sini:

- **Rutin** (anggota keluar/masuk): `add_member`/`remove_member` via proposal DAO 2-of-3 + cabut akses off-chain (SSH/deploy/Telegram/DB) + catat log.
- **Darurat** (dugaan kompromi satu key — CS-1): guardian pause SEGERA → `remove_member(compromised)` 2-of-3 → rotasi kunci & secret → unpause via DAO → postmortem.
- Prinsip: rotasi rutin = 2-of-3 (satu anggota tidak cukup); darurat = pause dulu (cepat, single-key guardian) lalu perbaiki via DAO. **Tidak ada jalur satu-akun yang bisa memindahkan dana.**
- Terkait: onboarding anggota (key-management.md §7), rotasi langkah-demi-langkah (§6), runbook recovery (§9).

## Desain guardian pause (mainnet)

> Masalah: bila `owner_id` = DAO account, panggilan `pause` single-key akan **gagal** cek owner. Pause harus tetap cepat (reaksi insiden), jadi tidak boleh menunggu proposal DAO.

```text
1. Daftar `pause_callers` ditetapkan saat deploy — akun guardian terpisah dari owner-DAO
   (role SECURITY; mis. 2 akun terpisah).
2. Guardian HANYA boleh `pause` — TIDAK boleh `unpause`, TIDAK boleh tarik dana, TIDAK boleh ubah param.
3. `unpause` HANYA via DAO 2-of-3 (+ timelock bila berlaku) — mencegah guardian menyandera kontrak.
4. Pause memblokir mutasi baru tetapi mengizinkan cancel/withdraw/refund (INV-022, TC-012).
5. Drill pause/unpause wajib di testnet sebelum mainnet (SEC-IR-001).
```

- **RESEARCH REQUIRED (kecil)**: integrasi Owner pattern near-sdk-contract-tools (owner = DAO account) dengan daftar `pause_callers` terpisah. Ini satu-satunya pekerjaan desain kontrak yang tersisa untuk model mainnet.

## Timeline migrasi mainnet

> Gate per milestone; detail pekerjaan key management ada di key-management.md §10. Urutan **terkunci**: jangan migrasi sebelum (a) DAO council teruji di testnet, (b) drill pause/rotasi/recovery hijau, (c) semua SEC P0 mainnet gate READY-FOR-IMPLEMENTATION.

| Milestone | Pekerjaan key management | Gate / bukti |
|---|---|---|
| **M1–M2** (MVP/testnet) | owner single-key + seed offline 2 lokasi; alert Telegram tx owner tak terduga (TASK-026) | SEC-KEY-001 |
| **M3** (pra-mainnet) | desain `pause_callers` + setup Sputnik DAO V2 council 2-of-3 via NEAR Treasury + timelock 24 jam | SEC-KEY-002 → READY-FOR-IMPLEMENTATION |
| **M3** (pra-mainnet) | drill rotasi council (§3a), onboarding (§7), pause/unpause guardian, recovery (§9) — di testnet | SEC-IR-001 |
| **M4** (mainnet gate) | migrasi owner → DAO; treasury → DAO-guarded; guardian aktif; reproducible build (NEP-330); keputusan Shamir/split-seed difinalkan | SEC-CONTRACT-012/006 |
| **Pasca-mainnet** | rotasi rutin tahunan + insidental; audit akses berkala | log rotasi |

## Consequences
Sebelum mainnet: SEC-KEY-002 harus status READY-FOR-IMPLEMENTATION; drill rotasi/pause dilakukan (SEC-IR-001).
