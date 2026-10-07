# Threat Model

> Model ancaman lengkap (STRIDE + Web3/NEAR-specific). Struktur dokumen keamanan: ancaman ini dijawab oleh kontrol di dokumen turunan — lihat DOCUMENTATION-MAP.
> Label: FACT = properti protokol; DESIGN = keputusan kita; RESEARCH REQUIRED = belum selesai.
> **Skala**: Likelihood (L) = Rendah / Sedang / Tinggi; Impact (I) = Rendah / Sedang / Tinggi / Kritis; **Residual** = risiko sisa setelah kontrol (Rendah / Sedang / Tinggi). Kolom **Status** = status SEC-ID terkait (`DECIDED` / `PROPOSED` / `READY-FOR-IMPLEMENTATION` — enum di [security-requirements.md](./security-requirements.md)).

## Aset yang dilindungi

→ Lengkap di [asset-inventory.md](./asset-inventory.md). Inti: NFT user (selalu di wallet), escrow Ⓝ offer, dana settlement, royalti, owner key, reputasi platform.

## Trust boundaries

→ Formal di [trust-boundaries.md](./trust-boundaries.md). Yang paling menentukan desain: **Market ↔ NFT Contract** (market menerima interaksi kontrak NFT mana pun — platform terbuka) dan **Browser ↔ metadata** (semua konten untrusted).

## Ancaman per kategori (STRIDE + Web3)

### Authentication

| Ancaman | Kontrol | SEC-ID | Status | L | I | Residual | Test |
|---|---|---|---|---|---|---|---|
| Signature replay (API) | nonce sekali pakai + TTL, consumed atomic | SEC-AUTH-001 | READY-FOR-IMPLEMENTATION | Sedang | Tinggi | Rendah | TC-028 |
| Nonce reuse | challenge nonce dikonsumsi atomik di DB | SEC-AUTH-001 | READY-FOR-IMPLEMENTATION | Sedang | Tinggi | Rendah | TC-028 |
| Session fixation | JWT pendek scoped, refresh rotasi | SEC-AUTH-003/006 | READY-FOR-IMPLEMENTATION | Sedang | Tinggi | Rendah | TC-029, TC-030 |
| Wrong-domain signature | **format utama NEP-413**: `recipient` = domain; fallback custom: domain di dalam message | SEC-AUTH-002 | DECIDED | Sedang | Tinggi | Rendah | TC-028 (TV-3) |
| Wrong-chain signature | network eksplisit di message (fallback) + transaksi dana tidak via API | SEC-AUTH-002 | DECIDED | Rendah | Sedang | Rendah | TC-028 |
| Wallet spoofing | verifikasi key↔account via RPC `view_access_key_list` | SEC-AUTH-004 | READY-FOR-IMPLEMENTATION | Sedang | Tinggi | Rendah | TC-028 |
| Brute force / spam login | rate limit + lockout (5 gagal/menit) | SEC-AUTH-005 | PROPOSED | Sedang | Sedang | Rendah | TC-027 |

Replay tx on-chain = protokol (FACT: nonce per access key + recent block hash) — bukan ancaman aplikasi.

### Order

| Ancaman | Kontrol | SEC-ID | Status | L | I | Residual | Test |
|---|---|---|---|---|---|---|---|
| Replay/double execution | order = state on-chain (ADR-012) — tidak ada signature order; INV-008/009 | SEC-SIGN-001, SEC-ORDER-005 | DECIDED | Rendah | Tinggi | Rendah | TC-017, TC-018 |
| Overfill/partial-fill bug | tidak ada partial fill | SEC-ORDER-005 | DECIDED | Rendah | Sedang | Rendah | TC-014 |
| Stale orders | dual verification saat settle + auto-stale | SEC-ORDER-004 | DECIDED | Sedang | Sedang | Rendah | TC-006 |
| Cancellation races | optimistic removal + atomic resolve | SEC-ORDER-001/006 | DECIDED | Sedang | Sedang | Rendah | TC-016 |
| Recipient substitution | payout dari `nft_transfer_payout`, tervalidasi (INV-002/003) | SEC-ORDER-002, SEC-CONTRACT-005 | DECIDED | Rendah | Kritis | Rendah | TC-003 |
| Private listing: buyer tak berhak | cek `allowed_buyer` on-chain saat buy (INV-026) | — | DECIDED | Rendah | Sedang | Rendah | TC-011 |
| Bundle partial failure | pre-validasi all-or-nothing + refund; residual mid-loop → kompensasi G7 (INV-025) | SEC-ORDER-001 | DECIDED | Rendah | Tinggi | Rendah | TC-009, TC-010 |
| Price manipulation | harga on-chain; FE re-verify via view call sebelum sign | SEC-ORDER-003 | PROPOSED | Sedang | Tinggi | Rendah | TC-022 |
| Kill switch tidak tersedia saat exploit aktif | **Pausable** — MVP owner-only; mainnet guardian `pause_callers` | SEC-CONTRACT-007 | DECIDED | Rendah | Kritis | Rendah | TC-012 |

