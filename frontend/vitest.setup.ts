import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

// Vitest berjalan tanpa globals, jadi auto-cleanup bawaan RTL tidak terdaftar;
// tanpa ini DOM menumpuk antar test dan query "single element" ikut gagal.
afterEach(cleanup);

// Alamat kontrak market untuk test. Nilai asli datang dari env saat build/deploy; test
// memakainya supaya jalur "kontrak terkonfigurasi" benar-benar dieksekusi — bukan jalur
// "belum dikonfigurasi" yang kebetulan lolos karena env kosong.
process.env.NEXT_PUBLIC_MARKET_CONTRACT_ID ??= "market.testnet";
