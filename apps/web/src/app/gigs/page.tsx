import Link from "next/link";
import { Search } from "lucide-react";
import { DashboardNav } from "@/components/dashboard/dashboard-nav";
import { Pill } from "@/components/ui/pill";
import { Button } from "@/components/ui/button";

export const metadata = { title: "Browse Jobs | DOODLE" };

const JOBS = [
  { id: "label-product-images", title: "Label product images (500)", budget: "$50", client: "★★★★★ (12 hires)", posted: "2h ago" },
  { id: "prompt-eval-chatbot", title: "Prompt eval set for chatbot", budget: "$80", client: "★★★★☆ (5 hires)", posted: "5h ago" },
  { id: "sentiment-tagging", title: "Sentiment tagging batch", budget: "$35", client: "★★★★★ (30 hires)", posted: "1d ago" },
  { id: "data-cleaning-ml", title: "Data cleaning for ML dataset", budget: "$120", client: "★★★★★ (8 hires)", posted: "1d ago" },
];

export default function BrowseJobsPage() {
  return (
    <section className="min-h-[calc(100vh-72px)] bg-[#050505]">
      <div className="border-b border-white/[0.06]">
        <DashboardNav role="freelancer" balance="₹12,450" />
      </div>

      <div className="max-w-[1200px] mx-auto px-6 py-10">
        <h1 className="text-2xl font-bold text-white mb-6">Browse jobs</h1>

        <div className="flex items-center gap-3 mb-4">
          <div className="flex-1 flex items-center h-11 rounded-full bg-white/[0.04] border border-white/10 px-4">
            <Search size={16} className="text-[#71717A] mr-2" />
            <input
              placeholder="Search jobs..."
              className="flex-1 bg-transparent text-sm text-[#F5F5F5] placeholder:text-[#71717A] outline-none"
            />
          </div>
          <Button size="lg">Search</Button>
        </div>

        <div className="flex items-center gap-2 mb-6">
          <Pill>Category ▾</Pill>
          <Pill>Budget ▾</Pill>
          <Pill>Client rating ▾</Pill>
        </div>

        <div className="rounded-2xl border border-white/10 overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-[10px] uppercase tracking-widest text-[#A1A1AA] bg-white/[0.02]">
                <th className="px-5 py-3 font-medium">Job</th>
                <th className="px-5 py-3 font-medium">Budget</th>
                <th className="px-5 py-3 font-medium">Client</th>
                <th className="px-5 py-3 font-medium">Posted</th>
                <th className="px-5 py-3 font-medium"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/[0.06]">
              {JOBS.map((j) => (
                <tr key={j.id} className="hover:bg-white/[0.02] transition-colors">
                  <td className="px-5 py-4 text-white font-medium">{j.title}</td>
                  <td className="px-5 py-4 text-[#D4D4D8]">{j.budget}</td>
                  <td className="px-5 py-4 text-[#A1A1AA]">{j.client}</td>
                  <td className="px-5 py-4 text-[#A1A1AA]">{j.posted}</td>
                  <td className="px-5 py-4 text-right">
                    <Link href={`/gigs/${j.id}`}>
                      <Button size="sm" variant="ghost" className="border border-white/10">
                        Apply
                      </Button>
                    </Link>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
}
