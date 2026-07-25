import Link from "next/link";
import { Briefcase, Hammer } from "lucide-react";
import { GlassCard } from "@/components/ui/glass-card";

export const metadata = { title: "Choose your role | DOODLE" };

export default function RoleSelectPage() {
  return (
    <section className="min-h-[calc(100vh-72px)] flex flex-col items-center justify-center px-6 py-24 bg-[#050505]">
      <div className="text-center mb-12">
        <h1
          className="text-white font-extrabold italic tracking-tight"
          style={{ fontSize: "clamp(32px, 5vw, 52px)", lineHeight: 1.05 }}
        >
          What brings you to DOODLE?
        </h1>
        <p className="text-[#A1A1AA] mt-3 text-base">
          Pick one — you can always switch later from settings.
        </p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-5 w-full max-w-[720px]">
        <Link href="/sign-in?role=freelancer">
          <GlassCard className="h-full flex flex-col items-center text-center gap-3 py-10 hover:border-[#FF6FAF]/50 transition-colors group">
            <div className="h-14 w-14 rounded-full border border-white/10 bg-[#FF6FAF]/10 flex items-center justify-center text-[#FF6FAF] group-hover:scale-105 transition-transform">
              <Hammer size={22} />
            </div>
            <div className="text-lg font-semibold text-white">I want to work</div>
            <p className="text-sm text-[#A1A1AA] max-w-[220px]">
              Find gigs and get paid instantly on delivery — escrow protects every milestone.
            </p>
            <span className="mt-2 text-sm font-semibold text-[#FF6FAF]">Continue →</span>
          </GlassCard>
        </Link>

        <Link href="/sign-in?role=client">
          <GlassCard className="h-full flex flex-col items-center text-center gap-3 py-10 hover:border-[#FF6FAF]/50 transition-colors group">
            <div className="h-14 w-14 rounded-full border border-white/10 bg-[#FF6FAF]/10 flex items-center justify-center text-[#FF6FAF] group-hover:scale-105 transition-transform">
              <Briefcase size={22} />
            </div>
            <div className="text-lg font-semibold text-white">I want to hire</div>
            <p className="text-sm text-[#A1A1AA] max-w-[220px]">
              Post a job and hire talent with milestone-based escrow protection.
            </p>
            <span className="mt-2 text-sm font-semibold text-[#FF6FAF]">Continue →</span>
          </GlassCard>
        </Link>
      </div>

      <p className="text-[11px] text-[#71717A] mt-10 max-w-[420px] text-center">
        Your role is saved to your profile after first sign-in — next time you&apos;ll land straight in your dashboard.
      </p>
    </section>
  );
}
