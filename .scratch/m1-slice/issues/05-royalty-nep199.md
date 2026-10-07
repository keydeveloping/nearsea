# 05: Royalti NEP-199

**What to build:** Saat market menyelesaikan penjualan, koleksi memberi tahu cara membagi uangnya — jatah kreator, dibatasi 10% dari harga. Ini paruh kedua tesis: royalti ditegakkan on-chain, bukan dijanjikan.

**Blocked by:** 04 — butuh token yang ada dan bisa di-transfer sebelum payout bisa diuji.

**Status:** ready-for-agent

- [ ] `nft_transfer_payout` mengembalikan payout map dengan kreator sebagai satu-satunya penerima.
- [ ] Amount = `floor(harga × royalty_bps / 10_000)`.
- [ ] Royalti dibatasi **10% per token**; konfigurasi di atas cap ditolak saat init.
- [ ] Payout diturunkan dari konfigurasi royalti **level kontrak** (bukan `TokenMetadata.extra`, bukan per-token).
- [ ] View `royalty_config()` mengembalikan `{ receiver, bps }` untuk dipakai market.
- [ ] Gas 15 Tgas yang di-attach market ke `nft_transfer_payout` **terbukti cukup** untuk jalur payout.
- [ ] Sandbox hijau: payout ≤10% untuk beberapa harga berbeda, termasuk batas bawah (dust) dan batas atas cap (INV-003/027, TC-003 sebagian).

**Done-when (TASK-003):** Payout royalti ≤10% teruji (INV-003/027).

**Spec:** [docs/contracts/nft-collection.md](../../../docs/contracts/nft-collection.md) §4 · [docs/features/payments.md](../../../docs/features/payments.md) §Algoritma · INV-003/027
