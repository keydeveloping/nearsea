# 04 — UX/UI Spec

> Apa yang user lihat dan lakukan. Halaman → komponen → state.
>
> **Ronde 2**: bahasa UI **English** (copy terpusat, siap i18n). Tema visual **custom branding** — butuh sesi desain terpisah; spec teknis (Tailwind theme tokens) menyusul setelah sesi itu.

## Halaman (final — ronde 8, diperluas ronde 13)

- Home / Explore (search + grid listing)
- Detail NFT
- Halaman Koleksi (+ badge verified, tab items/activity/mint, tombol Report)
- Profil user (owned / created / offers / activity, profil custom — ronde 13)
- Create Collection & Mint (launchpad wizard)
- Activity / Leaderboard
- Settings
- Onboarding guide (newcomer)
- Admin: antrean report & verifikasi koleksi

## Layout Global

### Header (ronde 8: top-nav ala OpenSea)
- Logo kiri ("NearSea")
- Search bar tengah (koleksi/token)
- Kanan: bell notifikasi (unread badge) · dropdown Create (Create Collection; aksi Jual/Bundle tersedia dari Profil) · WalletButton
- Mobile: logo + hamburger → drawer (search, menu, wallet)

### Navigasi menu
- Explore · Leaderboard · Create · Profile · Settings · Onboarding (guide)

## Komponen inti (draft — final setelah sesi branding TASK-008b)

| Komponen | State |
|---|---|
| NFT Card (media, nama, koleksi, harga, badge verified) | default/hover/skeleton/stale |
| WalletButton | disconnected/connecting/connected(alamat+saldo)/error |
| Buy Button | enabled/pending-tx/disabled(self-buy, harga berubah, stale, bukan allowed_buyer)/sold |
| Make Offer Modal | form harga+durasi → confirm → pending → success/error |
| List Modal | step 1/2 (approve → list) dengan progress indicator |
| Private Listing Modal | list + input alamat allowed_buyer + preview keterbatasan |
| Bundle Builder | pilih multi-NFT → set satu harga → approve batch → pending per token |
| Notification Panel | bell dropdown: unread badge, daftar item, klik→navigasi, empty state |
| Report Modal | pilih alasan (enum) → kirim → success/error |
| Trait Badge · Price Chip · Activity Table · Toast | — |

## State per Halaman (wajib semua halaman)

```text
Loading (skeleton) · Empty · Error (retry) · Success · Disabled (tx pending)
```

## Responsive (ronde 8: setara penuh)

- Breakpoint: mobile (<640) / tablet (640–1024) / desktop (>1024) — didesain setara.
- Grid NFT: 2 kolom mobile · 3–4 tablet · 5–6 desktop.
- Semua modal full-screen di mobile.

---

## Design Tokens (placeholder — menunggu sesi branding TASK-008b)

> **Semua nama token di bawah = placeholder** yang dikunci saat sesi desain branding. Nilai final mengisi Tailwind theme; komponen **hanya** memakai nama token (bukan nilai literal). Dilarang menaruh warna/ukuran hardcode di komponen.

| Kategori | Nama token (placeholder) | Contoh nilai | Catatan |
|---|---|---|---|
| **Color** | `color-bg`, `color-surface`, `color-surface-raised` | — | Latar & kartu |
| Color | `color-text`, `color-text-muted`, `color-text-inverse` | — | Hierarki teks |
| Color | `color-primary`, `color-primary-hover`, `color-primary-active` | — | Aksi utama |
| Color | `color-danger`, `color-warning`, `color-success`, `color-info` | — | Semantik status/toast |
| Color | `color-border`, `color-divider` | — | Garis |
| Color | `color-verified` | — | Badge verified |
| **Spacing** | `space-0`…`space-12` (skala 4px base) | 0/4/8/12/16/24/32/48 | Jarak konsisten |
| **Type** | `font-sans`, `font-mono` | — | Body & angka/alamat |
| Type | `text-xs`…`text-3xl` | 12/14/16/18/20/24/30 | Skala ukuran |
| Type | `font-regular`, `font-medium`, `font-semibold` | 400/500/600 | Bobot |
| **Radius** | `radius-sm`, `radius-md`, `radius-lg`, `radius-full` | 4/8/12/9999 | Sudut |
| **Shadow** | `shadow-sm`, `shadow-md`, `shadow-lg` | — | Elevasi |
| **Z-index** | `z-header`, `z-dropdown`, `z-modal`, `z-toast` | 100/200/300/400 | Lapisan |
| **Motion** | `duration-fast`, `duration-base`, `ease-standard` | 120ms/200ms | Hormati `prefers-reduced-motion` |
| **Breakpoint** | `bp-mobile`, `bp-tablet`, `bp-desktop` | 640/1024 | Responsive |
| **Icon** | `icon-sm`, `icon-md`, `icon-lg` | 16/20/24 | Ukuran ikon |

