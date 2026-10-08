# Test Cases

> Satu kasus uji = satu blok. ID stabil dipakai di PR dan task. Semua sandbox test memakai near-workspaces (2 kontrak: NFT + market).

## Format

```md
### TC-xxx — <nama>

Precondition: …
Steps: 1) … 2) …
Expected: …
Layer: unit | sandbox | api | e2e
Invariant: INV-xxx (bila berlaku)
```

## Kasus

### TC-001 — Mint NFT
Precondition: kontrak NFT ter-init, akun tester ada.
Steps: `nft_mint` dengan metadata valid.
Expected: `nft_token(id)` mengembalikan owner benar; event `nft_mint` muncul.
Layer: sandbox | Invariant: —

### TC-002 — List & Buy happy path
Precondition: token ada; storage deposit seller cukup.
Steps: approve → list → buy (deposit = harga).
Expected: ownership pindah; seller + royalti terbayar; fee 2% → treasury; listing SOLD.
Layer: sandbox | Invariant: INV-001, INV-011

### TC-003 — Payout tidak valid → refund
Precondition: NFT sandbox dengan royalti > 10 akun (mint khusus) ATAU royalti yang menghasilkan payout > harga.
Steps: list → buy.
Expected: tx resolve, buyer kembali dana penuh, listing tetap ada.
Layer: sandbox | Invariant: INV-002

### TC-004 — Offer escrow & expire
Precondition: token milik seller; buyer berbeda.
Steps: make_offer (1Ⓝ, durasi detik-level untuk sandbox) → produce_blocks sampai lewat expires_at → attempt accept.
Expected: escrow refund penuh; accept setelah expire ditolak.
Layer: sandbox | Invariant: INV-005, INV-010
Catatan: near-workspaces tidak punya "time warp" — set expiry pendek + `produce_blocks`, atau inject expiry di argumen.

### TC-005 — Accept offer + fee konsisten
Steps: make_offer → seller accept_offer.
Expected: NFT ke buyer; distribusi = offer − 2% fee → seller + royalti (sama dengan jalur buy now).
Layer: sandbox | Invariant: INV-005

### TC-006 — Stale listing
Steps: list → owner pindahkan NFT via nft_transfer langsung → panggil remove_stale_listing / cek view.
Expected: listing ditandai stale; tidak muncul di discovery; tidak bisa dibeli.
Layer: sandbox | Invariant: INV-016

### TC-007 — Launchpad whitelist phase
Steps: creator set phase whitelist + allowlist [alice] → alice mint (sukses) → bob mint (gagal) → phase public → bob mint sukses.
Expected: hanya allowlisted yang bisa mint di phase tsb; phase kedua terbuka sesuai waktu.
Layer: sandbox | Invariant: INV-017

### TC-008 — Offer limits
Steps: (a) make_offer 0.005Ⓝ → ditolak; (b) make_offer sukses → make_offer lagi buyer sama → ditolak; (c) self-offer → ditolak.
Expected: ketiganya revert dengan pesan benar.
Layer: sandbox | Invariant: INV-024, INV-023

### TC-009 — Bundle sukses
Steps: 2 token milik seller → bundle 1 harga → buyer beli.
Expected: kedua NFT pindah; royalti per token dijumlahkan; fee dipotong sekali.
Layer: sandbox | Invariant: INV-025

### TC-010 — Bundle gagal (pre-validasi) → abort + refund
Precondition: bundle 2 token; token kedua dipindahkan setelah listing bundle (stale).
Steps: buyer beli bundle.
Expected: pre-validasi gagal → abort bersih + refund penuh; tidak ada token yang pindah.
Layer: sandbox | Invariant: INV-025

### TC-011 — Private listing
Steps: seller set allowed_buyer=alice → bob buy → ditolak; alice buy → sukses.
Expected: bob ditolak ("bukan buyer yang ditunjuk"); alice sukses (settlement normal).
Layer: sandbox | Invariant: INV-026

