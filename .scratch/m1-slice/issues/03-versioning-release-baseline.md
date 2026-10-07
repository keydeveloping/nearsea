# 03: Baseline versioning & rilis

**What to build:** Setiap artefak punya versi yang bisa dilacak ke satu commit, sehingga deploy testnet pertama bisa diidentifikasi dan rollback punya sasaran yang jelas.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Skema SemVer per-artefak ditetapkan dan tertulis (kontrak / web / indexer).
- [ ] Tag anotasi pertama dibuat di `mainnet` (mis. `contract-v0.1.0`) — dengan tagger, tanggal, dan pesan rilis.
- [ ] `CHANGELOG.md` punya entri untuk rilis pertama, format Keep a Changelog.
- [ ] Jalur rollback tertulis dan bisa dieksekusi: revert merge commit promosi, atau deploy ulang tag sebelumnya.
- [ ] Versi artefak tertanam di build (versi kontrak) dan bisa dibaca dari luar.
- [ ] NEP-330 `contract_source_metadata` tersedia dan build reproducible **— AC ini menutup setelah paket kontrak pertama ter-build (tiket 04); boleh selesai lebih lambat dari AC lain di tiket ini.**

**Done-when (TASK-032):** Tag pertama + CHANGELOG terisi + versi NEP-330 terverifikasi.

**Spec:** [docs/development/versioning-and-release.md](../../../docs/development/versioning-and-release.md) · [docs/development/git-workflow.md](../../../docs/development/git-workflow.md) §7