- **Aturan**: token hanya didefinisikan sekali (Tailwind config); komponen memakai nama token; kontras warna memenuhi WCAG AA (§ Accessibility).

---

## Wireframe per Halaman (9 halaman MVP)

> Wireframe ASCII = kerangka layout (bukan visual final). Semua halaman punya state loading/empty/error (§ State per Halaman).

### 1. Home / Explore (`/`)

```text
┌──────────────────────────────────────────────────────────────┐
│ [Logo]   [ Search collections & items…      ]   [Bell][Create][Wallet] │
├──────────────────────────────────────────────────────────────┤
│ Explore · Leaderboard · Create · Profile · Settings · Guide   │
├──────────────────────────────────────────────────────────────┤
│ [Sort ▾]  [Filter harga ▾]                    [ tampil: 24 ]  │
├──────────────────────────────────────────────────────────────┤
│ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐                  │
│ │ media  │ │ media  │ │ media  │ │ media  │  ← NFT Card      │
│ │ Nama   │ │ Nama   │ │ Nama   │ │ Nama   │                  │
│ │ Koleksi│ │ Koleksi│ │ Koleksi│ │ Koleksi│  ✓ = verified    │
│ │ 5 Ⓝ    │ │ 2 Ⓝ    │ │ 1.2 Ⓝ  │ │ 8 Ⓝ    │                  │
│ └────────┘ └────────┘ └────────┘ └────────┘                  │
│                    [ Load more ]                              │
└──────────────────────────────────────────────────────────────┘
```

### 2. Detail NFT (`/token/:contract/:tokenId`)

```text
┌──────────────────────────────────────────────────────────────┐
│ [Logo]  [ Search ]                            [Bell][Wallet] │
├──────────────────────────────────────────────────────────────┤
│ ┌──────────────────────┐   Nama Token           ✓ Verified   │
│ │                      │   Koleksi › (link)                   │
│ │      media besar     │   Owner: alice.testnet               │
│ │                      │   ──────────────────────────────     │
│ │                      │   Harga: 10 Ⓝ                        │
│ │                      │   [ Buy now ] [ Make offer ]         │
│ └──────────────────────┘   (jika owner: [Sell][Cancel][Edit]) │
├──────────────────────────────────────────────────────────────┤
│ [ Details ] [ Offers ] [ Activity ]                          │
│ ─────────────────────────────────────────────────────────   │
│ (isi tab)                                                    │
├──────────────────────────────────────────────────────────────┤
│ [ Report ]                                                   │
└──────────────────────────────────────────────────────────────┘
```

### 3. Halaman Koleksi (`/collection/:id`)

```text
┌──────────────────────────────────────────────────────────────┐
│ [Logo]  [ Search ]                            [Bell][Wallet] │
├──────────────────────────────────────────────────────────────┤
│ ┌──────┐  Nama Koleksi  ✓ Verified        [ Report ]         │
│ │banner│  by creator.testnet · supply 1000 · royalti 5%      │
│ └──────┘  [ Items ] [ Activity ] [ Mint ]                    │
├──────────────────────────────────────────────────────────────┤
│ [Sort ▾] [Filter ▾]                                          │
│ ┌────────┐ ┌────────┐ ┌────────┐                             │
│ │ item   │ │ item   │ │ item   │   ← grid items             │
│ └────────┘ └────────┘ └────────┘                             │
│                    [ Load more ]                              │
└──────────────────────────────────────────────────────────────┘
```

### 4. Profil User (`/profile/:account`)

```text
┌──────────────────────────────────────────────────────────────┐
│ [Logo]  [ Search ]                            [Bell][Wallet] │
├──────────────────────────────────────────────────────────────┤
│ ┌────┐  Alice   (alias)                       [ Share ]       │
│ │avtr│  alice.testnet · bio…                                  │
│ └────┘                                                        │
│ [ Owned ] [ Created ] [ Offers ] [ Activity ]                │
├──────────────────────────────────────────────────────────────┤
│ ┌────────┐ ┌────────┐ ┌────────┐                             │
│ │ item   │ │ item   │ │ item   │   ← grid per tab            │
│ └────────┘ └────────┘ └────────┘                             │
└──────────────────────────────────────────────────────────────┘
```

