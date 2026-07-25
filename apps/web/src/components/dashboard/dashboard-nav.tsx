"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";
import { WalletWidget } from "./wallet-widget";

const FREELANCER_LINKS = [
  { href: "/dashboard", label: "Dashboard" },
  { href: "/gigs", label: "Browse Jobs" },
  { href: "/dashboard#services", label: "My Services" },
  { href: "/dashboard#portfolio", label: "Portfolio" },
];

const CLIENT_LINKS = [
  { href: "/client", label: "Dashboard" },
  { href: "/talent", label: "Browse Talent" },
  { href: "/transactions", label: "Transactions" },
  { href: "/services-taken", label: "Services Taken" },
];

export function DashboardNav({ role, balance }: { role: "freelancer" | "client"; balance: string }) {
  const pathname = usePathname();
  const links = role === "freelancer" ? FREELANCER_LINKS : CLIENT_LINKS;

  return (
    <div className="w-full max-w-[1200px] mx-auto flex items-center justify-between gap-6 py-4 px-6 md:px-0">
      <div className="flex items-center gap-6">
        {links.map((l) => (
          <Link
            key={l.label}
            href={l.href}
            className={cn(
              "text-sm font-medium transition-colors",
              pathname === l.href.split("#")[0]
                ? "text-white"
                : "text-[#A1A1AA] hover:text-white"
            )}
          >
            {l.label}
          </Link>
        ))}
      </div>
      <WalletWidget balance={balance} />
    </div>
  );
}