### Smart contract (adaptasi NEAR)

| Ancaman | Kontrol | SEC-ID | Status | L | I | Residual | Test |
|---|---|---|---|---|---|---|---|
| Reentrancy | FACT: cross-contract NEAR async — padanan = callback manipulation → `#[private]` + validate promise results | SEC-CONTRACT-003 | DECIDED | Rendah | Tinggi | Rendah | review + test |
| Arbitrary calls / unauthorized withdrawal | INV-014: tidak ada transfer ke alamat arbitrer | SEC-CONTRACT-003 | DECIDED | Rendah | Kritis | Rendah | review + fuzz |
| Access-control failure | Owner pattern + matrix; no admin catch-all | SEC-ADMIN-001 | PROPOSED | Rendah | Tinggi | Rendah | authz test |
| Initialization takeover | `#[init]` + `PanicOnDefault` | SEC-CONTRACT-002 | PROPOSED | Rendah | Kritis | Rendah | test + review |
| Upgrade takeover | MVP diterima (testnet); mainnet Sputnik DAO V2 2-of-3 + timelock (ADR-013) | SEC-CONTRACT-012 | DECIDED | Rendah | Kritis | Sedang (mainnet) / Tinggi (MVP testnet) | drill + review |
| Malicious callbacks (kontrak NFT jahat) | **G1 DECIDED — permissionless + mitigasi** (payout tervalidasi, gas caps, optimistic removal+revert, moderasi display-layer); residual = DoS/UX, **bukan** kehilangan dana | SEC-CONTRACT-003/005/010 | DECIDED | Sedang | Sedang | Sedang | TC-003 + fuzz |
| Accounting/rounding | INV-001..004, checked math | SEC-CONTRACT-005 | DECIDED | Rendah | Kritis | Rendah | TC-003, TC-015 |
| Approval abuse | 1-yocto + per-token approval + revoke saat cancel | SEC-CONTRACT-001 | DECIDED | Rendah | Sedang | Rendah | TC-002 |
| DoS via gas/external contract | INV-021 batas statis; batas gas tx 300 Tgas (FACT) + cap statis per operasi | SEC-CONTRACT-010 | PROPOSED | Sedang | Sedang | Sedang | TC-014 + fuzz |

### Token (adaptasi: kontrak NFT/FT jahat)

| Ancaman | Kontrol | SEC-ID | Status | L | I | Residual | Test |
|---|---|---|---|---|---|---|---|
| Malicious NFT contract (payout bohong, hooks, return value aneh) | payout validation; promise failure → refund; **G1 DECIDED — permissionless + mitigasi** | SEC-CONTRACT-005 | DECIDED | Sedang | Sedang | Sedang | TC-003 + fuzz |
| Fee-on-transfer/rebasing | TIDAK relevan untuk NEP-171 (unit NFT); FT fase 2 → `ft_on_transfer` cek saldo aktual | — | ⏳ open-by-design (fase 2) | — | Sedang | — | fase 2 |
| Non-standard return | resolve callback memvalidasi ketat; promise gagal → refund | SEC-CONTRACT-003 | DECIDED | Sedang | Sedang | Rendah | TC-003 |
| Blacklist/pause behavior | treat as failed transfer → revert & refund | SEC-ORDER-001 | DECIDED | Rendah | Sedang | Rendah | TC-003 |

### Backend/API

→ BOLA, authz, injection, SSRF, exhaustion, race: [api-security-architecture.md](./api-security-architecture.md). SSRF = kosong di MVP (ADR-014).

