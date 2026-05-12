"""
Floating island overlay - WisperFlow-style dark pill with live audio bars.
Idles as a small dot, grows when active, shrinks back after done.
"""

import tkinter as tk
import threading
import time
import math
import numpy as np

BG          = "#0a0a0a"
BG_IDLE     = "#0d0d0d"
FG          = "#ffffff"
FG_DIM      = "#888888"
RED         = "#ff3b30"
GREEN       = "#30d158"
ORANGE      = "#ff9f0a"

# sizes
IDLE_W      = 70
IDLE_H      = 28
ACTIVE_W    = 240
ACTIVE_H    = 52
CORNER      = 14

BAR_COUNT   = 20
BAR_W       = 3
BAR_GAP     = 2
BAR_MAX_H   = 26
BAR_MIN_H   = 3

SCREEN_BOTTOM_OFFSET = 120


class Overlay:
    def __init__(self):
        self._root = None
        self._canvas = None
        self._state = "idle"
        self._click_callback = None

        # animated dimensions
        self._w = float(IDLE_W)
        self._h = float(IDLE_H)
        self._w_target = float(IDLE_W)
        self._h_target = float(IDLE_H)

        self._alpha = 0.85
        self._bar_heights = [BAR_MIN_H] * BAR_COUNT
        self._bar_targets  = [BAR_MIN_H] * BAR_COUNT
        self._text = ""
        self._ready = threading.Event()
        threading.Thread(target=self._run, daemon=True).start()
        self._ready.wait(timeout=2.0)

    def _on_click(self):
        if self._click_callback:
            self._click_callback()

    def set_click_callback(self, fn):
        self._click_callback = fn

    def _run(self):
        self._root = tk.Tk()
        self._root.overrideredirect(True)
        self._root.attributes("-topmost", True)
        self._root.attributes("-alpha", self._alpha)
        self._root.configure(bg="#000001")
        self._root.attributes("-transparentcolor", "#000001")
        self._canvas = tk.Canvas(
            self._root, width=ACTIVE_W, height=ACTIVE_H,
            bg="#000001", highlightthickness=0
        )
        self._canvas.pack()
        self._canvas.bind("<Button-1>", lambda e: self._on_click())
        self._canvas.bind("<Enter>", lambda e: self._on_hover(True))
        self._canvas.bind("<Leave>", lambda e: self._on_hover(False))
        self._position(IDLE_W, IDLE_H)
        self._ready.set()
        self._root.after(16, self._tick)
        self._root.mainloop()

    def _position(self, w, h):
        sw = self._root.winfo_screenwidth()
        sh = self._root.winfo_screenheight()
        x = (sw - w) // 2
        y = sh - h - SCREEN_BOTTOM_OFFSET
        self._root.geometry(f"{int(w)}x{int(h)}+{int(x)}+{int(y)}")

    def _draw_pill(self, w, h, bg):
        self._canvas.config(width=w, height=h)
        self._canvas.delete("all")
        r = min(CORNER, h // 2)
        # pill shape
        self._canvas.create_arc(0, 0, r*2, r*2, start=90, extent=90, fill=bg, outline=bg)
        self._canvas.create_arc(w-r*2, 0, w, r*2, start=0, extent=90, fill=bg, outline=bg)
        self._canvas.create_arc(0, h-r*2, r*2, h, start=180, extent=90, fill=bg, outline=bg)
        self._canvas.create_arc(w-r*2, h-r*2, w, h, start=270, extent=90, fill=bg, outline=bg)
        self._canvas.create_rectangle(r, 0, w-r, h, fill=bg, outline=bg)
        self._canvas.create_rectangle(0, r, w, h-r, fill=bg, outline=bg)

    def _tick(self):
        if not self._root:
            return

        # lerp dimensions
        spd = 0.22
        self._w += (self._w_target - self._w) * spd
        self._h += (self._h_target - self._h) * spd
        w = max(IDLE_W, int(self._w))
        h = max(IDLE_H, int(self._h))

        self._render(w, h)
        self._position(w, h)
        self._root.after(16, self._tick)

    def _render(self, w, h):
        state = self._state
        progress = (self._w - IDLE_W) / max(1, ACTIVE_W - IDLE_W)  # 0..1

        bg = BG if progress > 0.1 else BG_IDLE
        self._draw_pill(w, h, bg)
        cx = w // 2
        cy = h // 2

        if state == "idle" or progress < 0.15:
            # small idle dot
            dot_r = 5
            self._canvas.create_oval(cx-dot_r, cy-dot_r, cx+dot_r, cy+dot_r,
                                      fill="#444444", outline="")
            return

        if state == "recording":
            self._render_audio_bars(cx, cy, w, progress)

        elif state == "transcribing":
            self._render_spinner(cx, cy, progress)

        elif state in ("done", "ready"):
            self._render_done(cx, cy, progress)

        elif state == "loading":
            self._render_loading(cx, cy)

        elif state == "error":
            self._render_error(cx, cy)

    def _render_audio_bars(self, cx, cy, w, progress):
        for i in range(BAR_COUNT):
            self._bar_heights[i] += (self._bar_targets[i] - self._bar_heights[i]) * 0.3

        visible_bars = max(1, int(BAR_COUNT * min(1.0, progress * 1.5)))
        total_w = visible_bars * BAR_W + (visible_bars - 1) * BAR_GAP
        x0 = cx - total_w // 2

        for i in range(visible_bars):
            h_bar = self._bar_heights[i]
            x = x0 + i * (BAR_W + BAR_GAP)
            dist = abs(i - visible_bars / 2) / max(1, visible_bars / 2)
            intensity = 1.0 - dist * 0.35
            r_val = int(255 * intensity)
            g_val = int(59 * intensity * 0.6)
            b_val = int(48 * intensity * 0.4)
            color = f"#{r_val:02x}{g_val:02x}{b_val:02x}"
            y1 = cy - h_bar // 2
            y2 = cy + h_bar // 2
            self._canvas.create_rectangle(x, y1, x + BAR_W, y2, fill=color, outline="")

        # mic dot
        self._canvas.create_oval(12, cy-5, 22, cy+5, fill=RED, outline="")

    def _render_spinner(self, cx, cy, progress):
        t = time.time()
        n = 8
        radius = 22
        for i in range(n):
            angle = (i / n) * 2 * math.pi - t * 3
            sx = cx + radius * math.cos(angle) * 0.6
            sy = cy + 5 * math.sin(angle)
            a = (math.sin(t * 5 + i) + 1) / 2
            gray = int(60 + 160 * a)
            color = f"#{gray:02x}{gray:02x}{gray:02x}"
            self._canvas.create_oval(sx-2.5, sy-2.5, sx+2.5, sy+2.5, fill=color, outline="")

    def _render_done(self, cx, cy, progress):
        short = self._text[:26] + "…" if len(self._text) > 26 else self._text
        self._canvas.create_oval(12, cy-5, 22, cy+5, fill=GREEN, outline="")
        self._canvas.create_text(cx + 8, cy, text=short or "ready",
                                  fill=FG, font=("Segoe UI", 10), anchor="center")

    def _render_loading(self, cx, cy):
        t = time.time()
        for i in range(3):
            phase = math.sin(t * 3 + i * 1.2)
            y_off = int(phase * 4)
            gray = int(100 + 100 * (phase + 1) / 2)
            color = f"#{gray:02x}{gray:02x}{gray:02x}"
            x = cx - 16 + i * 16
            self._canvas.create_oval(x-4, cy-4+y_off, x+4, cy+4+y_off, fill=color, outline="")
        self._canvas.create_text(cx + 30, cy, text="loading…",
                                  fill=FG_DIM, font=("Segoe UI", 9), anchor="w")

    def _render_error(self, cx, cy):
        self._canvas.create_text(cx, cy, text=f"⚠ {self._text}",
                                  fill=ORANGE, font=("Segoe UI", 10), anchor="center")

    def push_level(self, rms: float):
        """Called by AudioRecorder on every block — drives the bar visualiser."""
        for i in range(BAR_COUNT):
            sensitivity = 0.8 + 0.4 * math.sin(i * 1.3)
            val = min(1.0, rms * sensitivity * 80)
            h = BAR_MIN_H + int((BAR_MAX_H - BAR_MIN_H) * val)
            h += int(math.sin(time.time() * 14 + i * 0.9) * 2)
            self._bar_targets[i] = max(BAR_MIN_H, min(BAR_MAX_H, h))

    def _reset_bars(self):
        self._bar_targets = [BAR_MIN_H] * BAR_COUNT

    # ── public API ───────────────────────────────────────────────────────────

    def set_recording(self):
        def _go():
            self._state = "recording"
            self._w_target = float(ACTIVE_W)
            self._h_target = float(ACTIVE_H)
        if self._root: self._root.after(0, _go)

    def set_transcribing(self):
        def _go():
            self._reset_bars()
            self._state = "transcribing"
            self._w_target = float(ACTIVE_W)
            self._h_target = float(ACTIVE_H)
        if self._root: self._root.after(0, _go)

    def set_done(self, text: str = ""):
        def _go():
            self._text = text
            self._state = "done"
            self._w_target = float(ACTIVE_W)
            self._h_target = float(ACTIVE_H)
            self._root.after(500, self._shrink_to_idle)
        if self._root: self._root.after(0, _go)

    def _on_hover(self, hovering: bool):
        if self._state != "idle":
            return
        self._w_target = float(IDLE_W + 20) if hovering else float(IDLE_W)
        self._h_target = float(IDLE_H + 10) if hovering else float(IDLE_H)
        self._alpha = 1.0 if hovering else 0.85
        if self._root:
            self._root.attributes("-alpha", self._alpha)

    def set_loading(self):
        def _go():
            self._state = "loading"
            self._w_target = float(ACTIVE_W)
            self._h_target = float(ACTIVE_H)
        if self._root: self._root.after(0, _go)

    def set_error(self, msg: str = ""):
        def _go():
            self._text = msg[:22]
            self._state = "error"
            self._w_target = float(ACTIVE_W)
            self._h_target = float(ACTIVE_H)
            self._root.after(500, self._shrink_to_idle)
        if self._root: self._root.after(0, _go)

    def _shrink_to_idle(self):
        self._state = "idle"
        self._w_target = float(IDLE_W)
        self._h_target = float(IDLE_H)

    def destroy(self):
        if self._root:
            self._root.after(0, self._root.destroy)