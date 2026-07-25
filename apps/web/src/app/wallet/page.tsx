import { Plus, ArrowDownToLine, Activity } from "lucide-react";
import { GlassCard, Eyebrow } from "@/components/ui/glass-card";
import { Button } from "@/components/ui/button";

export const metadata = { title: "Wallet | DOODLE" };

export default function WalletPage() {
  return (
    <section className="min-h-[calc(100vh-72px)] bg-[#050505] px-6 py-16 flex justify-center">
      <GlassCard className="w-full max-w-[380px]">
        <Eyebrow>Wallet</Eyebrow>
        <div className="text-sm text-[#A1A1AA] mb-1">Balance</div>
        <div className="text-2xl font-bold text-white mb-6">₹12,450 <span className="text-sm text-[#71717A] font-normal">(≈ $148)</span></div>

        <div className="flex flex-col gap-2.5">
          <Button className="w-full">
            <Plus size={16} className="mr-2" /> Add funds (on-ramp)
          </Button>
          <Button variant="ghost" className="w-full border border-white/10">
            <ArrowDownToLine size={16} className="mr-2" /> Cash out (off-ramp)
          </Button>
          <Button variant="ghost" className="w-full border border-white/10">
            <Activity size={16} className="mr-2" /> View wallet activity
          </Button>
        </div>
      </GlassCard>
    </section>
  );
}
