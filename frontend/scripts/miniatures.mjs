// One isometric line miniature per registry component, light and dark, as plain SVG files.
// Usage: node scripts/miniatures.mjs [--check]   (writes public/components/svg/<name>.<mode>.svg)
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { bounds, faces, painted } from "../src/iso/engine.ts";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const registry = JSON.parse(readFileSync(join(root, "../registry.json"), "utf8"));
const out = join(root, "public/components/svg");
const check = process.argv.includes("--check");
const stale = [];

const MODES = {
  light: { ground: "#fcfaf7", line: "#1a1918", side: "#f3f0ec", shade: "#e9e5df", accent: "#3772bb" },
  dark: { ground: "#131110", line: "#d9d5d0", side: "#1a1816", shade: "#201e1c", accent: "#69a1e8" },
};

/** A seeded generator, so a component always draws the same. */
function seeded(text) {
  let h = 2166136261;
  for (const c of text) h = Math.imul(h ^ c.charCodeAt(0), 16777619);
  return () => {
    h = Math.imul(h ^ (h >>> 15), 2246822507);
    h = Math.imul(h ^ (h >>> 13), 3266489909);
    h ^= h >>> 16;
    return (h >>> 0) / 4294967296;
  };
}

const b = (x, z, w, d, h, y = 0, accent = false) => ({ x, z, w, d, h, y, accent, beat: 0 });
const base = (w, d) => b(0, 0, w, d, 0.4);
/** Rows of text as thin slabs, the last one short. */
const lines = (x, z, w, n, r, y = 0.4, gap = 0.9) =>
  Array.from({ length: n }, (_, i) => b(x, z + i * gap, i === n - 1 ? w * (0.4 + r() * 0.3) : w * (0.8 + r() * 0.2), 0.45, 0.14, y));

