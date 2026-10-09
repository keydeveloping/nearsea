import { defineConfig, globalIgnores } from "eslint/config";
import nextVitals from "eslint-config-next/core-web-vitals";
import nextTs from "eslint-config-next/typescript";

// Impor lintas-fitur: `features/<A>` → `features/<B>` dilarang, harus lewat `lib/`
// (frontend-architecture.md §1). Dipakai ulang oleh ketiga lapisan di bawah.
const crossFeatureImport = {
  group: ["@/features/*", "@/features/**"],
  message:
    "Impor lintas-fitur dilarang (frontend-architecture.md §1) — pindahkan ke lib/ atau components/ui/.",
};

const forbid = (patterns) => ["error", { patterns }];

const eslintConfig = defineConfig([
  ...nextVitals,
  ...nextTs,
  {
    settings: {
      // `@/*` adalah alias tsconfig untuk berkas repo, bukan paket eksternal;
      // tanpa ini import/order menaruhnya satu grup dengan react/next.
      "import/internal-regex": "^@/",
    },
    rules: {
      // Aturan proyek: `any` dilarang tanpa alasan eksplisit (code-standards.md §9).
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/consistent-type-imports": "error",
      "no-console": ["warn", { allow: ["warn", "error"] }],
      // Urutan import (code-standards.md §5).
      "import/order": [
        "error",
        {
          groups: ["builtin", "external", "internal", "parent", "sibling", "index", "type"],
          "newlines-between": "always",
          alphabetize: { order: "asc", caseInsensitive: true },
        },
      ],
      "react/jsx-no-useless-fragment": "error",
    },
  },
  // Kolom "Dilarang impor" tabel frontend-architecture.md §1, per lapisan. Tidak bisa
  // satu blok tunggal karena `app/` justru boleh mengimpor semuanya.
  {
    files: ["features/**/*.{ts,tsx}"],
    rules: { "no-restricted-imports": forbid([crossFeatureImport]) },
  },
  {
    files: ["lib/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": forbid([
        crossFeatureImport,
        {
          group: ["@/app/*", "@/app/**"],
          message: "`lib/` tidak boleh mengimpor `app/` (frontend-architecture.md §1).",
        },
        {
          group: ["@/components/ui/*", "@/components/ui/**"],
          message: "`lib/` tidak boleh mengimpor `components/ui/` (frontend-architecture.md §1).",
        },
      ]),
    },
  },
  {
    files: ["components/ui/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": forbid([
        crossFeatureImport,
        {
          group: ["@/app/*", "@/app/**"],
          message: "`components/ui/` tidak boleh mengimpor `app/` (frontend-architecture.md §1).",
        },
        {
          group: ["@/stores/*", "@/stores/**"],
          message:
            "`components/ui/` tidak boleh mengimpor `stores/` (frontend-architecture.md §1).",
        },
        // `components/ui/` hanya boleh memakai util murni `lib/format` (§1), bukan
        // `lib/near`/`lib/errors`/`lib/constants`.
        {
          group: ["@/lib/*", "@/lib/**", "!@/lib/format", "!@/lib/format/**"],
          message:
            "`components/ui/` hanya boleh mengimpor `lib/format` dari `lib/` (frontend-architecture.md §1).",
        },
      ]),
    },
  },
  globalIgnores([".next/**", "out/**", "build/**", "coverage/**", "next-env.d.ts"]),
]);

export default eslintConfig;
