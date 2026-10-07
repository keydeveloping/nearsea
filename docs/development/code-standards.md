# Code Standards

> Konvensi kode & komentar untuk NearSea. Ringkas — aturan yang mengikat.
> Pipeline: [ci-cd.md](./ci-cd.md). Aturan kerja agent: [AGENTS.md](../../AGENTS.md).

## 1. Kebijakan komentar (WAJIB dibaca)

> Permintaan eksplisit pemilik proyek: **komentar hanya di baris yang benar-benar penting.**

**Boleh dikomentari:**
- Keputusan non-obvious: *mengapa* kode begini, bukan *apa* yang dilakukan.
- Batasan protokol/keamanan (mis. `assert_one_yocto` mencegah function-call key abuse).
- Angka/konstanta sihir yang perlu alasan (mis. `MAX_FEE_BPS` = batas governance).
- Peringatan (mis. "jangan ubah urutan: dipakai sebagai storage key").
- Link ke dokumen spesifikasi/invariant bila kode mengimplementasikannya.

**DILARANG dikomentari:**
- Baris yang sudah jelas dari namanya (`// increment counter` → tidak perlu).
- Riwayat perubahan ("dulu begini", "diubah di PR #12") — itu tugas git & CHANGELOG.
- Komentar yang mengulang kode, atau menandai bagian yang belum jadi tanpa task.
- Blok komentar panjang berisi narasi; ringkas atau pindahkan ke `docs/`.

**Aturan praktis:** jika menghapus komentar tidak mengurangi pemahaman pembaca berikutnya,
komentar itu tidak perlu. `TODO` hanya boleh ada bila menunjuk task di
[tasks/backlog.md](../../tasks/backlog.md); selain itu dilarang.

## 2. Rust (kontrak)

- `cargo fmt` + `cargo clippy` **tanpa warning** (gate CI).
- Penamaan standar Rust (snake_case fungsi/var, CamelCase tipe).
- `near-sdk` + `near-sdk-contract-tools`; ikuti pola tutorial (RESEARCH.md §10).
- Semua aritmetika uang pakai **checked math** (`checked_add`/`checked_sub`) — dilarang `+`/`-` mentah pada saldo.
- Nilai u128 dikembalikan sebagai **string** di JSON.
- Callback cross-contract selalu `#[private]`.
- Setiap method mutasi wajib punya test sandbox (AGENTS.md Testing Rules).
- Dokumentasi layout state per rilis (SEC-CONTRACT-008).

## 3. TypeScript (frontend & API)

- **Strict mode**; dilarang `any` kecuali diberi alasan eksplisit di komentar (dan lebih baik dihindari).
- `eslint` + `prettier` (gate CI).
- Semua string UI lewat `i18n/` — **tidak ada hardcode** teks di komponen (bahasa Inggris).
- Nilai yoctoNEAR selalu `string`, tidak pernah `number`.
- Panggilan kontrak/RPC lewat satu lapisan klien terpusat (bukan fetch tersebar).
- Error ditangani lewat katalog terpusat: [error-handling.md](./error-handling.md) — kode/pesan
  hanya di satu modul error terpusat, diimpor (bukan literal ad-hoc). Lihat §8 dokumen itu.

## 4. Umum

- **Ikuti pola yang sudah ada.** Bila sebuah hal sudah punya konvensi/modul bersama (mis. error,
  klien RPC, format uang), gunakan itu — jangan menciptakan pola baru yang berdampingan.
- Line ending LF (lihat [.gitattributes](../../.gitattributes)); indentasi via [.editorconfig](../../.editorconfig).
- Tidak ada secret di kode — lihat [secrets-and-gitignore.md](./secrets-and-gitignore.md).
- Nama file & folder konsisten (kebab-case untuk dokumen, konvensi bahasa masing-masing untuk kode).
- Kode harus bisa di-build di CI tanpa akses secret produksi.

## 5. Urutan import

**TypeScript** — satu aturan, dijaga ESLint (`import/order`, PROPOSED dikonfigurasi di §9):

```text
1. node builtins        (node:fs, node:path)
2. external packages    (react, next, @tanstack/react-query)
3. internal alias       (@/lib/…, @/features/…, @/components/…)
4. relative parent      (../foo)
5. relative sibling     (./bar)
6. type-only imports    (import type { … })
```

- Satu baris kosong antar grup; urut alfabetis **di dalam** grup.
- Dilarang import lintas-fitur langsung (`features/a` → `features/b`) — lewat `lib/` (frontend-architecture §1).
- Import tipe selalu `import type` (atau `import { type X }`) agar tidak ikut bundle runtime.