const archetypes = {
  button: (r) => {
    const w = 8 + r() * 4;
    return [base(w + 6, 9), b(3, 2.5, w, 4, 0.9, 0.4, true), b(3 + w * 0.3, 3.9, w * 0.4, 0.6, 0.12, 1.3)];
  },
  input: (r) => [base(20, 9), b(1.5, 1.2, 17, 0.5, 0.14, 0.4), b(1.5, 2.6, 17, 3.6, 0.3, 0.4), b(2.2 + r() * 5, 3.6, 0.35, 1.8, 0.2, 0.7, true), ...lines(1.5, 7, 9, 1, r)],
  toggle: (r) => {
    const rows = 2 + Math.floor(r() * 2);
    return [base(18, 3 + rows * 3.2), ...Array.from({ length: rows }, (_, i) => [b(1.5, 1.4 + i * 3.2, 2.4, 2.4, 0.5, 0.4, i === 0), b(5.4, 1.9 + i * 3.2, 8 + r() * 4, 0.55, 0.14, 0.4)]).flat()];
  },
  slider: (r) => [base(20, 8), b(2, 3.6, 16, 0.7, 0.2, 0.4), b(2, 3.6, 4 + r() * 10, 0.7, 0.32, 0.4, true), b(2 + 6 + r() * 6, 3, 1.8, 1.8, 0.8, 0.4, true)],
  select: (r) => [base(20, 17), b(1.5, 1.5, 17, 3.2, 0.3, 0.4), b(14.5, 2.4, 2, 1.4, 0.16, 0.7, true), b(1.5, 5.4, 17, 9.2, 0.18, 0.4), ...Array.from({ length: 4 }, (_, i) => b(2, 6 + i * 2.1, 10 + r() * 5, 1.4, 0.14, 0.58, i === 1))],
  card: (r) => [base(20, 15), b(1.5, 1.5, 17, 12, 0.5, 0.4), b(2.5, 2.5, 14, 3.4, 0.2, 0.9, true), ...lines(2.5, 7, 14, 3 + Math.floor(r() * 2), r, 0.9, 1.2)],
  list: (r) => {
    const n = 4 + Math.floor(r() * 3);
    return [base(20, 3 + n * 2.6), ...Array.from({ length: n }, (_, i) => [b(1.5, 1.4 + i * 2.6, 17, 2.1, 0.3, 0.4, i === 1), b(2.2, 2 + i * 2.6, 1.2, 0.9, 0.1, 0.7), b(4.4, 2 + i * 2.6, 6 + r() * 7, 0.7, 0.1, 0.7)]).flat()];
  },
  table: (r) => {
    const rows = 4, cols = 4;
    const s = [base(21, 15), b(1, 1, 19, 2, 0.36, 0.4)];
    for (let i = 0; i < rows; i++) for (let j = 0; j < cols; j++) s.push(b(1.4 + j * 4.7, 3.4 + i * 2.7, 4.2, 2.2, 0.12 + r() * 0.1, 0.4, i === 1 && j === 2));
    return s;
  },
  tabs: (r) => {
    const n = 3 + Math.floor(r() * 3);
    return [base(22, 12), b(1, 1, 20, 2, 0.3, 0.4), ...Array.from({ length: n }, (_, i) => b(1.4 + i * (18 / n), 1.4, 18 / n - 0.8, 1.2, 0.24, 0.7, i === 0)), b(1, 3.6, 20, 7.4, 0.14, 0.4), ...lines(2, 4.6, 14, 3, r, 0.54)];
  },
  dialog: (r) => [base(22, 16), b(0.5, 0.5, 21, 15, 0.06, 0.4), b(4, 3, 14, 10, 0.7, 0.46), b(5, 4, 8, 0.9, 0.16, 1.16, true), ...lines(5, 6, 12, 2, r, 1.16), b(11, 10.8, 3, 1.4, 0.3, 1.16, true), b(14.4, 10.8, 3, 1.4, 0.3, 1.16)],
  chart: (r) => {
    const n = 7 + Math.floor(r() * 4);
    const s = [base(22, 12), b(1, 1, 20, 10, 0.08, 0.4)];
    let level = 1.6;
    for (let i = 0; i < n; i++) {
      level = Math.max(0.5, Math.min(5.5, level + (r() - 0.42) * 2));
      s.push(b(1.6 + i * (18 / n), 4, 18 / n - 0.5, 6, level * 0.4, 0.48, i === n - 2));
    }
    return s;
  },
  calendar: (r) => {
    const s = [base(23, 17), b(1, 1, 21, 1.6, 0.3, 0.4)];
    for (let row = 0; row < 5; row++) for (let col = 0; col < 7; col++) s.push(b(1.2 + col * 3, 3.2 + row * 2.6, 2.6, 2.2, 0.1, 0.4, row === 2 && col === 3));
    if (r() > 2) s.push(b(0, 0, 1, 1, 0));
    return s;
  },
  code: (r) => {
    const n = 7 + Math.floor(r() * 3);
    const s = [base(23, 4 + n * 1.2), b(1, 1, 21, 1.4, 0.3, 0.4), b(1, 2.8, 3, n * 1.2 + 0.4, 0.2, 0.4)];
    for (let i = 0; i < n; i++) s.push(b(4.8 + (i % 3) * 1.2, 3.1 + i * 1.2, 4 + r() * 11, 0.6, 0.14, 0.4, i === 3));
    return s;
  },
  media: (r) => [base(22, 16), b(1.5, 1.5, 19, 10, 0.4, 0.4), b(8.6, 4.5, 4.6, 4, 0.5, 0.8, true), b(1.5, 12.6, 19, 0.6, 0.14, 0.4), b(1.5, 12.6, 4 + r() * 10, 0.6, 0.22, 0.4, true)],
  chat: (r) => {
    const s = [base(22, 17)];
    for (let i = 0; i < 4; i++) {
      const mine = i % 2 === 1;
      const w = 8 + r() * 6;
      s.push(b(mine ? 20 - w : 1.6, 1.4 + i * 3.4, w, 2.6, 0.3, 0.4, mine && i === 3));
    }
    s.push(b(1.4, 15, 19, 1.4, 0.2, 0.4));
    return s;
  },
  tree: (r) => {
    const depth = [0, 1, 2, 2, 1, 2, 0, 1];
    return [base(20, 3 + depth.length * 1.9), ...depth.flatMap((d, i) => [b(1.5 + d * 2.4, 1.4 + i * 1.9, 1, 1, 0.14, 0.4, i === 3), b(3 + d * 2.4, 1.5 + i * 1.9, 4 + r() * 7, 0.7, 0.12, 0.4)])];
  },
  board: (r) => {
    const s = [base(23, 15)];
    for (let c = 0; c < 3; c++) {
      s.push(b(1 + c * 7.3, 1, 6.6, 13, 0.1, 0.4));
      const n = 2 + Math.floor(r() * 3);
      for (let i = 0; i < n; i++) s.push(b(1.4 + c * 7.3, 1.6 + i * 3.6, 5.8, 3, 0.32, 0.5, c === 1 && i === 0));
    }
    return s;
  },
  avatar: (r) => [base(18, 10), ...Array.from({ length: 3 }, (_, i) => b(2 + i * 3.2, 2.6, 3.4, 3.4, 0.5 + r() * 0.2, 0.4, i === 1)), b(12, 3.6, 3.6, 1.4, 0.3, 0.4, true)],
  progress: (r) => [base(20, 9), b(2, 2.2, 16, 1.4, 0.2, 0.4), b(2, 2.2, 3 + r() * 11, 1.4, 0.34, 0.4, true), b(2, 5, 4, 4, 0.1, 0.4), b(7, 5.4, 9, 0.6, 0.12, 0.4)],
  layout: (r) => {
    const left = r() > 0.4;
    return [base(22, 15), b(1, 1, 20, 1.8, 0.3, 0.4), ...(left ? [b(1, 3.4, 5, 10.6, 0.22, 0.4, true)] : []), b(left ? 7 : 1, 3.4, left ? 14 : 20, 5, 0.18, 0.4), b(left ? 7 : 1, 8.9, left ? 6.6 : 9.6, 5.1, 0.18, 0.4), b(left ? 14.4 : 11.4, 8.9, left ? 6.6 : 9.6, 5.1, 0.18, 0.4)];
  },
  map: (r) => {
    const s = [base(22, 15)];
    for (let i = 0; i < 4; i++) for (let j = 0; j < 3; j++) s.push(b(0.6 + i * 5.2, 0.6 + j * 4.7, 4.9, 4.4, 0.1 + r() * 0.16, 0.4));
    s.push(b(10.6, 6, 1.6, 1.6, 1.2, 0.5, true));
    return s;
  },
};

