import { DashboardNav } from "@/components/dashboard/dashboard-nav";
import { Pill } from "@/components/ui/pill";

export const metadata = { title: "Transactions | DOODLE" };

const ROWS: { date: string; type: string; freelancer: string; amount: string; status: string; tone: "default" | "good" | "accent" | "warn" }[] = [
  { date: "Jul 20", type: "Milestone funded", freelancer: "Freelancer A", amount: "-$20", status: "Locked", tone: "default" },
  { date: "Jul 18", type: "Milestone released", freelancer: "Freelancer C", amount: "-$75", status: "Paid", tone: "good" },
  { date: "Jul 15", type: "Partial timeout", freelancer: "Freelancer B", amount: "-$16", status: "Auto", tone: "warn" },
  { date: "Jul 10", type: "Dispute resolved", freelancer: "Freelancer D", amount: "-$40", status: "Jury", tone: "accent" },
  { date: "Jul 05", type: "On-ramp", freelancer: "—", amount: "+₹5,000", status: "Added", tone: "good" },
];

export default function TransactionsPage() {
  return (
    <section className="min-h-[calc(100vh-72px)] bg-[#050505]">
      <div className="border-b border-white/[0.06]">
        <DashboardNav role="client" balance="₹8,200" />
      </div>

      <div className="max-w-[1200px] mx-auto px-6 py-10">
        <h1 className="text-2xl font-bold text-white mb-2">Transactions</h1>
        <p className="text-sm text-[#A1A1AA] mb-6">
          Each row is a human-readable label over an on-chain event — a filtered view of your wallet&apos;s history.
        </p>

        <div className="rounded-2xl border border-white/10 overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-[10px] uppercase tracking-widest text-[#A1A1AA] bg-white/[0.02]">
                <th className="px-5 py-3 font-medium">Date</th>
                <th className="px-5 py-3 font-medium">Type</th>
                <th className="px-5 py-3 font-medium">Freelancer</th>
                <th className="px-5 py-3 font-medium">Amount</th>
                <th className="px-5 py-3 font-medium">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/[0.06]">
              {ROWS.map((r, i) => (
                <tr key={i} className="hover:bg-white/[0.02] transition-colors">
                  <td className="px-5 py-4 text-[#A1A1AA]">{r.date}</td>
                  <td className="px-5 py-4 text-white font-medium">{r.type}</td>
                  <td className="px-5 py-4 text-[#A1A1AA]">{r.freelancer}</td>
                  <td className="px-5 py-4 text-[#D4D4D8]">{r.amount}</td>
                  <td className="px-5 py-4">
                    <Pill tone={r.tone}>{r.status}</Pill>
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
