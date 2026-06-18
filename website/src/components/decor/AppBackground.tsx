export function AppBackground() {
  return (
    <div className="pointer-events-none fixed inset-0 z-0 overflow-hidden bg-black" aria-hidden="true">
      <div className="absolute inset-0 bg-[linear-gradient(to_right,rgba(255,255,255,0.09)_1px,transparent_1px),linear-gradient(to_bottom,rgba(255,255,255,0.09)_1px,transparent_1px)] bg-[size:64px_64px] [mask-image:radial-gradient(ellipse_at_top,black_48%,transparent_82%)]" />
      <div className="absolute left-1/2 top-[-14rem] h-[46rem] w-[46rem] -translate-x-1/2 rounded-full border border-white/15 bg-white/[0.07] blur-3xl" />
      <div className="absolute left-[8%] top-[18rem] h-72 w-72 rounded-full bg-white/[0.06] blur-3xl" />
      <div className="absolute right-[-8rem] top-[34rem] h-96 w-96 rounded-full border border-white/15 bg-zinc-800/80 blur-3xl" />
      <div className="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-white/40 to-transparent" />
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_0%,transparent,rgba(0,0,0,0.58)_76%)]" />
    </div>
  )
}