### 5. Create Collection & Mint (`/create`)

```text
┌──────────────────────────────────────────────────────────────┐
│ [Logo]                                        [Bell][Wallet] │
├──────────────────────────────────────────────────────────────┤
│ Create a collection                                          │
│ ── Step 1: Metadata ──────────────────────────────────────   │
│  Name [________]  Symbol [____]  Media [upload/URL]          │
│ ── Step 2: Supply & Royalty ──────────────────────────────   │
│  ( ) Open  ( ) Fixed max [____]     Royalty [ 5 ]% (max 10)  │
│ ── Step 3: Mint phases ───────────────────────────────────   │
│  Phase 1: name [____] price [__]Ⓝ alloc [__] max/wallet [__] │
│           window [start–end]  allowlist [Upload CSV]         │
│  [+ Add phase]                                               │
│ ─────────────────────────────────────────────────────────   │
│                    [ Review & deploy ]                        │
└──────────────────────────────────────────────────────────────┘
```

### 6. Activity / Leaderboard (`/leaderboard`)

```text
┌──────────────────────────────────────────────────────────────┐
│ [Logo]  [ Search ]                            [Bell][Wallet] │
├──────────────────────────────────────────────────────────────┤
│ Leaderboard                        [ filter periode ▾ ]      │
├──────────────────────────────────────────────────────────────┤
│ # │ Koleksi            │ Volume (Ⓝ) │ Floor │ Items          │
│ 1 │ Genesis ✓          │ 120.5      │ 2.0   │ 340            │
│ 2 │ Pixel Punks        │  88.0      │ 1.5   │ 120            │
│ 3 │ …                  │   …        │  …    │  …             │
│                          [ Load more ]                        │
└──────────────────────────────────────────────────────────────┘
```

### 7. Settings (`/settings`)

```text
┌──────────────────────────────────────────────────────────────┐
│ [Logo]                                        [Bell][Wallet] │
├──────────────────────────────────────────────────────────────┤
│ Settings                                                     │
│ ── Profile ───────────────────────────────────────────────   │
│  Display name [____________]  (≤32)                          │
│  Bio [__________________________] (0/280)                    │
│  Avatar URL [__________________]                             │
│                                    [ Save ]                   │
│ ── Wallet ────────────────────────────────────────────────   │
│  alice.testnet [copy]   Network: testnet   [ Disconnect ]    │
│ ── Notifications ─────────────────────────────────────────   │
│  [ Mark all as read ]  [ Clear ]                             │
│ ── Danger / advanced ─────────────────────────────────────   │
│  ⏳ open-by-design                                            │
└──────────────────────────────────────────────────────────────┘
```

### 8. Onboarding Guide (`/onboarding`)

```text
┌──────────────────────────────────────────────────────────────┐
│ [Logo]                                        [Bell][Wallet] │
├──────────────────────────────────────────────────────────────┤
│ Welcome to NearSea — get started in 4 steps                  │
│                                                              │
│  1. Create a wallet      → [ Official wallet link ]          │
│  2. Get testnet funds    → [ Faucet ]                        │
│  3. Connect your wallet  → [ Connect Wallet ]                │
│  4. Make your first trade→ [ Explore ]                       │
│                                                              │
│  ⚠ Security tips: always check the official domain,          │
│    verify the price in your wallet before signing.           │
└──────────────────────────────────────────────────────────────┘
```

### 9. Admin (`/admin`)

```text
┌──────────────────────────────────────────────────────────────┐
│ [Logo]                                          [Wallet]     │
├──────────────────────────────────────────────────────────────┤
│ Admin   [ Reports ] [ Blocklist ] [ Verified ]               │
├──────────────────────────────────────────────────────────────┤
│ # │ Target              │ Reason   │ Reporter │ Status │ Act  │
│ 1 │ Genesis (collection)│ fraud    │ bob…     │ open   │ [▾]  │
│ 2 │ token …             │ spam     │ carol…   │ open   │ [▾]  │
├──────────────────────────────────────────────────────────────┤
│ Aksi: Hide from discovery · Ignore · Mark verified           │
│ Step-up: [ Confirm with your wallet ]                        │
└──────────────────────────────────────────────────────────────┘
```

---

## Kontrak Komponen (Props / API + Varian)

