# 09: FE — connect wallet

**What to build:** Pengunjung bisa menyambungkan wallet NEAR-nya dan melihat dirinya terhubung, dengan jaringan yang benar ditampilkan. Ini pintu masuk untuk semua aksi berikutnya — tanpa ini, tidak ada yang bisa menandatangani transaksi.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Tombol connect membuka pemilih wallet dan berhasil menyambung di testnet.
- [ ] Wallet yang didukung near-connect tersedia sejak awal (daftar resmi), bukan satu wallet hardcoded.
- [ ] Alamat akun yang terhubung tampil di UI setelah connect.
- [ ] Banner/indikator jaringan tampil dan akurat: peringatan jelas bila wallet berada di jaringan yang salah.
- [ ] Disconnect berfungsi dan mengembalikan UI ke keadaan belum-connect.
- [ ] Aplikasi tetap berfungsi tanpa wallet (mode baca), bukan halaman kosong.
- [ ] Konfigurasi jaringan mengikuti env (`NEAR_NETWORK`) — bukan hardcoded di komponen.
- [ ] Tidak ada private key / seed / kredensial apa pun di kode FE atau bundle.
- [ ] Sandbox hijau: connect → disconnect, plus render tanpa wallet.

**Done-when (TASK-007):** Connect/disconnect + banner network jalan di testnet.

**Spec:** [docs/architecture/frontend-architecture.md](../../../docs/architecture/frontend-architecture.md) §Wallet integration · [docs/features/auth.md](../../../docs/features/auth.md) · SEC-AUTH-001
