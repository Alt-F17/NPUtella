export function AppBackground() {
  return (
    <div className="pointer-events-none fixed inset-0 -z-10 overflow-hidden bg-black" aria-hidden="true">
      <div className="absolute inset-0 bg-[linear-gradient(to_right,rgba(255,255,255,0.055)_1px,transparent_1px),linear-gradient(to_bottom,rgba(255,255,255,0.055)_1px,transparent_1px)] bg-[size:72px_72px] [mask-image:radial-gradient(ellipse_at_top,black_35%,transparent_78%)]" />
      <div className="absolute left-1/2 top-[-18rem] h-[42rem] w-[42rem] -translate-x-1/2 rounded-full border border-white/10 bg-white/[0.035] blur-3xl" />
      <div className="absolute left-[12%] top-[18rem] h-64 w-64 rounded-full bg-white/[0.035] blur-3xl" />
      <div className="absolute right-[-10rem] top-[34rem] h-96 w-96 rounded-full border border-white/10 bg-zinc-900/70 blur-3xl" />
      <div className="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-white/40 to-transparent" />
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_0%,transparent,rgba(0,0,0,0.78)_68%)]" />
    </div>
  )
}