**Rust** — dikelola `rustfmt` dengan konfigurasi (PROPOSED, `rustfmt.toml`):

```toml
# rustfmt.toml (PROPOSED — diaktifkan saat scaffold)
edition = "2021"
group_imports = "StdExternalCrate"
imports_granularity = "Module"
```

- Urutan: `std` → crate eksternal → `crate::`/`super::`/`self::`.
- Hindari `use super::*;` (wildcard) di kode produksi; eksplisit lebih mudah diaudit.
- `use` yang tidak terpakai = error CI (`cargo clippy -D warnings`).

## 6. Penamaan file & komponen FE

| Artefak | Pola | Contoh |
|---|---|---|
| Komponen React (file) | `PascalCase.tsx` | `BuyButton.tsx`, `ListingCard.tsx` |
| Hook | `useThing.ts` (camelCase, prefix `use`) | `useBuy.ts`, `useListing.ts` |
| Modul util/lib | `kebab-case.ts` | `format-near.ts`, `query-keys.ts` |
| Tipe | `types.ts` atau `<domain>.types.ts` | `marketplace.types.ts` |
| Folder | `kebab-case` | `features/marketplace/`, `components/ui/` |
| Halaman (App Router) | `page.tsx` / `layout.tsx` / `error.tsx` | `app/token/[contract]/[tokenId]/page.tsx` |
| Konstanta | `constants.ts` | `lib/constants/contracts.ts` |
| Fixture/test util | `test-utils.tsx` | — |

- Satu komponen = satu file; satu file mengekspor **satu** komponen utama (sub-komponen privat boleh menyertai).
- Nama komponen = nama file (tanpa `.tsx`).
- Dilarang nama file generik (`utils.ts`, `helpers.ts`, `misc.ts`) — beri nama sesuai domain.
- Barrel file (`index.ts`) hanya untuk `components/ui/` (design system); fitur tidak memakai barrel (menyulitkan tree-shaking & traceability).

## 7. Struktur modul

Selaras [frontend-architecture.md](../architecture/frontend-architecture.md) §1:

```text
features/<fitur>/
  components/   presentational + container
  hooks/        useX (orchestrasi state & tx)
  api/          panggilan view-call/REST (tipis, tanpa UI)
  types/        tipe domain fitur
  <Page>.tsx    komposisi halaman (dipanggil app/<route>/page.tsx)
```

- **Arah dependensi satu arah**: `app/ → features/ → lib/ → components/ui/`; tidak ada impor balik.
- Satu modul = satu tanggung jawab; file `api/` **tidak** memuat JSX; komponen **tidak** memanggil `fetch`/RPC langsung (lewat hook/`api/`).
- Modul lintas-fitur (`lib/`) bebas dari konsep fitur (tidak tahu "marketplace"/"launchpad").
- Modul error terpusat (`lib/errors/`) = satu-satunya tempat kode/pesan error didefinisikan ([error-handling.md](./error-handling.md) §8).

## 8. Penamaan & tata letak file test

Mengikuti konvensi di [testing-strategy.md](../testing/testing-strategy.md) §Konvensi penamaan test.

| Jenis test | Lokasi | Pola nama |
|---|---|---|
| Rust unit | bersebelahan (`src/foo.rs` → `#[cfg(test)] mod tests`) | `test_<method>_<kondisi>_<harapan>` |
| Rust sandbox/integration | `tests/<topik>.rs` | `mint_list_buy.rs` |
| Rust invariant | `tests/inv_<nnn>_<topik>.rs` | `inv_025_bundle_prevalidation.rs` |
| Fuzz | `fuzz/fuzz_targets/<target>.rs` | `payout_parse.rs` |
| TS unit/komponen | bersebelahan dengan sumber (`Button.tsx` → `Button.test.tsx`) | `*.test.ts(x)` |
| API integration | `tests/api/<method>_<path>.spec.ts` | `patch_profile.spec.ts` |
| E2E | `e2e/<flow>.spec.ts` | `buy-happy-path.spec.ts` |

- Setiap test mencantumkan ID di komentar header: `// TC-002 · INV-001, INV-011`.
- Dilarang folder `__tests__` campur untuk semua jenis — pisahkan unit (bersebelahan) vs integration/E2E (folder khusus).
- Fixture bersama di `tests/fixtures/` (atau `test-utils`), **bukan** disalin antar test.

## 9. Konfigurasi lint (aturan yang diaktifkan)

**TypeScript / ESLint** (`.eslintrc` — PROPOSED, TASK-001):

