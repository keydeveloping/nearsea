# 09: FE — connect wallet

**What to build:** Pengunjung bisa menyambungkan wallet NEAR-nya dan melihat dirinya terhubung, dengan jaringan yang benar ditampilkan. Ini pintu masuk untuk semua aksi berikutnya — tanpa ini, tidak ada yang bisa menandatangani transaksi.

**Blocked by:** 01

**Status:** done

- [x] Tombol connect membuka pemilih wallet dan berhasil menyambung di testnet.
      → Tombol memanggil `signIn()` dari `near-connect-hooks`, yang membuka **popup pemilih wallet
      bawaan near-connect**. **Belum diklaim**: connect dengan wallet testnet nyata — butuh akun
      testnet + browser; itu jalur emas Playwright tiket `10`.
- [x] Wallet yang didukung near-connect tersedia sejak awal (daftar resmi), bukan satu wallet hardcoded.
      → Tidak ada daftar wallet di kode: `lib/near/wallet-connector.ts` hanya meneruskan `network` +
      provider RPC; daftar berasal dari manifest resmi near-connect (termasuk wallet baru).
- [x] Alamat akun yang terhubung tampil di UI setelah connect.
      → Header menampilkan `accountId` (mono) + saldo Ⓝ hasil probe RPC.
- [x] Banner/indikator jaringan tampil dan akurat: peringatan jelas bila wallet berada di jaringan yang salah.
      → `NetworkBanner`: indikator permanen (`Network: testnet`); saat akun ter-connect **tidak ada** di
      jaringan terkonfigurasi → banner amber + flag gerbang `transactionsDisabled`.
      **Catatan jujur**: near-connect tidak mengekspos network wallet, jadi yang dibuktikan adalah
      "akun tidak ada di jaringan ini" — sebabnya bisa wallet di jaringan lain **atau** akun belum
      dibuat/di-fund. Copy menyebut keduanya, bukan mengklaim jaringan salah. Detail batasan:
      [docs/features/auth.md](../../../docs/features/auth.md) §Status implementasi.
- [x] Disconnect berfungsi dan mengembalikan UI ke keadaan belum-connect.
      → `disconnect()` → `signOut()`; state kembali `disconnected` (diuji). Selama proses, tombol
      menampilkan "Disconnecting…" (bukan "Connecting…").
- [x] Aplikasi tetap berfungsi tanpa wallet (mode baca), bukan halaman kosong.
      → `WalletProvider` tidak me-render lapisan wallet saat SSR; halaman tetap ter-prerender statis
      (`build` → 4/4 halaman ○ Static) dan menampilkan ajakan connect (diuji).
- [x] Konfigurasi jaringan mengikuti env (`NEAR_NETWORK`) — bukan hardcoded di komponen.
      → `lib/near/network.ts` membaca `NEXT_PUBLIC_NEAR_NETWORK` (+ `_RPC_URL`/`_RPC_FALLBACKS`);
      nilai tak dikenal = gagal saat start, bukan fallback diam-diam.
- [x] Tidak ada private key / seed / kredensial apa pun di kode FE atau bundle.
      → Tidak ada secret di source; `grep` bundle `.next/static` + `.next/server` bersih (tidak ada
      `ed25519:`/`privateKey`/seed/`JWT_SECRET`/`DATABASE_URL`, dan tidak ada kode penandatanganan
      `access_key::plugin`/`createLocalKeyFor`). Lihat catatan TASK-037.
- [x] Sandbox hijau: connect → disconnect, plus render tanpa wallet.
      → `pnpm test` = **52 test** (7 file). near-connect di-mock lewat `features/auth/near-connect.double.tsx`
      yang meniru **kontrak** `useNearWallet()` dan memancarkan perubahan akun seperti
      `wallet:signIn`/`wallet:signOut`, sehingga logika turunan (`status`, probe saldo,
      `transactionsDisabled`, pemetaan error) benar-benar dieksekusi — bukan sekadar mengulang nilai
      yang disuntik. Signature nyata diuji di sandbox/API, bukan unit FE (`testing-strategy.md`).

**Done-when (TASK-007):** Connect/disconnect + banner network jalan di testnet.
→ Connect/disconnect + banner network **terimplementasi dan teruji di unit/komponen**; bukti
"jalan di testnet" (wallet nyata + dua akun) menyusul di jalur emas Playwright tiket `10`.

**Bukti & catatan implementasi (ronde 25):**
- **Lokasi**: `frontend/features/auth/` (`components/WalletProvider.tsx`, `components/AppHeader.tsx`,
  `components/NetworkBanner.tsx`, `components/WalletButton.tsx`,
  `components/WalletReadOnlyNotice.tsx`, `hooks/useWallet.ts`, `hooks/useAccountProbe.ts`,
  `hooks/useWalletActions.ts`, `types/wallet.types.ts`, `wallet-api.ts`, `test-utils.tsx`,
  `near-connect.double.tsx`) + `frontend/lib/near/` (`network.ts`, `wallet-connector.ts`,
  `wallet-errors.ts`) + `frontend/lib/format/money.ts` + `i18n/en/auth.json`.
