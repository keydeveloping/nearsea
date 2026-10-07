# ADR-014: Metadata Isolation — Browser-only di MVP, Fetcher Terisolasi di Fase 2

## Status
Accepted (MVP); Fetcher design PROPOSED (fase 2)

## Problem
Metadata NFT (media URL, deskripsi, extra) = input user yang tidak dipercaya dari koleksi mana pun. Server yang mem-fetch URL arbitrer = SSRF surface klasik; browser yang merender = XSS/SVG surface.

## Context
MVP tidak punya fitur server-side fetch (thumbnail/rekap) — media dirender langsung FE via `<img>`. Indexer fase 2 akan menambah fetch server-side.

## Options
1. Server proxy semua metadata (render server-side) — kontrol penuh, tapi server menyentuh untrusted URL (SSRF surface) sejak MVP.
2. Browser-only (MVP) + isolated fetcher service (fase 2) — SSRF surface kosong di MVP; fetcher saat benar-benar dibutuhkan.
3. Tanpa isolasi (fetch bebas) — ditolak.

## Trade-offs
- Opsi 1: konsisten tapi membuka SSRF/decode-bomb sejak hari pertama tanpa manfaat MVP.
- Opsi 2: keamanan mengikuti kebutuhan; fetcher harus tetap dibangun terisolasi (allowlist, pin IP, limits, proses terpisah) — didesain sekarang (metadata-security.md §3).

## Security implications
Opsi 2 meniadakan SSRF di MVP (FACT: tidak ada code path server→internet untuk metadata). Browser tetap harus: img-only, CSP, gateway allowlist (SEC-META-001).

## Scalability / Operational / Cost
Opsi 2: nol infra tambahan di MVP; fetcher fase 2 = satu container.

## SSRF-bypass yang harus ditutup (enumerasi)

> Setiap kelas bypass di bawah wajib punya kontrol eksplisit di fetcher. Aturan kanonik: **https only; validasi host SEBELUM request; tolak localhost/loopback/private/link-local/reserved; anti-DNS-rebinding (pin IP); redirect ≤3, timeout 5s, response ≤5MB, dekompresi ≤25MB.** Detail nilai: [metadata-security.md](../security/metadata-security.md) §3/§7.

| # | Kelas bypass | Contoh | Kontrol wajib |
|---|---|---|---|
| SSRF-1 | **Loopback** | `https://127.0.0.1/…`, `https://[::1]/…` | tolak `127/8`, `::1` — hanya global-unicast |
| SSRF-2 | **Private (RFC1918)** | `https://10.0.0.5/…`, `172.16/12`, `192.168/16` | tolak private ranges |
| SSRF-3 | **Link-local / metadata** | `https://169.254.169.254/latest/meta-data/` (cloud metadata) | tolak `169.254/16`, `fe80::/10` |
| SSRF-4 | **CGNAT / shared** | `https://100.64.0.1/…` | tolak `100.64/10` |
| SSRF-5 | **Reserved / unspecified** | `0.0.0.0`, `::ffff:0:0/96`, `fc00::/7` | tolak semua yang bukan global-unicast |
| SSRF-6 | **DNS rebinding** (TOCTOU) | resolve → IP publik, request kedua resolve → IP internal | resolve → validasi IP → **koneksi ke IP yang sama (pin)**, bukan resolve ulang |
| SSRF-7 | **Redirect berantai** | URL publik → 302 ke `http://169.254.169.254/` | redirect ≤3 hop; **re-validasi protokol + SSRF guard tiap hop** |
| SSRF-8 | **Alternate IP encoding** | desimal/oktal/hex IP, `0x7f000001`, `2130706433` | parse host sebagai IP kanonik lalu validasi; jangan percaya string host |
| SSRF-9 | **DNS wildcard / nip.io** | `169.254.169.254.nip.io` | validasi **IP hasil resolve** (bukan hostname) → tertangkap oleh pin IP |
| SSRF-10 | **Skema non-https** | `http:`, `ftp:`, `file:`, `gopher:`, `data:` | protokol allowlist: hanya `https` + `ipfs://`→gateway; `data:` default TOLAK |
| SSRF-11 | **IPv6-mapped IPv4** | `::ffff:127.0.0.1` | normalisasi lalu tolak (bagian dari reserved `::ffff:0:0/96`) |

