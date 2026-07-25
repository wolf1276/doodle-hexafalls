import Link from "next/link";
import { GlassCard, Eyebrow } from "@/components/ui/glass-card";
import { Stars } from "@/components/ui/pill";
import { Button } from "@/components/ui/button";

export const metadata = { title: "Gig detail | DOODLE" };

const MILESTONES = [
  { name: "Sample batch", amount: "$20" },
  { name: "Full delivery", amount: "$130" },
];

export default async function GigDetailPage({
  params,
}: {
  params: Promise<{ gigId: string }>;
}) {
  const { gigId } = await params;
  const title = gigId
    .split("-")
    .map((w) => w[0]?.toUpperCase() + w.slice(1))
    .join(" ");

  return (
    <section className="min-h-[calc(100vh-72px)] bg-[#050505] px-6 py-10">
      <div className="max-w-[1000px] mx-auto grid grid-cols-1 lg:grid-cols-[1.5fr_260px] gap-8">
        <div>
          <div className="aspect-[16/7] rounded-2xl bg-white/[0.04] border border-white/10 mb-6" />
          <h1 className="text-2xl font-bold text-white mb-3">{title}</h1>
          <p className="text-sm text-[#A1A1AA] leading-relaxed max-w-[560px]">
            Deliver a clean, accurately labeled dataset in the required format. Full spec and
            reference examples are shared after you accept the milestone. Payment releases the
            moment each milestone is approved — no waiting on invoices.
          </p>

          <GlassCard className="mt-6">
            <Eyebrow>Milestones</Eyebrow>
            <div className="flex flex-col divide-y divide-white/[0.06]">
              {MILESTONES.map((m, i) => (
                <div key={m.name} className="flex items-center justify-between py-3 first:pt-0 last:pb-0">
                  <span className="text-sm text-[#D4D4D8]">
                    {i + 1}. {m.name}
                  </span>
                  <span className="text-sm font-semibold text-white">{m.amount}</span>
                </div>
              ))}
            </div>
          </GlassCard>
        </div>

        <div>
          <GlassCard tone="accent">
            <Eyebrow>Posted by</Eyebrow>
            <div className="flex items-center gap-3 mb-4">
              <div className="h-11 w-11 rounded-full bg-white/[0.06] border border-white/10" />
              <div>
                <div className="text-sm font-semibold text-white">Client A</div>
                <Stars filled={5} />
              </div>
            </div>
            <Link href={`/gigs/${gigId}/hire`}>
              <Button className="w-full">Hire &amp; fund milestone 1</Button>
            </Link>
          </GlassCard>
        </div>
      </div>
    </section>
  );
}