| Ancaman | Kontrol | SEC-ID | Status | L | I | Residual | Test |
|---|---|---|---|---|---|---|---|
| BOLA / akses data orang lain | resource scoped `sub`; chain data publik (FACT) | SEC-API-002 | PROPOSED | Rendah | Sedang | Rendah | TC-031, TC-032 |
| Broken function authorization | guard middleware + test authz per endpoint | SEC-API-002 | PROPOSED | Sedang | Tinggi | Rendah | TC-034..TC-039 |
| Injection (SQL) | ORM/parameterized only (Prisma) | SEC-API-002 | PROPOSED | Rendah | Tinggi | Rendah | review |
| Resource exhaustion | rate limit + pagination + body cap (16 KB) | SEC-API-001 | PROPOSED | Sedang | Sedang | Rendah | TC-027, TC-033 |

### Frontend

→ XSS, SVG, phishing, address substitution, tx manipulation: [frontend-security.md](./frontend-security.md) + [metadata-security.md](./metadata-security.md). Mitigasi kunci: preview=on-chain (SEC-ORDER-003) — user sign apa yang ditampilkan on-chain.

| Ancaman | Kontrol | SEC-ID | Status | L | I | Residual | Test |
|---|---|---|---|---|---|---|---|
| XSS via metadata/SVG | `<img>` saja, tanpa `innerHTML`, CSP strict | SEC-META-001, SEC-FE-001 | PROPOSED | Sedang | Tinggi | Rendah | CSP report + E2E |
| Address/tx substitution di UI | preview=on-chain + full address + copy | SEC-ORDER-003, SEC-FE-002 | PROPOSED | Sedang | Kritis | Rendah | TC-040 |
| Supply chain FE | lockfile + audit gate + tanpa third-party script | SEC-CICD-001 | PROPOSED | Sedang | Kritis | Sedang | CI review |
| Phishing mirror | domain resmi + interstitial link eksternal | SEC-IR-002 | PROPOSED | Sedang | Sedang | Sedang | — |

### Indexer (fase 2)

→ Reorg (TIDAK ada pasca-final — FACT), duplicate, stale, korup: [indexer-security.md](./indexer-security.md); prinsip SEC-INDEX-001.

| Ancaman | Kontrol | SEC-ID | Status | L | I | Residual | Test |
|---|---|---|---|---|---|---|---|
| Event palsu / poisoning | filter emitter + `standard` kita; ingest final-only | SEC-INDEX-001/002 | PROPOSED/DECIDED | Sedang | Sedang | Rendah | reconciliation |
| Duplicate/inconsistent | dedup `(receipt_id, event_index)` | SEC-INDEX-002 | DECIDED | Sedang | Sedang | Rendah | TC-024 (pola) |
| Ingestion bug → state korup | rebuild dari `events_raw` | SEC-INDEX-001 | PROPOSED | Sedang | Sedang | Rendah | reconciliation |
| Stale/lag | kolom `ingested_height` + indikator UI | SEC-INDEX-001 | PROPOSED | Sedang | Sedang | Rendah | lag alert |

### Infrastructure & CI/CD

→ VPS hardening, secrets, supply chain: [infrastructure-security.md](./infrastructure-security.md), [cicd-security.md](./cicd-security.md).

| Ancaman | Kontrol | SEC-ID | Status | L | I | Residual | Test |
|---|---|---|---|---|---|---|---|
| VPS compromise | hardening checklist (SSH key-only, fw, updates) | SEC-INFRA-001 | PROPOSED | Sedang | Kritis | Sedang | scan + review |
| DB compromise | bind localhost, app role non-superuser | SEC-INFRA-002 | PROPOSED | Rendah | Tinggi | Rendah | review |
| Secret leak (git) | `.gitignore` ketat + gitleaks + rotasi | SEC-CICD-002 | PROPOSED | Sedang | Kritis | Rendah | gitleaks |
| Dependency/artifact swap | lockfile + audit + post-deploy hash (NEP-330) | SEC-CICD-001, SEC-CONTRACT-006 | PROPOSED/DECIDED | Sedang | Kritis | Sedang | CI review |
| Workflow modification | perubahan file workflow = review wajib; branch protection | SEC-CICD-003 | PROPOSED | Sedang | Kritis | Rendah | review konfigurasi |

### Marketplace abuse

→ Wash trading, Sybil, fake collection, impersonation: [fraud-and-abuse.md](./fraud-and-abuse.md) — fraud layer tidak pernah menyentuh settlement.