### TC-012 — Pause
Steps: owner pause → coba list/buy/mint/make_offer/accept_offer/update_price (semua ditolak) → seller cancel_offer / storage_withdraw (tetap bisa) → unpause.
Layer: sandbox | Invariant: INV-022

### TC-013 — List di bawah harga minimum
Steps: list dengan 0.005Ⓝ.
Expected: ditolak.
Layer: sandbox | Invariant: INV-030

### TC-014 — Bundle melebihi batas
Steps: seller coba buat bundle 11 token.
Expected: ditolak ("maksimum 10 token per bundle").
Layer: sandbox | Invariant: INV-021

### TC-015 — Royalti per token > 10% ditolak; bundle agregat > 10% sah
Steps: (a) mint koleksi dengan royalti 15% → list → buy → ditolak; (b) bundle 3 token royalti 10% masing-masing (agregat 30%) → buy_bundle → sukses.
Expected: (a) ditolak + refund; (b) sukses (tanpa cap agregat).
Layer: sandbox | Invariant: INV-027

### TC-016 — Race: banyak pembeli untuk satu listing
Precondition: 1 listing aktif; 20 akun pembeli siap.
Steps: kirim `buy` dari 20 akun pada listing yang sama (paralel via sandbox).
Expected: **tepat 1 sukses** (NFT pindah, seller+royalti terbayar); 19 tx revert dan **deposit Ⓝ masing-masing kembali penuh**; tidak ada dua pembayar.
Layer: sandbox | Invariant: INV-007, INV-008, INV-009

### TC-017 — Double-submit pembeli yang sama
Precondition: 1 listing aktif; satu akun pembeli.
Steps: kirim dua `buy` berturut-turut dari akun yang sama (tx kedua sebelum final).
Expected: tx pertama sukses; tx kedua revert (sale sudah terhapus / nonce) tanpa efek ganda.
Layer: sandbox | Invariant: INV-007, INV-008

### TC-018 — Dua `accept_offer` pada offer yang sama
Precondition: satu offer aktif.
Steps: dua akun/upaya `accept_offer` pada offer yang sama.
Expected: satu sukses (settle); yang kedua revert; escrow tidak terdistribusi dua kali.
Layer: sandbox | Invariant: INV-009

### TC-019 — Offer auto-cancel saat offer lain di-accept (superseded)
Precondition: token C1.7 milik seller; offer AKTIF O-A (buyer `nearsea-buyer.testnet`, 1 Ⓝ) dan O-B (buyer `nearsea-bob.testnet`, 1.2 Ⓝ) pada token yang sama.
Steps: 1) seller `accept_offer(O-B)` 2) cek status O-A 3) bandingkan saldo buyer O-A sebelum/sesudah.
Expected: O-B ACCEPTED (NFT ke bob; distribusi = 1.2 Ⓝ − 2% fee → seller + royalti); O-A → SUPERSEDED + escrow 1 Ⓝ refund penuh ke buyer O-A; tepat satu NFT berpindah.
Layer: sandbox | Invariant: INV-005, INV-009
Catatan: perilaku auto-cancel dimiliki [features/marketplace.md](../features/marketplace.md) § Flow — Offer langkah 5; status `SUPERSEDED` di [order-protocol-security.md](../security/order-protocol-security.md) §2. Event refund mengikuti katalog [webhooks.md](../api/webhooks.md) (`market_offer_cancel` — tidak ada event terpisah untuk SUPERSEDED). Payload event yang diharapkan untuk O-A:
```json
{ "standard": "x-nearsea-market", "version": "1.0.0", "event": "market_offer_cancel",
  "data": [ { "nft_contract_id": "nearsea-genesis.testnet", "token_id": "7",
              "buyer": "nearsea-buyer.testnet", "amount_yocto": "1000000000000000000000000" } ] }
```

