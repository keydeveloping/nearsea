# 02: Proteksi 3 branch + secret scan

**What to build:** Model tiga branch ditegakkan oleh platform, bukan cuma kesepakatan — tidak ada yang (termasuk agent di masa depan) bisa push langsung ke `mainnet`/`testnet`/`dev` atau menghapusnya, dan secret yang tidak sengaja ter-push tertangkap sebelum merge.

**Blocked by:** 01 — butuh workflow CI/Security sudah jalan agar proteksi punya status check yang bisa diwajibkan.

**Status:** resolved — proteksi aktif di ketiga branch (ronde 18c), diverifikasi lewat penolakan push langsung.

- [x] Proteksi aktif di `mainnet`, `testnet`, `dev`: tanpa direct push, tanpa force-push, tanpa delete.
      → ketiganya: `required_pull_request_reviews` aktif (PR wajib), `allow_force_pushes=false`, `allow_deletions=false`, `enforce_admins=true` (termasuk admin).
- [x] PR wajib sebelum merge ke tiap branch permanen.
      → bukti: `git push origin dev` ditolak — `GH006: Protected branch update failed … Changes must be made through a pull request. 5 of 5 required status checks are expected.`
- [x] Proteksi tag untuk pola `contract-v*` dan `web-v*`.
      → ruleset `protect-release-tags` (target `tag`, `enforcement: active`) atas `contract-v*`/`web-v*`/`indexer-v*` dengan aturan `deletion` + `update`. (API `tags/protection` 404 untuk repo ini; ruleset adalah jalur yang bekerja.)
- [x] Secret scan berjalan di setiap PR dan **menggagalkan check** saat menemukan temuan.
      → job `Secret scanning (gitleaks)` di `security.yml`, di-pin SHA, dan **masuk daftar required status checks** → temuan memblokir merge, bukan sekadar laporan.
- [x] CODEOWNERS aktif untuk jalur sensitif (`.github/workflows/**` + file higienitas secret).
      → `.github/CODEOWNERS` ada dan mencakup `.github/workflows/`, `.gitignore`, `.env.example`, `.gitleaks.toml`, `docs/security/`, `docs/development/`. **Catatan:** `require_code_owner_reviews` belum diaktifkan (setara menambah required approval → mengunci repo dengan satu akun); CODEOWNERS tetap berfungsi sebagai reviewer otomatis.
- [x] `.gitignore` diverifikasi tetap mengecualikan `.env` dan material kunci (sudah ada — konfirmasi, jangan tulis ulang).
      → dikonfirmasi tanpa menulis ulang: aturan `.env`, `.env.*`, `!.env.example`, `*.env`, `*.pem`, `*.key`, `id_rsa`, `near-credentials/` ada; `git check-ignore -v .env` → `.gitignore:14:*.env`. Satu-satunya file cocok pola yang ter-track = `.env.example` (template, bukan kredensial).
- [x] Tidak ada kredensial usable di source/contoh/test (audit cepat, bukan asumsi).
      → `gitleaks detect --config .gitleaks.toml` pada 151 file ter-track: **0 temuan**; gitleaks juga hijau sebagai job CI di `dev`.

**Done-when (TASK-031):** Proteksi 3 branch + secret scan hijau + CODEOWNERS aktif. — ✅ terpenuhi.

**Catatan keputusan (ronde 18c):** required approval = **0** (ditunda), bukan 2/1 seperti spec §8, karena repo hanya punya satu akun GitHub dan self-approve dilarang — approval akan mengunci semua PR. Dinaikkan saat maintainer kedua ada. Blokir force-push justru **lebih ketat** dari spec awal (`enforce_admins: true` di ketiga branch).

**Spec:** [docs/development/git-workflow.md](../../../docs/development/git-workflow.md) §8 · [docs/development/secrets-and-gitignore.md](../../../docs/development/secrets-and-gitignore.md) · [docs/security/cicd-security.md](../../../docs/security/cicd-security.md)
