# 04: NFT core — mint, metadata, approval, satu fase publik

**What to build:** Koleksi NFT yang bisa di-deploy, di-mint, dan di-transfer — dengan metadata standar dan approval. Ini paruh pertama tesis slice: NFT-nya ada dan pemiliknya yang memegang, bukan kontrak.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Kontrak koleksi ter-deploy (testnet) dan `nft_mint` berhasil untuk wallet yang lolos validasi fase.
- [ ] Mint **bergantung fase**: tanpa fase terkonfigurasi, mint mustahil; dengan satu fase publik terbuka, mint jalan. `set_phases` minimal (satu fase publik, `allowlist_required = false`) ikut di tiket ini — tanpanya koleksi tidak bisa di-mint sama sekali.
- [ ] Deposit saat mint **cocok persis** dengan `harga fase × quantity` (bukan ≥); mismatch ditolak.
- [ ] `nft_transfer` mewajibkan tepat 1 yocto dan memindahkan kepemilikan.
- [ ] `nft_metadata` dan metadata per-token mengembalikan name/symbol/base_uri yang dikonfigurasi.
- [ ] NEP-178 lengkap: `nft_approve`, `nft_is_approved`, `nft_revoke`, `nft_revoke_all`.
- [ ] Event NEP-297 ter-emit untuk mint dan transfer (envelope `EVENT_JSON:`, satu baris).
- [ ] Storage mengikuti NEP-145: minter membayar storage token-nya; mint tanpa deposit cukup gagal (bukan diam-diam memakai saldo kontrak).
- [ ] Init menolak `royalty_bps` di luar `1..=1000`.
- [ ] Init memakai `PanicOnDefault` — memanggil method tanpa init = panic, tidak ada jalur init ulang.
- [ ] Storage-key prefix terdokumentasi dan tidak berubah sembarangan (layout stabil untuk upgrade).
- [ ] Sandbox hijau: mint → transfer → events (TC-001).

**Done-when (TASK-002):** `cargo test` hijau; mint + transfer + events lolos TC-001.

**Spec:** [docs/contracts/nft-collection.md](../../../docs/contracts/nft-collection.md) §1–§3, §6–§7 · [docs/features/launchpad.md](../../../docs/features/launchpad.md) · INV-017/018/019/021/029 · SEC-CONTRACT-002/008
