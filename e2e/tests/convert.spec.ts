import { test, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { apiRunJob, E2E_DIR, FIXTURES, openSubTab, openTopTab, runJob, workDir } from './helpers';

async function setCoordEpsg(page: import('@playwright/test').Page, epsg: string) {
  await openSubTab(page, 'coord');
  await page.locator('#coordEpsg').fill(epsg);
  await page.locator('#coordEpsg').dispatchEvent('change');
}

test.describe('前処理', () => {
  test('OBJ 表面処理: f 行の頂点順と vn 行の法線が反転する', async ({ page }) => {
    const out = workDir('obj-flip');
    const outFile = path.join(out, 'out.obj');
    await page.goto('/');
    await openTopTab(page, 'data-preprocess');
    await openSubTab(page, 'obj-surface');
    await page.locator('#objSurfaceInput').fill(path.join(FIXTURES, 'sample.obj'));
    await page.locator('#objSurfaceOutput').fill(outFile);

    const res = await runJob(page, 'objSurfaceFlipBtn');
    expect(res.statusText, res.log).toContain('完了');
    expect(fs.existsSync(outFile), res.log).toBeTruthy();

    const content = fs.readFileSync(outFile, 'utf8');
    expect(content).toContain('f 3/3/1 2/2/1 1/1/1');
    expect(content).toMatch(/vn .*-1/);
  });

  test('XY 反転: LAS の X/Y が入れ替わる', async ({ page }) => {
    const out = workDir('xyflip');
    const outFile = path.join(out, 'flipped.las');
    await page.goto('/');
    await openTopTab(page, 'data-preprocess');
    await openSubTab(page, 'xyflip');
    await page.locator('#xyflipInput').fill(path.join(FIXTURES, 'sample.las'));
    await page.locator('#xyflipOutput').fill(outFile);

    const res = await runJob(page, 'xyflipRunBtn');
    expect(res.statusText, res.log).toContain('完了');
    expect(fs.existsSync(outFile), res.log).toBeTruthy();
    expect(fs.statSync(outFile).size).toBeGreaterThan(0);
  });

  test('glTF 最適化: gltf-pipeline で GLB を出力', async ({ page }) => {
    const out = workDir('gltfopt');
    const outFile = path.join(out, 'out.glb');
    await page.goto('/');
    await openTopTab(page, 'data-preprocess');
    await openSubTab(page, 'gltfopt');
    await page.locator('#gltfoptInput').fill(path.join(FIXTURES, 'sample.glb'));
    await page.locator('#gltfoptOutput').fill(outFile);

    const res = await runJob(page, 'gltfoptRunBtn', 240_000);
    expect(res.statusText, res.log).toContain('完了');
    expect(fs.existsSync(outFile), res.log).toBeTruthy();
    expect(fs.statSync(outFile).size).toBeGreaterThan(0);
  });
});

test.describe('個別コンバータ', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await openTopTab(page, 'data-convert');
  });

  test('gocesiumtiler: LAS → pnts タイルセット', async ({ page }) => {
    const out = workDir('gocesiumtiler');
    await setCoordEpsg(page, '3857');
    await openSubTab(page, 'individual');
    await openSubTab(page, 'gocesiumtiler');
    await page.locator('#gocesiumInput').fill(path.join(FIXTURES, 'sample.las'));
    await page.locator('#gocesiumOutput').fill(out);

    const res = await runJob(page, 'gocesiumRunBtn', 180_000);
    expect(res.statusText, res.log).toContain('完了');
    expect(fs.existsSync(path.join(out, 'tileset.json')), res.log).toBeTruthy();
  });

  test('Py3DTiles: LAS → pnts タイルセット', async ({ page }) => {
    const out = workDir('py3dtiles');
    await setCoordEpsg(page, '3857');
    await openSubTab(page, 'individual');
    await openSubTab(page, 'py3dtiles');
    await page.locator('#pyInput').fill(path.join(FIXTURES, 'sample.las'));
    await page.locator('#pyOutput').fill(out);

    const res = await runJob(page, 'pyRunBtn', 180_000);
    expect(res.statusText, res.log).toContain('完了');
    expect(fs.existsSync(path.join(out, 'tileset.json')), res.log).toBeTruthy();
  });

  test('mago-3d-tiler: OBJ → b3dm タイルセット', async ({ page }) => {
    const out = workDir('mago');
    await setCoordEpsg(page, '3857');
    await openSubTab(page, 'individual');
    await openSubTab(page, 'mago');
    await page.locator('#input').fill(path.join(FIXTURES, 'mago_in'));
    await page.locator('#output').fill(out);
    await page.locator('#inputType').selectOption('obj');
    await page.locator('#outputType').selectOption('b3dm');

    const res = await runJob(page, 'runBtn', 240_000);
    expect(res.statusText, res.log).toContain('完了');
    expect(fs.existsSync(path.join(out, 'tileset.json')), res.log).toBeTruthy();
  });

  test('3D Tiles 1.1: tileset を upgrade して 1.1 化', async ({ page, request }) => {
    // 入力となる 1.0 タイルセットを mago で生成（UI 済み経路は API で代替）
    const src = workDir('tdt-src');
    await apiRunJob(
      request,
      '/api/convert',
      {
        input: path.join(FIXTURES, 'mago_in'),
        output: src,
        input_type: 'obj',
        output_type: 'b3dm',
        crs: '3857',
      },
      240_000,
    );
    expect(fs.existsSync(path.join(src, 'tileset.json'))).toBeTruthy();

    const out = workDir('tiles11');
    // NOTE: `npx 3d-tiles-tools` は Node 24 + better-sqlite3 の prebuild 不在で
    // 失敗するため、e2e/node_modules に入れた CLI を直接使う。
    // （upgrade は better-sqlite3 を使用しないため --ignore-scripts で導入可能）
    const cli = path.join(E2E_DIR, 'node_modules', '3d-tiles-tools', 'build', 'src', 'cli.mjs');
    await page.goto('/');
    await openTopTab(page, 'data-convert');
    await openSubTab(page, 'individual');
    await openSubTab(page, 'tiles11');
    await page.locator('#tdtCommand').fill(`node ${cli}`);
    await page.locator('#tdtTool').fill('upgrade');
    await page.locator('#tdtInput').fill(path.join(src, 'tileset.json'));
    await page.locator('#tdtOutput').fill(out);
    // -f: 出力ディレクトリが既存でも上書き許可（無いと 3d-tiles-tools が失敗する）
    await page.locator('#tdtExtraArgs').fill('--targetVersion 1.1 -f');

    const res = await runJob(page, 'tdtRunBtn', 240_000);
    expect(res.statusText, res.log).toContain('完了');
    const tileset = JSON.parse(fs.readFileSync(path.join(out, 'tileset.json'), 'utf8'));
    expect(tileset.asset.version).toBe('1.1');
  });
});

test.describe('自動変換', () => {
  test('LAS → pnts (gocesiumtiler に自動ルーティング)', async ({ page }) => {
    const out = workDir('auto-pnts');
    await page.goto('/');
    await openTopTab(page, 'data-convert');
    await setCoordEpsg(page, '3857');
    await openSubTab(page, 'auto');
    await page.locator('#autoInput').fill(path.join(FIXTURES, 'auto_in'));
    await page.locator('#autoOutput').fill(out);
    await openSubTab(page, '3d-tile-out');
    await page.locator('#autoOutputFormat3dTile').selectOption('pnts');

    const res = await runJob(page, 'autoRunBtn', 180_000);
    expect(res.statusText, res.log).toContain('完了');
    // gocesiumtiler folder モードは入力ファイルごとにサブディレクトリを作る
    expect(fs.existsSync(path.join(out, 'sample', 'tileset.json')), res.log).toBeTruthy();
  });
});
