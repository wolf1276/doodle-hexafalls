"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { CheckCircle2 } from "lucide-react";
import { GlassCard, Eyebrow } from "@/components/ui/glass-card";
import { Button } from "@/components/ui/button";
import { use } from "react";

const STEPS = ["Confirm milestone", "Sign in", "Locked"];

export default function HireFlowPage({
  params,
}: {
  params: Promise<{ gigId: string }>;
}) {
  const { gigId } = use(params);
  const [step, setStep] = useState(0);
  const router = useRouter();

  return (
    <section className="min-h-[calc(100vh-72px)] bg-[#050505] px-6 py-16 flex flex-col items-center">
      <h1 className="text-xl font-bold text-white mb-8">Hire &amp; fund escrow</h1>

      <div className="w-full max-w-[820px] grid grid-cols-1 sm:grid-cols-3 gap-5 mb-2">
        {STEPS.map((label, i) => (
          <div key={label} className="text-center text-[11px] uppercase tracking-widest text-[#A1A1AA]">
            <span className={i <= step ? "text-[#FF6FAF]" : ""}>Step {i + 1}</span> · {label}
          </div>
        ))}
      </div>

      <div className="w-full max-w-[820px] grid grid-cols-1 sm:grid-cols-3 gap-5">
        <GlassCard tone={step >= 0 ? "accent" : "default"}>
          <Eyebrow>Step 1</Eyebrow>
          <p className="text-sm text-[#D4D4D8] mb-4">Confirm milestone 1: $20</p>
          {step === 0 && (
            <Button size="sm" onClick={() => setStep(1)}>
              Continue
            </Button>
          )}
        </GlassCard>

        <GlassCard tone={step >= 1 ? "accent" : "default"}>
          <Eyebrow>Step 2 — Privy</Eyebrow>
          <p className="text-sm text-[#D4D4D8] mb-4">Signed in as you@email.com — no wallet popup</p>
          {step === 1 && (
            <Button size="sm" onClick={() => setStep(2)}>
              Confirm payment
            </Button>
          )}
        </GlassCard>

        <GlassCard tone={step >= 2 ? "good" : "default"}>
          <Eyebrow>Step 3</Eyebrow>
          {step >= 2 ? (
            <div className="flex items-center gap-2 text-sm text-emerald-300">
              <CheckCircle2 size={16} /> $20 locked in escrow
            </div>
          ) : (
            <p className="text-sm text-[#71717A]">Waiting…</p>
          )}
        </GlassCard>
      </div>

      {step >= 2 && (
        <Button className="mt-8" onClick={() => router.push(`/gigs/${gigId}`)}>
          Back to gig
        </Button>
      )}
    </section>
  );
}