| Ancaman | Kontrol | SEC-ID | Status | L | I | Residual | Test |
|---|---|---|---|---|---|---|---|
| Wash trading | self-buy ditolak (INV-023); heuristik fase 2 (display-only) | — | DECIDED | Tinggi | Rendah | Sedang | fase 2 |
| Sybil / bot mint | allowlist phase on-chain + max/wallet | — | DECIDED | Sedang | Sedang | Sedang | TC-007 |
| Fake collection / impersonation | report + similarity check + verified badge | SEC-ADMIN-001 | PROPOSED | Tinggi | Rendah | Sedang | TC-036 |
| Spam listing/report | min harga 0.01 Ⓝ + storage pre-deposit + rate limit | SEC-API-001 | PROPOSED | Sedang | Rendah | Rendah | TC-033 |

## Attack trees

> Notasi: `AND` = semua anak harus berhasil; `OR` = salah satu cukup. Daun = aksi penyerang; label `[kontrol]` = mitigasi yang memutus cabang.

### AT-1 — Owner key compromise (CS-1, impact Kritis)

```text
GOAL: kendali penuh kontrak market/factory (pause, fee, treasury, upgrade)
├── OR
│   ├── AND VPS compromise
│   │   ├── RCE via bug app/API (validasi input gagal)         [kontrol: hardening, input validation]
│   │   ├── Baca `~/.near-keys/` (chmod 600, app user terpisah)[kontrol: permission 600 + user OS terpisah]
│   │   └── Key tidak terenkripsi / LUKS tak tersedia          [kontrol: enkripsi disk — RESEARCH]
│   ├── AND Seed phrase bocor
│   │   ├── Phishing tim / screenshot / chat                   [kontrol: seed offline 2 lokasi, tanpa digital]
│   │   └── Akses fisik salinan kertas                         [kontrol: 2 lokasi terpisah]
│   ├── AND CI/pipeline leak
│   │   └── Key pernah lewat CI/deploy script                  [kontrol: key TIDAK pernah di CI — SEC-KEY-001]
│   └── AND Governance (mainnet)
│       └── Rebut 2 dari 3 council member                      [kontrol: timelock 24 jam + monitoring]
├── AND [setelah key didapat] gunakan key untuk aksi finansial
│   ├── Upgrade kontrak ke kode jahat                          [kontrol mainnet: DAO 2-of-3 + timelock]
│   ├── Drain escrow offer                                     [kontrol: offer expire 7 hari + user cancel_offer]
│   └── Naikkan fee / tarik treasury                           [kontrol: MAX_FEE_BPS immutable + INV-005]
└── BATAS STRUKTURAL: NFT user TIDAK PERNAH dipegang kontrak (ADR-007) → kepemilikan NFT tetap aman.
```

Residual: Sedang (mainnet, timelock) / Tinggi (MVP testnet tanpa dana). Pemutus utama = key hygiene + timelock + monitoring.

### AT-2 — Settlement manipulation / payout salah (CS-3, impact Kritis)

```text
GOAL: membuat dana terdistribusi ke penyerang / jumlah salah
├── OR
│   ├── AND Kendalikan input payout
│   │   ├── Kontrak NFT jahat mengembalikan payout bohong      [kontrol: payout divalidasi market — INV-002/003]
│   │   └── Duplikat receiver                                 [kontrol: merge per receiver — INV-003]
│   ├── AND Eksploitasi aritmetika
│   │   ├── Overflow/underflow u128                           [kontrol: overflow-checks + checked ops]
│   │   └── Fee dihitung dua kali/kurang                      [kontrol: fee dipotong sekali, sum ≤ harga−fee]
│   ├── AND Eksploitasi lifecycle
│   │   ├── Beli listing CANCELLED/EXPIRED/STALE              [kontrol: dual verification — INV-008/016]
│   │   └── Accept offer kedaluwarsa                          [kontrol: cek `expires_at` — INV-010]
│   └── AND Race
│       └── Dua pembeli membayar satu NFT                     [kontrol: unique key + optimistic removal — INV-007]
├── AND transfer berhasil
│   └── Transfer on-chain FINAL (FACT) → tidak bisa dibalik   [kontrol: validasi SEBELUM transfer]
└── DETEKSI: reconcile `sum(payout) == harga − fee`; alert selisih > 1 yocto.
```

Residual: Rendah (INV-001..004 + fuzz + test matriks royalti). Test: TC-003, TC-015.

### AT-3 — Frontend supply chain compromise (CS-6, impact Kritis)

