// TC-040 · AC-WALLET-1, AC-WALLET-2, AC-WALLET-4
import { fireEvent, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { asYoctoNear } from "@/lib/format/money";

import { renderHeader } from "../test-utils";

describe("wallet control in the app header", () => {
  it("offers Connect and calls connect on click", () => {
    const connect = vi.fn(async () => undefined);
    renderHeader({ connect });

    fireEvent.click(screen.getByRole("button", { name: "Connect Wallet" }));

    expect(connect).toHaveBeenCalledTimes(1);
  });

  it("shows the connected address and balance, and disconnects on click", () => {
    const disconnect = vi.fn(async () => undefined);
    renderHeader({
      status: "connected",
      accountId: "alice.testnet",
      balanceYocto: asYoctoNear("1500000000000000000000000"),
      accountStatus: "ready",
      disconnect,
    });

    expect(screen.getByText("alice.testnet")).toBeDefined();
    expect(screen.getByText("1.5 Ⓝ")).toBeDefined();

    fireEvent.click(screen.getByRole("button", { name: "Disconnect" }));

    expect(disconnect).toHaveBeenCalledTimes(1);
  });

  it("returns to the disconnected state once the account is gone (post-disconnect)", () => {
    // State yang dihasilkan near-connect setelah `wallet:signOut`.
    renderHeader({ status: "disconnected", accountId: null, balanceYocto: null });

    expect(screen.queryByText("alice.testnet")).toBeNull();
    expect(screen.getByRole("button", { name: "Connect Wallet" })).toBeDefined();
  });

  it("shows a disconnecting state on the disconnect control, not a connect spinner", () => {
    // Regresi: `pending` yang tidak membedakan aksi membuat disconnect tampil sebagai
    // "Connecting…" + hint approve.
    renderHeader({
      status: "disconnecting",
      accountId: "alice.testnet",
      accountStatus: "ready",
    });

    const button = screen.getByRole("button", { name: "Disconnecting…" }) as HTMLButtonElement;

    expect(button.disabled).toBe(true);
    expect(screen.queryByText("Approve the request in your wallet")).toBeNull();
    expect(screen.queryByRole("button", { name: /Connecting/ })).toBeNull();
  });

  it("shows the connecting state with an approval hint and a disabled button", () => {
    renderHeader({ status: "connecting" });

    const button = screen.getByRole("button", { name: /Connecting/ }) as HTMLButtonElement;

    expect(button.disabled).toBe(true);
    expect(screen.getByText("Approve the request in your wallet")).toBeDefined();
  });

  it("disables Connect while the wallet layer is not yet mounted", () => {
    renderHeader({ status: "loading" });

    const button = screen.getByRole("button", { name: "Connect Wallet" }) as HTMLButtonElement;

    expect(button.disabled).toBe(true);
  });

  it("maps a rejected connect to readable copy plus Retry (no raw wallet error text)", () => {
    const retry = vi.fn(async () => undefined);
    renderHeader({ status: "error", errorCode: "rejected", retry });

    expect(screen.getByRole("alert").textContent).toContain("Connection cancelled in your wallet.");

    fireEvent.click(screen.getByRole("button", { name: "Retry" }));

    expect(retry).toHaveBeenCalledTimes(1);
  });

  it("maps an unavailable wallet to its own copy", () => {
    renderHeader({ status: "error", errorCode: "wallet_unavailable" });

    expect(screen.getByRole("alert").textContent).toContain(
      "That wallet is not available. Install it, then try again.",
    );
  });

  it("still surfaces the error when a disconnect fails while connected", () => {
    // Disconnect gagal meninggalkan user ter-connect — pesannya tidak boleh hilang
    // hanya karena status utamanya "connected".
    renderHeader({
      status: "connected",
      accountId: "alice.testnet",
      accountStatus: "ready",
      errorCode: "connect_failed",
    });

    expect(screen.getByText("alice.testnet")).toBeDefined();
    expect(screen.getByRole("alert").textContent).toContain(
      "Could not reach the wallet. Try again.",
    );
  });

  it("omits the balance line while the account probe is still in flight", () => {
    renderHeader({
      status: "connected",
      accountId: "alice.testnet",
      balanceYocto: null,
      accountStatus: "unknown",
    });

    expect(screen.getByText("alice.testnet")).toBeDefined();
    expect(screen.queryByText(/Ⓝ/)).toBeNull();
  });
});
