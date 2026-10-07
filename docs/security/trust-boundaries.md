# Trust Boundaries

> Setiap batas: apa yang melintas, siapa yang kontrol, autentikasi/otorisasi, validasi, kegagalan, dan apa yang wajib di-log.
> Diagram konsolidasi ada di bagian 11; **attack tree per boundary di bagian 12**; **asersi testable per boundary di bagian 13**.

## 1. User → Wallet (trust boundary tertinggi)

- **Melintas**: intent user (klik Buy/List/Offer) → payload transaksi yang di-sign.
- **Kontrol**: user penuh; kita TIDAK bisa memaksa signing.
- **AuthN/AuthZ**: key wallet user (ed25519), nonce + block hash level protokol (FACT).
- **Validasi**: FE wajib menampilkan argumen final (harga, alamat) — tapi keputusan signing ada di wallet; wallet menampilkan deposit & receiver.
- **Gagal**: user menolak; mismatch network; payload berubah setelah ditampilkan.
- **Log**: FE log event UI (tanpa secret).

## 2. Wallet → NEAR Chain (transaksi)

- **Melintas**: signed transaction → RPC → pool → receipt chain.
- **Kontrol**: validator; user; KITA TIDAK ADA di jalur ini (non-custodial penuh).
- **Validasi on-chain**: kontrak = gatekeeper sebenarnya (assert deposit, ownership, approval, phase, payout).
- **Gagal**: tx gagal/panic → state revert; refund semantik receipt.
- **Log**: tx hash selalu ditampilkan & disimpan di FE untuk rekonsiliasi.

## 3. Frontend → Market/NFT Contract (view calls via RPC)

- **Melintas**: query read-only (`get_sale`, `nft_token`, `get_offers`).
- **Kontrol**: RPC provider pihak ketiga — **bisa salah/malicious** (data bukan dana).
- **AuthN**: tidak ada (publik). **AuthZ**: tidak relevan (read-only).
- **Validasi**: tampilan harga/ownership SELALU di-reverify via view call tepat sebelum transaksi; angka dari cache/NearBlocks tidak boleh jadi dasar settlement.
- **Gagal**: RPC down/lying → jangan pernah izinkan transaksi berbasis data yang tidak konsisten.
- **Log**: latency/error per provider.

## 4. Browser → Frontend (konten untrusted dari chain)

- **Melintas**: metadata token (media URL, deskripsi, extra JSON), nama koleksi — **semuanya user-generated & untrusted**.
- **Kontrol**: kreator koleksi mana pun (platform terbuka).
- **Validasi**: frontend-security.md — render via `<img>` saja, gateway IPFS allowlist, CSP strict, tanpa `innerHTML`.
- **Gagal**: SVG/script injection, phishing via nama/deskripsi, media 404.
- **Log**: violation CSP.

## 5. Browser → Report API (VPS)

- **Melintas**: report, profil, admin actions (JSON).
- **AuthN**: signature wallet **NEP-413 utama / custom fallback** (domain-bound, TTL 5 menit, sekali pakai) → session token (ADR-011).
- **AuthZ**: endpoint user vs admin (allowlist DB).
- **Validasi**: strict schema; rate limit; idempotency nonce.
- **Gagal**: replay challenge, spam report, privilege escalation.
- **Log**: setiap write dengan account_id + nonce hash (bukan signature mentah).

## 6. Report API → PostgreSQL

- **Melintas**: kueri SQL via ORM (Prisma).
- **Kontrol**: kita; akses hanya dari API process (least privilege, user DB non-superuser).
- **Validasi**: constraint + unique index; migrasi versioned.
- **Gagal**: injection (mitigasi ORM + parameterized), koneksi bocor.
- **Log**: query lambat, error; admin actions audit table.

## 7. Market Contract ↔ NFT Contract (receipt lintas-kontrak — boundary unik NEAR)

