import { GlassCard, Eyebrow } from "@/components/ui/glass-card";
import { Pill } from "@/components/ui/pill";
import { ProgressBar } from "@/components/ui/progress";

export const metadata = { title: "Profile | DOODLE" };

export default async function ProfilePage({
  params,
}: {
  params: Promise<{ wallet: string }>;
}) {
  const { wallet } = await params;

  return (
    <section className="min-h-[calc(100vh-72px)] bg-[#050505] px-6 py-10">
      <div className="max-w-[900px] mx-auto grid grid-cols-1 sm:grid-cols-[200px_1fr] gap-8">
        <div>
          <div className="h-[110px] w-[110px] rounded-full bg-white/[0.06] border border-white/10 mb-3" />
          <div className="text-lg font-semibold text-white">Alex Rivera</div>
          <Pill className="mt-2">Wallet: {wallet.slice(0, 4)}...{wallet.slice(-4)}</Pill>
        </div>

        <div>
          <div className="grid grid-cols-3 gap-4 mb-5">
            <GlassCard className="text-center">
              <Eyebrow>Points</Eyebrow>
              <div className="text-xl font-bold text-white">150</div>
            </GlassCard>
            <GlassCard className="text-center">
              <Eyebrow>Gigs done</Eyebrow>
              <div className="text-xl font-bold text-white">18</div>
            </GlassCard>
            <GlassCard className="text-center">
              <Eyebrow>Badges</Eyebrow>
              <div className="text-xl font-bold text-white">1</div>
            </GlassCard>
          </div>

          <GlassCard tone="accent">
            <Eyebrow>Progress</Eyebrow>
            <ProgressBar value={150} max={200} />
            <p className="text-xs text-[#A1A1AA] mt-2">150 / 200 pts to next badge</p>
          </GlassCard>
        </div>
      </div>
    </section>
  );
}
