import Link from "next/link";
import { Search } from "lucide-react";
import { DashboardNav } from "@/components/dashboard/dashboard-nav";
import { GlassCard } from "@/components/ui/glass-card";
import { Pill, Stars } from "@/components/ui/pill";
import { Button } from "@/components/ui/button";

export const metadata = { title: "Browse Talent | DOODLE" };

const TALENT = [
  { id: "7xK9pQ", name: "Freelancer A", rating: 5, points: "150 pts · badge minted", skills: "AI Labeling, Prompt Eng. · 18 gigs" },
  { id: "9mR2vL", name: "Freelancer B", rating: 4, points: "90 pts", skills: "Data Annotation · 9 gigs" },
  { id: "3tY7wZ", name: "Freelancer C", rating: 5, points: "210 pts · badge minted", skills: "Model Evaluation · 25 gigs" },
  { id: "5qN1dX", name: "Freelancer D", rating: 5, points: "300 pts · 2 badges", skills: "AI Labeling · 40 gigs" },
];

export default function BrowseTalentPage() {
  return (
    <section className="min-h-[calc(100vh-72px)] bg-[#050505]">
      <div className="border-b border-white/[0.06]">
        <DashboardNav role="client" balance="₹8,200" />
      </div>

      <div className="max-w-[1200px] mx-auto px-6 py-10">
        <h1 className="text-2xl font-bold text-white mb-6">Browse talent</h1>

        <div className="flex items-center gap-3 mb-4">
          <div className="flex-1 flex items-center h-11 rounded-full bg-white/[0.04] border border-white/10 px-4">
            <Search size={16} className="text-[#71717A] mr-2" />
            <input
              placeholder="Search talent..."
              className="flex-1 bg-transparent text-sm text-[#F5F5F5] placeholder:text-[#71717A] outline-none"
            />
          </div>
          <Button size="lg">Search</Button>
        </div>

        <div className="flex items-center gap-2 mb-6">
          <Pill>Skill ▾</Pill>
          <Pill>Points range ▾</Pill>
          <Pill>Rating ▾</Pill>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-5">
          {TALENT.map((t) => (
            <GlassCard key={t.id}>
              <div className="flex items-center gap-3">
                <div className="h-11 w-11 rounded-full bg-white/[0.06] border border-white/10" />
                <div>
                  <div className="text-sm font-semibold text-white">{t.name}</div>
                  <Stars filled={t.rating} />
                </div>
              </div>
              <p className="text-xs text-[#A1A1AA] mt-3 leading-relaxed">
                {t.points}
                <br />
                {t.skills}
              </p>
              <Link href={`/reputation/${t.id}`}>
                <Button size="sm" variant="ghost" className="border border-white/10 mt-3">
                  View profile
                </Button>
              </Link>
            </GlassCard>
          ))}
        </div>
      </div>
    </section>
  );
}
