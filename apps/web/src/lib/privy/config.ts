import type { PrivyClientConfig } from "@privy-io/react-auth";

export const PRIVY_APP_ID = process.env.NEXT_PUBLIC_PRIVY_APP_ID ?? "";

export const privyConfig: PrivyClientConfig = {
  appearance: {
    theme: "dark",
    accentColor: "#FF6FAF",
    logo: undefined,
  },
  loginMethods: ["email", "google", "wallet"],
  embeddedWallets: {
    solana: {
      createOnLogin: "users-without-wallets",
    },
  },
};
