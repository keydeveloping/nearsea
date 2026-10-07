# Infrastructure Security

> Keamanan infrastruktur self-hosted VPS (ronde 5) + dependensi. Least privilege & segmentation sesuai batas kecil tim.

## 1. Topologi MVP (DECIDED)

```text
Internet ── Caddy (TLS otomatis, rate limit, security headers) ── Next.js (FE + API routes) ── PostgreSQL (localhost only)
                                  └─ (opsional) indexer fase 2 (container terpisah)
VPS juga menyimpan: owner key dir (chmod 600, bukan web-root), .env server
```

## 2. Hardening VPS (checklist implementasi)

| Area | Kontrol |
|---|---|
| SSH | key-only, no root login, no password, fail2ban, port default boleh (obfuscation bukan kontrol) |
| Firewall | ufw/nftables: 80/443 in; 5432 DENY publik (localhost); SSH rate-limited |
| Updates | unattended-upgrades (security patches otomatis); reboot window |
| Process isolation | Docker Compose (app/db terpisah) — container non-root, read-only FS where possible |
| Least privilege | app user OS terpisah; DB app role non-superuser; tidak ada sudo untuk app process |
| Secrets | `.env` server-only chmod 600; tidak pernah di image/bundle; owner key terpisah dari app user (key-management.md) |
| Logging | auth.log + fail2ban + app logs → dipantau; alert Telegram untuk SSH gagal massal |

## 3. Segmentation & least privilege

- FE/API dan DB di satu VPS (accepted MVP trade-off — single box; risiko didokumentasikan di catastrophic-failure-scenarios).
- Object storage backup: kredensial write-only untuk backup job (baca restore manual).
- RPC/API pihak ketiga: kunci API (bila ada) di env server; frontend hanya butuh endpoint publik.
- Fase 2 indexer: container terpisah, DB role read-only terpisah, egress hanya ke RPC.

## 4. WAF / CDN / DoS

- MVP: tanpa CDN/WAF — mitigasi DoS = rate limit app + provider VPS; **OPEN QUESTION** (saat trafik naik): Cloudflare free tier di depan (anti-DDoS + sembunyikan origin IP).
- Origin IP terekspos via A record (tanpa CDN) = **risiko diterima di MVP** (target direct-DoS).
- **Accepted MVP risk (A2)**: owner key disimpan di VPS yang sama dengan app internet-facing — dikompensasi di key-management.md §2 (baca CS-1).

## 5. Secrets & observability

- Semua secret: env server; rotasi manual terdokumentasi; tidak ada secret manager di MVP (key-management.md).
- Observability: uptime-kuma (opsional), log app + auth, alert Telegram (ronde 12). Audit trail admin di DB (admin-security.md).
- Backup: pg_dump harian → object storage (database-security.md; restore drill SEC-DB-004).

## 6. Environment separation

- dev (lokal) / staging (subdomain, Docker terpisah) / testnet prod — konfigurasi env berbeda, DB terpisah.
- Konfigurasi produksi TIDAK boleh diubah manual di server (immutable via deploy) — sesuai AGENTS.md.

## 7. Firewall (aturan persis)

Default-deny inbound; hanya 3 port terbuka. Contoh `nftables` (di atas `ufw`/provider firewall):

```bash
# /etc/nftables.conf — PROPOSED, diterapkan saat provisioning (TASK-028).
#!/usr/sbin/nft -f
flush ruleset

table inet filter {
  chain input {
    type filter hook input priority 0; policy drop;

    ct state established,related accept          # koneksi balik
    iif lo accept                                # loopback

    # SSH: rate-limited (max 4 koneksi baru/menit per IP), port 22 (obfuscation bukan kontrol).
    tcp dport 22 ct state new limit rate 4/minute burst 8 packets accept

    # HTTP/HTTPS publik (Caddy).
    tcp dport { 80, 443 } ct state new accept

    # ICMP terbatas (diagnostik), rate-limited.
    ip protocol icmp icmp type { echo-request } limit rate 5/second accept

    # SISANYA DROP (termasuk 5432 PostgreSQL — tidak pernah terbuka ke publik).
    counter drop
  }
  chain forward { type filter hook forward priority 0; policy drop; }
  chain output  { type filter hook output priority 0; policy accept; }
}
```

