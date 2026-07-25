"use client";

import { useState } from "react";
import Link from "next/link";
import { Search, Menu, X, Bell, Mail, Heart, Flame } from "lucide-react";
import { usePrivy } from "@privy-io/react-auth";
import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
} from "@/components/ui/navigation-menu";
import { Button } from "@/components/ui/button";

function AccountButton({ className }: { className?: string }) {
  const { ready, authenticated, user, login, logout } = usePrivy();
  const label = authenticated
    ? user?.email?.address ?? user?.google?.email ?? "Account"
    : "Connect Wallet";

  return (
    <Button
      variant="secondary"
      size="sm"
      className={className}
      disabled={!ready}
      onClick={() => (authenticated ? logout() : login())}
    >
      {label}
    </Button>
  );
}

const CATEGORY_LINKS = [
  "Graphics & Design",
  "Programming & Tech",
  "Digital Marketing",
  "Video & Animation",
  "Writing & Translation",
  "Music & Audio",
  "Business",
  "Finance",
];

const TALENT_CATEGORIES = [
  { label: "Smart Contract Dev", description: "Solana, EVM & audits" },
  { label: "Frontend Engineering", description: "React, Next.js, wallets" },
  { label: "Protocol Design", description: "Tokenomics & architecture" },
  { label: "Security Audits", description: "Program & contract review" },
];

const AI_SERVICES = [
  { label: "AI Agents", description: "Custom autonomous agents" },
  { label: "Prompt Engineering", description: "Optimized LLM workflows" },
  { label: "Data Labeling", description: "Training data at scale" },
  { label: "Chatbots", description: "Support & sales automation" },
];

