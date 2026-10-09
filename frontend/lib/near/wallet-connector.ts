import type { NearConnectorOptions } from "@hot-labs/near-connect";

import { NEAR_NETWORK, configuredRpcUrls } from "./network";

/**
 * Konfigurasi wallet connector dari env. **Tidak ada daftar wallet di sini**:
 * daftar resmi dimuat near-connect dari manifest-nya sendiri, sehingga wallet baru
 * otomatis muncul tanpa perubahan kode (features/auth.md §Supported Wallets).
 */
export function buildConnectorConfig(): NearConnectorOptions {
  return {
    network: NEAR_NETWORK,
    providers: { [NEAR_NETWORK]: configuredRpcUrls() },
  };
}
