# 01: Scaffold workspace + gate CI

**What to build:** Repo tempat `cargo test` dan `pnpm build` benar-benar jalan (walau masih placeholder), dan CI hijau di `dev` — sehingga setiap tiket berikutnya punya loop verifikasi yang bisa dipercaya. Workspace memuat paket kontrak dan frontend; workflow CI + Security menyala di tiga branch permanen.

**Blocked by:** None (can start immediately)

**Status:** in-progress — semua artefak ada & gate lokal hijau; sisa: push ke remote GitHub untuk membuktikan CI + Security hijau di `dev`.

- [x] Workspace terbentuk: paket kontrak ter-compile untuk target `wasm32-unknown-unknown`, frontend ter-build dengan TypeScript strict.
      → Cargo workspace `contract/`+`market/`+`factory/`; frontend Next.js 16 + TS strict + Tailwind 4 + ESLint + Prettier + Vitest + i18n.
- [x] CI menjalankan `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, dan `cargo test` (boleh lolos atas test placeholder).
      → terverifikasi lokal: fmt bersih, clippy 0 warning, `cargo test --workspace` 4 test hijau.
- [x] CI menjalankan `pnpm lint`, `pnpm typecheck`, `pnpm test`, dan `pnpm build`.
      → plus `pnpm format:check`; terverifikasi lokal: 3 test Vitest hijau, build sukses.
- [x] Workflow Security menjalankan secret scan + audit dependency.
      → gitleaks-action v2 + rustsec/audit-check + `pnpm audit` + dependency-review; semua action di-pin SHA.
- [ ] Commit yang di-push ke `dev` menampilkan CI + Security **hijau** (bukti, bukan klaim).
      → **BLOCKED**: repo belum punya remote GitHub. Butuh keputusan user (buat repo + push).
- [x] Versi toolchain ter-pin (mengikuti `rust-toolchain.toml` dan `.nvmrc`) sehingga build lokal = CI.
      → Rust 1.93.1 + near-sdk 5.29.1 (naik dari rujukan riset 1.77.1/4.x — dependency tree 5.x butuh `edition2024`); Node 24 LTS.
- [x] Workflow `deploy-*.yml` ada tapi belum melakukan deploy nyata (tidak ada kredensial yang dibutuhkan).
      → ketiganya `workflow_dispatch` saja + guard variabel environment kosong; auto-deploy diaktifkan di TASK-029.

**Done-when (TASK-001):** CI + Security hijau di `dev`; semua manifest & workflow ada. — manifest/workflow ✅; "hijau di `dev`" menunggu remote.

**Spec:** [docs/development/ci-cd.md](../../../docs/development/ci-cd.md) · [docs/architecture/tech-stack.md](../../../docs/architecture/tech-stack.md) · [AGENTS.md](../../../AGENTS.md) §Build & Test Commands
