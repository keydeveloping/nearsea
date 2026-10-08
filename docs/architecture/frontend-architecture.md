# Frontend Architecture

> Struktur internal aplikasi frontend.

## Framework & struktur folder (ronde 2+12 final)

Next.js App Router + TypeScript strict + Tailwind. Struktur feature-based:

```text
frontend/
├── app/            # routing (app router)
├── features/       # marketplace/ collections/ profile/ launchpad/ admin/
│   └── <fitur>/    # components/ hooks/ api/ types/
├── lib/            # near client, rpc, nearblocks, format, constants
├── stores/         # zustand (wallet-ui, modal)
├── components/ui/  # design system primitives (setelah sesi branding TASK-008b di tasks/backlog.md)
└── i18n/           # copy terpusat EN (siap tambah locale)
```

## Data fetching & state (ronde 5 final)

- **Server state: TanStack Query** — view calls (listings, offers, koleksi) + REST API + NearBlocks.
- **Client state: Zustand** — wallet UI, modal, drawer.
- **URL state** — query params untuk search/filter/sort/pagination (shareable).
- Setelah tx sukses → invalidate query terkait (listing, saldo, profil).
- **Re-verification wajib (SEC-ORDER-003)**: sebelum menampilkan modal sign, FE re-verify harga + ownership + status via view call on-chain — data cache TIDAK boleh jadi dasar transaksi.
- Profil custom: write via REST + signature **NEP-413 utama / custom fallback** (api/authentication.md).

## Wallet integration

- `near-connect` + hooks resminya — semua wallet yang didukung near-connect aktif sejak v1 (daftar resmi: docs.near.org/tools/near-connect; HOT, Meteor, Nightly, dsb.).
- Jaringan: ikut `NEAR_NETWORK` env (testnet MVP).

**Implementasi (TASK-007)** — detail di `features/auth/`:

- Paket: `@hot-labs/near-connect` + `near-connect-hooks` (`NearProvider`, `useNearWallet`).
  `lib/near/wallet-connector.ts` membangun konfigurasi connector dari env; **tidak ada daftar
  wallet hardcoded** — daftar berasal dari manifest near-connect.
- `features/auth/components/WalletProvider.tsx` menyediakan state wallet app-wide lewat
  `WalletContext` + hook `useWallet()`. Lapisan near-connect **hanya di-mount di browser**
  (constructor-nya memakai `window`/IndexedDB + fetch manifest), sehingga halaman tetap
  ter-render statis saat SSR dan app jalan tanpa wallet.
- **Bentuk state**: satu tipe `WalletApi` (`status`/`accountId`/`network`/`balanceYocto`/
  `accountStatus`/`transactionsDisabled`/`errorCode` + aksi `connect`/`disconnect`/`retry`),
  dibangun satu tempat di `features/auth/wallet-api.ts`. Status: `loading` / `disconnected` /
  `connecting` / `connected` / `disconnecting` / `error`.
- Pemilih wallet = popup bawaan near-connect (bukan modal buatan sendiri); header app hanya
  menampilkan keadaan di atas.
- **Network mismatch → `transactionsDisabled`**: near-connect tidak mengekspos network wallet, jadi
  sinyalnya adalah **akun tidak ada di jaringan terkonfigurasi** (probe RPC `view_account` gagal
  "account does not exist"). Banner peringatan + flag gerbang `transactionsDisabled` disediakan di
  sini; **tombol aksi yang men-disable dirinya lewat flag itu milik TASK-008**. Batasan & positif/
  negatif palsu didokumentasikan di [features/auth.md](../features/auth.md) §Status implementasi.
- Nilai Ⓝ ditampilkan lewat `lib/format/money.ts` (`formatNear`, tipe `YoctoNear`) — aritmetika
  BigInt, tanpa float.

## Konversi & format

- Nilai Ⓝ selalu `NEAR.fromDecimal()` / `formatUnits` — dilarang aritmetika float pada yoctoNEAR.

## Metadata untrusted (keputusan keamanan — ADR-014)

- Render metadata hanya via `<img>`; tanpa `innerHTML`; gateway IPFS allowlist; CSP ketat — detail [../security/metadata-security.md](../security/metadata-security.md) & [../security/frontend-security.md](../security/frontend-security.md).

## Error & loading (default — bisa diubah saat build UI)

