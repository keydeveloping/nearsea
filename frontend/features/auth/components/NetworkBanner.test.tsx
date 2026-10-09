// TC-040 · AC-WALLET-1 (indikator jaringan)
import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { renderHeader } from "../test-utils";

describe("network banner in the app header", () => {
  it("always shows the configured network", () => {
    renderHeader();

    expect(screen.getByRole("status").textContent).toBe("Network: testnet");
  });

  it("warns when the connected account does not exist on the configured network", () => {
    renderHeader({
      status: "connected",
      accountId: "alice.testnet",
      accountStatus: "not_found",
    });

    const banner = screen.getByRole("status");
    expect(banner.textContent).toContain("Account not found on testnet.");
    // Copy tidak boleh mengklaim jaringan salah sebagai satu-satunya sebab.
    expect(banner.textContent).toContain("or this account has not been created or funded yet");
  });

  it("stays a plain indicator while the account probe is unknown or in flight", () => {
    renderHeader({
      status: "connected",
      accountId: "alice.testnet",
      accountStatus: "unknown",
    });

    expect(screen.getByRole("status").textContent).toBe("Network: testnet");
  });

  it("stays a plain indicator when the RPC probe itself failed", () => {
    // RPC bermasalah bukan bukti akun ada di jaringan lain.
    renderHeader({
      status: "connected",
      accountId: "alice.testnet",
      accountStatus: "unreachable",
    });

    expect(screen.getByRole("status").textContent).toBe("Network: testnet");
  });

  it("does not warn for a disconnected visitor", () => {
    renderHeader({ accountStatus: "not_found" });

    expect(screen.getByRole("status").textContent).toBe("Network: testnet");
  });
});