```jsonc
{
  "extends": ["next/core-web-vitals", "plugin:@typescript-eslint/strict"],
  "rules": {
    "@typescript-eslint/no-explicit-any": "error",
    "@typescript-eslint/consistent-type-imports": "error",
    "import/order": ["error", { "newlines-between": "always", "alphabetize": { "order": "asc" } }],
    "no-restricted-imports": ["error", { "patterns": ["../features/*", "@/features/*/../*"] }],
    "react/jsx-no-useless-fragment": "error",
    "no-console": ["warn", { "allow": ["warn", "error"] }]
  }
}
```

- `any` = error (bukan warning); pengecualian harus lewat komentar ber-alasan + `eslint-disable-next-line` — dan tetap dibahas di review.
- `no-restricted-imports` menegakkan larangan impor lintas-fitur (§5).
- Prettier untuk format (dijalankan lewat ESLint atau terpisah); konflik aturan diformat oleh Prettier.

**Rust**: `cargo fmt` (format) + `cargo clippy --all-targets -- -D warnings` (lint). Dilarang `#[allow(...)]` tanpa komentar alasan; `#[allow(clippy::…)]` yang menonaktifkan lint keamanan (`arithmetic_side_effects` bila diaktifkan) = review wajib.

## 10. Konvensi kunci i18n

Struktur file dimiliki [frontend-architecture.md](../architecture/frontend-architecture.md) §9; konvensi **kunci**:

```text
<namespace>.<section>.<element>            → camelCase segmen, dot notation
errors.<ERROR_CODE>                        → kode error VERBATIM dari registry
common.<element>                           → tombol/label lintas fitur
```

| Aturan | Contoh benar | Contoh salah |
|---|---|---|
| Segmen camelCase | `marketplace.buyButton.label` | `marketplace.buy_button.label` |
| Kode error verbatim | `errors.CONFLICT_SOLD` | `errors.conflictSold` |
| Placeholder bernama | `"offer.expiresIn": "Expires in {days} days"` | `"Expires in {}"` |
| Tidak ada string EN di kode | `t('common.retry')` | `"Retry"` di JSX |
| Tidak ada kalimat panjang di kode | kunci + `{count}` | menyusun kalimat di komponen |

- Kunci bertipe (TypeScript) sehingga kunci salah = **error build** (frontend-architecture §9).
- Kode error baru → tambahkan kunci `errors.<CODE>` bersamaan dengan update registry [error-handling.md](./error-handling.md) §3 (satu PR).
- Dilarang memakai teks sebagai kunci (`t("Retry")`) — kunci adalah identifier, bukan kalimat.

## 11. Konvensi tipe respons API

- Tipe respons didefinisikan **eksplisit** per endpoint di `features/<fitur>/types/` atau `lib/api/types.ts`; dilarang `any`/`object`.
- Nilai uang = **branded string type**, bukan `string` polos maupun `number`:

```ts
// lib/format/money.ts
export type YoctoNear = string & { readonly __brand: 'YoctoNear' };
export function asYocto(v: string): YoctoNear { /* validasi digit saja */ return v as YoctoNear; }
```

- Timestamp = `string` (ISO-8601 UTC), bukan `Date` mentah di boundary.
- Field opsional ditandai `?` + `| null` sesuai kontrak [endpoints.md](../api/endpoints.md) (mis. `allowed_buyer: string | null`).
- Envelope error = satu tipe bersama `ApiError { code; message; requestId }` — kode di-*narrow* ke union kode registry (bukan `string` bebas).
- Validasi runtime di boundary (respons eksternal RPC/NearBlocks) = **PROPOSED** (schema validator, mis. zod) — sampai dikunci, gunakan guard fungsi eksplisit; jangan percaya `as`.
- Respons daftar selalu `{ items: T[]; nextCursor: string | null }`.
- Klien **wajib** mengabaikan field tak dikenal (forward-compatible, api-overview) — tipe tidak boleh `exact`.

## 12. Batas ukuran fungsi & file

> Angka = **engineering default (DECIDED ronde 15)**; boleh dilampaui dengan alasan di komentar/review, bukan aturan buta.

| Unit | Batas lunak | Tindakan bila lewat |
|---|---|---|
| Fungsi Rust | ≤ 50 baris | Pecah jadi helper privat; pindahkan logika murni ke modul teruji |
| Fungsi TS | ≤ 40 baris | Ekstrak hook/helper |
| Komponen React | ≤ 200 baris | Pecah sub-komponen; pindahkan logika ke hook |
| File/modul | ≤ 400 baris | Pecah per tanggung jawab |
| Parameter fungsi | ≤ 4 | Pakai struct/objek argumen |
| Kedalaman nesting | ≤ 3 | Early return |