- Global: toast untuk hasil tx (success/fail), inline error + retry untuk data fetch.
- Pending tx: tombol disabled + spinner sampai receipt final; lalu invalidate query.
- Polling notifikasi: TanStack Query refetchInterval 30–60 dtk saat tab aktif.
- Urutan saat koneksi putus di tengah signing: **cek status tx via hash dulu**; reset state hanya bila tx tidak ditemukan/gagal.

## Routing (default — bisa diubah saat build UI)

```text
/                      explore (search + grid)
/collection/[id]       halaman koleksi (+ tab items/activity/mint)
/token/[contract]/[tokenId]   detail NFT
/profile/[account]     profil
/create                create collection (launchpad)
/settings              settings
/onboarding            guide newcomer
/leaderboard           top koleksi
/admin                 panel admin (guard allowlist)
```

---

## 1. Hierarki komponen & dependensi modul

> Aturan dependensi satu arah: `app/ → features/ → lib/ → components/ui/`; fitur **tidak** saling impor langsung (lewat `lib/` atau shared feature).

```text
app/                         (routing, layout, providers)
 │  ├── layout.tsx           Providers: QueryClient, WalletProvider, i18n, ErrorBoundary
 │  └── <route>/page.tsx ───► features/<fitur>/<Page>.tsx
 │
features/<fitur>/
 │  ├── components/          (presentational + container fitur)
 │  ├── hooks/               (useListing, useBuy, useProfile …)
 │  ├── api/                 (view-call & REST call — tipis)
 │  └── types/
 │
lib/
 │  ├── near/                (client, RPC failover, contract bindings)
 │  ├── nearblocks/          (riwayat tx)
 │  ├── format/              (yoctoNEAR ↔ display, alamat)
 │  ├── errors/              (modul error terpusat — error-handling.md §8)
 │  └── constants/
 │
stores/                     (zustand: wallet-ui, modal, notifikasi read-state)
components/ui/              (design-system primitives — tanpa dependensi fitur)
i18n/                       (kunci copy EN)
```

| Modul | Boleh impor | Dilarang impor |
|---|---|---|
| `app/` | `features/`, `lib/`, `components/ui/`, `stores/` | — |
| `features/<A>/` | `lib/`, `components/ui/`, `stores/` | `features/<B>/` (kecuali via `lib/`) |
| `lib/` | `lib/` internal | `features/`, `app/`, `components/ui/` |
| `components/ui/` | `lib/format` (util murni) | `features/`, `app/`, `stores/` |

- **Server-only** vs **client**: panggilan RPC write & wallet hanya di client (`"use client"`); data publik boleh SSR (lihat §3).
- **Nilai Ⓝ** selalu lewat `lib/format` (`NEAR.fromDecimal`/`formatUnits`) — dilarang aritmetika float pada yoctoNEAR.

## 2. Tabel route

> Guard: `publik` = tanpa wallet; `wallet` = butuh koneksi wallet; `admin` = allowlist + scope `admin`. SSR/CSR: `SSR` = render server untuk data publik awal, `CSR` = client (butuh wallet/cache hidup).

| Route | Guard | Params / query | Dependensi data | Loading state | Error state | Render |
|---|---|---|---|---|---|---|
| `/` | publik | `q`, `sort`, `cursor` | listing/koleksi (RPC view) | skeleton grid | inline retry | SSR shell + CSR data |
| `/collection/[id]` | publik | `id`, tab, `cursor` | `nft_metadata`/enumeration, stats (fase 2) | skeleton tab | inline retry | SSR shell + CSR |
| `/token/[contract]/[tokenId]` | publik | `contract`, `tokenId` | `nft_token`, sale, offers (view) | skeleton detail | "no longer available" bila stale | SSR shell + CSR |
| `/profile/[account]` | publik | `account`, tab, `cursor` | `nft_tokens_for_owner`, NearBlocks, profil (API) | skeleton per tab | empty state | CSR (data per akun) |
| `/create` | wallet | — | factory/launchpad config (view) | form skeleton | inline field error | CSR |
| `/settings` | wallet | — | profil (API), session | form skeleton | toast + field error | CSR |
| `/onboarding` | publik | — | statis | — | — | SSR statis |
| `/leaderboard` | publik | `cursor` | agregat (★ fase 2) | skeleton tabel | inline retry | CSR |
| `/admin` | admin | tab | report/blocklist (API) | tabel loading | 403 halaman | CSR |
| `/notifications` (opsional) | wallet | — | polling NearBlocks + offers (view) | list skeleton | badge berhenti update | CSR |

