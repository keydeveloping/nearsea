# 06: Market listing — storage, 2-tx, dual verification

**What to build:** Seller bisa memasang NFT untuk dijual dengan harga tetap **tanpa menyerahkan tokennya**. Listing butuh dua transaksi (approve, lalu list), dan market memverifikasi sendiri kepemilikan **dan** approval sebelum menerima — jadi listing tidak pernah lahir dari klaim sepihak.

**Blocked by:** 04 — butuh kontrak koleksi dengan `nft_token`/`nft_is_approved` untuk diverifikasi.

**Status:** ready-for-agent

- [ ] `list_nft_for_sale` membuat listing **hanya** bila verifikasi silang milik market lolos: pemanggil adalah pemilik token **dan** market sedang di-approve.
- [ ] Verifikasi dijalankan sebagai cross-contract call ke koleksi, bukan dari argumen pemanggil (tidak percaya klaim).
- [ ] NFT **tetap di wallet seller** selama listing — non-custodial, dibuktikan dengan pembacaan on-chain, bukan diklaim.
- [ ] Listing butuh deposit storage NEP-145; listing dengan deposit kurang gagal.
- [ ] `remove_sale` membatalkan listing dan mencabut approval market; mewajibkan tepat 1 yocto.
- [ ] `update_price` mengubah harga in-place dan menolak harga di bawah minimum.
- [ ] Harga di bawah 0,01 Ⓝ ditolak.
- [ ] Listing ganda untuk token yang sama ditolak.
- [ ] Harga/listing tidak bisa diubah oleh selain pemilik token.
- [ ] Event `market_list` dan `market_delist` ter-emit dengan payload sesuai katalog.
- [ ] Sandbox hijau: list → cancel, plus jalur gagal (TC-002 paruh list, TC-013, TC-020, TC-044).

**Done-when (TASK-004):** List/cancel + storage deposit lolos (INV-020, TC-002).

**Spec:** [docs/contracts/market.md](../../../docs/contracts/market.md) §2, §5 · [docs/features/marketplace.md](../../../docs/features/marketplace.md) §Flow — List · INV-007/013/020/030 · SEC-CONTRACT-001
