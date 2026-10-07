# ADR-011: Wallet Authentication untuk API Off-chain

## Status
Accepted — **updated 2026-10-01**: NEP-413 (status **Final**, dukungan wallet luas — Defuse/Intents, Privy 2025) dipilih sebagai format utama; custom challenge menjadi fallback. (riset web + docs.near.org wallet-login)

## Problem
Endpoint tulis API (report/profil/admin) butuh identitas anti-spoof tanpa password dan tanpa memindahkan kredensial.

## Context
Wallet-first platform; user terbiasa sign via wallet; NEAR punya nonce tx protokol tapi off-chain message signing belum seragam antar wallet.

## Options
1. NEP-413 (standar sign off-chain message).
2. Custom challenge string yang di-sign via wallet (sign message API wallet).
3. Tanpa auth (read-only total).

## Trade-offs
- Opsi 1 (NEP-413): terstandar & interoperable; **riset 2026-10-01 mengkonfirmasi status Final + dukungan wallet luas** (NEAR Intents/Defuse, Privy 2025).
- Opsi 2 (custom challenge): bekerja di semua wallet dengan sign-message, tapi format kita sendiri (harus disiplin domain-binding); dipertahankan sebagai **fallback**.
- Opsi 3: spam report tak terkendali.

## Security implications
Kedua jalur wajib: nonce sekali pakai + TTL + domain binding di dalam payload (SEC-AUTH-001/002). Verifikasi ownership key on-chain menambah kekuatan (SEC-AUTH-004).

## Scalability / Operational / Cost
Semua ringan; challenge state kecil di DB.

## Decision
**Opsi 1 (NEP-413) = format utama** — hasil riset 2026-10-01 (status Final, dukungan wallet luas). **Opsi 2 (custom challenge) = fallback** untuk wallet tanpa NEP-413, dengan template kanonik sama ketatnya. **Opsi 3 ditolak.**

## Rejected
Opsi 3 (tanpa auth); password-based auth (bertentangan wallet-first).

## Consequences
Protocol final di wallet-authentication.md; implementasi Fase 1 API report. Step-up admin memakai template terpisah dengan action name eksplisit.

## Matriks kompatibilitas wallet

> NEP-413 = jalur utama; wallet tanpa NEP-413 memakai fallback (template kanonik §3). Status per wallet di bawah **berbasis kelas kemampuan**; daftar wallet terverifikasi final = ⏳ open-by-design (diisi saat test vector §8 — jangan mengarang dukungan per-wallet).

| Kelas wallet | `signMessage` | Dukungan NEP-413 | Jalur auth | Catatan |
|---|---|---|---|---|
| Wallet modern (NEAR Intents/Defuse; Privy 2025) | ya | ya — **FACT (riset 2026-10-01)** | NEP-413 (utama) | format Final, interoperable |
| Wallet dengan `signMessage` generik (tanpa NEP-413) | ya | tidak | **fallback** custom challenge | template kanonik sama ketat; dipakai sampai wallet menambah NEP-413 |
| Wallet tanpa `signMessage` | tidak | tidak | — | API write tidak didukung; user tetap bisa transaksi on-chain (S1); arahkan ke wallet lain |
| Contract account / smart account | bervariasi | tidak | — | out-of-scope MVP (padanan ERC-1271 belum standar) — wallet-authentication.md §12 |
| Skema kunci non-ed25519 (secp256k1, ml-dsa-65) | — | — | NEP-413/fallback | verifikasi skema-agnostik via `view_access_key_list` (SEC-AUTH-004) |

- Aturan: FE **mendeteksi dukungan NEP-413** dan memilih jalur otomatis; kedua jalur menghasilkan payload yang diverifikasi dengan template kanonik yang sama (tidak ada jalur "longgar").

## Argumen kesetaraan keamanan fallback

Fallback (Opsi 2) **tidak boleh** menjadi jalur yang lebih lemah. Karena itu ia memakai properti keamanan yang identik dengan NEP-413:

| Properti | NEP-413 | Fallback custom | Setara? |
|---|---|---|---|
| Domain binding | field `recipient` + `Domain:` di message | field `Domain:` di message | ya (keduanya di dalam byte yang di-sign) |
| Nonce anti-replay | 32 B, di dalam message **dan** payload | 32 B, di dalam message | ya (konsumsi atomic, TTL 5 menit) |
| Chain/network binding | `Network:` di message | `Network:` di message | ya |
| Scope separation | `Scope:` di message | `Scope:` di message | ya |
| Verifikasi ownership key | `view_access_key_list` | `view_access_key_list` | ya (SEC-AUTH-004) |
| Urutan verifikasi | validasi dulu, konsumsi nonce belakangan | idem | ya (SEC-AUTH-001) |

- Perbedaan hanya pada **serialisasi byte**: NEP-413 memakai prefix `2^31+413` + Borsh (`SignMessage`), fallback memakai string kanonik. Karena kedua jalur memverifikasi `message` + `recipient` dengan **memcmp penuh** terhadap template kanonik (bukan parse longgar), tidak ada celah downgrade dari NEP-413 ke fallback.
- **Konsekuensi**: fallback diterima sebagai security-equivalent selama (a) nonce ada **di dalam message** (bukan hanya payload), (b) template kanonik dipatuhi persis, (c) key ownership dicek on-chain. Ketiganya **wajib** dan diuji (TC-028).

## Pointer protokol (SSOT)

Detail protokol lengkap — urutan langkah challenge-response, skema payload eksak, algoritma verifikasi pseudocode, session store, CSRF, ml-dsa-65 — dimiliki **[security/wallet-authentication.md](../security/wallet-authentication.md)**. ADR ini hanya memutuskan **format** (NEP-413 utama + fallback), bukan merinci protokol.

- **Template kanonik login & step-up dimiliki wallet-authentication.md §3** — pemilik tunggal (SSOT). Dokumen lain (mis. [signature-architecture.md](../security/signature-architecture.md) §1, api/authentication.md) **merujuk**, tidak mendefinisikan ulang.
- Urutan field login: `Domain → Network → Account → Nonce → Expires → Scope`; step-up admin: `Domain → Network → Account → Action → Target → Nonce → Expires`.
- Nilai domain pada contoh (`nearsea.example`) = placeholder kanonik; domain produksi dibaca dari konfigurasi (⏳ open-by-design).

## Rencana test vector

- Vektor byte-level dikunci oleh **[wallet-authentication.md §8](../security/wallet-authentication.md)** (TV-1..TV-8): NEP-413 valid, nonce mismatch/dipakai ulang/expired, domain/recipient mismatch, scope mismatch, target step-up mismatch, dan minimal satu skema non-ed25519 (secp256k1; ml-dsa-65 ⏳ saat wallet mendukung).
- **Status: ⏳ diisi saat implementasi** — nilai byte tidak dikarang; diambil dari wallet nyata. Setiap vektor wajib dijalankan sebagai test API (TC-028) sebelum endpoint auth dinyatakan selesai.
- Tambahan yang wajib diuji: **paritas fallback** — payload custom challenge harus menghasilkan keputusan verifikasi yang sama dengan NEP-413 untuk vektor ekuivalen (membuktikan argumen kesetaraan di atas), dan payload NEP-413 tidak bisa "di-downgrade" ke fallback dengan menghapus/mengubah field.
