import Link from "next/link";
import { DashboardNav } from "@/components/dashboard/dashboard-nav";
import { GlassCard, Eyebrow } from "@/components/ui/glass-card";
import { Button } from "@/components/ui/button";

export const metadata = { title: "Client Dashboard | DOODLE" };

const STATS = [
  { label: "Talent hired", value: "14" },
  { label: "Total spent", value: "$1,240" },
  { label: "Avg rating given", value: "4.7" },
];

export default function ClientDashboardPage() {
  return (
    <section className="min-h-[calc(100vh-72px)] bg-[#050505]">
      <div className="border-b border-white/[0.06]">
        <DashboardNav role="client" balance="₹8,200" />
      </div>

      <div className="max-w-[1200px] mx-auto px-6 py-10">
        <div className="flex items-center justify-between mb-8">
          <h1 className="text-2xl font-bold text-white">Welcome back</h1>
          <Link href="/gigs/new">
            <Button>Post a gig</Button>
          </Link>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-5 mb-6">
          <GlassCard>
            <Eyebrow>Active (2)</Eyebrow>
            <div className="flex flex-col divide-y divide-white/[0.06]">
              <div className="py-3 first:pt-0">
                <div className="text-sm font-medium text-white">Label product images</div>
                <div className="text-xs text-[#A1A1AA] mt-0.5">Freelancer working · 2d left</div>
              </div>
              <div className="py-3 last:pb-0">
                <div className="text-sm font-medium text-white">Data cleaning sprint</div>
                <div className="text-xs text-[#A1A1AA] mt-0.5">Freelancer working · 4d left</div>
              </div>
            </div>
          </GlassCard>

          <GlassCard tone="warn">
            <Eyebrow>Awaiting your review (1)</Eyebrow>
            <div className="text-sm font-medium text-white">Prompt eval set</div>
            <div className="text-xs text-[#A1A1AA] mt-0.5 mb-4">Delivered · review now</div>
            <Link href="/gigs/prompt-eval-chatbot">
              <Button size="sm">Review &amp; approve</Button>
            </Link>
          </GlassCard>
        </div>

        <Eyebrow>Quick stats</Eyebrow>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-5">
          {STATS.map((s) => (
            <GlassCard key={s.label} className="text-center">
              <div className="text-[11px] uppercase tracking-widest text-[#A1A1AA] mb-1">{s.label}</div>
              <div className="text-2xl font-bold text-white">{s.value}</div>
            </GlassCard>
          ))}
        </div>
      </div>
    </section>
  );
}