### TC-020 — Storage deposit kurang saat listing (INV-020)
Precondition: seller punya token C2.101; kebutuhan storage market dibaca dari view NEP-145 (`storage_balance_bounds`/`storage_balance_of`) = R.
Steps: 1) `list_nft_for_sale` dengan `attached_deposit` = R − 1 yocto 2) ulangi dengan deposit `0` yocto 3) `storage_deposit` sampai saldo = R 4) `list_nft_for_sale` lagi.
Expected: (1) dan (2) revert (storage kurang) dan tidak ada listing tersimpan; (3) sukses; (4) listing ACTIVE. `storage_withdraw` hanya menarik kelebihan, tidak boleh di bawah kebutuhan.
Layer: sandbox | Invariant: INV-020
Catatan: angka konkret yang dipakai = `required − 1 yocto`, `0 yocto`, dan `required` persis (dihitung dari view kontrak, bukan hardcode). Tidak ada nilai storage tetap di dokumen ini — nilainya berasal dari `storage_balance_bounds`.

### TC-021 — Launchpad phase overlap ditolak (INV-029)
Precondition: koleksi C1 punya phase P1 dengan window `[T0, T0+24j)`.
Steps: 1) creator set phase P2 dengan window overlap P1 (mis. `[T0+12j, T0+48j)`) 2) cek konfigurasi phase 3) set P2 berurutan `[T0+24j, T0+72j)`.
Expected: (1) ditolak ("phase tidak boleh overlap"); konfigurasi phase tidak berubah; (3) sukses. Saat `nft_mint`, maksimum satu phase aktif pada satu waktu.
Layer: sandbox | Invariant: INV-029
Catatan: aturan `ends_at > starts_at` + tidak overlap tercermin di `chk_launchpad_phases_window` ([database-schema.md](../database/database-schema.md)).

### TC-022 — Harga berubah saat signing (CONFLICT_PRICE_CHANGED)
Precondition: listing L1 harga 1 Ⓝ; buyer sudah preview harga 1 Ⓝ.
Steps: 1) seller `update_price` → 1.5 Ⓝ 2) buyer `buy` dengan deposit 1 Ⓝ 3) buyer `buy` dengan deposit 2 Ⓝ.
Expected: (2) revert (deposit < harga) → dana kembali, listing tetap ACTIVE, FE memetakan ke `CONFLICT_PRICE_CHANGED` ("Harga berubah — periksa ulang") sesuai [error-handling.md](../development/error-handling.md) §4; (3) sukses, kelebihan 0.5 Ⓝ dikembalikan ke buyer via `resolve_purchase`.
Layer: sandbox | Invariant: INV-001, INV-002, INV-016
Catatan: kontrak assert `deposit ≥ price`; selisih > 0 dikembalikan buyer via resolve (bukan hangus). Payload event yang diharapkan pada langkah (3) — listing L1 (token C2.101, royalti koleksi 0%) harga 1.5 Ⓝ, fee 2%:
```json
{ "standard": "x-nearsea-market", "version": "1.0.0", "event": "market_sale",
  "data": [ { "nft_contract_id": "nearsea-open.testnet", "token_id": "101",
              "buyer": "nearsea-buyer.testnet", "seller": "nearsea-seller.testnet",
              "price_yocto": "1500000000000000000000000",
              "payout": [ { "receiver_id": "nearsea-seller.testnet", "amount_yocto": "1470000000000000000000000" },
                          { "receiver_id": "treasury.testnet", "amount_yocto": "30000000000000000000000" } ] } ] }
```
(Koleksi C2 berroyalti 0%; total payout = harga − fee = 1.47 Ⓝ + 0.03 Ⓝ fee = 1.5 Ⓝ.)

### TC-023 — Notifikasi polling: offer baru muncul ≤ 60 detik
Precondition: user seller login; tab aktif; interval polling 30–60 detik ([features/notifications.md](../features/notifications.md)).
Steps: 1) buyer `make_offer` pada token seller 2) tunggu siklus polling berikutnya 3) cek bell + daftar notifikasi.
Expected: bell menampilkan ≥1 unread ("offer diterima") dalam ≤ 60 detik; item menunjuk token & nominal benar; sumber = view `get_offers` + diff riwayat NearBlocks.
Layer: e2e | Invariant: —

