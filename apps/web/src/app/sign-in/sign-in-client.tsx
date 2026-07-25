"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { Mail } from "lucide-react";
import { useLogin, useLoginWithOAuth, usePrivy } from "@privy-io/react-auth";
import { GlassCard } from "@/components/ui/glass-card";
import { Button } from "@/components/ui/button";
import { Pill } from "@/components/ui/pill";

export default function SignInClient({ role }: { role: "freelancer" | "client" }) {
  const isFreelancer = role === "freelancer";
  const dest = isFreelancer ? "/dashboard" : "/client";
  const router = useRouter();
  const { ready, authenticated } = usePrivy();
  const [pending, setPending] = useState<"email" | "google" | null>(null);

  const { login } = useLogin({
    onComplete: () => router.push(dest),
    onError: () => setPending(null),
  });

  const { initOAuth } = useLoginWithOAuth({
    onComplete: () => router.push(dest),
    onError: () => setPending(null),
  });

  useEffect(() => {
    if (ready && authenticated) router.replace(dest);
  }, [ready, authenticated, dest, router]);

  return (
    <section className="min-h-[calc(100vh-72px)] flex flex-col items-center justify-center px-6 py-24 bg-[#050505]">
      <GlassCard className="w-full max-w-[380px] text-center py-8">
        <Pill tone="accent" className="mx-auto mb-5">Secured by Privy</Pill>
        <h1 className="text-xl font-semibold text-white mb-1">
          Sign in to start {isFreelancer ? "working" : "hiring"}
        </h1>
        <p className="text-sm text-[#A1A1AA] mb-6">
          {isFreelancer
            ? "Browse gigs and get paid the moment work is approved."
            : "Post gigs and pay through protected on-chain escrow."}
        </p>

        <div className="flex flex-col gap-2.5">
          <Button
            className="w-full"
            size="lg"
            disabled={!ready || pending !== null}
            onClick={() => {
              setPending("email");
              login({ loginMethods: ["email"] });
            }}
          >
            <Mail size={16} className="mr-2" />
            {pending === "email" ? "Waiting for confirmation…" : "Continue with Email"}
          </Button>
          <Button
            variant="ghost"
            className="w-full border border-white/10"
            size="lg"
            disabled={!ready || pending !== null}
            onClick={() => {
              setPending("google");
              initOAuth({ provider: "google" });
            }}
          >
            {pending === "google" ? "Redirecting…" : "Continue with Google"}
          </Button>
        </div>

        <p className="text-[11px] text-[#71717A] mt-5">
          No wallet needed — we set one up for you automatically.
        </p>

        <Link
          href="/role"
          className="inline-block mt-6 text-[11px] text-[#A1A1AA] hover:text-white border border-white/10 rounded-full px-3 py-1"
        >
          ← Back to role select
        </Link>
      </GlassCard>
    </section>
  );
}