- **Redirect guard**: route `wallet` tanpa koneksi → modal connect (bukan redirect keras); route `admin` tanpa scope → halaman 403.
- **URL state**: search/filter/sort/pagination di query params (shareable) — [04-ux-ui-spec.md](../04-ux-ui-spec.md).

## 3. Keputusan SSR/CSR

| Kriteria | Pilihan | Alasan |
|---|---|---|
| Halaman publik yang butuh SEO/share | SSR shell + CSR data | Metadata OG dapat dirender; data tetap live dari RPC |
| Halaman butuh wallet/sign | CSR | Wallet hanya di client |
| Halaman admin | CSR + guard | Butuh session; tidak untuk SEO |
| Halaman statis (onboarding) | SSR/static | Tidak ada data dinamis |

> MVP self-hosted (ADR-009): SSR di server Next.js yang sama; tidak ada CDN pada MVP (scaling.md §6).

## 4. Konvensi query key & matriks invalidasi

> TanStack Query = server state. Key deterministik agar invalidasi presisi (bukan refetch global).

**Konvensi key** (array, dari umum → spesifik):

```text
['listings', { status, sort, cursor }]
['listing', contractId, tokenId]
['token', contractId, tokenId]           // nft_token + metadata
['offers', contractId, tokenId]
['offersByAccount', accountId]
['bundle', bundleId]
['collection', collectionId]
['tokensByOwner', accountId, cursor]
['profile', accountId]
['activity', accountId, cursor]
['nearBalance', accountId]
['launchpad', collectionId]
```

**Matriks invalidasi** (setelah tx final / write sukses):

| Aksi | Invalidate |
|---|---|
| List / update price / remove sale | `['listing',*]`, `['listings',*]`, `['tokensByOwner', account]` |
| Buy / accept offer | `['listing',*]`, `['listings',*]`, `['token',*]`, `['offers',*]`, `['nearBalance', buyer]`, `['tokensByOwner', buyer/seller]` |
| Make offer / cancel offer | `['offers',*]`, `['offersByAccount', account]`, `['nearBalance', buyer]` |
| Create / cancel bundle | `['bundle',*]`, `['listings',*]` (bundle memblokir list terpisah — INV-028) |
| Mint (launchpad) | `['launchpad',*]`, `['tokensByOwner', minter]`, `['nearBalance', minter]` |
| PATCH profil | `['profile', account]` |
| Admin hide/verified/blocklist | `['collection',*]`, `['listings',*]`, `['admin',*]` |

- **Aturan**: invalidasi **setelah receipt final**, bukan saat submit (error-handling.md §5).
- **Polling**: notifikasi `refetchInterval` 30–60 dtk saat tab aktif (features/notifications.md).
- **Re-verify** (SEC-ORDER-003) selalu fetch fresh, bukan dari cache (§11).

## 5. Strategi error boundary

> Taksonomi & kode: [development/error-handling.md](../development/error-handling.md). FE **wajib** memetakan panic kontrak → kode user (bukan teks mentah).

| Level | Komponen | Menangkap | Tampilan |
|---|---|---|---|
| Root | `app/error.tsx` (App Router) | error render tak terduga | halaman error + tombol retry |
| Route | `app/<route>/error.tsx` | error per halaman | inline error + retry |
| Fitur | `ErrorBoundary` di sekitar modul | error komponen fitur | fallback section (sisa halaman tetap hidup) |
| Query | TanStack Query `error` state | error fetch | inline error + retry |
| Tx | toast + state `pending-tx` | revert/timeout/drop | kode user (error-handling.md §4) |

- **Aturan**: error boundary **tidak** menampilkan stack trace ke user; detail ke log (error-handling.md §1/§7).
- **Koneksi putus saat signing**: cek status tx via hash dulu; reset state hanya bila tx tidak ditemukan/gagal (features/auth.md, error-handling.md §5).

## 6. Performance budget & code-splitting

> Angka di bawah **PROPOSED** (target engineering, bukan fakta) — diukur di CI saat scaffold (TASK-001) dan disesuaikan.