### TC-024 — Notifikasi idempotency (receipt_id, event_index)
Precondition: satu event notifikasi sudah tampil dengan `(receipt_id, event_index)` tetap.
Steps: 1) paksa beberapa siklus polling tanpa event baru 2) simulasi respons NearBlocks memuat event yang sama dua kali.
Expected: tidak ada duplikat item/toast; dedup key = `(receipt_id, event_index)` (bukan `tx_hash`); unread tidak bertambah.
Layer: e2e | Invariant: — (aturan dedup [features/notifications.md](../features/notifications.md) § Error Cases)

### TC-025 — Admin step-up signature (happy)
Precondition: `nearsea-admin.testnet` ada di allowlist admin DB; ada 1 report pending.
Steps: 1) login scope `admin` 2) `PATCH /api/v1/admin/reports/:id` dengan `step_up` signature kanonik (`Action`, `Target`, `Nonce`, `Expires`) 3) cek audit.
Expected: 200; report resolved; target tersembunyi dari discovery; baris `admin_audit` (actor, action, target, reason) tercatat.
Layer: api | Invariant: —
Catatan: template step-up kanonik dimiliki [wallet-authentication.md](../security/wallet-authentication.md) §3; audit wajib [admin-security.md](../security/admin-security.md) §5.

### TC-026 — Admin step-up ditolak (error path)
Precondition: session scope `admin` valid; satu akun ber-scope `admin` tetapi DI LUAR allowlist DB.
Steps: (a) aksi destruktif tanpa `step_up` (b) `step_up` dengan nonce sudah dipakai (c) nonce expired (d) signature atas action/target berbeda (e) akun di luar allowlist.
Expected: (a)–(d) ditolak 400/401 dengan kode `INVALID_*`/`AUTH_*` (bukan 200); (e) 403 `FORBIDDEN_ADMIN`; tidak ada perubahan state dan tidak ada audit sukses.
Layer: api | Invariant: —

### TC-027 — API `GET /api/v1/auth/nonce` (happy + error)
Precondition: API + DB jalan.
Steps: 1) GET `?account_id=alice.testnet` 2) GET tanpa `account_id` / format invalid 3) ulangi > 10×/menit dari IP sama.
Expected: (1) 200 `{nonce, domain, network, expires_at}`; nonce 32 byte hex tersimpan `used=false`; (2) 400 `INVALID_ACCOUNT_ID`; (3) 429 `RATE_LIMITED` + `Retry-After`.
Layer: api | Invariant: — (rate limit [api-security-architecture.md](../security/api-security-architecture.md) §1)

### TC-028 — API `POST /api/v1/auth/verify` (happy + error, NEP-413)
Precondition: nonce dari TC-027; test vector NEP-413 ([api/authentication.md](../api/authentication.md) § NEP-413 test vectors).
Steps: 1) verify signature valid 2) ulangi nonce sama 3) `message`/`recipient` mismatch 4) `public_key` bukan milik akun 5) signature salah 6) 5 gagal/menit.
Expected: (1) 200 `{access_token, refresh_token, expires_in:900, scope}`; (2) 401 `AUTH_NONCE_USED`; (3) 401 `AUTH_MESSAGE_MISMATCH`; (4) 401 `AUTH_KEY_NOT_FOUND`; (5) 401 `AUTH_SIGNATURE`; (6) 429 `RATE_LIMITED`. Kegagalan (3)–(5) TIDAK mengonsumsi nonce.
Layer: api | Invariant: —

### TC-029 — API `POST /api/v1/auth/refresh` (happy + error)
Steps: 1) refresh valid 2) pakai ulang refresh lama (reuse) 3) refresh kedaluwarsa 4) refresh scope `admin`.
Expected: (1) 200 pasangan access+refresh baru (rotasi); (2) 401 `AUTH_REFRESH_REUSED` + seluruh family sesi dicabut; (3) 401 `AUTH_REFRESH_INVALID`; (4) 403 (admin tanpa silent refresh).
Layer: api | Invariant: —

