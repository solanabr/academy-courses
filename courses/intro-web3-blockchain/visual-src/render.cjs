// Renders the course banner and the eight lesson cards in the panel style of the
// videos. Needs playwright on NODE_PATH, optionally CHROMIUM_PATH, and cwebp on PATH:
//   node courses/intro-web3-blockchain/visual-src/render.cjs
// The banner is text-free like every course banner: the catalog card overlays the
// title bottom-left and shows source y 0-770, with the status chip top-right.
const { chromium } = require("playwright");
const { execFileSync } = require("node:child_process");
const { mkdtempSync, writeFileSync, statSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");

const COURSE = path.resolve(__dirname, "..");
const W = 1600;
const H = 900;

const LOGO = `<svg class="mark" viewBox="0 0 397.7 311.7" aria-hidden="true">
<defs><linearGradient id="sol" gradientUnits="userSpaceOnUse" x1="360.9" y1="-37.5" x2="141.2" y2="383.3">
<stop offset="0" stop-color="#00FFA3"/><stop offset="1" stop-color="#DC1FFF"/></linearGradient></defs>
<path fill="url(#sol)" d="M64.6 237.9c2.4-2.4 5.7-3.8 9.2-3.8h317.4c5.8 0 8.7 7 4.6 11.1l-62.7 62.7c-2.4 2.4-5.7 3.8-9.2 3.8H6.5c-5.8 0-8.7-7-4.6-11.1l62.7-62.7z"/>
<path fill="url(#sol)" d="M64.6 3.8C67.1 1.4 70.4 0 73.8 0h317.4c5.8 0 8.7 7 4.6 11.1l-62.7 62.7c-2.4 2.4-5.7 3.8-9.2 3.8H6.5c-5.8 0-8.7-7-4.6-11.1L64.6 3.8z"/>
<path fill="url(#sol)" d="M333.1 120.1c-2.4-2.4-5.7-3.8-9.2-3.8H6.5c-5.8 0-8.7 7-4.6 11.1l62.7 62.7c2.4 2.4 5.7 3.8 9.2 3.8h317.4c5.8 0 8.7-7 4.6-11.1l-62.7-62.7z"/>
</svg>`;

// Wordmark letters drawn as strokes on a 110x100 cell, following the squared
// geometry of the video's logo.
const LETTERS = {
  S: "M104 9H20Q9 9 9 20V39Q9 50 20 50H90Q101 50 101 61V80Q101 91 90 91H6",
  O: "M20 9H90Q101 9 101 20V80Q101 91 90 91H20Q9 91 9 80V20Q9 9 20 9Z",
  L: "M9 0V80Q9 91 20 91H104",
  A: "M9 100V20Q9 9 20 9H90Q101 9 101 20V100M9 58H101",
  N: "M9 100V9L101 91V0",
};
const WORD = `<svg class="word" viewBox="0 0 ${6 * 128} 100" aria-label="Solana">${[..."SOLANA"]
  .map(
    (c, i) =>
      `<path transform="translate(${i * 128} 0)" d="${LETTERS[c]}" fill="none" stroke="#fff" stroke-width="21" stroke-linejoin="miter"/>`
  )
  .join("")}</svg>`;

const CSS = `
*{box-sizing:border-box;margin:0;padding:0}
html,body{width:${W}px;height:${H}px;background:#05070b;font-family:Inter,sans-serif;color:#fff;-webkit-font-smoothing:antialiased}
.frame{position:relative;width:${W}px;height:${H}px;overflow:hidden;display:flex}
.panel{position:relative;flex:1;height:${H}px;background:
  radial-gradient(760px 520px at -6% -8%,rgba(0,128,120,.34),transparent 72%),
  radial-gradient(900px 620px at 104% 108%,rgba(112,40,190,.44),transparent 72%),
  #05070b}
.strip{position:absolute;top:0;right:0;width:9px;height:100%;background:linear-gradient(180deg,#14F195 0%,#35a7e8 48%,#9945FF 100%)}
.head{display:flex;align-items:center;gap:16px}
.mark{height:34px;width:auto}
.word{height:23px;width:auto}
.rule{height:1.5px;background:rgba(255,255,255,.42);margin-top:30px}
.badge{display:inline-block;margin-top:34px;padding:7px 15px;border:1.5px solid rgba(255,255,255,.2);border-radius:8px;background:rgba(255,255,255,.08);font-size:20px;font-weight:600;letter-spacing:.04em;color:#dfe4ea}
h1{font-weight:700;letter-spacing:-.012em}
h1 span{background:linear-gradient(95deg,#14F195 0%,#2ad1b4 40%,#3f8cf0 100%);-webkit-background-clip:text;background-clip:text;color:transparent;-webkit-box-decoration-break:clone;box-decoration-break:clone;padding-bottom:6px}
.sub{color:#b4bac3;font-weight:400}
ul{list-style:none}
li{position:relative;font-weight:400;color:#f2f4f7}
li::before{content:"-";position:absolute;left:0}
li b{font-weight:700;color:#fff}
.card .panel{padding:92px 120px 0 112px;background:
  radial-gradient(1500px 620px at 8% -22%,rgba(0,128,120,.34),transparent 74%),
  radial-gradient(1600px 720px at 88% 122%,rgba(112,40,190,.5),transparent 74%),
  #05070b}
.card h1{margin-top:26px;font-size:76px;line-height:1.12}
.card .sub{margin-top:10px;font-size:36px;line-height:1.25}
.card ul{margin-top:42px}
.card li{font-size:40px;line-height:1.26;padding-left:38px;margin-top:22px}
.banner .panel{background:
  radial-gradient(1000px 640px at 2% -14%,rgba(0,128,120,.42),transparent 72%),
  radial-gradient(1200px 820px at 98% 120%,rgba(112,40,190,.6),transparent 72%),
  #05070b}
.bk{position:absolute;border-radius:50%;filter:blur(6px)}
.pool{position:absolute;left:1060px;top:690px;width:900px;height:170px;transform:translate(-50%,-50%);
  background:radial-gradient(closest-side,rgba(70,200,210,.35),rgba(130,70,230,.18) 60%,transparent);filter:blur(14px)}
.trail{position:absolute;inset:0}
.cube{position:absolute;transform-style:preserve-3d}
.face{position:absolute;inset:0;border:2px solid rgba(170,255,235,.6);border-radius:6px;
  background:linear-gradient(140deg,rgba(20,241,149,.20),rgba(60,140,240,.12) 50%,rgba(153,69,255,.26));
  box-shadow:inset 0 0 38px rgba(90,220,230,.35),0 0 26px rgba(120,90,255,.30)}
.face.front{display:flex;align-items:center;justify-content:center}
.face .mark{height:46%;filter:drop-shadow(0 0 16px rgba(80,255,190,.55))}
.tile{position:absolute;width:118px;height:118px;border-radius:26px;display:flex;align-items:center;justify-content:center;
  background:linear-gradient(150deg,rgba(255,255,255,.12),rgba(255,255,255,.03));border:1.5px solid rgba(255,255,255,.28);
  box-shadow:0 0 34px rgba(110,90,255,.35),inset 0 0 22px rgba(120,230,230,.18)}
.tile svg{width:58px;height:58px;stroke:#fff;fill:none;stroke-width:2.2;stroke-linecap:round;stroke-linejoin:round;
  filter:drop-shadow(0 0 6px rgba(160,240,255,.7))}
`;

const page = (body) => `<!doctype html><html lang="pt-BR"><head><meta charset="utf-8">
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap" rel="stylesheet">
<style>${CSS}</style></head><body>${body}</body></html>`;

const bullets = (items) => `<ul>${items.map((i) => `<li>${i}</li>`).join("")}</ul>`;

const card = (n, title, sub, items) =>
  page(`<div class="frame card"><div class="panel"><div class="head">${LOGO}${WORD}</div><div class="rule"></div>
<span class="badge">AULA ${n}</span><h1><span>${title}</span></h1><p class="sub">${sub}</p>${bullets(items)}</div><div class="strip"></div></div>`);

const CARDS = [
  ["evolucao-da-web-e-blockchain", "A evolução da Web<br>e blockchain", "Quem está controlando as suas informações?", [
    "<b>Web1:</b> a cidade das vitrines",
    "<b>Web2:</b> a cidade das plataformas",
    "<b>Web3:</b> a cidade inteligente e descentralizada",
    "A internet são as <b>ruas</b>, a web são os <b>prédios</b>",
  ]],
  ["explorando-a-rede-solana", "Explorando a<br>rede Solana", "Ambientes, explorador e transações", [
    "<b>Mainnet:</b> a cidade em operação",
    "<b>Devnet:</b> a oficina dos aplicativos",
    "<b>Testnet:</b> o campo de testes da infraestrutura",
    "Para começar, <b>basta navegar</b>",
  ]],
  ["carteira-digital", "Carteira digital", "Será que você já usa?", [
    "<b>Endereço público:</b> você compartilha, como no Pix",
    "<b>Chave privada:</b> o segredo que assina",
    "<b>Frase de recuperação:</b> uma chave mestra",
    "<b>Custódia</b> é o manobrista, na <b>autocustódia</b> a chave é sua",
  ]],
  ["do-intermediario-ao-registro-distribuido", "Do intermediário ao<br>registro distribuído", "Como funciona a blockchain", [
    "Nenhuma cédula atravessa: é uma <b>instrução</b>",
    "Copiar é fácil: o dinheiro digital precisa de <b>registro</b>",
    "O problema: <b>gasto duplo</b>",
    "2008: o <b>Bitcoin</b> e o livro-razão distribuído",
  ]],
  ["quem-decide-qual-historico-e-valido", "Quem decide qual<br>histórico é válido?", "Sem uma autoridade central, a rede precisa decidir", [
    "<b>Mais blocos posteriores</b> = maior consolidação",
    "<b>Nem toda blockchain</b> é Bitcoin",
    "Smart contract: uma <b>região intermediária</b>",
    "A blockchain <b>não elimina a confiança</b>",
  ]],
  ["cartorios-diario-oficial-e-validadores", "Cartórios, diário oficial<br>e validadores", "Dado compartilhado é o grande segredo", [
    "Vários <b>cartórios digitais</b>, cópias do mesmo livro",
    "Blocos: volumes do <b>diário oficial</b>",
    "Uma confirmação <b>não é garantia mágica</b>",
    "Validadores seguem <b>as regras do protocolo</b>",
  ]],
  ["do-toque-ao-pagamento-confirmado", "Do toque no botão ao<br>pagamento confirmado", "Uma regra simples para cada pagamento", [
    "Antes de pagar, <b>confira</b>",
    "Antes de assinar, <b>entenda</b>",
    "Depois de enviar, <b>verifique</b>",
    "Pagamento pendente? <b>Não aperte enviar de novo</b>",
  ]],
  ["um-cafe-pago-em-dolar-digital", "Um café pago em<br>dólar digital", "Pagamentos, stablecoins e economia real", [
    "<b>On-ramp:</b> do real ao USDC",
    "<b>Solana Pay:</b> da sua carteira para o comerciante",
    "A palavra-chave é <b>liquidação</b>",
    "<b>Off-ramp:</b> de volta ao mundo tradicional",
  ]],
];

const cube = (x, y, size, rx, ry, logo) => {
  const h = size / 2;
  const faces = [["front", `translateZ(${h}px)`], ["back", `rotateY(180deg) translateZ(${h}px)`], ["right", `rotateY(90deg) translateZ(${h}px)`],
    ["left", `rotateY(-90deg) translateZ(${h}px)`], ["top", `rotateX(90deg) translateZ(${h}px)`], ["bottom", `rotateX(-90deg) translateZ(${h}px)`]];
  return `<div style="position:absolute;left:${x - h}px;top:${y - h}px;width:${size}px;height:${size}px;perspective:1400px">
<div class="cube" style="width:${size}px;height:${size}px;transform:rotateX(${rx}deg) rotateY(${ry}deg)">${faces
    .map(([n, t]) => `<div class="face ${n}" style="transform:${t}">${n === "front" && logo ? LOGO : ""}</div>`).join("")}</div></div>`;
};
const ICONS = {
  wallet: '<path d="M6 16h40a4 4 0 0 1 4 4v24a4 4 0 0 1-4 4H10a4 4 0 0 1-4-4V12a4 4 0 0 1 4-4h30v8"/><path d="M50 26H38a5 5 0 0 0 0 10h12z"/>',
  key: '<circle cx="20" cy="36" r="10"/><path d="M27 29 48 8M40 16l6 6M34 22l5 5"/>',
  globe: '<circle cx="28" cy="28" r="22"/><path d="M6 28h44M28 6c-12 12-12 32 0 44M28 6c12 12 12 32 0 44"/>',
  lock: '<rect x="10" y="24" width="36" height="26" rx="5"/><path d="M18 24v-7a10 10 0 0 1 20 0v7M28 34v6"/>',
  coin: '<ellipse cx="28" cy="16" rx="18" ry="7"/><path d="M10 16v12c0 4 8 7 18 7s18-3 18-7V16M10 28v12c0 4 8 7 18 7s18-3 18-7V28"/>',
};
const tile = (x, y, icon, rot) => `<div class="tile" style="left:${x - 59}px;top:${y - 59}px;transform:rotate(${rot}deg)"><svg viewBox="0 0 56 56">${ICONS[icon]}</svg></div>`;
// Chain runs lower-left to upper-right through the middle band; the title overlay
// owns x<880 below y=600 and the status chip owns the top-right corner.
const CHAIN = [[760, 560, 96, -18, 34], [905, 470, 118, -20, 36], [1080, 380, 172, -22, 38, true], [1265, 300, 118, -20, 40], [1405, 240, 92, -18, 42]];
const BOKEH = [[220, 160, 10, "rgba(80,220,200,.35)"], [470, 90, 6, "rgba(160,120,255,.4)"], [640, 250, 4, "rgba(120,230,255,.4)"], [1500, 470, 8, "rgba(180,110,255,.35)"],
  [1180, 600, 5, "rgba(90,240,190,.35)"], [330, 420, 5, "rgba(150,110,255,.3)"], [1540, 150, 4, "rgba(120,230,255,.35)"], [980, 170, 6, "rgba(90,240,190,.3)"]];
const BANNER = page(`<div class="frame banner"><div class="panel">
${BOKEH.map(([x, y, r, c]) => `<div class="bk" style="left:${x}px;top:${y}px;width:${r * 2}px;height:${r * 2}px;background:${c}"></div>`).join("")}
<div class="pool"></div>
<svg class="trail" viewBox="0 0 ${W} ${H}"><defs><linearGradient id="tr" x1="0" y1="1" x2="1" y2="0"><stop offset="0" stop-color="#14F195"/><stop offset=".5" stop-color="#35a7e8"/><stop offset="1" stop-color="#9945FF"/></linearGradient>
<filter id="gl" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="7"/></filter></defs>
<g fill="none" stroke="url(#tr)" stroke-linecap="round">
<path d="M640 630 C 720 600, 760 560, 905 470 S 1080 380, 1265 300 S 1460 210, 1540 180" stroke-width="18" opacity=".7" filter="url(#gl)"/>
<path d="M640 630 C 720 600, 760 560, 905 470 S 1080 380, 1265 300 S 1460 210, 1540 180" stroke-width="3.5"/>
<path d="M880 230 Q 980 250 1040 330" stroke-width="2" opacity=".55"/><path d="M1180 170 Q 1150 240 1120 300" stroke-width="2" opacity=".55"/>
<path d="M1400 570 Q 1300 500 1160 440" stroke-width="2" opacity=".55"/><path d="M1000 610 Q 1020 520 1050 470" stroke-width="2" opacity=".5"/>
<path d="M1470 400 Q 1440 330 1405 270" stroke-width="2" opacity=".5"/></g></svg>
${CHAIN.map(([x, y, s, rx, ry, logo]) => cube(x, y, s, rx, ry, logo)).join("")}
${tile(860, 220, "wallet", -6)}${tile(1180, 150, "key", 5)}${tile(1400, 590, "globe", 6)}${tile(990, 640, "lock", -4)}${tile(1480, 420, "coin", -5)}
</div><div class="strip"></div></div>`);

async function shot(browser, html, out, quality) {
  const tmp = mkdtempSync(path.join(tmpdir(), "iwb-"));
  const htmlPath = path.join(tmp, "page.html");
  const pngPath = path.join(tmp, "page.png");
  writeFileSync(htmlPath, html);
  const ctx = await browser.newContext({ viewport: { width: W, height: H }, deviceScaleFactor: 1 });
  const p = await ctx.newPage();
  await p.goto(`file://${htmlPath}`, { waitUntil: "networkidle" });
  await p.evaluate(() => document.fonts.ready);
  const ok = await p.evaluate(() => !document.querySelector("h1") || document.fonts.check("700 40px Inter"));
  if (!ok) throw new Error("Inter did not load");
  const overflow = await p.evaluate(() => {
    const panel = document.querySelector(".panel").getBoundingClientRect();
    const items = [...document.querySelectorAll("h1,.sub,li")];
    return items.filter((e) => {
      const r = e.getBoundingClientRect();
      return r.right > panel.right - 20 || r.bottom > panel.bottom - 30;
    }).map((e) => e.textContent);
  });
  if (overflow.length) console.warn("overflow:", out, overflow);
  await p.screenshot({ path: pngPath });
  await ctx.close();
  execFileSync("cwebp", ["-quiet", "-q", String(quality), "-m", "6", pngPath, "-o", out]);
  console.log(path.relative(COURSE, out), Math.round(statSync(out).size / 1024) + " KB");
}

(async () => {
  const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || undefined });
  await shot(browser, BANNER, path.join(COURSE, "assets/banner.webp"), 90);
  for (const [i, [slug, title, sub, items]] of CARDS.entries()) {
    await shot(browser, card(i + 1, title, sub, items), path.join(COURSE, `lessons/${slug}/assets/card-aula-${i + 1}.webp`), 90);
  }
  await browser.close();
})();