> Nama & varian **PROPOSED** sampai sesi branding; tipe mengikuti TypeScript strict (tanpa `any`). Semua teks lewat i18n (kunci, bukan literal).

| Komponen | Props inti | Varian | Catatan |
|---|---|---|---|
| `Button` | `variant`, `size`, `loading`, `disabled`, `icon?`, `onClick` | primary/secondary/ghost/danger; sm/md/lg | `loading` → spinner + disable |
| `NFT Card` | `token`, `price?`, `verified?`, `stale?`, `onClick` | default/hover/skeleton/stale | Harga via `lib/format` |
| `WalletButton` | `state`, `accountId?`, `balanceYocto?`, `onConnect`, `onDisconnect` | disconnected/connecting/connected/error | — |
| `Buy Button` | `listing`, `disabledReason?`, `onBuy` | enabled/pending-tx/disabled/sold | Disabled: self-buy/harga berubah/stale/bukan allowed_buyer |
| `Make Offer Modal` | `token`, `open`, `onSubmit`, `onClose` | form/confirm/pending/success/error | Escrow + min 0.01 Ⓝ |
| `List Modal` | `token`, `open`, `step`, `onApprove`, `onList` | step-1/step-2/pending/success/error | Progress indicator 1/2 |
| `Private Listing Modal` | `token`, `allowedBuyer`, `onSubmit` | form/preview/pending/success | Preview keterbatasan |
| `Bundle Builder` | `items[]`, `price`, `onApprove`, `onCreate` | select/approve/pending-per-item/success | Maks 10 token |
| `Notification Panel` | `items[]`, `unread`, `onOpen`, `onMarkAllRead` | idle-empty/idle-read/unread/loading/stale/error | Dedup `(receipt_id, event_index)` |
| `Report Modal` | `target`, `reasons[]`, `onSubmit` | form/pending/success/error | Alasan = enum |
| `Modal` | `open`, `title`, `onClose`, `children`, `footer?` | dialog/confirm | Fokus trap; Esc |
| `Drawer` | `open`, `side`, `onClose`, `children` | left/right/bottom | Mobile: hamburger |
| `Tabs` | `items[]`, `value`, `onChange` | underline | Cursor independen per tab |
| `Toast` | `severity`, `message`, `action?` | info/warning/error | Error boleh punya retry |
| `Badge` | `variant`, `label` | verified/status/stale | Wajib punya label teks (bukan warna saja) |
| `Price Chip` | `amountYocto`, `denom` | default/muted | Desimal via `lib/format` |
| `Activity Table` | `rows[]`, `loading`, `onLoadMore` | loading/empty/error | Tipe dari [features/users.md](./features/users.md) |
| `Trait Badge` | `traitType`, `value` | default | Fase 2/3 |
| `EmptyState` | `title`, `description?`, `action?` | — | Semua tab kosong |
| `Skeleton` | `variant` | text/card/grid/table | Loading |
| `Avatar` | `accountId`, `src?` | image/identicon | Fallback identicon |
| `Tooltip` | `content`, `children` | — | Penjelasan fee/royalti |

---

## Modal / Drawer Behavior

| Aspek | Aturan |
|---|---|
| Fokus saat buka | Fokus pindah ke elemen pertama (judul/aksi); fokus terkunci di dalam modal |
| Fokus saat tutup | Kembali ke elemen pemicu |
| Tutup | `Esc` + klik overlay + tombol close — **kecuali** saat signing/pending (tidak bisa ditutup sampai receipt final) |
| Scroll | Body scroll dikunci saat modal terbuka |
| Mobile | Modal full-screen; Drawer untuk menu/hamburger |
| Toast | Tampil di atas (`z-toast`); maks 1 toast per aksi; antre sisanya ke panel notifikasi |
| Konfirmasi destruktif | Butuh aksi eksplisit (Cancel listing, Delete) |
| ARIA | `role="dialog"` + `aria-modal="true"` + `aria-labelledby` |

---

## State per Halaman (Detail)

> Melengkapi "State per Halaman (wajib semua halaman)" di atas. Tiap halaman wajib punya: loading (skeleton) · empty · error (retry) · success · disabled (tx pending).

### Collection Header

| State | Tampilan |
|---|---|
| Loading | Skeleton banner + judul |
| Loaded | Banner + nama + creator + supply + royalti + tab |
| Verified | Badge ✓ (label teks) |
| Not verified | Tanpa badge |
| Empty (tanpa item) | Tab Items → EmptyState |
| Error | Inline retry (header tetap tampil) |