### TC-030 — API `POST /api/v1/auth/logout` (happy + error)
Steps: 1) logout dengan Bearer valid + refresh 2) logout ulang 3) tanpa/invalid Bearer.
Expected: (1) 204; sesi dihapus + `jti` masuk denylist (access token lama ditolak); (2) 204 (idempoten); (3) 401 `AUTH_SESSION_EXPIRED`/`AUTH_TOKEN_INVALID`.
Layer: api | Invariant: —

### TC-031 — API `GET /api/v1/accounts/:id/profile` (happy + error)
Steps: 1) GET akun yang punya profil 2) akun tanpa profil / format invalid.
Expected: (1) 200 objek profil (`account_id`, `alias`, `bio`, `avatar_url`, `updated_at`) + ETag; `If-None-Match` cocok → 304; (2) 404 `NOT_FOUND_ACCOUNT` / 400 untuk format invalid.
Layer: api | Invariant: —

### TC-032 — API `PATCH /api/v1/accounts/me/profile` + signature NEP-413 (happy + error)
Precondition: session scope `profile` dari alur nonce/verify NEP-413 (TC-027/TC-028).
Steps: 1) PATCH `{alias:"Alice", bio:"collector"}` 2) `alias` 33 char 3) `bio` 281 char 4) `avatar_url` di luar gateway allowlist 5) tanpa Bearer / scope `report` 6) > 10×/jam.
Expected: (1) 200 profil ter-update + `GET` menampilkannya; (2) 400 `INVALID_ALIAS`; (3) 400 `INVALID_BIO`; (4) 400 `INVALID_AVATAR`; (5) 401 `AUTH_*` / 403 `FORBIDDEN_SCOPE`; (6) 429 `RATE_LIMITED`. Data tidak tersimpan pada error.
Layer: api | Invariant: — (batas field [features/users.md](../features/users.md))

### TC-033 — API `POST /api/v1/reports` (happy + error)
Steps: 1) kirim report valid (scope `report`) 2) kirim target sama di hari yang sama 3) target tidak ada 4) > 5/akun/hari.
Expected: (1) 201 `{id, status:"open", created_at}`; (2) 409 `CONFLICT_REPORT_EXISTS` (unik per reporter+target+hari); (3) 404 `NOT_FOUND_COLLECTION`/`NOT_FOUND_TOKEN`; (4) 429 `RATE_LIMITED`.
Layer: api | Invariant: —

### TC-034 — API `GET /api/v1/admin/reports` (happy + error)
Steps: 1) admin allowlist GET antrean (+ filter `status`, cursor, limit) 2) non-admin 3) tanpa Bearer.
Expected: (1) 200 `{items[], nextCursor}` dengan field report; (2) 403 `FORBIDDEN_ADMIN`; (3) 401 `AUTH_*`.
Layer: api | Invariant: —

### TC-035 — API `PATCH /api/v1/admin/reports/:id` (happy + error)
Steps: 1) admin + step-up `{action:"hide_target", reason}` 2) report sudah diputuskan 3) id tidak ada 4) action invalid 5) tanpa step-up.
Expected: (1) 200 resolved + baris `admin_audit` + target hilang dari discovery; (2) 409 `CONFLICT_ALREADY_DECIDED`; (3) 404 `NOT_FOUND_REPORT`; (4) 400 `INVALID_*`; (5) 401/403.
Layer: api | Invariant: —

### TC-036 — API `PATCH /api/v1/admin/collections/:id/verified` (happy + error)
Steps: 1) admin + step-up `{verified:true, reason}` 2) koleksi tidak ada 3) tanpa step-up.
Expected: (1) 200 `{verified:true, decided_by, decided_at}` + `admin_audit`; badge tampil; (2) 404 `NOT_FOUND_COLLECTION`; (3) 401/403.
Layer: api | Invariant: —

### TC-037 — API `GET /api/v1/admin/blocklist` (happy + error)
Steps: 1) admin GET (+ filter `target_type`, cursor) 2) non-admin.
Expected: (1) 200 `{items[], nextCursor}`; (2) 403 `FORBIDDEN_ADMIN`.
Layer: api | Invariant: —

