# Seed Data

> Data awal untuk development & demo. Target environment: **testnet** (keputusan ronde 3 — bukan sandbox/localnet).

## Kebutuhan

- 4 akun testnet: creator, seller, buyer, admin (report/verified).
- 2 koleksi contoh: (1) whitelist+public phases untuk uji launchpad, (2) simple open collection.
- ±20 token + 5 listing aktif + 2 offer + 1 bundle (maks 10 token) + 1 sold + 1 report + 1 entri verified.
- **Profil contoh untuk 4 akun** (alias/bio/avatar) — tabel app-mutable yang butuh data contoh.

## Sumber

- `scripts/seed.ts` (dijalankan `tsx scripts/seed.ts`) — satu metode: mint via `near-api-js` ke kontrak testnet.
- Kontrak NFT testnet publik boleh dipakai untuk aset contoh: `nft.examples.testnet` (lihat [../../RESEARCH.md](../../RESEARCH.md)).
- Data chain-derived (listing/offer/bundle/sold) dibuat lewat **transaksi kontrak**, bukan insert DB langsung; hanya data app-mutable (profil/report/verified) yang di-insert ke DB report API.

## Skrip

- `scripts/seed.ts` — mint token, buat listing/offer/bundle via kontrak, insert profil/report ke DB.

---

## 1. Dataset seed (eksak)

> Semua nama di bawah adalah **placeholder kanonik** untuk testnet; nilai final bisa di-override lewat env (lihat §7). Jangan pakai nama ini di mainnet.

### Akun (4)

| Peran | Account ID | Dipakai untuk |
|---|---|---|
| Creator | `nearsea-creator.testnet` | deploy 2 koleksi, set phase + allowlist, royalti |
| Seller | `nearsea-seller.testnet` | mint/own token, buat listing, bundle, accept offer |
| Buyer | `nearsea-buyer.testnet` | buy, make_offer |
| Admin | `nearsea-admin.testnet` | report queue, set verified, blocklist (allowlist DB) |

Akun tambahan untuk test negatif (bukan bagian 4 inti): `nearsea-bob.testnet` (private listing / allowlist miss), `nearsea-outsider.testnet` (self-buy/race).

### Koleksi (2)

| # | Account kontrak | Nama | Tipe | Phase | Royalti |
|---|---|---|---|---|---|
| C1 | `nearsea-genesis.testnet` | NearSea Genesis | launchpad whitelist → public | 2 phase | 5% |
| C2 | `nearsea-open.testnet` | NearSea Open | open collection (tanpa phase) | — | 0% |

### Token (20)

| Koleksi | Token IDs | Pemilik awal | Catatan |
|---|---|---|---|
| C1 | `1`..`12` | creator | mint via phase 1 (allowlist) untuk `1`..`4`, phase 2 (public) untuk `5`..`12` |
| C2 | `101`..`108` | creator → transfer | `101`..`104` ditransfer ke seller; `105`..`108` tetap creator |

### Listing (5 aktif + 1 sold + 1 private)

| # | Koleksi.Token | Seller | Harga | Tipe |
|---|---|---|---|---|
| L1 | C2.101 | seller | 1 Ⓝ | normal |
| L2 | C2.102 | seller | 2.5 Ⓝ | normal |
| L3 | C2.103 | seller | 0.01 Ⓝ | batas minimum (INV-030) |
| L4 | C1.5 | creator | 5 Ⓝ | normal |
| L5 | C1.6 | creator | 3 Ⓝ | private `allowed_buyer = nearsea-buyer.testnet` |
| LSOLD | C2.104 | seller | 1.5 Ⓝ | sudah dibeli buyer (riwayat sale) |

### Offer (2) & Bundle (1)

| # | Target | Buyer | Nominal | Status |
|---|---|---|---|---|
| O1 | C1.7 (tidak di-list) | buyer | 1 Ⓝ | ACTIVE |
| O2 | C1.8 | buyer | 0.5 Ⓝ | ACTIVE |

| # | Bundle | Seller | Item | Harga |
|---|---|---|---|---|
| B1 | `bundle-1` | seller | C2.105, C2.106, C2.107 (3 token) | 4 Ⓝ |

### Phase launchpad (C1)

| Phase | `phase_index` | Nama | Harga | Alokasi | Max/wallet | Window | Allowlist |
|---|---|---|---|---|---|---|---|
| P1 | 0 | Whitelist | 0.5 Ⓝ | 4 | 1 | T0 → T0+24 jam | ya (`allowlist-C1.csv`) |
| P2 | 1 | Public | 0.8 Ⓝ | 8 | 2 | T0+24 jam → T0+72 jam | tidak |

