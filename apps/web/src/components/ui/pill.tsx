import { cn } from "@/lib/utils";

export function Pill({
  children,
  className,
  tone = "default",
}: {
  children: React.ReactNode;
  className?: string;
  tone?: "default" | "accent" | "good" | "warn" | "bad";
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-full border px-3 py-1 text-[11px] font-medium",
        {
          "border-white/15 bg-white/[0.03] text-[#D4D4D8]": tone === "default",
          "border-[#FF6FAF]/40 bg-[#FF6FAF]/10 text-[#FF6FAF]": tone === "accent",
          "border-emerald-400/30 bg-emerald-400/10 text-emerald-300": tone === "good",
          "border-amber-400/30 bg-amber-400/10 text-amber-300": tone === "warn",
          "border-red-400/30 bg-red-400/10 text-red-300": tone === "bad",
        },
        className
      )}
    >
      {children}
    </span>
  );
}

export function Stars({ count = 5, filled = 5 }: { count?: number; filled?: number }) {
  return (
    <span className="text-[#FF6FAF] text-xs tracking-[2px]">
      {"★".repeat(filled)}
      <span className="text-white/20">{"★".repeat(Math.max(0, count - filled))}</span>
    </span>
  );
}
