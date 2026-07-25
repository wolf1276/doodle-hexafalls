import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";

export default function Home() {
  return (
    <div className="relative">
      <Image
        src="/images/hero-banner.png"
        alt="DOODLE"
        width={1600}
        height={900}
        className="w-full h-auto"
        priority
      />

      <div className="absolute inset-0 flex flex-col items-center justify-end pb-16 gap-4">
        <div className="flex items-center gap-4">
          <Link href="/role">
            <Button size="lg" className="rounded-full px-8">
              Start Hiring →
            </Button>
          </Link>
          <Link href="/talent">
            <Button size="lg" variant="ghost" className="rounded-full px-8 border border-white/10">
              Browse Talent
            </Button>
          </Link>
        </div>
      </div>
    </div>
  );
}