Window harus **berurutan, tidak overlap** (INV-029). T0 = waktu seed (deterministik dari env, §2).

### App-mutable (profil/report/verified)

| Tabel | Entri |
|---|---|
| `profiles` | 4 baris (creator/seller/buyer/admin) dengan alias + bio + avatar placeholder |
| `reports` | 1 baris: reporter `buyer`, target koleksi C2, reason `spam`, status `pending` |
| `collections.verified` | C1 = `true` (badge verified), C2 = `false` |
| `blocklist` | 1 baris: `target_type='token'`, `target_id='C2.108'`, reason seed |

> Data chain-derived (listing/offer/bundle/sale/mint) dibuat lewat **transaksi kontrak**, bukan insert DB langsung (lihat bagian "Sumber"). Hanya tabel app-mutable di atas yang di-insert langsung ke DB.

## 2. Kebijakan determinisme & randomness

| Aspek | Kebijakan |
|---|---|
| Akun | **deterministik** dari env `SEED_NAMESPACE` (default `nearsea`): `<ns>-creator.testnet`, dst. |
| Token ID | **deterministik**: C1 = `1..12`, C2 = `101..108` (urutan tetap) |
| Harga | **deterministik** (tabel §1); tidak ada harga acak |
| T0 (window phase) | deterministik: `SEED_T0` env bila diisi; jika tidak → `floor(now / 1 jam) * 1 jam` agar re-run dalam jam yang sama menghasilkan window identik |
| Timestamp DB | `created_at`/`decided_at` memakai `now()` — **tidak** dipaksa deterministik (bukan bagian asersi) |
| Metadata/trait | **deterministik**: atribut dihasilkan dari indeks token (mis. `Background = palette[token_id % N]`), bukan RNG |
| Media | aset tetap (lihat §6), nama file deterministik `token-<id>.png` |
| Randomness | hanya untuk nonce/keypair bila diperlukan; **tidak** untuk data yang di-assert |

Prinsip: seed yang dijalankan dua kali pada window yang sama menghasilkan **state logis identik** (token, harga, phase, profil). Hash media boleh berbeda hanya jika provider menyematkan timestamp — hindari.

## 3. Idempotency, re-run, reset & teardown

**Idempotency** — setiap langkah seed aman diulang:

```text
[ ] Cek akun: buat hanya bila belum ada (else reuse).
[ ] Cek koleksi: deploy hanya bila kontrak belum ada (else skip).
[ ] Cek token: mint hanya bila nft_token(token_id) belum ada.
[ ] Cek listing/offer/bundle: buat hanya bila belum ada (view call kontrak).
[ ] DB app-mutable: UPSERT (ON CONFLICT (account_id) DO UPDATE) untuk profil;
    report/blocklist: INSERT ... ON CONFLICT DO NOTHING (pakai natural key).
```

Contoh upsert profil (idempoten):

```sql
INSERT INTO profiles (account_id, alias, bio, avatar_url)
VALUES ($1, $2, $3, $4)
ON CONFLICT (account_id)
DO UPDATE SET alias = EXCLUDED.alias,
              bio   = EXCLUDED.bio,
              avatar_url = EXCLUDED.avatar_url,
              updated_at = now();
```

Contoh idempotent report (pakai unique harian):

```sql
INSERT INTO reports (reporter, contract_account_id, token_id, reason, status)
VALUES ($1, $2, NULL, 'spam', 'pending')
ON CONFLICT DO NOTHING;
```

**Reset (hapus state seed):**

```text
1. Kontrak: cancel semua listing/offer/bundle milik akun seed (via kontrak), lalu burn/transfer token seed.
2. DB: DELETE app-mutable seed saja (jangan sentuh data non-seed):
   DELETE FROM reports      WHERE reporter LIKE '<ns>-%';
   DELETE FROM profiles     WHERE account_id LIKE '<ns>-%';
   DELETE FROM blocklist    WHERE target_id LIKE '%' AND decided_by LIKE '<ns>-%';
   UPDATE collections SET verified = false, blocklisted = false WHERE contract_account_id LIKE '<ns>-%';
3. (fase 2) TRUNCATE tabel chain-derived seed bila ada → rebuild dari events_raw.
```

**Teardown penuh** (dev): drop database seed + redeploy kontrak; `scripts/seed-reset.ts --yes` wajib flag konfirmasi. Dilarang menjalankan reset/teardown terhadap testnet publik atau mainnet.

**Re-run**: `tsx scripts/seed.ts` (idempoten). `tsx scripts/seed.ts --reset` = reset lalu seed ulang.

## 4. Struktur file fixture