- **Melintas**: `nft_token`, `nft_is_approved`, `nft_transfer_payout` (market→NFT) dan `nft_on_approve` (NFT→market).
- **Kontrol**: **kontrak NFT apa pun bisa berinteraksi** (platform terbuka!) — termasuk kontrak malicious.
- **AuthN (dua mekanisme per arah)**:
  - Market→NFT: market hanya mempercayai hasil promise yang ia panggil sendiri; callback internal `#[private]` (hanya self).
  - NFT→market (`nft_on_approve`): call MASUK dari kontrak NFT mana pun (predecessor = kontrak NFT, TIDAK #[private]) → market wajib memvalidasi payload NEP-178 (owner_id, approval_id, msg) dan bahwa predecessor adalah kontrak NFT yang sah untuk token tersebut.
- **Validasi**: payout dari kontrak NFT = untrusted → divalidasi ketat (≤ harga, ≤10 penerima, sisa 0..1 yocto) sebelum distribusi; gas caps.
- **Gagal**: kontrak NFT jahat: return payout bohong, gas bomb, revert transfer setelah state diubah (→ optimistic removal + revert pattern), event palsu.
- **Log**: event `market_*` + kegagalan promise.

## 8. Developer → Git → CI/CD → VPS (supply chain)

- **Melintas**: commit → build (wasm/bundle) → artifact → deploy via SSH.
- **Kontrol**: maintainer + GitHub infra.
- **AuthN**: branch protection, signed commits (RECOMMENDED, belum aktif); deploy key sebagai GitHub secret.
- **Validasi**: test gate (workspaces), `cargo audit`/`npm audit`, **verifikasi code hash kontrak post-deploy** (SEC-CONTRACT-006).
- **Gagal**: dependency compromise, secret leak, build tanpa review.
- **Log**: audit log GitHub + auth.log VPS.

## 9. Admin → Panel → Privileged API

- **Melintas**: keputusan moderasi (hide/verified).
- **AuthN**: signature wallet + allowlist; destructive op = step-up re-sign (SEC-ADMIN-003).
- **Gagal**: allowlist manipulation (butuh DB access), session theft.
- **Log**: audit table `admin_audit` wajib.

## 10. Indexer → Chain (fase 2)

- **Melintas**: block/receipt/event ingestion (sumber stream: **Neardata** — ADR-015).
- **Kontrol**: tim (self-host, container terpisah).
- **AuthN/Validasi**: filter emitter = kontrak kita; dedup (receipt_id, event_index); ingest final-only untuk data uang.
- **Gagal**: ingestion bug, lag, RPC provider tidak sehat → indikator lag + reconciliation (indexer-security.md).
- **Log**: lag ingestion, mismatch reconciliation.
- **Aturan mutlak**: indexer = proyeksi pencarian; **TIDAK PERNAH** otoritas ownership/settlement (SEC-INDEX-001). Verifikasi uang selalu ke kontrak via RPC.
- **Catatan DNS**: domain/registrar bukan boundary runtime, tapi DNS hijack = full-control phishing — mitigasi di catastrophic-failure-scenarios.md CS-8 (registrar lock + cron dig).

## 11. Final Security Architecture Diagram (konsolidasi)

```text
                    USER
                     │  (intent)
                WALLET (sign)          ── user key; nonce+blockhash (protokol)
                     │ signed tx (TIDAK lewat server kita)
                     ▼
                NEAR RPC ──────────► BLOCKCHAIN
                     ▲                  │
        view re-verify│                 receipts
                     │                  ▼
   ┌──────────FRONTEND (browser)   SMART CONTRACTS
   │                 │             (NFT / Market+Factory+Launchpad)
   │                 │ JSON+sig      │   ▲
   │                 ▼               │   │ cross-contract receipts
   │          REPORT API (VPS) ──────┘   │ (boundary #7)
   │          │        │                 │
   │          ▼        ▼                 │
   │     PostgreSQL   NearBlocks ────────┘ (riwayat, non-otoritatif)
   │          ▲
   │     ADMIN PANEL (signature + allowlist + audit)
   │
   └─ CSP/media gateway allowlist (metadata untrusted)

DEVELOPER → GIT → CI/CD (test+audit gates) → SSH → VPS → PRODUCTION
PLATFORM OWNER → OWNER KEY (MVP: VPS single-key; mainnet: Sputnik DAO V2 council 2-of-3 + timelock — DECIDED ADR-013)
```

## 12. Attack tree per boundary

> Notasi: `AND` = semua harus berhasil; `OR` = salah satu cukup; `[kontrol]` = pemutus cabang. Boundary mengacu nomor bagian di atas.

### AT-B1 — Boundary #1 User → Wallet

```text
GOAL: membuat user menandatangani payload yang bukan maksudnya
├── OR
│   ├── AND UI menampilkan argumen berbeda dari yang di-sign
│   │   ├── Cache basi (harga/ownership lama)              [kontrol: preview=on-chain — SEC-ORDER-003]
│   │   └── Konten untrusted menimpa tampilan              [kontrol: render teks, CSP — SEC-FE-001]
│   └── AND Wallet tidak menampilkan receiver/deposit
│       └── Payload dimanipulasi sebelum sign              [kontrol: wallet menampilkan receiver; user verifikasi]
└── BATAS: signing = keputusan user; kita tidak bisa memaksa (non-custodial).
```

### AT-B2 — Boundary #4 Browser → Frontend (metadata untrusted)

```text
GOAL: eksekusi script di browser korban lewat metadata
├── OR
│   ├── AND SVG/HTML ber-script dirender inline            [kontrol: hanya <img>, tanpa innerHTML — SEC-META-001]
│   ├── AND URL metadata menunjuk host jahat               [kontrol: gateway allowlist + CSP img-src]
│   └── AND Link eksternal tanpa peringatan                [kontrol: interstitial + rel=noopener]
└── DETEKSI: violation CSP (report-only) → alert.
```

### AT-B3 — Boundary #5 Browser → Report API

```text
GOAL: menyamar sebagai user/admin lain
├── OR
│   ├── AND Replay challenge lama                          [kontrol: nonce sekali pakai + TTL 5m — SEC-AUTH-001]
│   ├── AND Pakai signature dari domain lain               [kontrol: recipient=domain NEP-413 — SEC-AUTH-002]
│   ├── AND Klaim public_key yang bukan miliknya           [kontrol: view_access_key_list — SEC-AUTH-004]
│   └── AND Curi session token                             [kontrol: TTL 15m + rotasi + denylist — SEC-AUTH-006]
└── AND lolos ke endpoint admin
    └── Butuh scope admin + allowlist + step-up            [kontrol: SEC-ADMIN-001/003]
```

### AT-B4 — Boundary #7 Market ↔ NFT Contract (paling unik)

```text
GOAL: kontrak NFT jahat memaksa market transfer dana/NFT ke arah yang salah
├── OR
│   ├── AND Kirim `nft_on_approve` palsu (predecessor bukan NFT sah)
│   │   └── Market membuat listing tanpa approval nyata   [kontrol: validasi payload NEP-178 + predecessor — INV-013]
│   ├── AND Kembalikan payout bohong dari `nft_transfer_payout`
│   │   └── Dana dialihkan ke attacker                    [kontrol: payout ≤ harga−fee, ≤10 receiver, sisa ≤1y — INV-002/003]
│   ├── AND Gas bomb / revert setelah state diubah
│   │   └── State tidak konsisten                         [kontrol: optimistic removal + revert — SEC-ORDER-001]
│   └── AND Emit event format kita dari kontrak lain
│       └── Indexer salah catat                           [kontrol: filter emitter — SEC-INDEX-001]
└── BATAS STRUKTURAL: G1 DECIDED — permissionless; kontrak jahat tidak bisa paksa payout palsu
    (selalu divalidasi market). Residual = DoS/UX, bukan kehilangan dana.
```

### AT-B5 — Boundary #8 Developer → Git → CI/CD → VPS

```text
GOAL: mendeploy artifact/kontrak jahat
├── OR
│   ├── AND Dependency malicious lolos audit              [kontrol: lockfile + audit gate — SEC-CICD-001]
│   ├── AND Ubah workflow tanpa review                    [kontrol: perubahan workflow = review wajib — SEC-CICD-003]
│   ├── AND Secret ter-commit                             [kontrol: gitleaks + .gitignore — SEC-CICD-002]
│   └── AND Artifact swap saat deploy                     [kontrol: post-deploy hash NEP-330 — SEC-CONTRACT-006]
└── AND mainnet tanpa gate
    └── Deploy lolos tanpa approval                       [kontrol: manual-trigger + environment approval + timelock]
```

### AT-B6 — Boundary #9 Admin → Panel → Privileged API

```text
GOAL: admin (atau penyerang) melakukan aksi destruktif tanpa jejak
├── OR
│   ├── AND Replay step-up signature                      [kontrol: nonce per aksi — SEC-ADMIN-003]
│   ├── AND Session admin dicuri                          [kontrol: TTL 15m tanpa silent refresh]
│   └── AND Edit/hapus audit trail                        [kontrol: REVOKE UPDATE/DELETE — SEC-ADMIN-002/SEC-DB-003]
└── AND privilege escalation via allowlist
    └── Tambah diri ke allowlist                          [kontrol: 2-admin approval (mainnet) + audit]
```

## 13. Asersi testable per boundary

> Setiap asersi adalah **klaim yang bisa diuji otomatis** (sandbox/api/e2e) dan dipetakan ke SEC-ID + test. Ini kontrak antara desain boundary dan test suite.

| # | Boundary | Asersi (harus benar) | SEC-ID | Test |
|---|---|---|---|---|
| AS-1 | #1 User→Wallet | Tx yang di-sign memakai argumen hasil view-call terkini (bukan cache) | SEC-ORDER-003 | TC-022 |
| AS-2 | #1 User→Wallet | FE tidak pernah meminta seed phrase / menyimpan private key | SEC-FE-002 | review + E2E |
| AS-3 | #2 Wallet→Chain | Tx gagal → state revert; deposit kembali (tidak ada dana hangus) | SEC-ORDER-001 | TC-016 |
| AS-4 | #3 FE→Contract (view) | Settlement tidak pernah memakai angka cache/NearBlocks | SEC-ORDER-003 | review + TC-022 |
| AS-5 | #4 Browser→FE | Metadata tidak pernah dirender sebagai HTML/inline SVG | SEC-META-001 | E2E + CSP report |
| AS-6 | #5 Browser→API | Nonce hanya bisa dikonsumsi sekali; replay → 401 | SEC-AUTH-001 | TC-028 |
| AS-7 | #5 Browser→API | Message/recipient non-kanonik → 401 (memcmp penuh) | SEC-AUTH-002 | TC-028 (TV-3) |
| AS-8 | #5 Browser→API | public_key bukan milik account → 401 | SEC-AUTH-004 | TC-028 |
| AS-9 | #5 Browser→API | Scope admin tanpa allowlist → 403 | SEC-ADMIN-001 | TC-026 |
| AS-10 | #6 API→PostgreSQL | Tidak ada query raw concat; semua parameterized | SEC-API-002 | review |
| AS-11 | #6 API→PostgreSQL | `admin_audit` menolak UPDATE/DELETE | SEC-ADMIN-002, SEC-DB-003 | test DB perms |
| AS-12 | #7 Market↔NFT | `nft_on_approve` dari predecessor non-NFT → revert | SEC-CONTRACT-003 | sandbox test |
| AS-13 | #7 Market↔NFT | Payout > harga−fee / >10 receiver / sisa >1y → tolak + refund | SEC-CONTRACT-005 | TC-003, TC-015 |
| AS-14 | #7 Market↔NFT | Kontrak NFT yang revert setelah state diubah → state pulih + refund | SEC-ORDER-001 | sandbox test |
| AS-15 | #7 Market↔NFT | Event kontrak lain tidak di-ingest indexer | SEC-INDEX-001 | fase 2 test |
| AS-16 | #8 Dev→CI/CD | Deploy mainnet gagal bila hash artifact ≠ on-chain | SEC-CONTRACT-006 | CI gate |
| AS-17 | #8 Dev→CI/CD | Perubahan file workflow terdeteksi butuh review | SEC-CICD-003 | review konfigurasi |
| AS-18 | #8 Dev→CI/CD | Secret ter-commit terdeteksi gitleaks | SEC-CICD-002 | gitleaks |
| AS-19 | #9 Admin→API | Aksi destruktif tanpa step-up → 401/403, tanpa perubahan state | SEC-ADMIN-003 | TC-026 |
| AS-20 | #9 Admin→API | Setiap aksi admin sukses menulis satu baris `admin_audit` | SEC-ADMIN-002 | TC-025, TC-035..TC-039 |
| AS-21 | #10 Indexer→Chain | Ingest hanya blok final; dedup `(receipt_id,event_index)` | SEC-INDEX-002 | fase 2 test |
| AS-22 | #10 Indexer→Chain | Indexer tidak pernah jadi dasar transaksi uang | SEC-INDEX-001 | review arsitektur |
| AS-23 | #7 Market↔NFT | Pause memblokir mutasi baru tapi mengizinkan cancel/withdraw | SEC-CONTRACT-007 | TC-012 |
| AS-24 | #9 Admin→API | Break-glass menulis `admin_audit` manual + postmortem | SEC-ADMIN-002, SEC-IR-001 | drill evidence |
