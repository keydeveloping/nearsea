"use client";

import { useEffect, useRef } from "react";

/**
 * Penanda komponen masih ter-mount. Modal bisa ditutup di tengah transaksi, dan tanpa penanda
 * ini hasil `await` yang datang terlambat akan menulis state ke komponen yang sudah dilepas.
 */
export function useMountedRef() {
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  return mounted;
}
