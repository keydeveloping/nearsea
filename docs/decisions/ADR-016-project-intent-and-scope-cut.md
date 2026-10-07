# ADR-016: Niat Proyek (Pembelajaran/Portofolio) + Pemotongan Scope M1

## Status

**Accepted** (ronde 17, 2026-10-07) — **keputusan user langsung** (jawaban atas pertanyaan grilling Q2 + Q4).
**Mengubah** asumsi implisit di seluruh dokumen: proyek ini **bukan** usaha komersial.

## Problem

Seluruh dokumentasi (94 file, ~181k kata) dibangun dengan aparatus **produksi**: gate audit eksternal,
governance Sputnik DAO 2-of-3, 50 security requirement, bug bounty, rencana DR/RTO, dan gate M4.
Aparatus itu mahal dan hanya masuk akal kalau proyek ini dikejar sebagai **bisnis**.

Dua pertanyaan terbuka yang menentukan seluruh cabang:
1. **Kenapa proyek ini ada?** (bukan "apa"-nya — itu sudah jelas: marketplace NFT di NEAR)
2. **Seberapa besar scope yang harus selesai sebelum bisa disebut "jadi"?**

Keduanya tidak pernah dijawab di dokumen mana pun. Akibatnya M1 membengkak jadi **23 dari 35 task
(66% seluruh pekerjaan)** — termasuk launchpad, moderasi, admin panel, dan infrastruktur VPS —
sebelum ada satu orang pun yang mencoba intinya.

## Context

**Fakta pasar (riset 2026-10-07, [docs/research/near-nft-market-2026.md](../research/near-nft-market-2026.md)):**
- Paras (marketplace NFT andalan NEAR) → pivot ke `narrativeprotocol.com` (engine AI, bukan NFT). **Terverifikasi langsung.**
- MITTE (klaim ~91% pangsa NEAR, 35k MAU) tutup 2025-04-10.
- Mintbase paused; Few and Far (danai $10,5M + grant NEAR Foundation) jadi blog; TradePort drop NEAR.
- Magic Eden **tidak pernah** support NEAR; Tensor Solana-only.
- Satu-satunya yang hidup: HotCraft, volume koleksi teratas ≈ **$742/minggu**.
- NEAR absen dari ranking CryptoSlam. Volume NFT global turun ~37% (2025 vs 2024).
- "Royalti dipaksa on-chain" — pembeda utama di README/ADR-005 — kalah di perang royalti 2022–2023.

**Jawaban user (ronde 17):**
- **Q2 (niat):** *"gapapa saya mau build ini aja, mau ada trader atau engga juga saya aman aman saja."*
  → Proyek **pembelajaran/portofolio**. Tidak mengejar pengguna, pendapatan, atau pangsa pasar.
- **Q3 (resource):** full-time.
- **Q4 (potong scope):** *"iya yauda atur baiknya aja gimana."* → user menyerahkan bentuk potongannya.

## Options

1. **Lanjutkan sebagai produk** — perbaiki 3 lubang kritis, tambah rencana permintaan/cold-start, kejar audit + DAO + mainnet.
2. **Pembelajaran/portofolio, scope penuh** — tetap kerjakan 23 task M1, tapi tanpa audit/DAO/bug bounty.
3. **Pembelajaran/portofolio, potong ke vertical slice** — buktikan tesis inti dalam slice kecil yang jalan, sisanya jadi backlog.

## Trade-offs

| | Opsi 1 (produk) | Opsi 2 (penuh) | Opsi 3 (slice) |
|---|---|---|---|
| Sesuai niat user (Q2) | tidak | ya | **ya** |
| Waktu ke hasil pertama | 9+ bulan | 3–6 bulan | **minggu** |
| Membuktikan tesis inti | ya | ya, tapi lambat | **ya** |
| Risiko proyek mangkrak | tinggi | sedang | **rendah** |
| Biaya (audit, VPS, IPFS) | besar | sedang | **hampir nol** |
| Yang tidak didapat | — | umpan balik dini | fitur lanjutan (sementara) |

## Security implications

- **Gate yang menjadi opsional** (karena tidak ada dana nyata + bukan produk): audit eksternal (TASK-027),
  Sputnik DAO + timelock (SEC-CONTRACT-012/SEC-KEY-002), bug bounty (G14), insurance fund G7,
  drill pause/restore (SEC-IR-001), CDN/WAF (G12).
- **Yang TETAP wajib** (murah, dan inti pembelajaran): `assert_one_yocto`, callback `#[private]`,
  validasi payout, NEP-145 storage, dual verification, reentrancy-safe promise handling,
  secret hygiene (`.gitignore` + gitleaks), tidak ada kredensial di repo.
- **Alasan**: kalau tujuannya belajar, mempelajari **pola keamanan kontrak** adalah bagian intinya;
  mempelajari **prosedur governance organisasi** (DAO council, audit vendor, bug bounty) tidak.
- **Wajib dicatat**: dokumen tetap **dilarang** menyatakan proyek ini "aman" atau "siap produksi".
  Tanpa audit, tidak ada klaim keamanan — status tetap PROPOSED.

## Scalability / Operational / Cost

- Infrastruktur turun drastis: **testnet saja**, kontrak di-deploy manual, FE bisa jalan lokal atau
  hosting statis gratis. VPS + PostgreSQL + monitoring (TASK-028/029/030) **ditunda** — bukan dibatalkan.
