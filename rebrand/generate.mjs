// klava-nevinovata brand asset generator — v4 (2026-07-11)
// v4: halo got an ink outline (matches capsule), cradle arc hugs the capsule
//     (uniform 55-unit gap; the old deep bowl wasted ~15% of canvas height),
//     mark enlarged +20%, composition vertically balanced (was 9px from top).
//     Tray glyph: uniform 6-unit stroke (halo was 4.6 — sub-pixel at 16px).
// v3: keycap dropped — mark is now "mic + halo" (keycap didn't survive 16-24px;
//     Egor's call). Klava meaning is carried by the wordmark.
// v2: increased glyph scale.
// Writes master SVGs and renders PNGs into D:/dev/Handy/rebrand/
import { Resvg } from "@resvg/resvg-js";
import { mkdirSync, writeFileSync } from "fs";
import { join } from "path";

const OUT = "D:/dev/Handy/rebrand";
for (const d of ["", "svg", "tray", "wordmark"]) mkdirSync(join(OUT, d), { recursive: true });

// ---- palette ----
const INK = "#2A2419";       // warm near-black
const AMBER = "#E3A33C";     // halo / light accent
const AMBER_DEEP = "#C4791B";// mic capsule / deep accent
const WARM_WHITE = "#FFFDF4";
const PLATE = "#F5EFDF";
const REC = "#C74A33";
const MONO_WHITE = "#FFFFFF"; // tray icons on dark taskbar (no *_dark suffix)
const MONO_INK = "#1E1913";   // tray icons on light taskbar (*_dark suffix)

const svgOpen = (vb) => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${vb}">`;

// ---- app logo 1024: big mic + halo on warm plate ----
// Halo is two concentric strokes (ink under amber) — SVG strokes can't have
// their own outline. Halo is drawn first so its lower edge tucks behind the
// capsule.
const logo = `${svgOpen("0 0 1024 1024")}
  <rect x="32" y="32" width="960" height="960" rx="212" fill="${PLATE}"/>
  <ellipse cx="524" cy="238" rx="258" ry="80" fill="none" stroke="${INK}" stroke-width="94" transform="rotate(-8 524 238)"/>
  <ellipse cx="524" cy="238" rx="258" ry="80" fill="none" stroke="${AMBER}" stroke-width="62" transform="rotate(-8 524 238)"/>
  <rect x="332" y="347" width="360" height="420" rx="180" fill="${AMBER_DEEP}" stroke="${INK}" stroke-width="30"/>
  <path d="M 238 587 a 274 274 0 0 0 548 0" fill="none" stroke="${INK}" stroke-width="48" stroke-linecap="round"/>
  <line x1="512" y1="861" x2="512" y2="895" stroke="${INK}" stroke-width="48" stroke-linecap="round"/>
</svg>`;

// ---- tray icons 64: mic + halo, glyph fills the canvas ----
// idle = outline capsule, recording = filled capsule, transcribing = three dots.
const HALO = (c) =>
  `<ellipse cx="32.5" cy="8" rx="19" ry="5.8" fill="none" stroke="${c}" stroke-width="6" transform="rotate(-9 32.5 8)"/>`;
const ARC = (c, sw = 6) =>
  `<path d="M 12 40 a 20 20 0 0 0 40 0" fill="none" stroke="${c}" stroke-width="${sw}" stroke-linecap="round"/>
   <line x1="32" y1="59.5" x2="32" y2="60.5" stroke="${c}" stroke-width="${sw}" stroke-linecap="round"/>`;
const CAPSULE = (attrs) => `<rect x="19.5" y="17" width="25" height="28" rx="12.5" ${attrs}/>`;
const DOTS = (c) =>
  `<circle cx="19" cy="31" r="5" fill="${c}"/><circle cx="32" cy="31" r="5" fill="${c}"/><circle cx="45" cy="31" r="5" fill="${c}"/>`;

function trayMono(c, state) {
  let body = "";
  if (state === "idle") body = CAPSULE(`fill="none" stroke="${c}" stroke-width="6"`) + ARC(c);
  if (state === "recording") body = CAPSULE(`fill="${c}"`) + ARC(c);
  if (state === "transcribing") body = DOTS(c) + ARC(c);
  return `${svgOpen("0 0 64 64")}${HALO(c)}${body}</svg>`;
}

// Colored theme: arc/stem warm-white (colored tray lives on dark taskbars;
// ink would vanish there).
function trayColored(state) {
  let body = "";
  if (state === "idle") body = CAPSULE(`fill="${AMBER_DEEP}"`) + ARC(WARM_WHITE);
  if (state === "recording") body = CAPSULE(`fill="${REC}"`) + ARC(WARM_WHITE);
  if (state === "transcribing") body = DOTS(AMBER_DEEP) + ARC(WARM_WHITE);
  return `${svgOpen("0 0 64 64")}${HALO(AMBER)}${body}</svg>`;
}

// ---- wordmark ----
function wordmark(fg1, fg2) {
  return `${svgOpen("0 0 940 140")}
  <text x="0" y="102" font-family="Consolas, 'Cascadia Mono', 'DejaVu Sans Mono', monospace" font-size="96" letter-spacing="0">
    <tspan font-weight="700" fill="${fg1}">klava</tspan><tspan fill="${fg2}">-nevinovata</tspan>
  </text>
  <rect x="858" y="26" width="24" height="88" fill="${AMBER}"/>
</svg>`;
}

// ---- job list ----
const jobs = [
  ["svg/logo.svg", "logo.png", 1024, logo],
  ["svg/wordmark-for-light-bg.svg", "wordmark/wordmark-for-light-bg.png", 1880, wordmark("#2A2419", "#6E6350")],
  ["svg/wordmark-for-dark-bg.svg", "wordmark/wordmark-for-dark-bg.png", 1880, wordmark("#EDE4CF", "#A2947A")],
];
for (const [state, file] of [
  ["idle", "tray_idle"],
  ["recording", "tray_recording"],
  ["transcribing", "tray_transcribing"],
]) {
  jobs.push([`svg/${file}.svg`, `tray/${file}.png`, 64, trayMono(MONO_WHITE, state)]);
  jobs.push([`svg/${file}_dark.svg`, `tray/${file}_dark.png`, 64, trayMono(MONO_INK, state)]);
}
for (const state of ["idle", "recording", "transcribing"]) {
  jobs.push([`svg/colored_${state}.svg`, `tray/colored_${state}.png`, 64, trayColored(state)]);
}

for (const [svgPath, pngPath, width, svg] of jobs) {
  writeFileSync(join(OUT, svgPath), svg);
  const r = new Resvg(svg, {
    fitTo: { mode: "width", value: width },
    font: { loadSystemFonts: true, defaultFontFamily: "Consolas" },
    background: "rgba(0,0,0,0)",
  });
  writeFileSync(join(OUT, pngPath), r.render().asPng());
  console.log(`ok ${pngPath} (${width}px)`);
}
console.log(`\ndone → ${OUT}`);
