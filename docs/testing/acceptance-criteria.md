# Acceptance Criteria

> Kriteria terima per fitur, format Given/When/Then. Dipakai QA dan agent sebagai definisi "selesai".
> (Revisi audit 2026-10-01 — selaras dengan docs/features/*.md dan INV.)
> Setiap AC punya ID stabil `AC-<area>-<n>` untuk traceability ke TC, PR, dan task.

## Format

```md
## <Fitur>
- **AC-<AREA>-<n>** Given: <kondisi awal>
  When: <aksi user>
  Then: <hasil teramati>
  And: <invariant / catatan>
```

- ID `AC-<area>-<n>` **stabil** — dipakai di PR, task, dan komentar test (mis. `// AC-BUY-2`).
- AC negatif/edge diberi nomor lanjut di area yang sama (bukan area terpisah).
- Area: `WALLET`, `COLL`, `MINT`, `LIST`, `BUY`, `OFFER`, `BUNDLE`, `STALE`, `PAUSE`, `PROFILE`,
  `NOTIF`, `REPORT`, `ADMIN`, `BADGE`, `LEADER`, `ONBOARD`, `SETTINGS`, `NETWORK`, `RATE`, `PERF`.

## Wallet / Auth

- **AC-WALLET-1** Given belum connect, When klik Connect lalu approve, Then alamat + saldo tampil di header dan bertahan setelah reload.
- **AC-WALLET-2** Given modal Connect terbuka, When user menolak popup, Then toast "dibatalkan" muncul dan state kembali `disconnected` (tanpa error mentah).
- **AC-WALLET-3** Given wallet yang dipilih belum terpasang, When user klik wallet tsb, Then tombol disabled + hint install (bukan crash).
- **AC-WALLET-4** Given wallet terhubung, When user disconnect via menu, Then alamat dihapus dari state dan kembali ke `disconnected`.

## Create Collection (launchpad)

- **AC-COLL-1** Given creator terhubung, When submit form koleksi + phases (allowlist terupload), Then kontrak koleksi ter-deploy via factory dan muncul di discovery.
- **AC-COLL-2** Given royalti diisi > 10%, When submit, Then ditolak (validasi kontrak + UI).
- **AC-COLL-3** Given dua phase dengan window yang overlap, When submit, Then ditolak (INV-029).
- **AC-COLL-4** Given `max_mints` di allowlist melebihi `max_per_wallet` phase, When upload allowlist, Then ditolak dengan pesan baris yang salah.

## Mint (launchpad)

- **AC-MINT-1** Given fase whitelist aktif dan user ada di allowlist, When mint dengan deposit = harga fase (+ storage pemicu mint), Then token ter-mint ke wallet user.
- **AC-MINT-2** Given user TIDAK ada di allowlist, When mint, Then tx gagal "not in allowlist" dan deposit tidak hangus.
- **AC-MINT-3** Given fase belum mulai / sudah selesai, When mint, Then ditolak.
- **AC-MINT-4** Given deposit ≠ harga fase (kurang atau lebih), When mint, Then ditolak (INV-018).
- **AC-MINT-5** Given alokasi fase habis atau mint melebihi max/wallet, When mint, Then ditolak (INV-017).

## NFT Listing

- **AC-LIST-1** Given user memiliki NFT dan storage deposit cukup, When list dengan harga ≥ 0.01 Ⓝ, Then listing aktif dan muncul di discovery.
- **AC-LIST-2** Given NFT yang sama, When di-list dua kali, Then percobaan kedua ditolak; NFT tetap di wallet seller (INV-007).
- **AC-LIST-3** Given harga < 0.01 Ⓝ, When list, Then ditolak (INV-030).
- **AC-LIST-4** Given storage deposit market kurang, When list, Then ditolak dan UI meminta `storage_deposit` dulu (INV-020).
- **AC-LIST-5** Given bukan owner token, When memanggil aksi seller (cancel/update price), Then revert tanpa efek (INV-012).

## Buy NFT

- **AC-BUY-1** Given listing aktif milik orang lain, When buyer mengirim deposit = harga, Then NFT pindah ke buyer, seller + royalti terbayar, fee 2% → treasury.
- **AC-BUY-2** Given settlement gagal, When resolve berjalan, Then refund 100% otomatis dan listing tetap ada.
- **AC-BUY-3** Given seller membeli listing sendiri, When buy, Then ditolak (INV-023).
- **AC-BUY-4** Given deposit > harga (harga berubah saat signing), When resolve, Then kelebihan dikembalikan ke buyer (bukan hangus).
- **AC-BUY-5** Given token sudah dipindah di luar market (stale), When buy, Then ditolak dan listing tidak bisa dibeli (INV-016).
- **AC-BUY-6** Given 20 pembeli menyasar satu listing, When semua tx diproses, Then tepat 1 sukses; 19 revert dengan deposit kembali penuh (INV-007/008/009).

## Offer

- **AC-OFFER-1** Given token milik orang lain, When make_offer ≥ 0.01 Ⓝ, Then escrow tersimpan dan seller melihat offer.
- **AC-OFFER-2** Given sudah ada offer aktif dari buyer yang sama, When make_offer lagi, Then ditolak (batal dulu — INV-024).
- **AC-OFFER-3** Given offer aktif, When expire, Then escrow kembali penuh ke buyer.
- **AC-OFFER-4** Given offer di-accept, Then distribusi = offer − fee 2% → seller + royalti; offer lain pada token yang sama auto-cancel + refund (perilaku spec di marketplace.md).
- **AC-OFFER-5** Given buyer membatalkan offer aktifnya, When cancel, Then refund penuh.
- **AC-OFFER-6** Given nominal offer < 0.01 Ⓝ, When make_offer, Then ditolak (INV-030).
- **AC-OFFER-7** Given buyer adalah owner token, When make_offer, Then ditolak (INV-023).
- **AC-OFFER-8** Given token TIDAK di-list, When make_offer ≥ 0.01 Ⓝ, Then offer sah (offer boleh pada token apa pun — PRD §13).

## Private Listing & Bundle

- **AC-BUNDLE-1** Given private listing untuk alice, When bob mencoba membeli, Then ditolak (INV-026).
- **AC-BUNDLE-2** Given bundle 3 token dengan satu harga, When dibeli, Then ketiga NFT pindah, royalti per token dijumlahkan, fee 2% dipotong sekali.
- **AC-BUNDLE-3** Given bundle N token, When settlement gagal pre-validasi, Then abort bersih + refund penuh; gagal residual → status PARTIAL + kompensasi G7 (INV-025).
- **AC-BUNDLE-4** Given bundle > 10 token, When dibuat, Then ditolak (INV-021).
- **AC-BUNDLE-5** Given token tergabung dalam bundle aktif, When token tsb di-list/di-offer terpisah, Then ditolak (INV-028).

## Stale Listing

- **AC-STALE-1** Given listing aktif, When token dipindah di luar market, Then listing ditandai stale, disembunyikan dari discovery, dan tidak bisa dibeli (INV-016).

## Pause (cakupan = INV-022)

- **AC-PAUSE-1** Given kontrak paused oleh owner, When user mencoba list/buy/mint/make_offer/accept_offer/update_price, Then semua ditolak.
- **AC-PAUSE-2** Given kontrak paused, When user (seller/buyer) menarik escrow/refund/cancel/withdraw, Then tetap bisa.

## Profil Custom

- **AC-PROFILE-1** Given user terhubung, When set alias/bio/avatar (signature NEP-413/fallback), Then profil publik menampilkan data tsb.
- **AC-PROFILE-2** Given alias > 32 char / bio > 280 char / avatar di luar gateway allowlist, When simpan, Then 400 dengan pesan field spesifik dan data tidak tersimpan.
- **AC-PROFILE-3** Given signature profil gagal diverifikasi, When simpan, Then data tidak tersimpan dan user bisa retry.

## Notifikasi

- **AC-NOTIF-1** Given seller memiliki offer baru, When offer dibuat, Then bell menampilkan 1 unread dalam ≤ 60 detik (tab aktif).
- **AC-NOTIF-2** Given notifikasi sama (event id `(receipt_id, event_index)` identik), When polling berikutnya, Then tidak terjadi duplikat (idempotency).
- **AC-NOTIF-3** Given NearBlocks gagal/rate-limit, When polling gagal, Then badge berhenti update dengan retry backoff dan UI tetap berfungsi (tanpa error mentah).
- **AC-NOTIF-4** Given offer milik user lewat expire, When FE mengecek expiry lokal, Then notifikasi "offer expired" muncul tanpa menunggu tx on-chain.

## Report System

- **AC-REPORT-1** Given user terhubung dengan scope `report`, When mengirim report pada koleksi/token, Then 201 dan report masuk antrean review admin.
- **AC-REPORT-2** Given user sudah report target yang sama di hari kalender yang sama, When mengirim lagi, Then 409 `CONFLICT_REPORT_EXISTS` (idempoten per hari).
- **AC-REPORT-3** Given user mengirim > 5 report dalam sehari, When report berikutnya, Then 429 `RATE_LIMITED` + `Retry-After`.
- **AC-REPORT-4** Given target koleksi/token tidak ada, When report, Then 404 `NOT_FOUND_COLLECTION`/`NOT_FOUND_TOKEN`.
- **AC-REPORT-5** Given belum login / scope salah, When report, Then 401 `AUTH_*` / 403 `FORBIDDEN_SCOPE`.
- **AC-REPORT-6** Given report diputuskan `hide_target`, When target disembunyikan, Then kepemilikan & transaksi on-chain TIDAK berubah (takedown hanya layer tampilan).

## Admin Panel

- **AC-ADMIN-1** Given akun ada di allowlist admin DB, When login scope `admin`, Then panel `/admin` dapat diakses.
- **AC-ADMIN-2** Given akun ber-scope `admin` tetapi TIDAK di allowlist DB, When akses `/admin`, Then 403 `FORBIDDEN_ADMIN`.
- **AC-ADMIN-3** Given aksi destruktif (hide/verified/blocklist), When `step_up` signature tidak ada/invalid/expired/replay, Then aksi ditolak dan tidak ada perubahan state (SEC-ADMIN-003).
- **AC-ADMIN-4** Given aksi destruktif berhasil, When selesai, Then baris `admin_audit` (actor, action, target, reason) tercatat append-only.
- **AC-ADMIN-5** Given session admin, When access token lewat 15 menit, Then re-login + step-up diminta (tanpa silent refresh).
- **AC-ADMIN-6** Given report di antrean, When admin memutuskan `ignore`, Then report berstatus ignored tanpa menyentuh discovery/chain.

## Verified Badge

- **AC-BADGE-1** Given koleksi `verified = true`, When tampil di discovery/detail, Then badge verified terlihat.
- **AC-BADGE-2** Given koleksi `verified = false`, When tampil, Then tidak ada badge verified.
- **AC-BADGE-3** Given bukan admin / tanpa step-up, When mencoba set verified, Then ditolak (403/401).

## Leaderboard

- **AC-LEADER-1** Given endpoint leaderboard aktif (★ fase 2), When dibuka, Then koleksi diurutkan berdasarkan volume/statistik dari penjualan riil dengan pagination cursor.
- **AC-LEADER-2** Given belum ada penjualan, When leaderboard dibuka, Then empty state tampil (bukan error).
- **AC-LEADER-3** Given volume mencurigakan (wash trading 2 akun), When stats dihitung, Then volume ditandai "tidak terverifikasi" pada stats — TIDAK memengaruhi settlement (fraud-and-abuse.md).

## Onboarding

- **AC-ONBOARD-1** Given newcomer membuka `/onboarding`, When halaman tampil, Then langkah (buat wallet → faucet testnet → connect wallet → transaksi pertama) tampil berurutan.
- **AC-ONBOARD-2** Given user belum punya wallet, When membuka `/onboarding`, Then halaman tetap publik/statis (tanpa perlu connect).
- **AC-ONBOARD-3** Given halaman onboarding, When user klik tautan wallet/faucet, Then tautan menuju kanal resmi (bukan mirror) dan tips keamanan ditampilkan.

## Settings

- **AC-SETTINGS-1** Given user terhubung, When membuka `/settings`, Then form alias/bio/avatar tampil dengan nilai profil saat ini.
- **AC-SETTINGS-2** Given user menyimpan perubahan dengan signature valid, When sukses, Then profil ter-update dan tercermin di halaman publik.
- **AC-SETTINGS-3** Given user menolak signature, When simpan, Then data tidak tersimpan dan toast "dibatalkan" muncul.
- **AC-SETTINGS-4** Given user belum connect, When membuka `/settings`, Then UI meminta connect wallet lebih dulu.

## Network-Mismatch Banner

- **AC-NETWORK-1** Given network wallet ≠ `NEAR_NETWORK` (mis. wallet mainnet, app testnet), When terdeteksi, Then banner peringatan tampil dan aksi transaksi (list/buy/offer/mint) disabled.
- **AC-NETWORK-2** Given network wallet = `NEAR_NETWORK`, When halaman dibuka, Then tidak ada banner peringatan dan transaksi aktif.

## Rate Limiting

- **AC-RATE-1** Given klien melebihi limit endpoint (mis. `GET /auth/nonce` > 10/IP/menit), When request berikutnya, Then 429 `RATE_LIMITED` + header `Retry-After`.
- **AC-RATE-2** Given 5 `POST /auth/verify` gagal/menit (IP+account), When percobaan ke-6, Then 429 lockout sementara; nonce yang belum dikonsumsi tetap valid sampai TTL.
- **AC-RATE-3** Given request ditolak rate limit, When diperiksa, Then TIDAK ada efek samping (tidak ada baris DB/report/audit baru).
- **AC-RATE-4** Given admin melakukan > 30 aksi/menit, When request berikutnya, Then 429 + `Retry-After` (admin juga terkena limit).

## Performance

- **AC-PERF-1** Given pemanggilan `buy`/`accept_offer`/`buy_bundle`/`nft_mint`, When dieksekusi, Then gas terpakai ≤ 300 Tgas per call (PRD §14).
- **AC-PERF-2** Given jalur `resolve_purchase` (sukses maupun gagal), When berjalan, Then gas ≤ budget 115 Tgas (payments.md).
- **AC-PERF-3** Given `buy_bundle` dengan 10 token royalti maksimum, When dieksekusi, Then gas ≤ budget dan payout ≤ 10 receiver unik (INV-021).
- **AC-PERF-4** Given endpoint API MVP, When diuji beban, Then p95 latensi ≤ 300 ms di luar waktu RPC (usulan; dikunci saat implementasi).
- **AC-PERF-5** Given discovery (★ fase 2) diakses, When melebihi 60/IP/menit, Then 429 — limit ditegakkan tanpa menurunkan akurasi data.
