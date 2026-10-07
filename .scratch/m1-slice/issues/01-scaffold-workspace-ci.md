# 01: Scaffold workspace + gate CI

**What to build:** Repo tempat `cargo test` dan `pnpm build` benar-benar jalan (walau masih placeholder), dan CI hijau di `dev` — sehingga setiap tiket berikutnya punya loop verifikasi yang bisa dipercaya. Workspace memuat paket kontrak dan frontend; workflow CI + Security menyala di tiga branch permanen.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Workspace terbentuk: paket kontrak ter-compile untuk target `wasm32-unknown-unknown`, frontend ter-build dengan TypeScript strict.
- [ ] CI menjalankan `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, dan `cargo test` (boleh lolos atas test placeholder).
- [ ] CI menjalankan `pnpm lint`, `pnpm typecheck`, `pnpm test`, dan `pnpm build`.
- [ ] Workflow Security menjalankan secret scan + audit dependency.
- [ ] Commit yang di-push ke `dev` menampilkan CI + Security **hijau** (bukti, bukan klaim).
- [ ] Versi toolchain ter-pin (mengikuti `rust-toolchain.toml` dan `.nvmrc`) sehingga build lokal = CI.
- [ ] Workflow `deploy-*.yml` ada tapi belum melakukan deploy nyata (tidak ada kredensial yang dibutuhkan).

**Done-when (TASK-001):** CI + Security hijau di `dev`; semua manifest & workflow ada.

**Spec:** [docs/development/ci-cd.md](../../../docs/development/ci-cd.md) · [docs/architecture/tech-stack.md](../../../docs/architecture/tech-stack.md) · [AGENTS.md](../../../AGENTS.md) §Build & Test Commands