| Metrik | Budget | Alat |
|---|---|---|
| JS initial bundle (route masuk) | ≤ 200 KB gzip (PROPOSED) | `next build` + bundle analyzer |
| JS per-route chunk | ≤ 100 KB gzip (PROPOSED) | bundle analyzer |
| LCP (4G mid-tier) | ≤ 2.5 dtk (PROPOSED) | Lighthouse / Web Vitals |
| CLS | ≤ 0.1 (PROPOSED) | Lighthouse |
| INP | ≤ 200 ms (PROPOSED) | Web Vitals |
| Gambar | lazy + ukuran responsif; `next/image` | Next.js |

- **Code-splitting**: route-based otomatis (App Router); komponen berat (modal tx, chart fase 2) via `dynamic(() => …, { ssr: false })`; design-system primitives tree-shakeable.
- **Aturan**: tidak ada third-party script di MVP (frontend-security.md §2) — menjaga budget & CSP.

## 7. Env var frontend

> Nilai PUBLIC saja (prefix `NEXT_PUBLIC_`); daftar kanonik [deployment/environments.md](../deployment/environments.md).

| Var | Prefix | Isi | Catatan |
|---|---|---|---|
| `NEAR_NETWORK` | `NEXT_PUBLIC_` | testnet/mainnet | Banner network permanen |
| `NEAR_RPC_URL` + fallbacks | `NEXT_PUBLIC_` | daftar provider | Urutan failover |
| `MARKET_CONTRACT_ID`, `FACTORY_CONTRACT_ID` | `NEXT_PUBLIC_` | alamat kontrak | — |
| base API report | — | same-origin `/api/*` (default) | `API_BASE_URL` **PROPOSED** hanya bila API dipisah origin |
| `IPFS_GATEWAY` | `NEXT_PUBLIC_` | gateway allowlist | fase lanjut |
| `JWT_SECRET`, `DATABASE_URL` | **tanpa** prefix | — | **DILARANG** di bundle FE |

- **Nama literal yang dibaca kode** (TASK-007): `NEXT_PUBLIC_NEAR_NETWORK`,
  `NEXT_PUBLIC_NEAR_RPC_URL`, `NEXT_PUBLIC_NEAR_RPC_FALLBACKS` — modul `lib/near/network.ts`.
  `.env.example` memuat keduanya (tanpa & dengan prefix) agar tidak ada nilai yang diam-diam
  berbeda antara proses server dan bundle browser.
- Nilai `NEAR_NETWORK` di luar `testnet`/`mainnet` = **gagal saat start**, bukan fallback
  diam-diam ke testnet: salah jaringan = transaksi di jaringan yang salah.

## 8. Pendekatan testing frontend

> Strategi: [testing-strategy.md](../testing/testing-strategy.md).

| Lapis | Alat | Cakupan |
|---|---|---|
| Unit (util/hook) | Vitest + React Testing Library | format yoctoNEAR, hook query, guard |
| Komponen | RTL | state loading/error/empty, tombol pending |
| E2E jalur emas | Playwright | connect → list (2 langkah) → buy; offer → accept |
| E2E keamanan | Playwright | re-verify sebelum sign (SEC-ORDER-003), pemetaan panic → kode user |
| A11y | axe (opsional) | komponen design-system |

- **Mock**: RPC & API di-mock di unit; E2E memakai testnet (bukan mock chain).

## 9. Struktur i18n

> Copy UI = **English**, terpusat; tidak ada string hardcode di komponen (AGENTS.md).

```text
i18n/
├── en/
│   ├── common.json      // tombol, label umum, error generik
│   ├── marketplace.json // listing/buy/offer/bundle
│   ├── auth.json        // connect/login
│   ├── profile.json
│   ├── launchpad.json
│   ├── admin.json
│   └── errors.json      // pemetaan kode error → pesan user (error-handling.md)
└── index.ts             // loader + tipe kunci (strict)
```

- **Kunci error** mengikuti registry kode ([error-handling.md](../development/error-handling.md) §3/§4): mis. `errors.CONFLICT_SOLD`.
- **Struktur siap multi-locale**: menambah locale = menambah folder `i18n/<locale>/`; tidak ada perubahan komponen.
- **Aturan**: kunci bertipe (TypeScript) agar kunci salah = error build.

## 10. Inventaris primitif design system

> Menunggu sesi branding (TASK-008b); daftar di bawah **PROPOSED** (kerangka sebelum sesi desain).