### TC-038 — API `POST /api/v1/admin/blocklist` (happy + error)
Steps: 1) admin + step-up tambah entri 2) target sudah ada 3) `target_type` invalid.
Expected: (1) 201 entri; target tersembunyi dari discovery; `admin_audit`; (2) 409 `CONFLICT_ALREADY_BLOCKED`; (3) 400 `INVALID_*`.
Layer: api | Invariant: —

### TC-039 — API `DELETE /api/v1/admin/blocklist/:id` (happy + error)
Steps: 1) admin + step-up hapus entri 2) id tidak ada 3) non-admin.
Expected: (1) 204; target kembali tampil; `admin_audit`; (2) 404 `NOT_FOUND_BLOCKLIST`; (3) 403.
Layer: api | Invariant: —

### TC-040 — E2E jalur emas: connect → list → buy
Precondition: environment E2E siap (seed testnet; [testing-strategy.md](./testing-strategy.md) § Penyediaan environment E2E).
Steps: 1) buyer connect wallet 2) seller list token C2.101 (approve → list) 3) buyer buka detail & Buy 4) konfirmasi di wallet.
Expected: listing ACTIVE muncul di discovery; setelah buy, NFT di wallet buyer, seller + royalti terbayar, fee 2% → treasury, listing SOLD, toast sukses (bukan teks panic).
Layer: e2e | Invariant: INV-001, INV-011

### TC-041 — E2E jalur emas: offer → accept
Steps: 1) buyer `make_offer` 1 Ⓝ pada token seller 2) seller melihat notifikasi offer 3) seller `accept_offer`.
Expected: escrow terlihat; notifikasi ≤ 60 detik; setelah accept, NFT ke buyer, distribusi = offer − 2% fee → seller + royalti.
Layer: e2e | Invariant: INV-005

### TC-042 — E2E jalur error: kalah race → refund
Precondition: dua sesi browser pembeli pada listing yang sama.
Steps: 1) buyer A selesaikan buy 2) buyer B submit buy atas listing yang sama.
Expected: B menerima satu pesan `CONFLICT_SOLD` ("Sudah terjual — dana kamu kembali"), dana kembali, listing hilang dari discovery; tidak ada teks panic mentah.
Layer: e2e | Invariant: INV-007, INV-008

### TC-043 — List/offer token yang ada di bundle aktif
Precondition: seller memiliki 3 token; `create_bundle` berisi token T sukses (bundle AKTIF).
Steps: 1) seller coba `list_nft_for_sale` untuk T 2) akun lain coba `make_offer` untuk T.
Expected: kedua aksi ditolak `CONFLICT_BUNDLE_ITEM_INVALID`; bundle tetap AKTIF; tidak ada state sale/offer tercipta.
Layer: sandbox | Invariant: INV-028

### TC-044 — Cancel listing (remove_sale) + storage kembali
Precondition: 1 listing aktif milik seller; storage deposit terpenuhi.
Steps: 1) seller `remove_sale` (1 yocto) 2) cek state `get_sale` 3) coba buy listing yang sama.
Expected: sale hilang dari state; storage seller kembali (dapat ditarik via `storage_withdraw`); buy berikutnya gagal; event `market_delist`. **Approval market TIDAK dicabut** oleh `remove_sale` (NEP-178 owner-only — [contracts/market.md](../contracts/market.md) §2a); re-list memerlukan `nft_revoke` lalu `nft_approve` baru.
Layer: sandbox | Invariant: INV-012, INV-016, INV-020

### TC-045 — Cancel offer + refund buyer
Precondition: 1 offer aktif (escrow terisi exact).
Steps: 1) buyer `cancel_offer` 2) cek saldo buyer 3) coba `storage_withdraw` offer.
Expected: refund exact = amount; offer hilang; event `market_offer_cancel`; **storage deposit offer TIDAK otomatis kembali** — ditarik manual via `storage_withdraw` (anti gas-DoS refund massal).
Layer: sandbox | Invariant: INV-005, INV-024