- **5432 PostgreSQL WAJIB tidak diekspos** — DB bind `127.0.0.1` (compose `ports: ["127.0.0.1:5432:5432"]` atau tanpa publish) — SEC-INFRA-002.
- Port 80 hanya untuk redirect ACME ke 443; bisa ditutup setelah sertifikat stabil (opsional).
- Verifikasi aturan aktif: `sudo nft list ruleset`; uji dari luar: `nmap -Pn -p 22,80,443,5432 <host>` → hanya 22/80/443 `open`, 5432 `closed/filtered`.

## 8. sshd_config (direktif persis)

```text
# /etc/ssh/sshd_config.d/99-nearsea.conf — PROPOSED (TASK-028).
PermitRootLogin no
PasswordAuthentication no
KbdInteractiveAuthentication no
PubkeyAuthentication yes
AuthenticationMethods publickey
MaxAuthTries 3
MaxSessions 3
LoginGraceTime 20
ClientAliveInterval 300
ClientAliveCountMax 2
AllowUsers deploy
X11Forwarding no
AllowAgentForwarding no
AllowTcpForwarding no
PermitTunnel no
```

- `AllowUsers deploy` = hanya akun deploy (non-root, `sudo` terbatas) yang bisa SSH.
- Deploy key GitHub di-`command=`-restrict (forced command) ke skrip deploy (cicd-security §2).
- Verifikasi: `sudo sshd -T | grep -E 'permitrootlogin|passwordauthentication|allowusers|maxauthtries'` harus sesuai.
- Reload: `sudo systemctl reload ssh`; uji login baru di sesi terpisah SEBELUM menutup sesi lama.

## 9. Docker & Docker Compose (opsi keamanan)

```yaml
# docker-compose.yml — potongan keamanan (PROPOSED; final saat TASK-028).
services:
  web:
    image: nearsea/web:${TAG}
    user: "10001:10001"            # non-root
    read_only: true                # rootfs read-only
    tmpfs: ["/tmp"]                # tulis hanya ke tmpfs
    cap_drop: ["ALL"]              # buang semua capability
    security_opt: ["no-new-privileges:true"]
    restart: unless-stopped
    env_file: [.env.server]        # chmod 600, di luar web-root
    depends_on: [db]
    # tanpa `ports:` publik — Caddy yang mengekspos
  db:
    image: postgres:16
    user: "999:999"
    volumes: ["pgdata:/var/lib/postgresql/data"]
    ports: ["127.0.0.1:5432:5432"] # localhost only
    cap_drop: ["ALL"]
    cap_add: ["CHOWN", "SETUID", "SETGID", "DAC_OVERRIDE"] # minimum agar postgres start
    security_opt: ["no-new-privileges:true"]
  caddy:
    image: caddy:2
    ports: ["80:80", "443:443"]
    volumes: ["./Caddyfile:/etc/caddy/Caddyfile:ro", "caddy_data:/data"]
    cap_drop: ["ALL"]
    cap_add: ["NET_BIND_SERVICE"]
volumes: { pgdata: {}, caddy_data: {} }
```

- Semua container `no-new-privileges`; app & indexer **non-root**, read-only FS where possible.
- Tanpa `privileged`, tanpa mount socket Docker (`/var/run/docker.sock`) ke container app.
- Image di-pin per tag/digest (bukan `latest`); diperbarui via patch SLA §10.
- Verifikasi: `docker compose config` (validasi), `docker inspect --format '{{.HostConfig.Privileged}} {{.Config.User}}'` per container.

## 10. Patch SLA

| Kelas patch | Contoh | SLA terapkan | Mekanisme |
|---|---|---|---|
| Kritis (CVSS ≥ 9.0 / aktif dieksploitasi) | RCE OpenSSH, Docker escape | ≤ 48 jam | unattended-upgrades + manual bila perlu reboot |
| Tinggi (CVSS 7.0–8.9) | library dengan PoC publik | ≤ 7 hari | unattended-upgrades |
| Sedang (CVSS 4.0–6.9) | non-eksploit publik | ≤ 30 hari | window maintenance bulanan |
| Rendah (CVSS < 4.0) | kosmetik | ≤ 90 hari / saat upgrade rutin | maintenance |
| Kernel / butuh reboot | — | reboot window (mingguan, mis. Minggu 03:00 UTC) | `needrestart` + notifikasi |