- **Lapisan near-connect hanya di-mount di browser** (`useSyncExternalStore` sebagai pembeda
  server/client): constructor `NearConnector` memakai `window`/IndexedDB dan mengambil manifest
  lewat fetch, jadi merendernya saat SSR salah **dan** membuat build butuh jaringan. `children`
  tetap di posisi pohon yang sama sebelum/sesudah mount → konten halaman tidak ter-remount.
- **Satu tipe `WalletApi`, bukan sekumpulan boolean lepas**; dibangun di satu tempat
  (`wallet-api.ts`) sehingga konteks default dan fixture test tidak bisa menyimpang. Saldo selalu
  **string yoctoNEAR** bertipe `YoctoNear`; aritmetika BigInt di `formatNear` (tanpa float),
  pecahan **dipotong** agar tidak melebihkan saldo.
- **Temuan saat review kode (semuanya diperbaiki, dengan test regresi):**
  1. **`pending` tidak membedakan aksi** → disconnect tampil sebagai "Connecting…" + hint approve.
     Diperbaiki: `useWalletActions` menyimpan aksi yang berjalan, status `disconnecting` ditambahkan.
  2. **Retry selalu connect** meski yang gagal disconnect. Diperbaiki: `retry` mengulang aksi yang gagal.
  3. **Error disconnect hilang** karena hanya dirender di keadaan belum-connect. Diperbaiki:
     `ErrorLine` dirender di kedua keadaan.
  4. **Placeholder `{network}` tidak disubstitusi** di judul banner (bug copy nyata). Diperbaiki.
  5. **Efek probe berjalan tanpa henti**: `getBalance` dari `near-connect-hooks` dibuat ulang tiap
     render dan sempat dipakai sebagai dependensi `useEffect`. Diperbaiki dengan ref.
  6. **Dua komponen publik dalam satu file** (`WalletButton` + `WalletReadOnlyNotice`) melanggar
     `code-standards.md` §6. Diperbaiki: `WalletReadOnlyNotice.tsx` dipisah.
  7. **Dependency kritis tidak di-pin exact** (`frontend-security.md` §9) dan `near-api-js` dideklarasikan
     langsung padahal tidak pernah diimpor. Diperbaiki: pin exact untuk `near-connect`/`hooks`; `near-api-js`
     dihapus dari `dependencies` (tetap ada sebagai dependency transitif).
- **Koreksi/penyesuaian lain** (prasyarat agar AC tiket ini lolos):
  1. `tsconfig.target` **ES2017 → ES2020** — literal `BigInt` tidak tersedia di bawah ES2020, dan
     aturan proyek mewajibkan aritmetika BigInt untuk nilai yoctoNEAR.
  2. `i18n` diperluas ke **kedalaman kunci bebas** (sebelumnya dua segmen). Konvensi
     `code-standards.md` §10 memakai tiga segmen (`marketplace.buyButton.label`).
  3. `vitest.setup.ts` ditambahkan (`cleanup()` RTL) — Vitest jalan **tanpa globals**, jadi
     auto-cleanup RTL tidak terdaftar dan DOM menumpuk antar test.
  4. `@testing-library/react` + `@testing-library/dom` ditambahkan sebagai devDependency —
     `testing-strategy.md` menyebut RTL untuk lapis komponen; sebelumnya belum ada.
- **Env**: kode membaca `NEXT_PUBLIC_NEAR_NETWORK`, `NEXT_PUBLIC_NEAR_RPC_URL`,
  `NEXT_PUBLIC_NEAR_RPC_FALLBACKS` (Next.js hanya mengekspos prefix `NEXT_PUBLIC_`). `.env.example`
  kini memuat keduanya — sebelumnya hanya versi tanpa prefix, yang akan membuat aplikasi diam-diam
  memakai default testnet meski env diisi. Didokumentasikan di `frontend-architecture.md` §7 dan
  `environments.md`.
- **Temuan keamanan (dicatat, tidak diperbaiki di tiket ini)**: `near-connect-hooks` menarik
  `function-call-key-plugin`, yang menyimpan **private key** function-call di `localStorage` dan
  menandatangani transaksi lokal. Proyek ini tidak pernah memanggil `signIn({ addFunctionCallKey })`,
  jadi jalurnya tidak aktif; plugin hanya di-`use()` untuk `signOut`, dan `createLocalKeyFor` tidak
  pernah dipanggil → kode penandatanganan ter-tree-shake dari bundle (diverifikasi: string
  `access_key::plugin`/`createLocalKeyFor` tidak ada di `.next/static` maupun `.next/server`).
  Ketergantungan transitifnya tetap perlu keputusan tertulis → **TASK-037** (`tasks/backlog.md`).
- **Belum diklaim**: connect/sign dengan wallet testnet nyata, persistensi setelah reload, deep-link
  mobile, modal `wallet-picker`/`approving` versi kustom, dan **penonaktifan tombol aksi** berbasis
  `transactionsDisabled` (baru ada tombol aksi di tiket `10`).

**Spec:** [docs/architecture/frontend-architecture.md](../../../docs/architecture/frontend-architecture.md) §Wallet integration, §7 · [docs/features/auth.md](../../../docs/features/auth.md) · SEC-AUTH-001