```text
scripts/
  seed.ts                 # entrypoint: orkestrasi (akun → koleksi → mint → order → DB)
  seed-reset.ts           # reset/teardown (flag --yes)
  seed/
    accounts.json         # daftar akun seed + peran
    collections.json      # definisi koleksi + royalti + base_uri
    tokens.json           # token per koleksi + owner awal + trait deterministik
    orders.json           # listing/offer/bundle (harga, allowed_buyer)
    phases.json           # phase launchpad (harga/alokasi/window/allowlist_required)
    profiles.json         # alias/bio/avatar 4 akun
    reports.json          # report seed
    allowlist-C1.csv      # daftar akun allowlist phase 1
    media/                # aset contoh (§6)
      token-1.png … token-12.png
      token-101.png … token-108.png
```

Contoh `tokens.json` (potongan):

```json
{
  "collection": "nearsea-open.testnet",
  "tokens": [
    { "token_id": "101", "owner": "nearsea-seller.testnet",
      "attributes": [{ "trait_type": "Background", "value": "Blue" }] },
    { "token_id": "102", "owner": "nearsea-seller.testnet",
      "attributes": [{ "trait_type": "Background", "value": "Green" }] }
  ]
}
```

Contoh `orders.json` (potongan; harga = string yoctoNEAR):

```json
{
  "listings": [
    { "contract": "nearsea-open.testnet", "token_id": "101",
      "seller": "nearsea-seller.testnet", "price_yocto": "1000000000000000000000000" },
    { "contract": "nearsea-open.testnet", "token_id": "103",
      "seller": "nearsea-seller.testnet", "price_yocto": "10000000000000000000000" }
  ],
  "offers": [
    { "contract": "nearsea-genesis.testnet", "token_id": "7",
      "buyer": "nearsea-buyer.testnet", "amount_yocto": "1000000000000000000000000" }
  ],
  "bundles": [
    { "seller": "nearsea-seller.testnet", "price_yocto": "4000000000000000000000000",
      "items": [
        { "contract": "nearsea-open.testnet", "token_id": "105" },
        { "contract": "nearsea-open.testnet", "token_id": "106" },
        { "contract": "nearsea-open.testnet", "token_id": "107" }
      ] }
  ]
}
```

> Catatan yoctoNEAR: 1 Ⓝ = `1000000000000000000000000`; 0.01 Ⓝ = `10000000000000000000000`.

## 5. Post-seed assertion checklist

Dijalankan otomatis di akhir `seed.ts` (gagal → exit non-zero):

```text
[ ] 4 akun seed ada; saldo cukup (>= 1 Ⓝ setelah seed) kecuali yang sengaja habis.
[ ] 2 koleksi ter-deploy; C1.verified = true, C2.verified = false.
[ ] 20 token ter-mint; owner awal sesuai tabel §1.
[ ] 5 listing ACTIVE + 1 SOLD + 1 private (allowed_buyer terisi).
[ ] 2 offer ACTIVE; tepat 1 offer ACTIVE per (token,buyer) — cek uq_offers_active.
[ ] 1 bundle ACTIVE berisi 3 item (≤ 10).
[ ] phase C1: 2 baris, tidak overlap, phase_index 0 & 1.
[ ] launchpad_mints: phase 1 berisi akun allowlist; mint di luar allowlist TIDAK ada.
[ ] profiles: 4 baris, alias ≤ 32 & bio ≤ 280.
[ ] reports: 1 baris pending.
[ ] blocklist: 1 baris (token C2.108).
[ ] semua harga di DB/event = string yoctoNEAR (tidak ada JSON number).
[ ] tidak ada duplikat event: dedup (receipt_id, event_index) unik (fase 2).
```

Contoh asersi SQL (fase 2):

```sql
-- Harus 0 baris: offer aktif ganda
SELECT contract_account_id, token_id, buyer, count(*)
FROM offers WHERE status = 'ACTIVE'
GROUP BY 1,2,3 HAVING count(*) > 1;

-- Harus 5 baris: listing aktif
SELECT count(*) FROM listings WHERE status = 'ACTIVE';
```

## 6. Media / aset seed

- 20 file PNG kecil (≤ 100 KB) di `scripts/seed/media/`, nama `token-<id>.png`.
- Metadata on-chain hanya berisi **URL + hash** (AGENTS Blockchain Rules); gambar TIDAK disimpan on-chain.
- Pada MVP, media boleh memakai gateway placeholder (mis. `https://ipfs.io/ipfs/<CID>` atau URL contoh) — **IPFS pinning masih ⏳ open-by-design** (tech-stack). Jangan mengunci provider di seed.
- `media_hash` diisi hash file seed (deterministik dari isi file).
- Avatar profil memakai URL placeholder yang lolos allowlist gateway (bukan file lokal).

