# 02: Proteksi 3 branch + secret scan

**What to build:** Model tiga branch ditegakkan oleh platform, bukan cuma kesepakatan — tidak ada yang (termasuk agent di masa depan) bisa push langsung ke `mainnet`/`testnet`/`dev` atau menghapusnya, dan secret yang tidak sengaja ter-push tertangkap sebelum merge.

**Blocked by:** 01 — butuh workflow CI/Security sudah jalan agar proteksi punya status check yang bisa diwajibkan.

**Status:** ready-for-agent

- [ ] Proteksi aktif di `mainnet`, `testnet`, `dev`: tanpa direct push, tanpa force-push, tanpa delete.
- [ ] PR wajib sebelum merge ke tiap branch permanen.
- [ ] Proteksi tag untuk pola `contract-v*` dan `web-v*`.
- [ ] Secret scan berjalan di setiap PR dan **menggagalkan check** saat menemukan temuan.
- [ ] CODEOWNERS aktif untuk jalur sensitif (`.github/workflows/**` + file higienitas secret).
- [ ] `.gitignore` diverifikasi tetap mengecualikan `.env` dan material kunci (sudah ada — konfirmasi, jangan tulis ulang).
- [ ] Tidak ada kredensial usable di source/contoh/test (audit cepat, bukan asumsi).

**Catatan:** butuh remote GitHub — langkah yang hanya bisa dilakukan manusia (buat repo, aktifkan proteksi, pasang secrets). Sisanya (CODEOWNERS, config) bisa dikerjakan agent.

**Done-when (TASK-031):** Proteksi 3 branch + secret scan hijau + CODEOWNERS aktif.

**Spec:** [docs/development/git-workflow.md](../../../docs/development/git-workflow.md) §8 · [docs/development/secrets-and-gitignore.md](../../../docs/development/secrets-and-gitignore.md) · [docs/security/cicd-security.md](../../../docs/security/cicd-security.md)