### TC-046 — Cancel bundle + revoke semua token
Precondition: bundle AKTIF berisi 3 token.
Steps: 1) seller `cancel_bundle` 2) cek approval ketiga token 3) cek bundle state.
Expected: approval ketiga token false; bundle hilang; event `market_bundle_cancel`; token bebas di-list lagi.
Layer: sandbox | Invariant: INV-028

### TC-047 — withdraw_fees owner-only
Precondition: 2 penjualan sukses (fee terkumpul di kontrak); treasury address terisi.
Steps: 1) non-owner panggil `withdraw_fees` 2) owner panggil `withdraw_fees` 3) cek saldo treasury.
Expected: non-owner revert; owner sukses — fee pindah exact ke treasury; event `treasury_withdraw`.
Layer: sandbox | Invariant: SEC-CONTRACT-012 (owner-only MVP)

### TC-048 — Callback `nft_on_approve` dipalsukan
Precondition: kontrak penyerang ter-deploy di sandbox.
Steps: 1) kontrak penyerang memanggil `nft_on_approve` market langsung dengan payload NEP-178 palsu 2) cek state sale.
Expected: revert (`#[private]` + predecessor check); TIDAK ada sale tercipta; tidak ada storage berubah.
Layer: sandbox | Invariant: INV-013

### TC-049 — Verify via custom challenge fallback
Precondition: wallet tanpa NEP-413 (fallback path); nonce sudah di-mint.
Steps: 1) wallet sign template kanonik (tanpa payload NEP-413) 2) `POST /auth/verify` 3) ulang dengan message yang satu field diubah.
Expected: (1) 200 + token sesi scope sesuai; nonce terkonsumsi atomic; (2) 401 `AUTH_MESSAGE_MISMATCH` tanpa mengonsumsi nonce.
Layer: api | Invariant: SEC-AUTH-002 (fallback parity)

### TC-050 — Body size cap 16 KB
Precondition: server jalan dengan body cap 16 KB.
Steps: 1) `POST /reports` dengan body > 16 KB.
Expected: ditolak (413/400 sesuai spec endpoints) sebelum diproses; rate limit report TIDAK terkonsumsi; tidak ada baris DB.
Layer: api | Invariant: SEC-API-001

### TC-051 — CORS preflight
Precondition: origin FE terdaftar di allowlist CORS.
Steps: 1) `OPTIONS` dari origin allowlist 2) `OPTIONS` dari origin asing.
Expected: (1) header `Access-Control-Allow-Origin/Headers` lengkap (termasuk `If-None-Match`, `X-Request-Id`); (2) tanpa header CORS (browser memblokir).
Layer: api | Invariant: SEC-FE-001

### TC-052 — Health endpoint
Precondition: API jalan; DB reachable.
Steps: 1) `GET /api/health` saat DB up 2) hentikan DB 3) `GET /api/health` lagi.
Expected: (1) 200; (2) 503 — dipakai smoke test deploy & uptime check (ci-cd.md §smoke, monitoring.md).
Layer: api | Invariant: —

### TC-053 — Stale karena approval dicabut (kasus B)
Precondition: 1 listing aktif; token TETAP di wallet seller.
Steps: 1) seller panggil `nft_revoke(token_id, market)` 2) buyer coba `buy` 3) panggil `remove_stale_listing` (akun ketiga).
Expected: (1) `buy` ditolak `CONFLICT_STALE` (bukan `CHAIN_REVERT` generik); (2) listing tetap ada tapi tidak bisa dibeli; (3) `remove_stale_listing` berhasil membersihkan (membuktikan approval invalid via XCC); event `market_stale_detected` dengan `reason: "approval_revoked"`.
Layer: sandbox | Invariant: INV-016 (kasus B)

