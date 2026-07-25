import { cn } from "@/lib/utils";

export function GlassCard({
  className,
  children,
  tone = "default",
  id,
}: {
  className?: string;
  children: React.ReactNode;
  tone?: "default" | "accent" | "warn" | "good" | "bad";
  id?: string;
}) {
  return (
    <div
      id={id}
      className={cn(
        "rounded-2xl border bg-white/[0.03] backdrop-blur-md p-5",
        {
          "border-white/10": tone === "default",
          "border-[#FF6FAF]/40": tone === "accent",
          "border-amber-400/40": tone === "warn",
          "border-emerald-400/30": tone === "good",
          "border-red-400/30": tone === "bad",
        },
        className
      )}
    >
      {children}
    </div>
  );
}

export function Eyebrow({ children, className }: { children: React.ReactNode; className?: string }) {
  return (
    <div className={cn("text-[10px] font-semibold uppercase tracking-widest text-[#A1A1AA] mb-2", className)}>
      {children}
    </div>
  );
}
