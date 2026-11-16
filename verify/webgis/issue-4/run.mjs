import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

let chromium;
try { ({ chromium } = await import('playwright')); }
catch {
  const req = createRequire('file:///C:/devin/kasuga_converter/e2e/package.json');
  ({ chromium } = req('playwright'));
}
if (!chromium) { console.error('NO_PLAYWRIGHT'); process.exit(2); }

const HERE = path.dirname(fileURLToPath(import.meta.url));
const SHOTS = path.join(HERE, 'shots');
const KASC = 'https://raw.githubusercontent.com/yamamoto-ryuzo/kasugai_canvas_webGIS/refs/heads/main/default/kasugai_canvas.kasc';
const BASE = 'https://yamamoto-ryuzo.github.io/kasugai-canvas-web';
const JOBS = [
  ['unstable','cesium'],['unstable','maplibre'],['unstable','openlayers'],
  ['unstable','leaflet'],['unstable','navara'],['unstable','gsimaps'],
  ['stable','cesium'],['stable','maplibre'],['stable','leaflet'],
];
const ok = v => /^[a-zA-Z0-9_-]+$/.test(v);
const report = {};
const t0 = Date.now();

async function listVals(page, sel, attr) {
  return page.$$eval(sel, (els, a) => [...new Set(els.map(e => e.getAttribute(a)))].filter(Boolean), attr);
}
async function clickAttr(page, attr, value) {
  const loc = page.locator(`[${attr}="${value}"]`);
  const n = await loc.count();
  for (let i = 0; i < n; i++) {
    const el = loc.nth(i);
    const clickable = await el.evaluate(e =>
      e.tagName === 'BUTTON' || e.tagName === 'A' || /tab|nav|item|btn|entry/i.test(e.className || ''));
    if (clickable) { await el.click({ timeout: 8000 }); return true; }
  }
  for (let i = 0; i < n; i++) { const el = loc.nth(i); if (await el.isVisible()) { await el.click({ timeout: 8000 }); return true; } }
  return false;
}