| Primitif | Varian | Dipakai |
|---|---|---|
| `Button` | primary/secondary/ghost/danger; size sm/md/lg | semua aksi |
| `Modal` | dialog/confirm; fokus trap | modal tx (2 langkah), konfirmasi |
| `Input` / `Textarea` | default/error/disabled | form profil, harga |
| `Select` | single | filter/sort |
| `Card` | token/collection/stat | grid discovery |
| `Badge` | verified/status/stale | badge verified, status listing |
| `Tabs` | underline | halaman koleksi/profil |
| `Toast` | info/warning/error | hasil tx (error-handling.md §6) |
| `Skeleton` | text/card/grid | loading state |
| `EmptyState` | — | tab kosong, akun tak ada |
| `Avatar` | identicon fallback | profil/wallet |
| `Tooltip` | — | penjelasan fee/royalti |
| `Spinner` | — | pending tx |

- **Aturan**: primitives tidak bergantung fitur (§1); semua teks lewat i18n (§9); warna/font dari Tailwind theme hasil branding.

## 11. Sesi & penyimpanan token

> Kebijakan keamanan: [frontend-security.md](../security/frontend-security.md) §6.

| Data | Tempat | TTL | Catatan |
|---|---|---|---|
| Wallet connection state | localStorage | — | Alamat publik saja |
| Session JWT (API) | **in-memory** | 15 menit | **Bukan** localStorage (SEC-AUTH-003) |
| Refresh token | in-memory (rotasi) | ≤ 12 jam | Rotasi tiap pakai (SEC-AUTH-006) |
| Notification read-state | localStorage per akun | — | features/notifications.md |
| Admin session | in-memory | 15 menit | **Tanpa** silent refresh (admin-security.md §2) |

- **Dilarang**: menyimpan private key/seed phrase (FACT: tidak perlu) — features/auth.md.
- **Logout**: hapus session server-side + hapus in-memory client.
- **CSRF**: token di header `Authorization` (bukan cookie) → CSRF tidak relevan; jika pindah cookie: `SameSite=Strict` + CSRF token (wallet-authentication.md §4).

## 12. Implementasi re-verification (SEC-ORDER-003)

> **Wajib**: sebelum menampilkan modal sign, FE re-verify state kritis via **view call on-chain** — data cache TIDAK boleh jadi dasar transaksi. Traceability: ADR-010, SEC-ORDER-003, SEC-FE-002.

| Aksi | View call yang dipanggil sebelum sign | Yang divalidasi |
|---|---|---|
| **Buy** (listing) | `get_sale(contract, token)` + `nft_token(contract, token)` | harga terkini, seller masih owner, status aktif, bukan stale |
| **Buy bundle** | `get_bundle(bundleId)` + `nft_token` per item | semua item masih valid (ownership + approval) — pre-validasi FE (INV-025/028) |
| **Accept offer** | `get_offer(contract, token, buyer)` + `nft_token` | offer masih aktif & belum expire, pemanggil masih owner |
| **Make offer** | `nft_token` + `get_offers(contract, token)` | bukan owner token (INV-023), belum ada offer aktif oleh buyer sama (INV-024) |
| **List (2 langkah)** | `nft_is_approved` + `nft_token` sebelum Tx-2 | approval benar-benar tercatat, owner benar |
| **Cancel/update** | `get_sale` | pemanggil = seller, listing masih ada |

**Alur implementasi** (contoh `useBuy`):

```text
1. User klik Buy → hook memanggil view call fresh (bukan cache) — bypass TanStack cache
2. Bandingkan hasil vs yang ditampilkan di modal (harga, seller, status)
3. Cocok  → tampilkan modal sign dengan data TERKINI
   Berubah → tampilkan CONFLICT_PRICE_CHANGED / CONFLICT_STALE; refresh UI, jangan sign
4. Sign → submit → tunggu receipt final → invalidate query (§4)
5. Revert → petakan panic ke kode user (error-handling.md §4), tampilkan toast
```

- **Aturan**: modal menampilkan argumen tx yang **sama** dengan yang akan di-sign (preview = reality); fee 2% & royalti di breakdown dihitung dari data view, bukan cache.
- **Rujukan implementasi**: hook per fitur di `features/marketplace/hooks/`; modul view-call di `features/marketplace/api/`.