## 7. Alamat kontrak & konfigurasi env

Nilai dibaca dari env (placeholder kanonik; final saat deploy — [../deployment/environments.md](../deployment/environments.md)):

| Env | Contoh (testnet) | Sumber |
|---|---|---|
| `NEAR_NETWORK` | `testnet` | `.env.example` |
| `NEAR_RPC_URL` | `https://rpc.testnet.near.org` | `.env.example` |
| `MARKET_CONTRACT_ID` | `market.nearsea.testnet` | deploy |
| `FACTORY_CONTRACT_ID` | `factory.nearsea.testnet` | deploy |
| `NFT_CONTRACT_ID_C1` | `nearsea-genesis.testnet` | deploy seed |
| `NFT_CONTRACT_ID_C2` | `nearsea-open.testnet` | deploy seed |
| `DATABASE_URL` | `postgresql://…` | secret server |
| `SEED_NAMESPACE` | `nearsea` | opsional (default) |
| `SEED_T0` | ISO-8601 | opsional (default = jam berjalan) |
| `SEED_MASTER_ACCOUNT` | `nearsea.testnet` | akun funding |

> `NFT_CONTRACT_ID_C1/C2` = tambahan khusus seed (bukan daftar kanonik environments.md); nilainya hanya dipakai skrip seed, tidak masuk runtime aplikasi.

## 8. Allowlist CSV (contoh)

`scripts/seed/allowlist-C1.csv`:

```csv
account_id,max_mints
nearsea-buyer.testnet,1
nearsea-bob.testnet,1
nearsea-outsider.testnet,1
```

Aturan:
- Header wajib: `account_id,max_mints`.
- Satu akun satu baris; duplikat ditolak saat parse.
- `max_mints` ≤ `launchpad_phases.max_per_wallet` phase terkait.
- Allowlist di-upload/di-set **on-chain** oleh creator (phase whitelist), bukan hanya di DB (INV-017; [../decisions/ADR-008-launchpad-phases.md](../decisions/ADR-008-launchpad-phases.md)).
- Akun di luar CSV dipakai untuk test negatif "not in allowlist".

## 9. Faucet & pendanaan

Urutan pendanaan sebelum seed:

```text
1. Buat/impor akun seed (deterministik dari SEED_NAMESPACE) via near-cli-rs.
2. Faucet testnet: https://near-faucet.io / helper `near account create-account fund-myself …`
   untuk setiap akun seed (butuh ±5 Ⓝ: storage + harga listing + fee).
3. Akun kontrak (market/factory/koleksi): alokasi storage deposit awal dari SEED_MASTER_ACCOUNT.
4. Verifikasi saldo minimum sebelum menjalankan seed:
     near state <account>  → saldo ≥ SEED_MIN_BALANCE (default 5 Ⓝ).
5. Storage deposit market (NEP-145) untuk seller/creator dipenuhi skrip seed sebelum listing.
```

- Testnet faucet = sumber dana gratis; **tidak ada** penggunaan dana mainnet di seed.
- Bila saldo kurang di tengah seed → skrip berhenti dengan pesan actionable (jangan lanjut parsial).
- Semua nilai saldo dibaca sebagai string yoctoNEAR.

## 10. Kaitan dataset → test case

Dataset §1 dirancang agar test case terkait bisa dijalankan langsung setelah seed:

| Entri seed | Test case |
|---|---|
| L1/L2 (listing normal) | TC-002 (list & buy happy path) |
| L3 harga 0.01 Ⓝ | TC-013 (batas minimum listing, INV-030) |
| L5 private + `nearsea-bob` | TC-011 (private listing, INV-026) |
| O1/O2 (offer aktif) | TC-004/TC-005 (offer escrow/accept), TC-008 (limit offer) |
| B1 (bundle 3 token) | TC-009/TC-010 (bundle sukses/gagal pre-validasi) |
| P1 allowlist + CSV | TC-007 (launchpad whitelist phase, INV-017/INV-029) |
| LSOLD (riwayat sale) | agregasi volume/payout (INV-001..003) |
| profiles 4 akun | AC Profil Custom |
| reports pending | alur admin moderasi (acceptance-criteria) |
| verified C1 | tampilan badge verified |

> Bila dataset diubah, cek kolom ini agar test case tetap punya fixture yang cocok (DOCUMENTATION-MAP T6/T10).

## Status

- Dataset — **DECIDED** (placeholder kanonik; nilai bisa di-override env).
- Target environment — testnet (ronde 3; bukan sandbox/localnet).
- Media/IPFS provider — ⏳ open-by-design (menunggu keputusan fitur mint).
