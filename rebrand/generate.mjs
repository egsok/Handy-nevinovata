// klava-nevinovata brand asset generator — v3 (2026-07-09)
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
const logo = `${svgOpen("0 0 1024 1024")}
  <rect x="32" y="32" width="960" height="960" rx="212" fill="${PLATE}"/>
  <ellipse cx="530" cy="150" rx="240" ry="75" fill="none" stroke="${AMBER}" stroke-width="52" transform="rotate(-9 530 150)"/>
  <rect x="362" y="292" width="300" height="356" rx="150" fill="${AMBER_DEEP}" stroke="${INK}" stroke-width="28"/>
  <path d="M 282 590 a 230 230 0 0 0 460 0" fill="none" stroke="${INK}" stroke-width="44" stroke-linecap="round"/>
  <line x1="512" y1="820" x2="512" y2="890" stroke="${INK}" stroke-width="44" stroke-linecap="round"/>
</svg>`;

// ---- tray icons 64: mic + halo, glyph fills the canvas ----
// idle = outline capsule, recording = filled capsule, transcribing = three dots.
const HALO = (c) =>
  `<ellipse cx="33" cy="7" rx="17" ry="5" fill="none" stroke="${c}" stroke-width="4.6" transform="rotate(-9 33 7)"/>`;
const ARC = (c, sw = 5) =>
  `<path d="M 14 38 a 18 18 0 0 0 36 0" fill="none" stroke="${c}" stroke-width="${sw}" stroke-linecap="round"/>
   <line x1="32" y1="56" x2="32" y2="61" stroke="${c}" stroke-width="${sw}" stroke-linecap="round"/>`;
const CAPSULE = (attrs) => `<rect x="21" y="15" width="22" height="28" rx="11" ${attrs}/>`;
const DOTS = (c) =>
  `<circle cx="20" cy="29" r="4.5" fill="${c}"/><circle cx="32" cy="29" r="4.5" fill="${c}"/><circle cx="44" cy="29" r="4.5" fill="${c}"/>`;

function trayMono(c, state) {
  let body = "";
  if (state === "idle") body = CAPSULE(`fill="none" stroke="${c}" stroke-width="5"`) + ARC(c);
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