- Kontrak: method `#[payable]` publik sebaiknya tipis — logika berat di helper privat (lebih mudah diuji & diaudit).
- File besar yang tak terhindarkan (mis. registry error) boleh dikecualikan; sebut alasan di review.

## 13. Contoh baik / buruk

**Komentar:**

```rust
// BURUK — mengulang kode
// increment counter
counter += 1;

// BAIK — menjelaskan alasan non-obvious
// assert_one_yocto mencegah function-call key menyentuh mutasi seller (SEC-CONTRACT-001).
assert_one_yocto();
```

**Nilai uang:**

```ts
// BURUK — presisi hilang & melanggar aturan
const price = Number(response.price_yocto);

// BAIK — string yoctoNEAR, format hanya saat tampil
const price = asYocto(response.price_yocto);
const display = formatNear(price); // "1.00 NEAR"
```

**Error:**

```ts
// BURUK — pesan ad-hoc, tidak terdaftar
throw new Error('Listing sudah terjual');

// BAIK — kode dari modul terpusat + kunci i18n
throw new AppError('CONFLICT_SOLD'); // errors.CONFLICT_SOLD
```

**Impor & lapisan:**

```ts
// BURUK — komponen memanggil RPC langsung + impor lintas-fitur
import { getSale } from '../../profile/api/rpc';
const sale = await fetch(rpcUrl, …);

// BAIK — lewat hook fitur, data dari api/ fitur sendiri
import { useListing } from '../hooks/useListing';
const { data } = useListing(contractId, tokenId);
```

## 14. Pre-commit hooks (daftar)

Status **PROPOSED** — dipasang saat scaffold (TASK-001/031). Memakai framework `pre-commit` (menjalankan hook lintas bahasa) + `commitlint` untuk pesan commit.

```yaml
# .pre-commit-config.yaml — PROPOSED
repos:
  - repo: https://github.com/gitleaks/gitleaks
    rev: <pin>
    hooks: [{ id: gitleaks }]                 # secret scan (wajib, selaras SEC-CICD-002)
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt --check
        entry: cargo fmt --all -- --check
        language: system
        files: \.rs$
      - id: cargo-clippy
        name: cargo clippy (deny warnings)
        entry: cargo clippy --all-targets -- -D warnings
        language: system
        files: \.rs$
        pass_filenames: false
      - id: eslint
        name: eslint
        entry: pnpm --dir frontend lint
        language: system
        files: \.(ts|tsx)$
        pass_filenames: false
      - id: prettier-check
        name: prettier --check
        entry: pnpm --dir frontend exec prettier --check
        language: system
        files: \.(ts|tsx|json|md)$
      - id: conventional-commit
        name: commitlint
        entry: pnpm exec commitlint --edit
        language: system
        stages: [commit-msg]
```

Daftar hook minimum:

| Hook | Tahap | Tujuan |
|---|---|---|
| gitleaks | `pre-commit` | Cegah secret ter-commit (SEC-CICD-002) |
| `cargo fmt --check` | `pre-commit` | Format Rust |
| `cargo clippy -D warnings` | `pre-commit` | Lint Rust |
| eslint | `pre-commit` | Lint TS (aturan §9) |
| prettier `--check` | `pre-commit` | Format TS/JSON/MD |
| commitlint | `commit-msg` | Format conventional commit (git-workflow §7) |
| unit test cepat (opsional) | `pre-push` | Smoke unit sebelum push |

```bash
# Pemasangan (setelah clone)
pipx install pre-commit          # atau: pip install --user pre-commit
pre-commit install               # hook pre-commit
pre-commit install --hook-type commit-msg
pnpm --dir frontend exec commitlint --version   # verifikasi commitlint tersedia
pre-commit run --all-files       # uji seluruh hook sekali
```

- Hook lokal **melengkapi**, bukan menggantikan CI: CI tetap menjalankan gitleaks + lint + test (ci-cd.md §3).
- Hook boleh di-*bypass* (`--no-verify`) **hanya** untuk alasan darurat yang dicatat; CI tetap menjadi penjaga akhir.
- Clippy penuh bisa lambat — bila mengganggu, jalankan `cargo clippy` di `pre-push` dan `cargo fmt --check` di `pre-commit` (PROPOSED).

## 15. Status

- **DECIDED (ronde 14)** — berlaku sejak scaffold Fase 1.
- Urutan import, penamaan, struktur modul, konvensi i18n & tipe API, batas ukuran, pre-commit (§5–§14) — **DECIDED (ronde 15)**; konfigurasi lint/hook final ditulis saat scaffold (TASK-001).