const rules = [
  ["button", /button|fab|chip|action|split_button/],
  ["toggle", /switch|checkbox|radio|toggle|choice|multi_?select|option/],
  ["slider", /slider|range|knob|dial|stepper|rating|scrub|number|spin/],
  ["input", /input|field|textarea|search|otp|pin|password|tag_?input|mask|editable|upload|drop_?zone|form/],
  ["select", /select|combo|dropdown|picker|cascade|autocomplete|menu|suggest|palette|command/],
  ["dialog", /dialog|modal|popover|tooltip|toast|sheet|drawer|overlay|hover|lightbox|spotlight|tour|hotspot|notification/],
  ["tabs", /tab|nav|breadcrumb|pagination|wizard|segment|sidebar|toolbar|titlebar|menubar|stepper|header|footer/],
  ["calendar", /calendar|date|time|schedule|agenda|week|month|event|gantt|roadmap|clock|timeline/],
  ["table", /table|grid|datagrid|matrix|heatmap|pivot|spreadsheet|sheet|cell|column|row/],
  ["chart", /chart|plot|graph|sparkline|gauge|funnel|radar|treemap|sankey|candle|ohlc|depth|histogram|axis|ticker|quote|price/],
  ["code", /code|editor|terminal|diff|markdown|document|text|rich|latex|console|shell|repl|json|blame|commit|git|merge|log|math|formula/],
  ["media", /image|video|audio|camera|gallery|player|waveform|crop|annotat|viewer|qr|barcode|canvas|model|photo|picture|screen|stream|webcam/],
  ["map", /map|geo|globe|location|marker|world/],
  ["chat", /chat|message|prompt|conversation|mention|reply|thread|comment|mail|inbox|agent|reasoning|tool|citation|assistant|bubble/],
  ["tree", /tree|outline|folder|file|explorer|path|archive/],
  ["board", /kanban|board|swimlane|mind|node|flow|diagram|connector|whiteboard|sticky/],
  ["avatar", /avatar|badge|tag|status|presence|dot|label|pill|user|member|profile/],
  ["progress", /progress|spinner|loading|skeleton|shimmer|countdown|timer|meter|busy|shimmer|wait/],
  ["list", /list|feed|item|queue|history|result|activity|notification_list/],
  ["card", /card|panel|section|group|accordion|collapse|callout|alert|banner|empty|stat|kpi|widget|surface|paper/],
  ["layout", /stack|flex|split|dock|pane|page|layout|container|shell|workspace|window|center|spacer|divider|scroll|aspect|responsive|viewport|frame|portal|slot/],
];

