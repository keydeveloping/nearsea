# Security Roadmap

> Pembagian pekerjaan keamanan per fase. Fase ini = **Phase A+B** (riset & pra-implementasi). Tidak ada Phase C+ yang boleh diklaim selesai.
> Setiap fase punya **Owner** (peran pemilik, bukan orang) dan **Exit criteria** (kondisi yang bisa dibuktikan). Milestone: [tasks/milestones.md](../../tasks/milestones.md).
>
> **⚠️ ADR-016 (ronde 17):** proyek ini **pembelajaran/portofolio**, bukan produk. Karena itu
> **Phase E (audit eksternal) dan Phase F (production hardening) TIDAK WAJIB** — keduanya berlaku
> hanya **bila** proyek beralih ke jalur produk nyata. **Phase A–D tetap berlaku** (inti pembelajaran
> keamanan kontrak). Tanpa audit, dokumen **dilarang** menyatakan proyek "aman" atau "siap produksi".

## Phase A — Research (SEKARANG, status)

**Owner**: security (MVP: PLATFORM_OWNER)

- [x] Asset inventory, trust boundaries, threat model, catastrophic scenarios
- [x] Arsitektur auth/signature/order/contract/invariants
- [x] Access control, key management (MVP), API/metadata/indexer/FE/DB/fraud/admin/infra/CI-CD models
- [x] Security requirements register (SEC-*) + gap analysis
- [ ] Finalisasi open questions (lihat gap-analysis) — **sisa Phase A**

**Exit criteria**: semua gap G1..G16 punya status (resolved / task / ⏳ open-by-design dengan owner+date); register SEC-* lengkap dengan Owner+Target; tidak ada open question tanpa klasifikasi. **Item terbuka Phase A**: finalisasi open questions (G8/G11/G12/G13/G14/G15 tetap terlihat di [security-gap-analysis.md](./security-gap-analysis.md)).

## Phase B — Pre-Implementation (sebelum/paralel Fase 1 coding)

**Owner**: security + contract

- [x] Finalisasi ADR-010..015 (semua Accepted; ADR-013 mainnet model diputuskan: Sputnik DAO V2)
- [x] Invariants → diterjemahkan jadi test plan di testing/test-cases
- [x] Keputusan tooling invariant test: near-workspaces + quickcheck/proptest (near-prop pattern) + cargo-fuzz
- [x] Keputusan pola multisig mainnet: Sputnik DAO V2 council 2-of-3 (legacy 2FA deprecated)
- [x] Keputusan reproducible build: NEP-330 + cargo-near (SEC-CONTRACT-006 diperkuat)
- [ ] Audit requirements ditetapkan final (kandidat sudah riset: OtterSec/Halborn/Block Security/Blocksec — pilihan & budget = keputusan bisnis)

**Exit criteria**: semua requirement P0 punya status minimal `DECIDED` sebelum task dimulai (khususnya SEC-ORDER-003, SEC-CONTRACT-002/009/010); scope audit (kontrak market + launchpad) disetujui; test plan invariants ada di [testing/test-cases.md](../testing/test-cases.md).

## Phase C — Implementation (Fase 1+ coding)

**Owner**: contract + backend + frontend + infra

- Semua requirement P0 READY-FOR-IMPLEMENTATION harus diimplement + test sejak awal (bukan retrofit)
- Security gates di CI (SEC-CICD-001) aktif sejak scaffold

**Exit criteria**: semua SEC-* P0 (MVP) punya test TC-* hijau; gate CI (gitleaks, audit, branch protection) aktif dan terbukti memblokir; tidak ada P0 tersisa `PROPOSED` tanpa justifikasi tertulis.

## Phase D — Internal Security Testing

**Owner**: security + contract

- Unit + integration (near-workspaces), property/fuzz pada invariants
- SAST: cargo clippy/audit, npm audit; DAST ringan (header/CSP scan, authz test)
- Restore drill DB + drill pause/unpause (bukti untuk SEC-DB-004, SEC-IR-001)

**Exit criteria**: drill restore DB sukses terdokumentasi (SEC-DB-004 naik dari BLOCKED); drill pause/unpause testnet lulus (SEC-IR-001); fuzz/property test hijau pada semua invariant; hasil SAST/DAST bersih atau tercatat.

## Phase E — External Audit (mainnet gate, M4)

**Owner**: security (pemilik kontrak keputusan: bisnis)

- Wajib diaudit independen: **market contract** (settlement/payout/escrow) dan **launchpad/factory** (mint/alokasi/allowlist)
- Direkomendasikan: auth API + flow escrow-refund
- Kandidat auditor sudah diriset (katalog Veridise `near_audits`): OtterSec, Halborn, Block Security, Blocksec — pilihan final & biaya = keputusan bisnis.