- Biaya berjalan ≈ nol (faucet testnet gratis; tidak ada domain, VPS, IPFS pinning, atau audit).
- Konsekuensi: jalur "self-hosted VPS" (ADR-009) tetap berlaku untuk fase lanjut, tapi tidak memblokir M1.

## Decision

**Opsi 3.** Proyek ini **pembelajaran/portofolio**; M1 dipotong ke **vertical slice** yang membuktikan
tesis inti. Keputusan turunan:

### 1. Niat proyek (mengikat)
- Proyek **bukan** usaha komersial. Tidak ada target pengguna, volume, atau pendapatan.
- Sukses = **tesis inti terbukti jalan** + penulisnya paham cara kerjanya.
- Riset pasar ([near-nft-market-2026.md](../research/near-nft-market-2026.md)) **diterima sebagai fakta**
  dan **tidak** menjadi alasan berhenti — audiens proyek ini adalah penulisnya sendiri (dan perekrut),
  bukan trader.
- Klaim komersial di dokumen (mis. "keunggulan kita", value proposition vs OpenSea) **diturunkan nadanya**
  menjadi deskripsi teknis.

### 2. M1 = vertical slice (8 task)
Tesis yang harus terbukti: **mint → list (2-tx, non-custodial) → orang lain beli → royalti + fee terbayar
→ NFT tidak pernah dipegang kontrak.**

| Task | Peran di slice |
|---|---|
| TASK-001 | Scaffold repo + CI (gate tunggal) |
| TASK-002 | Kontrak NFT: mint, metadata, NEP-178 |
| TASK-003 | Royalti NEP-199 |
| TASK-004 | Market: storage NEP-145 + listing 2-tx + dual verification |
| TASK-005 | Buy + settle + resolve/refund |
| TASK-006 | Sandbox test (2 kontrak) |
| TASK-007 | FE: connect wallet |
| TASK-008 | FE: browse / list / buy |

- **Koleksi di-deploy manual** (`cargo near deploy`) — **factory tidak diperlukan** di slice.
- **Testnet saja.** Tidak ada VPS, domain, IPFS pinning, atau audit.
- **Market hanya menerima koleksi NearSea** di slice (menutup H1 — lihat §3).

### 3. Sisanya → M1+ (bukan dibuang)
Semua task M1 lama yang lain pindah ke **M1+ (MVP completion)**: TASK-008b, 009, 010, 012, 015, 018, 019,
020, 021, 022, 023, 026, 028, 029, 030, 033. **ID task tidak berubah** — hanya pengelompokan milestone.
M2/M3/M4 tidak berubah.

### 4. Bundle & fitur terkait ditunda (dan itu menutup 3 temuan kritis)
Tiga temuan kritis dari review desain (ronde 17) semuanya berpusat di **bundle**:
- **C1** — `PARTIAL` tanpa alokasi kerugian + fund G7 tanpa jalur pendanaan
- **C2** — gas `buy_bundle` melebihi 300 Tgas setelah pre-validasi
- **C3** — dana buyer bisa nyangkut bila callback gagal (tanpa method recovery)

**TASK-010 (bundle) ditunda ke M1+**, sehingga ketiganya **tidak memblokir M1**. Sebelum bundle
dikerjakan, ketiganya **wajib diselesaikan lebih dulu** (dicatat sebagai syarat di backlog).

## Rejected

- **Opsi 1 (produk)** — ditolak user: *"mau ada trader atau engga juga saya aman aman saja."* Bukti pasar
  juga menunjukkan bentuk umum marketplace akan dapat ≈0 pengguna.
- **Opsi 2 (scope penuh tanpa potong)** — ditolak: 23 task sebelum umpan balik pertama = risiko mangkrak
  tinggi; untuk tujuan pembelajaran, memperlambat pembelajaran itu sendiri.

## Consequences

**Positif:**
- Hasil pertama bisa didemokan dalam **minggu**, bukan bulan.
- Biaya ≈ nol; tidak ada ketergantungan vendor/audit.
- Tesis inti (mekanika non-custodial + split payout) terbukti lebih dulu; fitur lanjutan dibangun di atas
  fondasi yang sudah terbukti.
- 3 temuan kritis (C1–C3) tertunda bersama bundle, bukan diabaikan.

**Negatif / diterima:**
- Tanpa audit → **tidak ada klaim keamanan**; dokumen wajib tetap berstatus PROPOSED.
- Tanpa mainnet → tidak ada penggunaan nyata; ini konsekuensi yang diterima.
- Fitur "marketplace lengkap" (launchpad, offers, bundle, moderasi) tertunda.
- Risiko: slice bisa terasa terlalu kecil → dimitigasi dengan mendefinisikan M1+ sebagai lanjutan eksplisit.

**Wajib ditindaklanjuti:**
- [x] Premis `01-PRD.md` §2 diperbarui dengan bukti pasar.
- [x] `tasks/milestones.md` + `tasks/backlog.md`: M1 dipotong, M1+ didefinisikan.
- [ ] Temuan H1–H4 + M1–M7 dari review desain dicatat di dokumen kontrak dengan status pasca-cut.
- [ ] Gate audit/DAO/bug-bounty di `deployment.md`, `security-roadmap.md`, `milestones.md` ditandai
      **"wajib hanya bila proyek beralih ke jalur produk"**.

## Related

[ADR-001](./ADR-001.md) (indeks) · [research/near-nft-market-2026.md](../research/near-nft-market-2026.md) ·
[01-PRD.md](../01-PRD.md) §2 · [tasks/milestones.md](../../tasks/milestones.md) ·
[tasks/backlog.md](../../tasks/backlog.md)
