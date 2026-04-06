import { invoke } from "@tauri-apps/api/core";

export type Point = [number, number];
export type Brush = {
  color: string;
  width: number;
  opacity: number;
  fill: boolean;
};
export type HistoryState = { undo: boolean; redo: boolean };

function context(canvas: HTMLCanvasElement, fast = false) {
  const ctx = canvas.getContext("2d", { desynchronized: fast });
  if (!ctx) throw new Error("Canvas is unavailable");
  return ctx;
}

type Live = {
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  brush: Brush;
  points: Point[];
  id: number;
};

export class DrawingPad {
  private base = document.createElement("canvas");
  private tail = Promise.resolve();
  private live?: Live;
  private layers = new Set<Live>();
  private spare = document.createElement("canvas");
  private observer: ResizeObserver;
  private listeners = new AbortController();
  private width = 0;
  private height = 0;
  private ratio = 0;
  private blocked = false;
  private timings: { delivery: number; paint: number }[] = [];
  private diagnostics = false;
  private localOnly = false;
  private scratch = document.createElement("canvas");
  private resolution?: MediaQueryList;

  constructor(
    private host: HTMLElement,
    private brush: () => Brush,
    private history: (s: HistoryState) => void,
    private report: (s: string) => void,
    private previewMode: (enabled: boolean) => void,
  ) {
    this.base.className = "paper";
    this.scratch.className = "scratch";
    host.append(this.base, this.scratch);
    const options = { signal: this.listeners.signal };
    host.addEventListener("pointerdown", this.down, options);
    const motion =
      "onpointerrawupdate" in window ? "pointerrawupdate" : "pointermove";
    host.addEventListener(motion, this.move as EventListener, options);
    host.addEventListener("pointerup", this.up, options);
    host.addEventListener("lostpointercapture", this.finish, options);
    host.addEventListener("pointercancel", this.finish, options);
    host.addEventListener("contextmenu", (e) => e.preventDefault(), options);
    window.addEventListener("blur", this.finish, options);
    this.observer = new ResizeObserver(this.resize);
    this.observer.observe(host);
    this.resize();
    this.watchResolution();
  }

  private watchResolution = () => {
    this.resize();
    this.resolution = matchMedia(`(resolution: ${devicePixelRatio}dppx)`);
    this.resolution.addEventListener("change", this.watchResolution, {
      once: true,
      signal: this.listeners.signal,
    });
  };

  private resize = () => {
    const width = Math.max(1, this.host.clientWidth);
    const height = Math.max(1, this.host.clientHeight);
    if (width !== this.width || height !== this.height) {
      this.width = width;
      this.height = height;
      this.queue(async () => {
        await invoke("grow", { width, height });
        this.extend(this.base, width, height);
      });
    }
    this.ratio = devicePixelRatio || 1;
    this.fitPreview(this.spare);
    for (const live of this.layers) this.sizeLayer(live);
  };

  private extend(canvas: HTMLCanvasElement, width: number, height: number) {
    if (width <= canvas.width && height <= canvas.height) return;
    const saved = document.createElement("canvas");
    saved.width = canvas.width;
    saved.height = canvas.height;
    if (canvas.width && canvas.height) context(saved).drawImage(canvas, 0, 0);
    canvas.width = Math.max(canvas.width, width);
    canvas.height = Math.max(canvas.height, height);
    if (saved.width && saved.height) context(canvas).drawImage(saved, 0, 0);
  }

  private fitPreview(canvas: HTMLCanvasElement) {
    const w = Math.ceil(this.width * this.ratio);
    const h = Math.ceil(this.height * this.ratio);
    if (canvas.width !== w) canvas.width = w;
    if (canvas.height !== h) canvas.height = h;
    canvas.style.width = this.width + "px";
    canvas.style.height = this.height + "px";
  }

  private sizeLayer(live: Live) {
    const { canvas, ctx, brush, points } = live;
    this.fitPreview(canvas);
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    canvas.style.opacity = String(brush.opacity / 255);
    ctx.setTransform(this.ratio, 0, 0, this.ratio, 0, 0);
    ctx.lineWidth = brush.width;
    ctx.lineCap = ctx.lineJoin = "round";
    ctx.strokeStyle = ctx.fillStyle = brush.color;
    this.paint(live, points, 0);
  }

