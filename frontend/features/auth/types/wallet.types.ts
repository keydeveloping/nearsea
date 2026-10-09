import type { YoctoNear } from "@/lib/format/money";
import type { NearNetwork } from "@/lib/near/network";
import type { WalletConnectErrorCode } from "@/lib/near/wallet-errors";

/**
 * `loading` = lapisan wallet belum ter-mount di browser (SSR / pra-hidrasi);
 * aksi connect di-disable selama status ini.
 */
export type WalletStatus =
  "loading" | "connecting" | "disconnecting" | "connected" | "disconnected" | "error";

/**
 * Hasil probe akun ke jaringan terkonfigurasi.
 *
 * `not_found` sengaja **tidak** dinamai "wrong network": akun yang tidak ada di jaringan ini
 * bisa berarti wallet di jaringan lain **atau** akun belum dibuat/di-fund. Keduanya butuh
 * tindakan user yang sama (periksa jaringan wallet + saldo), jadi digabung — tetapi copy UI
 * menyebut kedua sebabnya, bukan mengklaim jaringan yang salah.
 */
export type AccountStatus = "unknown" | "ready" | "not_found" | "unreachable";

/** Bentuk state wallet yang dikonsumsi UI — satu tipe, bukan beberapa boolean lepas. */
export interface WalletApi {
  status: WalletStatus;
  accountId: string | null;
  network: NearNetwork;
  /** Saldo yoctoNEAR sebagai string (u128) — dilarang `number`. */
  balanceYocto: YoctoNear | null;
  accountStatus: AccountStatus;
  /**
   * Gerbang transaksi: `true` selama akun belum terbukti ada di jaringan terkonfigurasi
   * (features/auth.md §Account / Network Switching — mismatch = hard block). TASK-008
   * **wajib** men-disable tombol aksi berbasis flag ini, bukan sekadar menampilkan banner.
   */
  transactionsDisabled: boolean;
  errorCode: WalletConnectErrorCode | null;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  /** Mengulang aksi terakhir yang gagal — bukan selalu connect (mis. disconnect yang gagal). */
  retry: () => Promise<void>;
}
