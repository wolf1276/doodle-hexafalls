"use client";

import { useState } from "react";
import { ChevronDown, Plus, ArrowDownToLine, Activity } from "lucide-react";
import { cn } from "@/lib/utils";

export function WalletWidget({ balance }: { balance: string }) {
  const [open, setOpen] = useState(false);

  return (
    <div className="relative">
      <button
        onClick={() => setOpen((v) => !v)}
        className="flex items-center gap-2 rounded-full border border-white/10 bg-white/[0.04] pl-1.5 pr-3 py-1.5 text-sm hover:border-[#FF6FAF]/40 transition-colors"
      >
        <span className="h-6 w-6 rounded-full bg-[#FF6FAF]/20 flex items-center justify-center text-[10px] text-[#FF6FAF] font-semibold">
          A
        </span>
        <span className="font-medium text-[#F5F5F5]">{balance}</span>
        <ChevronDown size={14} className="text-[#A1A1AA]" />
      </button>

      {open && (
        <div className="absolute right-0 mt-2 w-64 rounded-2xl border border-white/10 bg-[#0A0A0B]/95 backdrop-blur-xl p-3 shadow-2xl z-50">
          <div className="px-2 pb-2 text-[11px] uppercase tracking-widest text-[#A1A1AA]">
            Balance
          </div>
          <div className="px-2 pb-3 text-lg font-semibold text-white">{balance}</div>
          <div className="flex flex-col gap-1">
            <WalletAction icon={<Plus size={14} />} label="Add funds" />
            <WalletAction icon={<ArrowDownToLine size={14} />} label="Cash out" />
            <WalletAction icon={<Activity size={14} />} label="View wallet activity" />
          </div>
        </div>
      )}
    </div>
  );
}

function WalletAction({ icon, label, className }: { icon: React.ReactNode; label: string; className?: string }) {
  return (
    <button
      className={cn(
        "flex items-center gap-2 rounded-xl px-2 py-2 text-sm text-[#D4D4D8] hover:bg-white/5 hover:text-white transition-colors text-left",
        className
      )}
    >
      <span className="text-[#FF6FAF]">{icon}</span>
      {label}
    </button>
  );
}
