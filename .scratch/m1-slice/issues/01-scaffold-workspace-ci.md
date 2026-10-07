# 01: Scaffold workspace + gate CI

**What to build:** Repo tempat `cargo test` dan `pnpm build` benar-benar jalan (walau masih placeholder), dan CI hijau di `dev` — sehingga setiap tiket berikutnya punya loop verifikasi yang bisa dipercaya. Workspace memuat paket kontrak dan frontend; workflow CI + Security menyala di tiga branch permanen.

**Blocked by:** None (can start immediately)

**Status:** resolved — PR #1 squash-merge ke `dev` (commit `223015b`); CI + Security hijau di `dev` dengan bukti run.

- [x] Workspace terbentuk: paket kontrak ter-compile untuk target `wasm32-unknown-unknown`, frontend ter-build dengan TypeScript strict.
      → Cargo workspace `contract/`+`market/`+`factory/`; frontend Next.js 16 + TS strict + Tailwind 4 + ESLint + Prettier + Vitest 4 + i18n.
- [x] CI menjalankan `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, dan `cargo test` (boleh lolos atas test placeholder).
      → hijau di CI **dan** lokal: fmt bersih, clippy 0 warning, `cargo test --workspace` 4 test.
- [x] CI menjalankan `pnpm lint`, `pnpm typecheck`, `pnpm test`, dan `pnpm build`.
      → plus `pnpm format:check`; hijau di CI **dan** lokal: 3 test Vitest, build sukses.
- [x] Workflow Security menjalankan secret scan + audit dependency.
      → gitleaks-action v2 + `cargo install cargo-audit --locked` + `cargo audit` + `pnpm audit` + dependency-review; semua action di-pin SHA.
- [x] Commit yang di-push ke `dev` menampilkan CI + Security **hijau** (bukti, bukan klaim).
      → bukti: CI run [37627825466](https://github.com/keydeveloping/nearsea/actions/runs/37627825466) **success** + Security run [37627825490](https://github.com/keydeveloping/nearsea/actions/runs/37627825490) **success** (ketiga job Security success), keduanya pada commit `223015b` di branch `dev`.
- [x] Versi toolchain ter-pin (mengikuti `rust-toolchain.toml` dan `.nvmrc`) sehingga build lokal = CI.
      → Rust 1.93.1 + near-sdk 5.29.1 (naik dari rujukan riset 1.77.1/4.x — dependency tree 5.x butuh `edition2024`); Node 24 LTS; CI membaca file pin, bukan angka lepas.
- [x] Workflow `deploy-*.yml` ada tapi belum melakukan deploy nyata (tidak ada kredensial yang dibutuhkan).
      → ketiganya `workflow_dispatch` saja + guard variabel environment kosong; auto-deploy diaktifkan di TASK-029.

**Done-when (TASK-001):** CI + Security hijau di `dev`; semua manifest & workflow ada. — ✅ terpenuhi.

**Catatan temuan (run CI pertama menemukan 3 bug nyata yang tidak terlihat lokal):**
1. `Cannot find name 'LayoutProps'` — typecheck CI gagal karena tipe Next.js belum di-generate (`.next/types`); lokal tertutup oleh `.next/` sisa `next build`. Perbaikan: script `typecheck` menjalankan `next typegen` lebih dulu.
2. `tinypool` (critical, prototype pollution → RCE) lewat vitest 3 → naik ke vitest 4 (tidak lagi memakai tinypool).
3. gitleaks 403 — action butuh `pull-requests: read`, workflow hanya punya `contents: read`.

Sisa satu advisory **tanpa patch upstream** (`braces <=3.0.3`, high, lewat toolchain `eslint-config-next`) — pengecualian ditulis eksplisit di `frontend/pnpm-workspace.yaml`, dilacak sebagai TASK-035.

**Spec:** [docs/development/ci-cd.md](../../../docs/development/ci-cd.md) · [docs/architecture/tech-stack.md](../../../docs/architecture/tech-stack.md) · [AGENTS.md](../../../AGENTS.md) §Build & Test Commands
