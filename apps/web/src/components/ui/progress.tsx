export function ProgressBar({ value, max = 100 }: { value: number; max?: number }) {
  const pct = Math.min(100, Math.round((value / max) * 100));
  return (
    <div className="h-2 w-full rounded-full bg-white/[0.06] overflow-hidden">
      <div
        className="h-full rounded-full bg-gradient-to-r from-[#FF6FAF] to-[#FF9FCB]"
        style={{ width: `${pct}%` }}
      />
    </div>
  );
}