**Exit criteria**: laporan audit diterima; **semua temuan kritis/tinggi ditutup** (bukan sekadar checklist internal); re-test temuan oleh auditor; laporan + bukti penutupan diarsipkan; TASK-027 ditutup. Pilihan auditor final = **G8** (keputusan bisnis, M4).

## Phase F — Production Hardening (pasca-mainnet)

**Owner**: security + infra (mainnet: FINANCE ≠ SECURITY ≠ DEVELOPER)

- Monitoring ketat (anomali settlement, owner tx alert)
- Incident response drill
- Implementasi/aktivasi Sputnik DAO 2-of-3 + timelock di mainnet, bug bounty (PROPOSED, keputusan bisnis)

**Exit criteria**: monitoring + alert Telegram aktif dengan ambang terkalibrasi; drill IR berjalan terjadwal (incident-response §12); DAO 2-of-3 + timelock aktif di mainnet (SEC-CONTRACT-012/SEC-KEY-002); program bug bounty diputuskan (G14 — keputusan bisnis).

## Dependency graph (fase → prasyarat)

```text
Phase A (research)
   │  exit: gap terklasifikasi, register lengkap
   ▼
Phase B (decisions)  ◄── ADR-010..015 Accepted
   │  exit: P0 ≥ DECIDED, scope audit disetujui
   ▼
Phase C (implementation)  ◄── scaffold TASK-001, CI gates aktif
   │  exit: P0 punya test TC-* hijau
   ▼
Phase D (internal testing)  ◄── butuh infra TASK-028 (drill DB) + kontrak (drill pause)
   │  exit: drill DB + drill pause lulus
   ▼
Phase E (external audit)  ◄── butuh Phase C+D selesai + kontrak beku (code freeze)
   │  exit: temuan kritis/tinggi ditutup (TASK-027)
   ▼
Phase F (production hardening)  ◄── butuh M4 (deploy mainnet) + DAO aktif
   │  exit: monitoring + DAO + IR drill berjalan
   ▼
   (loop) temuan produksi → kembali ke C/D untuk patch
```

- **Ketergantungan keras**: E tidak boleh mulai sebelum D lulus; F tidak boleh mulai sebelum M4.
- **Ketergantungan lunak**: C bisa paralel B (coding dimulai saat P0 sudah DECIDED).
- SEC-CICD-001 (gate CI) adalah prasyarat lintas-fase: aktif di Phase C dan mengikat C–F.

## SEC-ID coverage matrix (fase penutup)

> Fase = kapan requirement **wajib tertutup** (test/drill/setup terbukti). Status per ID di [security-requirements.md](./security-requirements.md).

| Fase | SEC-ID |
|---|---|
| C (MVP, P0) | SEC-AUTH-001/002/004/006, SEC-SIGN-001, SEC-ORDER-001/002/004/005/006, SEC-CONTRACT-001/002/003/004/005/007/011, SEC-META-001, SEC-API-001/002, SEC-FE-001/002, SEC-ADMIN-001/002/003, SEC-INFRA-001/002, SEC-DB-001/002/003, SEC-CICD-001/002/003, SEC-IR-002/003, SEC-KEY-001 |
| C (MVP, P1) | SEC-AUTH-003/005, SEC-ORDER-003, SEC-CONTRACT-008/009/010 |
| D (internal testing) | SEC-DB-004 (restore drill), SEC-IR-001 (drill pause/unpause), SEC-CONTRACT-006 (verifikasi hash — latihan) |
| E (external audit) | TASK-027 (market + launchpad) — menutup temuan kontrak; syarat SEC-CONTRACT-012 |
| F (pasca-mainnet) | SEC-CONTRACT-006 (mainnet), SEC-CONTRACT-012, SEC-KEY-002, SEC-IR-001 (produksi), G14 (bug bounty) |
| fase 2 (M2) | SEC-META-002, SEC-INDEX-001/002 |

- Tidak ada ID di luar register (semua SEC-* terdaftar); matrix ini tidak menyalin status — hanya memetakan fase.
- ID yang muncul di >1 fase berarti **gate berulang** (mis. SEC-CONTRACT-006: latihan di D, wajib di F).

## Status

- Phase A+B — **sedang berjalan** (sisa Phase A: finalisasi open questions).
- Phase C–F — **belum boleh diklaim selesai**; exit criteria di atas = syarat klaim.
- Owner & exit criteria — **PROPOSED** (peran final saat mainnet: FINANCE ≠ SECURITY ≠ DEVELOPER).