### TC-054 — Recovery dana nyangkut (C3)
Precondition: `buy` disimulasikan gagal di callback (inject: callback revert / gas habis — mis. mock `nft_transfer_payout` yang panic).
Steps: 1) buyer `buy` dengan deposit penuh 2) callback gagal → cek `pending_purchases[sale_key]` masih ada 3) majukan `block_height` > `RECOVERY_DELAY_BLOCKS` 4) akun **ketiga** (bukan buyer/seller) panggil `recover_stuck_purchase`.
Expected: (1) entri pending ada setelah callback gagal; (2) `recover_stuck_purchase` sebelum delay → revert (terlalu dini); (3) setelah delay → buyer menerima refund **penuh**, Sale dipulihkan, entri pending dihapus; event `market_purchase_recovered`. Buyer tidak bergantung pada owner/governance.
Layer: sandbox | Invariant: INV-031

> **Cakupan slice M1 (TASK-006):** TC yang runnable tanpa M1+ = TC-001 (mint), TC-002 (list&buy), TC-013 (harga < min), TC-016 (race 20 pembeli), TC-017 (double-submit), TC-020 (storage kurang), TC-022 (harga berubah), TC-044 (remove_sale), TC-047 (withdraw_fees owner-only), TC-048 (callback dipalsukan), **TC-053** (stale approval), **TC-054** (recover stuck). TC-003/TC-012 sebagian (butuh setup khusus / subset market-pause).
> **Deferred ke M1+:** TC-004/005/008/018/019/041 (offers), TC-009/010/011/014/015/043/046 (bundle/private), TC-006 (stale ownership — cleanup), TC-007/021 (launchpad), TC-023/024 (notifikasi), TC-025..TC-039 (API/admin/profil/report), TC-040 (E2E penuh, butuh factory+seed), TC-049..TC-052 (API/CORS/health).

> **Sudah dibuktikan di level unit (bukan pengganti sandbox):** TC-001 (mint → transfer → events) dan
> **sisi koleksi TC-003** — `nft_transfer_payout` memindahkan kepemilikan dan mengembalikan payout
> royalti ≤10% (dust + cap diuji), wajib 1 yocto, menolak pengirim tanpa approval, dan approval lama
> invalid setelah transfer (INV-011). Lihat 11 test `test_transfer_payout_*` di `contract/src/lib.rs`
> (TASK-003, ronde 21). Yang **belum** dibuktikan: validasi payout di sisi **market** (≥1 penerima,
> Σ ≤ harga−fee, sisa ≤1 yocto, refund saat invalid) — itu bagian sandbox TC-003 milik TASK-006,
> bersama angka gas 15 Tgas.

> **Sudah dibuktikan di level unit untuk paruh listing (ronde 22, TASK-004):** 26 test di
> `market/src/lib.rs` menutup paruh **list** dari TC-002 (dual verification: ownership **dan** approval,
> termasuk jalur `nft_token` gagal / `None` / approval `false`), TC-013 (harga < min, plus batas
> inklusif tepat 0.01 Ⓝ), TC-020 (storage kurang → revert; storage kembali saat `remove_sale`),
> TC-044 (`remove_sale` 1 yocto + owner-only + boleh saat paused + event), serta duplikat listing
> (INV-007), `update_price` (in-place + min + owner-only + paused), `approval_id` di luar rentang
> `u32`, dan paginasi view. Non-custodial dibuktikan dengan **membaca receipt** yang dibuat jalur
> listing: hanya `nft_token` + `nft_is_approved` + callback, tidak ada `nft_transfer*`.
> Yang **belum**: paruh **buy** TC-002, TC-016/017 (race/double-submit), TC-022 (harga berubah),
> TC-048 (callback dipalsukan oleh kontrak penyerang), dan angka gas penuh — semuanya butuh dua
> kontrak nyata (TASK-005/006).

> Kasus lanjutan (TC-019..TC-052) mengikuti skenario wajib di [testing-strategy.md](./testing-strategy.md): offer auto-cancel, storage deposit, phase overlap, harga berubah saat signing, profil custom API (NEP-413), notifikasi polling/idempotency, admin step-up, test endpoint API (happy + error), E2E jalur emas, plus perlindungan bundle (TC-043..046) dan jalur owner/admin (TC-047..052). Kasus discovery ★ fase 2 ditambahkan saat indexer aktif.