```text
GOAL: user menandatangani tx jahat dari UI yang tampak asli
├── OR
│   ├── AND Dependency malicious (npm)
│   │   ├── Package typosquat / maintainer diambil alih      [kontrol: lockfile + audit + minimal deps]
│   │   └── Postinstall script inject receiver swap          [kontrol: review + tanpa third-party script]
│   ├── AND Akun GitHub maintainer takeover
│   │   ├── Push langsung ke branch deploy                   [kontrol: branch protection + 2FA GitHub]
│   │   └── Ubah workflow deploy                             [kontrol: perubahan workflow = review wajib]
│   └── AND CDN/runtime script
│       └── Script pihak ketiga menukar receiver             [kontrol: CSP strict, tanpa CDN runtime — SEC-FE-001]
├── AND user sign tanpa sadar
│   ├── Preview tidak di-reverify on-chain                   [kontrol: preview=on-chain — SEC-ORDER-003]
│   └── Wallet tidak menampilkan receiver berbeda            [kontrol: user edukasi; wallet menampilkan receiver]
└── DETEKSI: CSP report logs; anomali Dependabot; laporan user.
```

Residual: Sedang (rantai build panjang). Pemutus = preview on-chain + CSP + review diff + SRI.

## Skenario bencana paling berbahaya

→ [catastrophic-failure-scenarios.md](./catastrophic-failure-scenarios.md). **CS-1 (owner key compromise)** = worst case; mitigasi struktural: NFT tidak pernah dipegang kontrak + MAX_FEE_BPS immutable + (mainnet) Sputnik DAO V2 2-of-3 + timelock.

## Matriks prioritas ancaman (likelihood × impact)

| Ancaman | L (MVP) | I | Prioritas | Residual | Test |
|---|---|---|---|---|---|
| Owner key / upgrade | Rendah | Kritis | Tertinggi | Sedang (mainnet) | drill |
| Settlement/payout salah | Rendah | Kritis | Tinggi | Rendah | TC-003/015 |
| Drain escrow | Rendah | Kritis | Tinggi | Rendah | TC-018 |
| FE supply chain | Sedang | Kritis | Tinggi | Sedang | CI review |
| CI/CD compromise | Rendah | Kritis | Tinggi | Sedang | review |
| Metadata XSS/SSRF | Sedang | Sedang/Tinggi | Sedang | Rendah | CSP report |
| Bypass lifecycle | Sedang | Sedang | Sedang | Rendah | TC-006/016 |
| Indexer poisoning | — (fase 2) | Sedang | Fase 2 | Rendah | reconciliation |

## Out of scope

- Keamanan wallet pihak ketiga; ketersediaan gateway IPFS publik; spam report masif (rate-limited); kontrak pihak ketiga yang diintegrasikan nanti (bridge/on-ramp); eksploitasi OS/patch level VPS provider.

## Prosedur pembaruan threat-model (ronde 16)

> Kapan dan bagaimana tabel ancaman + attack tree di dokumen ini direvisi — supaya threat-model
> tidak menjadi dokumen statis yang usang.

```text
PICU PEMBARUAN bila salah satu:
1. Fitur baru / perubahan arsitektur   → identifikasi ancaman baru (STRIDE + Web3 per kategori),
                                          tambah baris + SEC-ID + test, evaluasi CS terkait.
2. Temuan audit / bug bounty / insiden → tambah ancaman terbukti + likelihood/impact aktual;
                                          perbarui attack tree (AT-*) yang bersangkutan.
3. Riset ekosistem baru (exploit NEAR) → RESEARCH.md dulu (fakta), lalu threat-model.
4. Perubahan SEC-*/drill               → sinkron kolom Test & status; jangan biarkan sel kosong menggantung.

LANGKAH:
1. Tambah/ubah baris ancaman di tabel kategori yang tepat (likelihood, impact, residual, test).
2. Ancaman baru bernilai tinggi → buat attack tree (AT-xx) + pertimbangkan CS baru
   (catastrophic-failure-scenarios.md — format 9-field).
3. Wajib ada kontrol → tambah/perbarui SEC-* di security-requirements.md
   (prosedur: register bagian "Prosedur: menambah SEC requirement baru").
4. Update matriks prioritas di atas + jalankan checklist crosscheck DOCUMENTATION-MAP.
5. Catat pemicu + tanggal di riwayat sinkronisasi DOCUMENTATION-MAP.
```