### Leaderboard

| State | Tampilan |
|---|---|
| Loading | Skeleton tabel |
| Empty | EmptyState "No sales yet" |
| Success | Tabel + pagination cursor |
| Unverified volume | Chip "unverified" (wash-trading flag) |
| Error | Inline retry |
| Fase 2 | Endpoint ★ fase 2 (MVP belum ada) |

### Settings

| State | Tampilan |
|---|---|
| Belum connect | Prompt connect wallet |
| Loading | Form skeleton |
| Loaded | Form terisi nilai profil saat ini |
| Saving | Tombol disabled + spinner |
| Field error | Inline error per field (`INVALID_*`) |
| Success | Toast "Profile updated" |
| Signature ditolak | Toast "dibatalkan"; data tak tersimpan |

### Onboarding

| State | Tampilan |
|---|---|
| Publik | Selalu tampil tanpa connect |
| Statis | Tanpa data dinamis |
| Link | Menuju kanal resmi + tips keamanan |
| Error | Tidak ada (statis) |

### Admin

| State | Tampilan |
|---|---|
| Tanpa scope | Halaman 403 |
| Loading | Tabel skeleton |
| Empty | EmptyState "No reports" |
| Success | Tabel + aksi |
| Step-up pending | Modal "Confirm with your wallet" |
| Step-up gagal | Toast + aksi tidak dieksekusi |
| Session expired | Re-login (tanpa silent refresh) |

---

## Form Field Validation Messages (EN)

| Form | Field | Aturan | Pesan (EN) |
|---|---|---|---|
| List | Price | ≥ 0.01 Ⓝ | "Minimum price is 0.01 Ⓝ" |
| List | Price | angka valid | "Enter a valid amount" |
| List | Allowed buyer | AccountId valid | "Enter a valid account ID" |
| Offer | Amount | ≥ 0.01 Ⓝ | "Offer must be at least 0.01 Ⓝ" |
| Offer | Duration | > 0 | "Choose a valid duration" |
| Bundle | Items | 1–10 | "Select between 1 and 10 items" |
| Bundle | Price | ≥ 0.01 Ⓝ | "Minimum price is 0.01 Ⓝ" |
| Create | Name | wajib | "Name is required" |
| Create | Royalty | ≤ 10% | "Royalty must be 10% or less" |
| Create | Phase window | `end > start`, non-overlap | "Phases cannot overlap" |
| Create | Allowlist | `max_mints ≤ max_per_wallet` | "Row {n}: mint limit exceeds max per wallet" |
| Settings | Alias | ≤ 32 | "Display name must be 32 characters or fewer" |
| Settings | Bio | ≤ 280 | "Bio must be 280 characters or fewer" |
| Settings | Avatar URL | https/ipfs allowlist | "Avatar URL is not allowed" |
| Report | Reason | enum wajib | "Choose a reason" |
| Admin | Step-up | signature valid | "Confirmation failed — try again" |

---

## Empty / Error / Loading Copy (EN)

| Konteks | Copy |
|---|---|
| Empty — discovery | "No items found" |
| Empty — collection items | "No items in this collection yet" |
| Empty — profile tab | "Nothing here yet" |
| Empty — notifications | "No notifications yet" |
| Empty — leaderboard | "No sales yet" |
| Empty — admin queue | "No reports to review" |
| Error — fetch | "Something went wrong" + [Retry] |
| Error — tx failed | "Transaction failed — your funds are on the way back" |
| Error — network | "You're offline — check your connection" |
| Error — stale listing | "This listing is no longer valid" |
| Loading — grid | skeleton cards |
| Loading — table | skeleton rows |
| Loading — tx pending | "Waiting for confirmation…" |
| Loading — re-verify | "Checking the latest price…" |
| Success — buy | "Purchase complete" |
| Success — list | "Your item is listed" |
| Success — offer | "Offer placed" |
| Success — mint | "Minted successfully" |
| Success — profile | "Profile updated" |

---

## Accessibility (WCAG 2.1 AA)

