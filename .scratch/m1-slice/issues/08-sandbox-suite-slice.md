# 08: Suite sandbox dua-kontrak (slice)

**What to build:** Satu suite yang membuktikan seluruh slice berjalan end-to-end di chain lokal sungguhan: mint → list → buy → royalti dan fee terbayar → jalur refund → race 20 pembeli. Inilah artefak yang membuat tesis jadi **terbukti**, bukan sekadar terimplementasi.

**Blocked by:** 04, 05, 06, 07 — suite ini menguji keempatnya; tidak ada yang bisa diuji sebelum ada.

**Status:** ready-for-agent

- [ ] Suite menjalankan **dua kontrak nyata** (koleksi + market) di sandbox chain, bukan mock.
- [ ] Jalur bahagia hijau: mint → list (2 tx) → buy → kreator + treasury + seller terbayar dengan angka yang benar.
- [ ] NFT **terbukti** berada di wallet seller selama window listing (diasersi on-chain, bukan disimpulkan).
- [ ] Jalur gagal hijau: payout tidak valid → refund penuh; storage kurang; listing stale (dua kasus); harga di bawah minimum.
- [ ] Race: 20 pembeli bersamaan → tepat 1 penjualan, 19 refund penuh.
- [ ] Pemulihan pembelian nyangkut hijau (TC-054).
- [ ] Stale karena approval dicabut hijau (TC-053) **dan** stale karena kepemilikan pindah (TC-006).
- [ ] Setiap invariant subset slice punya **minimal satu test**: INV-001..016, 023, 030, 031. Invariant M1+ (bundle/launchpad-penuh/pause) di-defer eksplisit, bukan dibiarkan tanpa jejak.
- [ ] Test case slice yang disepakati semuanya runnable dan hijau: TC-001, 002, 013, 016, 017, 020, 022, 044, 047, 048, 053, 054.
- [ ] Suite berjalan di CI dan **hijau** (bukan hanya hijau di lokal).

**Done-when (TASK-006):** Suite sandbox hijau; semua INV **slice** punya test (INV M1+ di-defer eksplisit).

**Spec:** [docs/testing/test-cases.md](../../../docs/testing/test-cases.md) §Cakupan slice M1 · [docs/testing/testing-strategy.md](../../../docs/testing/testing-strategy.md) · [docs/security/smart-contract-invariants.md](../../../docs/security/smart-contract-invariants.md)
