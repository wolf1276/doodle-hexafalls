import Link from "next/link";
import { Award, Plus } from "lucide-react";
import { DashboardNav } from "@/components/dashboard/dashboard-nav";
import { GlassCard, Eyebrow } from "@/components/ui/glass-card";
import { Pill, Stars } from "@/components/ui/pill";
import { ProgressBar } from "@/components/ui/progress";

export const metadata = { title: "Dashboard | DOODLE" };

const ONGOING = [
  { title: "Label 500 product images", status: "Milestone 1/2 · due in 2d" },
  { title: "Prompt eval set (100)", status: "Awaiting your delivery" },
  { title: "Data annotation sprint", status: "Delivered · 54h auto-release timer" },
];

const COMPLETED = [
  { title: "Sentiment tagging batch", amount: "$60", rating: 5 },
  { title: "Chatbot QA review", amount: "$40", rating: 4 },
  { title: "Image classification", amount: "$75", rating: 5 },
];

export default function FreelancerDashboardPage() {
  return (
    <section className="min-h-[calc(100vh-72px)] bg-[#050505]">
      <div className="border-b border-white/[0.06]">
        <DashboardNav role="freelancer" balance="₹12,450" />
      </div>

      <div className="max-w-[1200px] mx-auto px-6 py-10">
        <h1 className="text-2xl font-bold text-white mb-8">Welcome back, Alex</h1>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-5 mb-6">
          <GlassCard>
            <Eyebrow>Ongoing (3)</Eyebrow>
            <div className="flex flex-col divide-y divide-white/[0.06]">
              {ONGOING.map((t) => (
                <div key={t.title} className="py-3 first:pt-0 last:pb-0">
                  <div className="text-sm font-medium text-white">{t.title}</div>
                  <div className="text-xs text-[#A1A1AA] mt-0.5">{t.status}</div>
                </div>
              ))}
            </div>
          </GlassCard>

          <GlassCard>
            <Eyebrow>Completed (18)</Eyebrow>
            <div className="flex flex-col divide-y divide-white/[0.06]">
              {COMPLETED.map((t) => (
                <div key={t.title} className="py-3 first:pt-0 last:pb-0 flex items-center justify-between">
                  <div className="text-sm font-medium text-white">{t.title}</div>
                  <div className="flex items-center gap-2 text-xs text-[#A1A1AA]">
                    <span>{t.amount}</span>
                    <Stars filled={t.rating} />
                  </div>
                </div>
              ))}
            </div>
          </GlassCard>
        </div>

        <GlassCard id="services" className="mb-6" tone="accent">
          <div className="flex items-center justify-between gap-4 flex-wrap">
            <div className="flex items-center gap-3">
              <div className="h-11 w-11 rounded-full border-2 border-[#FF6FAF] flex items-center justify-center text-[#FF6FAF]">
                <Award size={18} />
              </div>
              <div>
                <div className="text-sm font-semibold text-white">100-pt Verified Pro badge minted</div>
                <div className="text-xs text-[#A1A1AA]">150 / 200 pts to next badge</div>
              </div>
            </div>
            <Link href="/reputation/7xK9pQ" className="text-xs font-semibold text-[#FF6FAF]">
              View public profile →
            </Link>
          </div>
          <div className="mt-4">
            <ProgressBar value={150} max={200} />
          </div>
        </GlassCard>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
          <GlassCard id="portfolio">
            <div className="flex items-center justify-between mb-3">
              <Eyebrow className="mb-0">Portfolio</Eyebrow>
            </div>
            <div className="grid grid-cols-4 gap-2">
              {[1, 2, 3].map((i) => (
                <div key={i} className="aspect-square rounded-lg bg-white/[0.04] border border-white/10" />
              ))}
              <button className="aspect-square rounded-lg border border-dashed border-white/15 flex items-center justify-center text-[#A1A1AA] hover:text-white hover:border-white/30">
                <Plus size={16} />
              </button>
            </div>
          </GlassCard>

          <GlassCard>
            <Eyebrow>Feedback</Eyebrow>
            <div className="flex flex-col gap-3">
              <div>
                <Stars filled={5} />
                <p className="text-sm text-[#D4D4D8] mt-1">
                  &ldquo;Fast, accurate, delivered early.&rdquo; — Client A
                </p>
              </div>
              <div>
                <Stars filled={4} />
                <p className="text-sm text-[#D4D4D8] mt-1">
                  &ldquo;Good work, minor revisions.&rdquo; — Client B
                </p>
              </div>
            </div>
          </GlassCard>
        </div>
      </div>
    </section>
  );
}