| Aspek | Aturan |
|---|---|
| **Level target** | WCAG 2.1 **AA** |
| Kontras teks | ≥ 4.5:1 (teks normal), ≥ 3:1 (teks besar) |
| Kontras non-teks | ≥ 3:1 (ikon, border, kontrol) |
| Fokus | Indikator fokus terlihat jelas; urutan fokus logis (kiri→kanan, atas→bawah) |
| Focus order | Header → konten utama → footer; modal: terkunci di dalam, kembali ke pemicu saat tutup |
| Focus trap | Modal/drawer menahan Tab di dalam (implementasi `Modal`/`Drawer`) |
| ARIA | `role="dialog"`+`aria-modal`, `aria-live="polite"` untuk status tx, `aria-label` untuk ikon-only button |
| Keyboard | Semua aksi bisa via keyboard; Esc menutup (kecuali pending) |
| Skip link | "Skip to content" di awal halaman |
| Reduced motion | Hormati `prefers-reduced-motion` |
| Warna makna | Tidak hanya warna — sertakan label/ikon (verified/stale/status) |
| Form | `<label>` eksplisit + `aria-describedby` untuk pesan error |
| Gambar | `alt` deskriptif; media untrusted via `<img>` (ADR-014) |
| Zoom | Layout tetap fungsional hingga 200% zoom |
| Uji | axe (opsional) di komponen design-system ([frontend-architecture.md](./architecture/frontend-architecture.md) §8) |

---

## Navigation Map (Route → Screen)

> Selaras [frontend-architecture.md](./architecture/frontend-architecture.md) §2. Guard: publik/wallet/admin.

| Route | Screen | Guard | Masuk dari |
|---|---|---|---|
| `/` | Home / Explore | publik | Logo, nav "Explore" |
| `/collection/[id]` | Halaman Koleksi | publik | Card koleksi, search |
| `/token/[contract]/[tokenId]` | Detail NFT | publik | NFT card, notifikasi, share |
| `/profile/[account]` | Profil User | publik | Alamat mana pun, nav "Profile" |
| `/create` | Create Collection | wallet | Nav "Create", dropdown Create |
| `/leaderboard` | Leaderboard | publik | Nav "Leaderboard" |
| `/settings` | Settings | wallet | Dropdown wallet, nav "Settings" |
| `/onboarding` | Onboarding Guide | publik | Nav "Guide", landing CTA |
| `/admin` | Admin | admin | URL langsung (guard 403) |
| `/notifications` (opsional) | Notifikasi | wallet | Bell → "View all" |

```text
[Logo] ──► /
Nav ──► / , /leaderboard , /create , /profile/:me , /settings , /onboarding
NFT card ──► /token/:c/:id ──► /collection/:id
Wallet dropdown ──► /profile/:me , /settings , Disconnect
Bell ──► dropdown ──► /notifications ──► destination (token/profile)
Admin (URL) ──► /admin
```

---

## Image Aspect Ratios

| Konteks | Rasio | Catatan |
|---|---|---|
| NFT Card (grid) | 1:1 | Crop tengah; lazy-load |
| NFT detail (media besar) | 1:1 (default) | Mendukung rasio lain dengan letterbox |
| Collection banner | 4:1 (desktop) · 2:1 (mobile) | — |
| Avatar / identicon | 1:1 | Bulat (`radius-full`) |
| Activity thumbnail | 1:1 | Kecil |
| Open Graph card | 1.91:1 | SSR shell |
| Media non-1:1 | letterbox di atas `color-surface` | Tidak pernah distorsi (stretch) |

- **Aturan**: `next/image` + ukuran responsif; SVG dirender via `<img>` (bukan `innerHTML`); placeholder saat gagal load.

---

## i18n Text Expansion

| Aspek | Aturan |
|---|---|
| Bahasa MVP | English (`en`); struktur siap multi-locale |
| Ekspansi desain | Rancang toleran **+30–40%** panjang string (locale lain bisa lebih panjang) |
| Tombol | Hindari lebar tetap ketat; `min-width` + padding; teks boleh wrap |
| Label form | Jangan singkat berlebihan; sediakan ruang untuk label panjang |
| Navigasi | Item nav tidak diasumsikan 1 kata |
| Angka/tanggal | `Intl` — ikuti locale (pemisah ribuan/desimal) |
| Mata uang | Simbol Ⓝ tetap; angka via `lib/format` |
| Plural | ICU plural (mis. `{count, plural, one {# item} other {# items}}`) |
| Placeholder | `{var}` — bukan konkatenasi string |
| RTL | ⏳ open-by-design; hindari asumsi LTR keras (mis. `margin-left` → logical properties) |
| Kunci | Bertipe (TypeScript) — kunci salah = error build |
| Konten user | Alias/bio/nama koleksi = data (tidak diterjemahkan) |

