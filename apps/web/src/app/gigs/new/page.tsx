"use client";

import { useState } from "react";
import { GlassCard, Eyebrow } from "@/components/ui/glass-card";
import { Button } from "@/components/ui/button";

export default function PostGigPage() {
  const [milestones, setMilestones] = useState([{ name: "", amount: "" }]);

  return (
    <section className="min-h-[calc(100vh-72px)] bg-[#050505] px-6 py-10">
      <div className="max-w-[620px] mx-auto">
        <h1 className="text-2xl font-bold text-white mb-6">Post a gig</h1>

        <GlassCard>
          <div className="flex flex-col gap-5">
            <div>
              <label className="text-xs font-medium text-[#A1A1AA] mb-1.5 block">Title</label>
              <input
                placeholder="e.g. Label 500 product images"
                className="w-full h-11 rounded-xl bg-white/[0.04] border border-white/10 px-4 text-sm text-white placeholder:text-[#71717A] outline-none focus:border-[#FF6FAF]/50"
              />
            </div>

            <div>
              <label className="text-xs font-medium text-[#A1A1AA] mb-1.5 block">Description</label>
              <textarea
                placeholder="Describe the task..."
                rows={4}
                className="w-full rounded-xl bg-white/[0.04] border border-white/10 px-4 py-3 text-sm text-white placeholder:text-[#71717A] outline-none focus:border-[#FF6FAF]/50 resize-none"
              />
            </div>

            <div>
              <Eyebrow>Milestones</Eyebrow>
              <div className="flex flex-col gap-2">
                {milestones.map((_, i) => (
                  <div key={i} className="flex gap-2">
                    <input
                      placeholder="Milestone name"
                      className="flex-[2] h-10 rounded-xl bg-white/[0.04] border border-white/10 px-3 text-sm text-white placeholder:text-[#71717A] outline-none focus:border-[#FF6FAF]/50"
                    />
                    <input
                      placeholder="$ amount"
                      className="flex-1 h-10 rounded-xl bg-white/[0.04] border border-white/10 px-3 text-sm text-white placeholder:text-[#71717A] outline-none focus:border-[#FF6FAF]/50"
                    />
                  </div>
                ))}
              </div>
              <button
                onClick={() => setMilestones((m) => [...m, { name: "", amount: "" }])}
                className="text-xs font-semibold text-[#FF6FAF] mt-2"
              >
                + Add milestone
              </button>
            </div>

            <Button className="w-full mt-2" size="lg">
              Publish gig
            </Button>
          </div>
        </GlassCard>
      </div>
    </section>
  );
}
