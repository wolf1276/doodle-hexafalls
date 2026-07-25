import { GlassCard, Eyebrow } from "@/components/ui/glass-card";
import { Pill } from "@/components/ui/pill";
import { Button } from "@/components/ui/button";

export const metadata = { title: "Dispute | DOODLE" };

export default async function DisputePage({
  params,
}: {
  params: Promise<{ caseId: string }>;
}) {
  const { caseId } = await params;

  return (
    <section className="min-h-[calc(100vh-72px)] bg-[#050505] px-6 py-10">
      <div className="max-w-[1000px] mx-auto">
        <div className="flex items-center gap-3 mb-6">
          <h1 className="text-2xl font-bold text-white">Dispute &amp; jury voting</h1>
          <Pill tone="warn">Case #{caseId}</Pill>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-[1fr_240px] gap-6">
          <div className="flex flex-col gap-5">
            <GlassCard>
              <Eyebrow>Case summary</Eyebrow>
              <p className="text-sm text-[#D4D4D8] leading-relaxed">
                Freelancer delivered milestone 2 early, but the client says the labels don&apos;t match
                the spec on ~15% of the batch. Freelancer disputes the client&apos;s assessment.
              </p>
            </GlassCard>
            <GlassCard>
              <Eyebrow>Evidence</Eyebrow>
              <div className="flex gap-3">
                <div className="h-[50px] w-[70px] rounded-lg bg-white/[0.04] border border-white/10" />
                <div className="h-[50px] w-[70px] rounded-lg bg-white/[0.04] border border-white/10" />
              </div>
            </GlassCard>
          </div>

          <GlassCard tone="accent">
            <Eyebrow>Jury (3 wallets)</Eyebrow>
            <div className="text-sm text-[#D4D4D8] leading-relaxed mb-4">
              Juror 1: freelancer
              <br />
              Juror 2: freelancer
              <br />
              Juror 3: pending
            </div>
            <Button className="w-full">Resolve case</Button>
          </GlassCard>
        </div>
      </div>
    </section>
  );
}
