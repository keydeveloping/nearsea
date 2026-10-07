# 10: FE — browse / list / buy

**What to build:** Demo M1 yang bisa dilihat orang: dua akun testnet membuka aplikasi, satu mem-mint dan memasang NFT untuk dijual lewat dua langkah yang jelas, satu lagi membelinya — dan breakdown pembayaran (royalti kreator + fee 2%) terlihat di UI. Inilah tiket yang menutup tesis slice.

**Blocked by:** 09 (wallet connect) dan 07 (kontrak lengkap). Dua jalur berbeda — FE butuh wallet, dan jalur emas butuh kontrak nyata untuk dijalankan end-to-end.

**Status:** ready-for-agent

- [ ] Grid/browse menampilkan listing yang dibaca langsung dari kontrak via view call RPC.
- [ ] Halaman detail NFT menampilkan metadata, harga, dan status listing.
- [ ] Pemilik token melihat tombol Sell; bukan-pemilik melihat Buy.
- [ ] Alur listing memakai modal **dua langkah** dengan progres yang menjelaskan langkah 1/2 (approve, lalu list) — bukan satu tombol yang tiba-tiba minta dua tanda tangan.
- [ ] Alur buy mengirim deposit = harga dan menampilkan **breakdown fee + royalti** sebelum konfirmasi.
- [ ] Hasil transaksi jelas: sukses dan gagal sama-sama memberi umpan balik yang terbaca (bukan diam).
- [ ] Saat transaksi menunggu, tombol disabled + indikator progres; setelah final, data di-refresh.
- [ ] Listing stale disembunyikan dari discovery; halaman token menampilkan status "tidak lagi tersedia" bila stale.
- [ ] Semua angka Ⓝ diformat dari string yoctoNEAR — **tidak ada aritmetika float** pada nilai yocto.
- [ ] Semua string UI lewat `i18n/` (bahasa Inggris); tidak ada string hardcoded di komponen.
- [ ] Metadata dari luar dirender lewat `<img>` tanpa `innerHTML`.
- [ ] Jalur emas Playwright hijau: browse → list → buy dengan dua akun di testnet.

**Done-when (TASK-008):** Browse->list->buy end-to-end via UI (jalur emas Playwright).

**Spec:** [docs/features/marketplace.md](../../../docs/features/marketplace.md) §UI · [docs/architecture/frontend-architecture.md](../../../docs/architecture/frontend-architecture.md) §2–§4, §8–§9 · [docs/04-ux-ui-spec.md](../../../docs/04-ux-ui-spec.md) · [tasks/milestones.md](../../../tasks/milestones.md) §Demo M1
