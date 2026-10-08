import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

// Vitest berjalan tanpa globals, jadi auto-cleanup bawaan RTL tidak terdaftar;
// tanpa ini DOM menumpuk antar test dan query "single element" ikut gagal.
afterEach(cleanup);