  private paint(live: Live, points: Point[], start: number) {
    const { ctx } = live;
    if (points.length === start) return;
    if (!start) {
      ctx.beginPath();
      ctx.arc(...points[0], live.brush.width / 2, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.beginPath();
    ctx.moveTo(...points[Math.max(0, start - 1)]);
    for (let i = Math.max(1, start); i < points.length; i++)
      ctx.lineTo(...points[i]);
    ctx.stroke();
  }

  private point(e: PointerEvent, rect: DOMRect): Point {
    return [
      Math.max(0, Math.min(this.width - 1, e.clientX - rect.left)),
      Math.max(0, Math.min(this.height - 1, e.clientY - rect.top)),
    ];
  }

  private down = (e: PointerEvent) => {
    if (this.blocked || !e.isPrimary || e.button !== 0 || this.live) return;
    e.preventDefault();
    const brush = this.brush();
    if (brush.fill) {
      if (!this.localOnly)
        this.edit("fill", {
          point: this.point(e, this.host.getBoundingClientRect()),
          color: rgb(brush.color),
        });
      return;
    }
    this.host.setPointerCapture(e.pointerId);
    const canvas = this.spare;
    this.spare = document.createElement("canvas");
    const live: Live = {
      canvas,
      ctx: context(canvas, true),
      brush,
      points: [],
      id: e.pointerId,
    };
    this.host.append(canvas);
    this.live = live;
    this.layers.add(live);
    this.sizeLayer(live);
    this.sample(e);
  };

  private sample(e: PointerEvent) {
    const live = this.live;
    if (!live || live.id !== e.pointerId) return;
    const now = this.diagnostics ? performance.now() : 0;
    const start = live.points.length;
    const rect = this.host.getBoundingClientRect();
    const append = (sample: PointerEvent) => {
      const point = this.point(sample, rect);
      const prev = live.points.at(-1);
      if (!prev || prev[0] !== point[0] || prev[1] !== point[1])
        live.points.push(point);
    };
    for (const sample of e.getCoalescedEvents?.() ?? []) append(sample);
    append(e);
    this.paint(live, live.points, start);
    if (this.diagnostics && this.timings.length < 8192) {
      this.timings.push({
        delivery: now - e.timeStamp,
        paint: performance.now() - now,
      });
    }
  }

  private move = (e: PointerEvent) => this.sample(e);
  private up = (e: PointerEvent) => {
    if (e.pointerId !== this.live?.id) return;
    this.sample(e);
    this.finish();
  };

  private finish = (event?: Event) => {
    const live = this.live;
    if (!live || (event instanceof PointerEvent && event.pointerId !== live.id))
      return;
    this.live = undefined;
    const remove = () => {
      this.layers.delete(live);
      live.canvas.remove();
      this.spare = live.canvas;
    };
    if (this.localOnly) {
      this.extend(this.scratch, this.width, this.height);
      const ctx = context(this.scratch);
      ctx.globalAlpha = live.brush.opacity / 255;
      ctx.drawImage(
        live.canvas,
        0,
        0,
        live.canvas.width / this.ratio,
        live.canvas.height / this.ratio,
      );
      remove();
    } else {
      this.queue(async () => {
        try {
          const { width, color, opacity } = live.brush;
          this.apply(
            await invoke<ArrayBuffer>("commit", {
              points: live.points,
              width,
              rgba: [...rgb(color), opacity],
            }),
          );
        } finally {
          remove();
        }
      });
    }
    if (this.diagnostics && this.timings.length) {
      const p95 = (items: number[]) =>
        items
          .sort((a, b) => a - b)
          [Math.floor((items.length - 1) * 0.95)].toFixed(1);
      this.report(
        `p95 delivery ${p95(this.timings.map((t) => t.delivery))} ms · draw calls ${p95(this.timings.map((t) => t.paint))} ms`,
      );
      this.timings = [];
    }
  };

  setBlocked(value: boolean) {
    if (value) this.finish();
    this.blocked = value;
  }

  toggleDiagnostics() {
    this.diagnostics = !this.diagnostics;
    this.report(this.diagnostics ? "Pen timing on" : "Pen timing off");
  }

  toggleLocalOnly() {
    this.finish();
    this.localOnly = !this.localOnly;
    this.scratch.width = 0;
    this.previewMode(this.localOnly);
    this.report(
      this.localOnly ? "Preview-only pen test" : "Rust commits enabled",
    );
  }

  private queue(job: () => Promise<void>, errorMessage?: string) {
    this.tail = this.tail.then(job).catch((e) => {
      if (errorMessage) console.error(e);
      this.report(errorMessage ?? String(e));
    });
    return this.tail;
  }

  private apply(bytes: ArrayBuffer) {
    // Sheet size, dirty rectangle and history flags precede the RGBA patch.
    const header = new DataView(bytes);
    const [sw, sh, x, y, w, h, flags] = Array.from({ length: 7 }, (_, i) =>
      header.getUint32(i * 4, true),
    );
    if (sw !== this.base.width || sh !== this.base.height) {
      this.base.width = sw;
      this.base.height = sh;
    }
    if (w && h) {
      const data = new Uint8ClampedArray(bytes, 28);
      context(this.base).putImageData(new ImageData(data, w, h), x, y);
    }
    this.history({ undo: !!(flags & 1), redo: !!(flags & 2) });
  }

  edit(command: string, args = {}, errorMessage?: string) {
    this.finish();
    return this.queue(
      async () => this.apply(await invoke<ArrayBuffer>(command, args)),
      errorMessage,
    );
  }

  async idle() {
    this.finish();
    await this.tail;
  }
  size() {
    return { width: this.width, height: this.height };
  }

  dispose() {
    this.listeners.abort();
    this.observer.disconnect();
    this.host.replaceChildren();
  }
}

function rgb(hex: string) {
  return [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
}