/** Chapter slugs that settle a name no rule matched. */
const byChapter = {
  buttons: "button", forms: "input", charts: "chart", finance: "chart", calendar: "calendar", tables: "table", lists: "list",
  navigation: "tabs", menus: "select", overlays: "dialog", feedback: "card", editor: "code", terminal: "code", documents: "code",
  git: "code", devtools: "code", chat: "chat", messaging: "chat", mail: "chat", agent: "chat", collab: "avatar", media: "media",
  maps: "map", canvas: "board", project: "board", layout: "layout", shell: "layout", dashboard: "chart", settings: "toggle",
  account: "input", onboarding: "dialog", data: "avatar", data_display: "avatar", motion: "progress", i18n: "select",
};

function pick(c) {
  const hay = `${c.name} ${c.export_type.replace(/([a-z])([A-Z])/g, "$1_$2").toLowerCase()}`;
  for (const [name, re] of rules) if (re.test(hay)) return name;
  return byChapter[c.category] ?? byChapter[c.source_dir?.replace("src/", "")] ?? "card";
}

const n2 = (n) => (Math.round(n * 100) / 100).toString();
const poly = (pts) => pts.map(([x, y]) => `${n2(x)},${n2(y)}`).join(" ");

let rough = 0;
/** Back to front: exactly when the blocks allow it, else by depth then height. */
function order(model) {
  try {
    return painted(model);
  } catch {
    rough++;
    return [...model].sort((a, c) => a.x + a.z + a.w + a.d - (c.x + c.z + c.w + c.d) || a.y - c.y);
  }
}

function svg(model, mode, blocks) {
  const ink = MODES[mode];
  const [x0, y0, x1, y1] = bounds(blocks);
  const pad = 0.8;
  const [w, h] = [x1 - x0 + pad * 2, y1 - y0 + pad * 2];
  const body = blocks
    .map((blk) => {
      const f = faces(blk, blk.h);
      const cls = blk.accent ? ' class="a"' : "";
      return `<g${cls}><polygon class="r" points="${poly(f.right)}"/><polygon class="f" points="${poly(f.front)}"/><polygon class="t" points="${poly(f.top)}"/></g>`;
    })
    .join("");
  return (
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${n2(x0 - pad)} ${n2(y0 - pad)} ${n2(w)} ${n2(h)}" role="img">` +
    `<style>polygon{stroke:${ink.line};stroke-width:.07;stroke-linejoin:round}.r{fill:${ink.shade}}.f{fill:${ink.side}}.t{fill:${ink.ground}}` +
    `.a polygon{stroke:${ink.accent}}.a .t{fill:${ink.accent};fill-opacity:.22}</style>${body}</svg>\n`
  );
}

if (!check) {
  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });
}
const used = {};
const seen = new Set();
for (const c of registry.components) {
  const file = c.name.replace(/[^a-z0-9_]/g, "_");
  if (seen.has(file)) throw new Error(`two components share the file name ${file}`);
  seen.add(file);
  const kind = pick(c);
  used[kind] = (used[kind] ?? 0) + 1;
  // Heights read better at about twice the drafted ones.
  const taller = archetypes[kind](seeded(c.name)).map((k) => ({ ...k, y: k.y * 2.2, h: k.h * 2.2 }));
  const blocks = order(taller);
  for (const mode of Object.keys(MODES)) {
    const path = join(out, `${file}.${mode}.svg`);
    const text = svg(blocks, mode, blocks);
    if (!check) writeFileSync(path, text);
    else if (!existsSync(path) || readFileSync(path, "utf8") !== text) stale.push(path);
  }
}
if (check) {
  if (stale.length || readdirSync(out).length !== registry.components.length * 2) {
    console.error(`miniatures are stale (${stale.length} differ): run node scripts/miniatures.mjs`);
    process.exit(1);
  }
  console.log("[ok] miniatures are current");
  process.exit(0);
}
const files = readdirSync(out).length;
if (files !== registry.components.length * 2) throw new Error(`expected ${registry.components.length * 2} files, wrote ${files}`);
console.log(`[ok] ${files} svg files for ${registry.components.length} components (${rough} depth-sorted)`);
console.log(Object.entries(used).sort((a, b) => b[1] - a[1]).map(([k, v]) => `${k} ${v}`).join(", "));
