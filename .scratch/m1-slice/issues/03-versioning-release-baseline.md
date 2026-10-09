# 03: Baseline versioning & rilis

**What to build:** Setiap artefak punya versi yang bisa dilacak ke satu commit, sehingga deploy testnet pertama bisa diidentifikasi dan rollback punya sasaran yang jelas.

**Blocked by:** 01

**Status:** done — mekanisme lengkap & terverifikasi lokal (ronde 20). Sisa satu item (tag pertama) sengaja **tidak** dikerjakan: tag sah hanya di `testnet`/`mainnet` setelah PR promosi, dan merge ke sana butuh persetujuan user.

- [x] Skema SemVer per-artefak ditetapkan dan tertulis (kontrak / web / indexer).
      → [versioning-and-release.md](../../../docs/development/versioning-and-release.md) §1 + matriks bump §9
      (DECIDED ronde 14/15); kini **ditegakkan mesin**, bukan sekadar dokumen.
- [ ] Tag anotasi pertama dibuat di `mainnet` (mis. `contract-v0.1.0`) — dengan tagger, tanggal, dan pesan rilis.
      → ⏸️ **Sengaja belum.** Aturan [git-workflow.md](../../../docs/development/git-workflow.md) §12:
      tag hanya di `testnet`/`mainnet` setelah PR promosi merge, dan merge ke sana **wajib tanya user**
      (§3). Yang sudah siap: prosedur langkah-demi-langkah + perintah tag di
      [CHANGELOG.md](../../../CHANGELOG.md) §Rilis pertama. Rilis pertama yang disiapkan: `contract-v0.1.0`.
- [x] `CHANGELOG.md` punya entri untuk rilis pertama, format Keep a Changelog.
      → Entri `### Contract` di `Unreleased` sudah memuat pekerjaan rilis pertama (TASK-002 + TASK-032);
      heading `## [contract-v0.1.0] - YYYY-MM-DD` dibuat **saat tag di-push** (§5 langkah 7) — sengaja
      tidak ditulis sekarang supaya tidak menjanjikan rilis yang belum terjadi.
- [x] Jalur rollback tertulis dan bisa dieksekusi: revert merge commit promosi, atau deploy ulang tag sebelumnya.
      → [CHANGELOG.md](../../../CHANGELOG.md) §Rilis pertama → tabel rollback (branch target / kontrak
      ter-deploy / tag); aturan pemiliknya [versioning-and-release.md](../../../docs/development/versioning-and-release.md) §12
      + [ci-cd.md](../../../docs/development/ci-cd.md) §16 + [git-workflow.md](../../../docs/development/git-workflow.md) §15.
- [x] Versi artefak tertanam di build (versi kontrak) dan bisa dibaca dari luar.
      → `version` + `repository` di ketiga `Cargo.toml` → metadata NEP-330 tertanam di wasm. Dibuktikan
      dengan membaca string JSON langsung dari wasm: `version=0.1.0`,
      `link=https://github.com/keydeveloping/nearsea` (sebelumnya `link=null`). Diverifikasi **tiap PR**
      oleh [ci.yml](../../../.github/workflows/ci.yml) dan saat rilis oleh
      [release.yml](../../../.github/workflows/release.yml).
- [x] NEP-330 `contract_source_metadata` tersedia dan build reproducible.
      → `contract_source_metadata` tertanam (`{"version":…,"link":…,"standards":[{"standard":"nep330",…}]}`);
      build reproducible dikonfigurasi lewat `[package.metadata.near.reproducible_build]` (image Docker
      **ter-pin by digest**) dan dijalankan di `release.yml`. **Bukti eksekusi reproducible + ABI milik CI**
      (butuh Docker/Linux — di Windows lokal langkah ABI gagal linking, [ci-cd.md](../../../docs/development/ci-cd.md) §14).

**Done-when (TASK-032):** Tag pertama + CHANGELOG terisi + versi NEP-330 terverifikasi.
→ ✅ **CHANGELOG terisi** dan ✅ **versi NEP-330 terverifikasi** (lokal + gate CI tiap PR).
⏸️ **Tag pertama tertunda atas keputusan user** — bukan pekerjaan yang bisa diselesaikan agent tanpa
melanggar aturan wajib-tanya-user. Semua prasyaratnya (mekanisme, prosedur, bukti) sudah ada.

**Bukti gate lokal (ronde 20):** `cargo fmt --all -- --check` ✅ ·
`cargo clippy --all-targets -- -D warnings` (0 warning) ✅ · `cargo test --workspace` ✅ (34 test:
1 factory + 2 market + 31 koleksi) · `cargo metadata --locked` ✅ · metadata NEP-330 ketiga wasm ✅.

**Spec:** [docs/development/versioning-and-release.md](../../../docs/development/versioning-and-release.md) · [docs/development/git-workflow.md](../../../docs/development/git-workflow.md) §7/§12