for (const [stage, engine] of JOBS) {
  const key = `${stage}/${engine}`;
  const dir = path.join(SHOTS, stage, engine);
  fs.mkdirSync(dir, { recursive: true });
  const url = `${BASE}/${stage}/${engine}/?kasc=${encodeURIComponent(KASC)}`;
  const rec = report[key] = { url, http: null, steps: [], consoleErrors: [], pageErrors: [], failedReqs: [], dialogs: [] };
  console.log('=== ' + key + ' ===');
  const browser = await chromium.launch();
  try {
    const ctx = await browser.newContext({ viewport: { width: 1366, height: 850 }, locale: 'ja-JP' });
    const page = await ctx.newPage();
    page.on('console', m => { if (m.type() === 'error') rec.consoleErrors.push(m.text().slice(0, 240)); });
    page.on('pageerror', e => rec.pageErrors.push(String((e && e.message) || e).slice(0, 240)));
    page.on('requestfailed', r => rec.failedReqs.push(`${r.failure()?.errorText} ${r.url().slice(0, 140)}`));
    page.on('dialog', d => { rec.dialogs.push(`${d.type()}: ${d.message().slice(0, 120)}`); d.dismiss().catch(() => {}); });
    const step = async (name, act) => {
      try {
        if (act) await act();
        await page.waitForTimeout(700);
        // active付与・可視性・コンテンツ描画を確認（#4 再検証要件）
        const check = await page.evaluate(() => {
          const attrs = ['data-panel','data-settings-panel','data-manage-pane','data-manage-sub','data-tab'];
          const active = [...document.querySelectorAll('.active')]
            .filter(e => e.offsetParent !== null)
            .map(e => {
              for (const a of attrs) { const v = e.getAttribute(a); if (v) return a + '=' + v; }
              return (e.id || e.className || e.tagName).toString().slice(0, 40);
            }).slice(0, 8);
          const visibleContainers = [...document.querySelectorAll('[id$="-panel"],[id$="-window"],[id="walk-help"],.panel,.pane')]
            .filter(e => e.offsetParent !== null && e.children.length > 0).length;
          return { active, visibleContainers };
        }).catch(() => null);
        await page.screenshot({ path: path.join(dir, `${name}.jpg`), type: 'jpeg', quality: 70 });
        const rec2 = { name, ok: true, check };
        if (check && !check.active.length && !check.visibleContainers) rec2.warn = 'no active/visible content';
        rec.steps.push(rec2);
      } catch (e) {
        rec.steps.push({ name, ok: false, error: String(e).slice(0, 200) });
        console.log('  NG ' + name + ' :: ' + String(e).slice(0, 120));
      }
    };
    const resp = await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 90000 });
    rec.http = resp ? resp.status() : null;
    try { await page.waitForLoadState('networkidle', { timeout: 25000 }); } catch {}
    await page.waitForTimeout(6000);
    rec.inventory = await page.evaluate(() => {
      const out = {};
      for (const a of ['data-panel','data-settings-panel','data-manage-pane','data-manage-sub','data-tab'])
        out[a] = [...new Set([...document.querySelectorAll(`[${a}]`)].map(e => e.getAttribute(a)))];
      out.manageOpen = !!document.getElementById('manage-open');
      out.toolsOpen = !!document.getElementById('tools-open');
      out.helpOpen = !!document.getElementById('help-open');
      out.walkHelp = !!document.getElementById('walk-help');
      out.title = document.title;
      return out;
    }).catch(e => ({ evalError: String(e).slice(0, 200) }));
    await step('00-initial');
    for (const p of (rec.inventory['data-panel'] || []).filter(ok)) {
      await step(`10-panel-${p}`, () => clickAttr(page, 'data-panel', p));
      if (p === 'display-panel') {
        for (const s of (rec.inventory['data-settings-panel'] || []).filter(v => v.startsWith('display-') && ok(v)))
          await step(`11-${s}`, () => clickAttr(page, 'data-settings-panel', s));
      }
    }
    if (rec.inventory.toolsOpen) {
      await step('20-tools-open', () => page.click('#tools-open', { timeout: 8000 }));
      for (const s of (rec.inventory['data-settings-panel'] || []).filter(v => v.startsWith('tools-') && ok(v)))
        await step(`21-${s}`, () => clickAttr(page, 'data-settings-panel', s));
    }
    if (rec.inventory.manageOpen) {
      await step('30-manage-open', () => page.click('#manage-open', { timeout: 8000 }));
      for (const p of (rec.inventory['data-manage-pane'] || []).filter(ok)) {
        await step(`31-${p}`, () => clickAttr(page, 'data-manage-pane', p));
        const subs = page.locator('[data-manage-sub]:visible');
        const n = await subs.count();
        for (let i = 0; i < n; i++) {
          const v = await subs.nth(i).getAttribute('data-manage-sub');
          if (!v || !ok(v)) continue;
          await step(`32-${v}`, () => subs.nth(i).click({ timeout: 8000 }));
        }
      }
      await step('39-manage-still-open');
      if (await page.locator('#manage-close').count())
        await page.click('#manage-close', { timeout: 5000 }).catch(() => {});
    }
    if (rec.inventory.helpOpen) {
      await step('40-help-open', () => page.click('#help-open', { timeout: 8000 }));
      for (const t of (rec.inventory['data-tab'] || []).filter(ok))
        await step(`41-tab-${t}`, () => clickAttr(page, 'data-tab', t));
    }
    await step('90-final');
    rec.consoleErrors = [...new Set(rec.consoleErrors)].slice(0, 30);
    rec.pageErrors = [...new Set(rec.pageErrors)].slice(0, 30);
    rec.failedReqs = [...new Set(rec.failedReqs)].slice(0, 30);
    await ctx.close();
  } catch (e) {
    rec.fatal = String(e).slice(0, 300);
    console.log('FATAL ' + key + ' :: ' + rec.fatal);
  } finally { await browser.close(); }
  fs.writeFileSync(path.join(HERE, 'report.json'), JSON.stringify(report, null, 2));
}

const md = ['# Issue #4 CDN WebGIS 検証レポート', '', `- kasc: \`${KASC}\``, `- 実施: ${new Date().toISOString()}`, ''];
for (const [k, r] of Object.entries(report)) {
  md.push(`## ${k} — HTTP ${r.http}`);
  if (r.fatal) md.push(`- **FATAL**: \`${r.fatal}\``);
  const ng = r.steps.filter(s => !s.ok);
  const warns = r.steps.filter(s => s.warn);
  md.push(`- steps: **${r.steps.length - ng.length}/${r.steps.length}** OK`);
  for (const s of ng) md.push(`  - NG \`${s.name}\`: ${s.error}`);
  for (const s of warns) md.push(`  - WARN \`${s.name}\`: ${s.warn}`);
  md.push(`- console errors: ${r.consoleErrors.length}`, ...r.consoleErrors.slice(0, 6).map(e => `  - \`${e}\``));
  md.push(`- page errors: ${r.pageErrors.length}`, ...r.pageErrors.slice(0, 6).map(e => `  - \`${e}\``));
  md.push(`- failed requests: ${r.failedReqs.length}`, ...r.failedReqs.slice(0, 6).map(e => `  - \`${e}\``));
  md.push(`- dialogs: ${r.dialogs.length}`, ...r.dialogs.slice(0, 6).map(e => `  - \`${e}\``));
  md.push('');
}
fs.writeFileSync(path.join(HERE, 'REPORT.md'), md.join('\n'));
fs.writeFileSync(path.join(HERE, 'report.json'), JSON.stringify(report, null, 2));
console.log('ALL DONE in ' + Math.round((Date.now() - t0) / 1000) + 's');