- `unattended-upgrades` untuk security patches otomatis; reboot terjadwal terpisah.
- Patch dependensi app (npm/cargo) mengikuti gate CI (cicd-security §2), bukan SLA OS.
- Alert bila ada paket `security` tertunda > SLA (monitoring §11).

## 11. Verifikasi hardening & ambang monitoring

Perintah verifikasi (jalankan setelah provisioning + tiap kuartal):

```bash
# 1. Firewall
sudo nft list ruleset | grep -E 'dport (22|80|443|5432)'
# 2. SSH
sudo sshd -T | grep -E 'permitrootlogin|passwordauthentication|allowusers'
# 3. fail2ban aktif
sudo fail2ban-client status sshd
# 4. Docker non-root / no-new-privileges
docker ps --format '{{.Names}}' | xargs -I{} docker inspect --format '{{.Name}} user={{.Config.User}} priv={{.HostConfig.Privileged}}' {}
# 5. DB tidak terekspos
ss -tlnp | grep 5432          # harus 127.0.0.1:5432, bukan 0.0.0.0
# 6. unattended-upgrades
sudo unattended-upgrade --dry-run -d 2>&1 | tail
# 7. Port terbuka dari luar (dari mesin lain)
nmap -Pn -p 22,80,443,5432 <host>
```

Ambang monitoring (selaras [monitoring.md](../deployment/monitoring.md) §SLO):

| Metrik | Warning | Critical | Aksi |
|---|---|---|---|
| SSH gagal (auth.log) | > 20/menit | > 100/menit | fail2ban ban + alert Telegram |
| Disk VPS | > 80% | > 90% | bersihkan log/WAL; resize |
| Memori VPS | > 85% | > 95% | restart service; naik tier |
| CPU load (5 mnt) | > 70% | > 90% | investigasi (abuse/DoS) |
| Sertifikat TLS | ≤ 21 hari | ≤ 7 hari | cek Caddy/ACME |
| Paket security tertunda | > SLA kelas | > 2× SLA | patch manual |
| Container restart | > 3/jam | > 10/jam | cek log; rollback |

## 12. Enkripsi backup (detail)

| Aspek | Keputusan |
|---|---|
| Saat transit | TLS ke object storage (S3-compatible HTTPS) |
| Saat diam (at-rest) | SSE provider (default) **DAN** enkripsi dump sebelum upload: `age`/`gpg` dengan kunci publik backup |
| Kunci enkripsi | Kunci privat `age` disimpan offline (2 lokasi, terpisah dari VPS & dari seed owner); kunci publik di VPS untuk enkripsi |
| Verifikasi | `sha256sum -c` + `pg_restore --list`; restore drill per kuartal (SEC-DB-004) |
| Jika kunci bocor | Data minim-PII → dampak rendah; rotasi keypair `age` + re-encrypt backup berikutnya |
| Retensi | 7 harian + 4 mingguan + 3 bulanan (disaster-recovery.md) |

```bash
# Enkripsi dump sebelum upload (PROPOSED — TASK-030).
pg_dump -U "$POSTGRES_USER" -Fc -d "$POSTGRES_DB" \
  | age -r "$BACKUP_AGE_PUBLIC_KEY" -o "/backup/nearsea_$(date -u +%Y%m%dT%H%M%SZ).dump.age"
sha256sum /backup/*.dump.age > /backup/checksums.sha256
```

- Backup **tidak pernah** disimpan hanya di VPS yang sama (SEC-DB-001); object storage terpisah.
- Kredensial object storage write-only untuk job backup; restore butuh kredensial read (manual, terpisah).

## 13. Status

- Hardening checklist, firewall, sshd, Docker options, patch SLA, enkripsi backup — **PROPOSED** (diterapkan TASK-028/030; diverifikasi via perintah §11).
- Tanpa CDN/WAF — risiko MVP diterima (A2); CDN/WAF = G12 (⏳, saat trafik naik).
