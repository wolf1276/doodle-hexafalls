import { DashboardNav } from "@/components/dashboard/dashboard-nav";
import { GlassCard } from "@/components/ui/glass-card";
import { Stars } from "@/components/ui/pill";
import { Button } from "@/components/ui/button";

export const metadata = { title: "Services Taken | DOODLE" };

const SERVICES = [
  { title: "Label product images", freelancer: "Freelancer A", date: "Jul 18", amount: "$75", note: "You rated", rating: 5, tone: "default" as const },
  { title: "Prompt eval set", freelancer: "Freelancer B", date: "Jul 15", amount: "$40", note: "Auto-released (20%)", rating: 0, tone: "default" as const },
  { title: "Data cleaning batch", freelancer: "Freelancer D", date: "Jul 10", amount: "$40", note: "Disputed → Resolved", rating: 0, tone: "bad" as const },
];

export default function ServicesTakenPage() {
  return (
    <section className="min-h-[calc(100vh-72px)] bg-[#050505]">
      <div className="border-b border-white/[0.06]">
        <DashboardNav role="client" balance="₹8,200" />
      </div>

      <div className="max-w-[1200px] mx-auto px-6 py-10">
        <h1 className="text-2xl font-bold text-white mb-6">Services taken</h1>

        <div className="flex flex-col gap-4">
          {SERVICES.map((s) => (
            <GlassCard key={s.title} tone={s.tone} className="flex items-center justify-between gap-4 flex-wrap">
              <div>
                <div className="text-sm font-semibold text-white">{s.title}</div>
                <div className="text-xs text-[#A1A1AA] mt-1">
                  {s.freelancer} · Completed {s.date} · {s.amount} · {s.note}
                  {s.rating > 0 && (
                    <span className="ml-1 align-middle">
                      <Stars filled={s.rating} />
                    </span>
                  )}
                </div>
              </div>
              <Button size="sm" variant="ghost" className="border border-white/10">
                View details
              </Button>
            </GlassCard>
          ))}
        </div>
      </div>
    </section>
  );
}
