# 07: Market buy — settlement, split royalti + fee, refund

**What to build:** Buyer bisa membeli NFT yang ter-list. Market memindahkan NFT, membagi pembayaran (royalti kreator, fee platform 2%, sisanya ke seller), dan **otomatis me-refund buyer bila ada yang gagal**. Setiap deposit yang masuk punya jalan keluar — termasuk bila callback settlement mati.

**Blocked by:** 06

**Status:** ready-for-agent

- [ ] `buy` dengan deposit ≥ harga memindahkan NFT ke buyer dan membayar kreator + treasury + seller dalam satu settlement.
- [ ] Payout dari kontrak koleksi diperlakukan **untrusted**: jumlah penerima, amount > 0, `Σpayout ≤ harga − fee`, dan sisa ≤ 1 yocto semuanya divalidasi.
- [ ] Fee = `floor(harga × fee_bps / 10_000)`; `fee_bps` default 200 dan tidak bisa melampaui cap 500.
- [ ] Payout tidak valid (receiver > batas, jumlah melebihi harga, sisa > 1 yocto) → **refund penuh buyer** dan listing dipulihkan.
- [ ] Kelebihan deposit di-refund ke buyer.
- [ ] Self-buy ditolak.
- [ ] Listing stale ditolak saat buy — **kedua** kasus: kepemilikan sudah pindah **atau** approval sudah tidak valid.
- [ ] Pembelian yang nyangkut (callback settlement gagal) bisa dipulihkan **oleh siapa pun** setelah jeda — buyer di-refund penuh, tanpa bergantung pada governance.
- [ ] Event `market_sale` ter-emit dengan payout map final.
- [ ] Anggaran gas `resolve_purchase` diverifikasi terhadap worst case (10 penerima payout + fee + seller + refund + event).
- [ ] Sandbox hijau: buy sukses, payout tidak valid → refund, dan **race 20 pembeli → tepat 1 menang, 19 refund penuh** (TC-002 paruh buy, TC-003, TC-016, TC-017, TC-022, TC-054).

**Done-when (TASK-005):** Buy sukses + refund + race 20 pembeli lolos (INV-001/016, TC-003/016).

**Spec:** [docs/contracts/market.md](../../../docs/contracts/market.md) §3, §3a, §3b · [docs/features/payments.md](../../../docs/features/payments.md) · [docs/development/concurrency-and-races.md](../../../docs/development/concurrency-and-races.md) · INV-001/002/003/009/016/031 · SEC-ORDER-001