export function Navbar() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  return (
    <header className="sticky top-0 z-50 w-full bg-[#050505]/95 backdrop-blur-md border-b border-white/[0.06]">
      {/* TOP ROW */}
      <div className="w-full flex items-center justify-center">
        <div className="w-full max-w-[1440px] h-[72px] px-6 md:px-[48px] flex items-center gap-6">
          {/* Logo */}
          <Link href="/" className="flex-shrink-0">
            <span className="text-white font-bold text-xl tracking-tight">
              DOODLE<span className="text-[#FF6FAF]">.</span>
            </span>
          </Link>

          {/* Search Bar (Desktop) */}
          <div className="hidden md:flex flex-1 max-w-[640px] items-center">
            <div className="w-full flex items-center h-11 rounded-full bg-white/[0.04] border border-white/10 focus-within:border-[#FF6FAF]/60 transition-colors duration-200">
              <input
                type="text"
                placeholder="What service are you looking for today?"
                className="flex-1 bg-transparent px-5 text-sm text-[#F5F5F5] placeholder:text-[#71717A] outline-none"
              />
              <button
                aria-label="Search"
                className="h-11 w-11 flex items-center justify-center flex-shrink-0 rounded-full bg-[#FF6FAF] text-[#050505] hover:bg-[#FF6FAF]/90 transition-colors duration-200"
              >
                <Search size={16} strokeWidth={2.5} />
              </button>
            </div>
          </div>

          {/* Right Actions (Desktop) */}
          <div className="hidden md:flex items-center gap-5 ml-auto">
            <button
              aria-label="Notifications"
              className="text-[#D4D4D8] transition-colors duration-200 hover:text-[#FF6FAF]"
            >
              <Bell size={19} strokeWidth={2} />
            </button>
            <button
              aria-label="Messages"
              className="text-[#D4D4D8] transition-colors duration-200 hover:text-[#FF6FAF]"
            >
              <Mail size={19} strokeWidth={2} />
            </button>
            <button
              aria-label="Saved"
              className="text-[#D4D4D8] transition-colors duration-200 hover:text-[#FF6FAF]"
            >
              <Heart size={19} strokeWidth={2} />
            </button>
            <Link
              href="/transactions"
              className="text-[#D4D4D8] font-medium text-sm transition-colors duration-200 hover:text-[#FF6FAF]"
            >
              Orders
            </Link>

            <div className="w-px h-6 bg-white/10" />

            <AccountButton className="rounded-full font-semibold" />
          </div>

          {/* Mobile toggles */}
          <div className="flex md:hidden items-center gap-4 ml-auto">
            <button aria-label="Search" className="text-[#D4D4D8]">
              <Search size={20} strokeWidth={2} />
            </button>
            <button
              aria-label="Toggle menu"
              className="text-white"
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
            >
              {mobileMenuOpen ? <X size={24} /> : <Menu size={24} />}
            </button>
          </div>
        </div>
      </div>

      {/* CATEGORY ROW (Desktop) */}
      <div className="hidden md:flex w-full items-center justify-center border-t border-white/[0.06]">
        <div className="w-full max-w-[1440px] px-6 md:px-[48px]">
          <NavigationMenu className="max-w-none justify-start">
            <NavigationMenuList className="justify-start gap-1 h-12">
              <NavigationMenuItem>
                <NavigationMenuLink
                  href="#"
                  className="flex items-center gap-1.5 text-sm font-medium text-[#F5F5F5] hover:bg-white/5"
                >
                  <Flame size={14} className="text-[#FF6FAF]" />
                  Trending
                </NavigationMenuLink>
              </NavigationMenuItem>

              <NavigationMenuItem>
                <NavigationMenuTrigger className="text-[#D4D4D8] data-[state=open]:text-[#FF6FAF]">
                  Find Talent
                </NavigationMenuTrigger>
                <NavigationMenuContent>
                  <ul className="grid w-[440px] grid-cols-2 gap-1 p-2">
                    {TALENT_CATEGORIES.map((item) => (
                      <li key={item.label}>
                        <NavigationMenuLink href="#" className="flex-col items-start gap-0.5">
                          <span className="text-sm font-medium text-[#F5F5F5]">
                            {item.label}
                          </span>
                          <span className="text-xs text-[#A1A1AA]">
                            {item.description}
                          </span>
                        </NavigationMenuLink>
                      </li>
                    ))}
                  </ul>
                </NavigationMenuContent>
              </NavigationMenuItem>

              <NavigationMenuItem>
                <NavigationMenuTrigger className="text-[#D4D4D8] data-[state=open]:text-[#FF6FAF]">
                  AI Services
                </NavigationMenuTrigger>
                <NavigationMenuContent>
                  <ul className="grid w-[440px] grid-cols-2 gap-1 p-2">
                    {AI_SERVICES.map((item) => (
                      <li key={item.label}>
                        <NavigationMenuLink href="#" className="flex-col items-start gap-0.5">
                          <span className="text-sm font-medium text-[#F5F5F5]">
                            {item.label}
                          </span>
                          <span className="text-xs text-[#A1A1AA]">
                            {item.description}
                          </span>
                        </NavigationMenuLink>
                      </li>
                    ))}
                  </ul>
                </NavigationMenuContent>
              </NavigationMenuItem>

              {CATEGORY_LINKS.map((item) => (
                <NavigationMenuItem key={item}>
                  <NavigationMenuLink
                    href="#"
                    className="text-sm font-medium text-[#D4D4D8] hover:text-[#F5F5F5] hover:bg-white/5 whitespace-nowrap"
                  >
                    {item}
                  </NavigationMenuLink>
                </NavigationMenuItem>
              ))}
            </NavigationMenuList>
          </NavigationMenu>
        </div>
      </div>

      {/* MOBILE MENU DROPDOWN */}
      {mobileMenuOpen && (
        <div className="md:hidden border-t border-white/10 flex flex-col p-6 gap-4 bg-[#050505]">
          <div className="flex items-center h-11 rounded-full bg-white/[0.04] border border-white/10 mb-2">
            <input
              type="text"
              placeholder="What service are you looking for today?"
              className="flex-1 bg-transparent px-5 text-sm text-[#F5F5F5] placeholder:text-[#71717A] outline-none"
            />
          </div>

          <Link
            href="/talent"
            className="text-[#D4D4D8] font-medium text-base transition-colors duration-200 hover:text-[#FF6FAF]"
          >
            Find Talent
          </Link>
          {["AI Services", ...CATEGORY_LINKS].map((item) => (
            <a
              key={item}
              href="#"
              className="text-[#D4D4D8] font-medium text-base transition-colors duration-200 hover:text-[#FF6FAF]"
            >
              {item}
            </a>
          ))}

          <div className="flex items-center gap-6 pt-4 mt-2 border-t border-white/10">
            <button aria-label="Notifications" className="text-[#D4D4D8]">
              <Bell size={20} strokeWidth={2} />
            </button>
            <button aria-label="Messages" className="text-[#D4D4D8]">
              <Mail size={20} strokeWidth={2} />
            </button>
            <button aria-label="Saved" className="text-[#D4D4D8]">
              <Heart size={20} strokeWidth={2} />
            </button>
            <Link href="/transactions" className="text-[#D4D4D8] font-medium text-sm">
              Orders
            </Link>
          </div>

          <AccountButton className="w-full rounded-full font-semibold mt-2" />
        </div>
      )}
    </header>
  );
}
