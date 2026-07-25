"use client";

import { motion } from "framer-motion";
import Image from "next/image";
import { CheckCircle2, Star, DollarSign } from "lucide-react";
import { Button } from "@/components/ui/button";

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
      delayChildren: 0.1,
    },
  },
};

const itemVariants = {
  hidden: { y: 20, opacity: 0 },
  visible: {
    y: 0,
    opacity: 1,
    transition: { duration: 0.8, ease: [0.16, 1, 0.3, 1] as const },
  },
};

const floatingAnimation = {
  y: ["-10px", "10px", "-10px"],
  transition: {
    duration: 6,
    repeat: Infinity,
    ease: "easeInOut" as const,
  },
};

const floatDrift1 = {
  y: ["-5px", "15px", "-5px"],
  x: ["-5px", "5px", "-5px"],
  transition: { duration: 7, repeat: Infinity, ease: "easeInOut" as const },
};

const floatDrift2 = {
  y: ["10px", "-10px", "10px"],
  x: ["5px", "-5px", "5px"],
  transition: { duration: 8, repeat: Infinity, ease: "easeInOut" as const, delay: 1 },
};

export function Hero() {
  return (
    <section className="relative min-h-screen w-full flex flex-col justify-center overflow-hidden bg-[#050505]">
      {/* Subtle Background Glow behind right column */}
      <div className="absolute right-[10%] top-1/2 -translate-y-1/2 z-0 pointer-events-none">
        <div className="w-[600px] h-[600px] bg-[#FF6FAF]/15 rounded-full blur-[120px] mix-blend-screen" />
      </div>

      <motion.div
        variants={containerVariants}
        initial="hidden"
        animate="visible"
        className="w-full max-w-[1440px] mx-auto px-8 lg:px-16 relative z-10 flex flex-col lg:flex-row items-center justify-between h-full min-h-screen pt-[72px]"
      >
        {/* LEFT COLUMN - 45% */}
        <div className="w-full lg:w-[45%] flex flex-col justify-center z-20 pb-12 lg:pb-0 pt-12 lg:pt-0">
          <motion.div variants={itemVariants} className="flex flex-col gap-6">
            
            {/* Small Badge */}
            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-white/[0.08] bg-white/[0.02] backdrop-blur-md w-fit">
              <span className="text-yellow-400 text-xs">⚡</span>
              <span className="text-[11px] font-semibold text-white/80 uppercase tracking-widest">
                Powered by Solana
              </span>
            </div>

            {/* Large Heading */}
            <h1 
              className="text-white font-extrabold italic tracking-tight"
              style={{ fontSize: "clamp(48px, 6vw, 84px)", lineHeight: 1.05 }}
            >
              Hire world-class freelancers. <br className="hidden lg:block" />
              <span className="text-white/60">Protected by on-chain escrow.</span>
            </h1>

            {/* Paragraph */}
            <p className="text-[17px] text-white/60 max-w-[560px] leading-relaxed">
              Milestone-based escrow. Instant payouts. Portable reputation. Hire confidently without payment disputes.
            </p>

            {/* Buttons */}
            <div className="flex items-center gap-4 pt-2">
              <Button size="lg" className="bg-white text-black hover:bg-white/90 h-12 px-6 text-sm font-semibold rounded-full shadow-[0_0_40px_rgba(255,255,255,0.1)]">
                Start Hiring <span className="ml-1">→</span>
              </Button>
              <Button size="lg" variant="ghost" className="h-12 px-6 text-sm font-semibold border border-white/10 hover:bg-white/5 rounded-full">
                Browse Talent
              </Button>
            </div>

            {/* Stats */}
            <div className="flex items-center gap-8 pt-12 border-t border-white/[0.08] mt-4">
              <div className="flex flex-col gap-1">
                <span className="text-xl font-bold text-white">12K+</span>
                <span className="text-[11px] text-white/50 uppercase tracking-wider font-semibold">Projects Completed</span>
              </div>
              <div className="flex flex-col gap-1">
                <span className="text-xl font-bold text-white">95%</span>
                <span className="text-[11px] text-white/50 uppercase tracking-wider font-semibold">Successful Deliveries</span>
              </div>
              <div className="flex flex-col gap-1">
                <span className="text-xl font-bold text-white">0.5%</span>
                <span className="text-[11px] text-white/50 uppercase tracking-wider font-semibold">Platform Fee</span>
              </div>
              <div className="flex flex-col gap-1">
                <span className="text-xl font-bold text-white">2 sec</span>
                <span className="text-[11px] text-white/50 uppercase tracking-wider font-semibold">Average Payout</span>
              </div>
            </div>

          </motion.div>
        </div>

        {/* RIGHT COLUMN - 55% */}
        <div className="w-full lg:w-[55%] relative flex items-center justify-center min-h-[500px] lg:min-h-0">
          
          {/* Background Typography */}
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 0.08, scale: 1 }}
            transition={{ duration: 1.5, ease: "easeOut" }}
            className="absolute z-0 select-none pointer-events-none"
          >
            <h2 
              className="text-white font-black italic tracking-tighter"
              style={{ fontSize: "clamp(120px, 18vw, 260px)", lineHeight: 0.8 }}
            >
              ESCROW
            </h2>
          </motion.div>

          {/* Laptop Image */}
          <motion.div
            animate={floatingAnimation}
            className="relative z-10 w-full max-w-[700px] aspect-[16/10] drop-shadow-[0_40px_80px_rgba(0,0,0,0.8)]"
          >
            <Image
              src="/images/laptop.png"
              alt="DOODLE Marketplace"
              fill
              className="object-contain"
              priority
            />

            {/* Floating UI Cards */}
            <motion.div
              animate={floatDrift1}
              className="absolute top-[15%] -left-[5%] bg-black/40 backdrop-blur-xl border border-white/10 rounded-2xl p-3 flex items-center gap-3 shadow-2xl"
            >
              <div className="w-7 h-7 rounded-full bg-white/10 flex items-center justify-center">
                <CheckCircle2 className="text-white w-4 h-4" />
              </div>
              <span className="text-[13px] font-medium text-white pr-2">Escrow Locked</span>
            </motion.div>

            <motion.div
              animate={floatDrift2}
              className="absolute top-[50%] -right-[10%] bg-black/40 backdrop-blur-xl border border-white/10 rounded-2xl p-3 flex items-center gap-3 shadow-2xl"
            >
              <div className="flex items-center gap-1">
                {[1, 2, 3, 4, 5].map((i) => (
                  <Star key={i} className="text-white w-3 h-3 fill-white" />
                ))}
              </div>
              <span className="text-[13px] font-medium text-white pr-2">Client Approved</span>
            </motion.div>

            <motion.div
              animate={floatDrift1}
              style={{ animationDelay: "2s" }}
              className="absolute bottom-[20%] left-[5%] bg-black/40 backdrop-blur-xl border border-white/10 rounded-2xl p-3 flex items-center gap-3 shadow-2xl"
            >
              <div className="w-7 h-7 rounded-full bg-white/10 flex items-center justify-center">
                <DollarSign className="text-white w-4 h-4" />
              </div>
              <span className="text-[13px] font-medium text-white pr-2">USDC Released</span>
            </motion.div>

          </motion.div>

        </div>
      </motion.div>
    </section>
  );
}
