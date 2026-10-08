import { WalletReadOnlyNotice } from "@/features/auth/components/WalletReadOnlyNotice";
import { ExplorePage } from "@/features/marketplace/ExplorePage";

import { MarketChainProvider } from "./MarketChainProvider";

export default function Home() {
  return (
    <MarketChainProvider>
      <WalletReadOnlyNotice />
      <ExplorePage />
    </MarketChainProvider>
  );
}