- **Urutan wajib**: validasi **sebelum** membuka koneksi (fail-closed). Pelanggaran → tolak fetch, bukan retry tanpa batas.
- Kontrol tambahan: `GET`/`HEAD` saja, Content-Type allowlist (`image/*`, `application/json`), timeout 5s/hop & 10s total, cap ukuran terkompresi 5 MB, hasil dekompresi 25 MB (rasio ≤100:1), MIME tidak dipercaya dari ekstensi.

## Arsitektur fetcher terisolasi (ASCII)

```text
                    INDEXER / API (VPS)
                          │  butuh thumbnail/JSON metadata
                          ▼
                 ┌───────────────────┐
                 │  FETCHER CONTAINER │  (proses terpisah; TIDAK punya kredensial DB utama)
                 │                    │
   URL untrusted │ 1. parse + validasi skema (https / ipfs→gateway allowlist)
   ─────────────►│ 2. resolve host → validasi IP global-unicast (SSRF-1..5,8,11)
                 │ 3. PIN IP → buka koneksi ke IP itu (anti-DNS-rebinding SSRF-6)
                 │ 4. follow redirect ≤3, re-validasi TIAP hop (SSRF-7)
                 │ 5. timeout 5s/hop, cap 5MB, dekompresi ≤25MB
                 │ 6. Content-Type allowlist; GET/HEAD saja
                 │ 7. hitung sha256 byte mentah → cache hash-keyed
                 └─────────┬──────────┘
                           │  hasil (thumbnail / JSON) + hash
                           ▼
                    CACHE (hash-keyed, TTL)  ──►  INDEXER DB / API (proyeksi)
                           │
                  egress terbatas: HANYA internet; DILARANG menjangkau
                  jaringan internal/DB/loopback (network policy)
```

- **Isolasi**: container terpisah dari API utama; egress dibatasi; tanpa kredensial DB utama; rate limit sendiri. Kegagalan fetch = placeholder, **bukan** error user.

## Cost / ops fetcher

| Aspek | Nilai | Jenis |
|---|---|---|
| Infra tambahan MVP | **0** (tidak ada fetch server-side) | FACT (desain) |
| Infra fase 2 | **1 container** di VPS yang sama (tanpa VM/DB tambahan) | PROPOSED |
| Kredensial DB utama di fetcher | **tidak ada** (least privilege) | DECIDED (metadata-security §3) |
| Rate limit global fetcher | 50 request/detik | PROPOSED (§7) |
| Rate limit per-host tujuan | 10 request/detik | PROPOSED (§7) |
| Dedup in-flight per URL | 1 request (single-flight) | PROPOSED (§7) |
| Cache TTL sukses | 7 hari (konten IPFS immutable) | PROPOSED (§7) |
| Cache TTL gagal | 5 menit | PROPOSED (§7) |
| Circuit breaker per host | buka setelah 10 kegagalan beruntun, 5 menit, lalu half-open | PROPOSED (§7) |
| Resource container | kecil (fetch I/O-bound; CPU rendah; RAM & disk terikat cache) — angka final = ESTIMASI, diukur saat implementasi | ESTIMASI |

- Biaya utama = **bandwidth egress** dan **CPU dekompresi** (dibatasi 25 MB/response) — keduanya dikontrol oleh cap & rate limit di atas; tanpa cap, fetcher bisa jadi vektor DoS keluar atau diblokir gateway.
- Fetcher tidak menambah biaya storage signifikan: cache hash-keyed dengan TTL panjang (konten immutable) → hit rate tinggi.

## Decision
**Opsi 2.** Dilarang menambah server-side fetch apa pun sebelum fetcher terisolasi (SEC-META-002) diimplementasi.

## Rejected
- **Opsi 1 (server proxy sejak MVP)** — menambah SSRF/decode-bomb surface tanpa manfaat MVP.
- **Opsi 3 (fetch bebas tanpa isolasi)** — ditolak; fetcher tanpa SSRF guard = akses jaringan internal.

## Consequences
FE rules (frontend-security) wajib sejak TASK-008; TASK-014 (indexer) berprasyarat SEC-META-002.
