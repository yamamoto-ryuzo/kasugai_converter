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

  test('mago-3d-tiler: FBX → b3dm タイルセット', async ({ page }) => {
    const out = workDir('mago-fbx');
    await setCoordEpsg(page, '3857');
    await openSubTab(page, 'individual');
    await openSubTab(page, 'mago');
    await page.locator('#input').fill(path.join(FIXTURES, 'fbx_in'));
    await page.locator('#output').fill(out);
    await page.locator('#inputType').selectOption('fbx');
    await page.locator('#outputType').selectOption('b3dm');

    const res = await runJob(page, 'runBtn', 240_000);
    expect(res.statusText, res.log).toContain('完了');
    expect(fs.existsSync(path.join(out, 'tileset.json')), res.log).toBeTruthy();
  });

  test('FBX → 3D Tiles 1.1 GLB（input_type 自動判定）', async ({ request }) => {
    const out = workDir('fbx-3dtiles11');
    // npx 3d-tiles-tools は Node 24 + better-sqlite3 の prebuild 不在で失敗するため、
    // e2e/node_modules の CLI を直接使う。
    const cli = path.join(E2E_DIR, 'node_modules', '3d-tiles-tools', 'build', 'src', 'cli.mjs');
    const job = await apiRunJob(
      request,
      '/api/convert/obj-3dtiles11',
      {
        input: path.join(FIXTURES, 'fbx_in'),
        output: out,
        crs: '3857',
        tdt_command: `node ${cli}`,
        tdt_extra_args: '--targetVersion 1.1 -f',
      },
      300_000,
    );
    expect(
      fs.existsSync(path.join(out, job.id, 'tiles_1_1', 'tileset.json')),
      job.output,
    ).toBeTruthy();
  });

  test('mago-3d-tiler: テクスチャ付き FBX → b3dm（glb にテクスチャ埋込）', async ({ request }) => {
    const out = workDir('mago-fbx-tex');
    await apiRunJob(
      request,
      '/api/convert',
      {
        input: path.join(FIXTURES, 'fbx_tex_in'),
        output: out,
        input_type: 'fbx',
        output_type: 'b3dm',
        crs: '3857',
      },
      240_000,
    );
    const dataDir = path.join(out, 'data');
    const tiles = fs.readdirSync(dataDir).filter((f) => /\.(glb|b3dm)$/.test(f));
    expect(tiles.length).toBeGreaterThan(0);
    let textured = false;
    for (const f of tiles) {
      const buf = fs.readFileSync(path.join(dataDir, f));
      let off = 0;
      if (buf.readUInt32LE(0) === 0x6d336462) {
        // 'b3dm' — glb payload follows feature/batch tables
        off = 28 + buf.readUInt32LE(8) + buf.readUInt32LE(12) + buf.readUInt32LE(16) + buf.readUInt32LE(20);
      }
      const jsonLen = buf.readUInt32LE(off + 12);
      const json = JSON.parse(buf.slice(off + 20, off + 20 + jsonLen).toString('utf8'));
      if ((json.images || []).length > 0) textured = true;
    }
    expect(textured, '出力 glb にテクスチャ画像が含まれていない').toBeTruthy();
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

  test('FBX → b3dm (mago-3d-tiler に自動ルーティング)', async ({ page }) => {
    const out = workDir('auto-b3dm-fbx');
    await page.goto('/');
    await openTopTab(page, 'data-convert');
    await setCoordEpsg(page, '3857');
    await openSubTab(page, 'auto');
    await page.locator('#autoInput').fill(path.join(FIXTURES, 'auto_fbx_in'));
    await page.locator('#autoOutput').fill(out);
    await openSubTab(page, '3d-tile-out');
    await page.locator('#autoOutputFormat3dTile').selectOption('b3dm');

    const res = await runJob(page, 'autoRunBtn', 240_000);
    expect(res.statusText, res.log).toContain('完了');
    expect(fs.existsSync(path.join(out, 'tileset.json')), res.log).toBeTruthy();
  });

  test('GIS オプション: proj / geoid / recursive が mago に引き渡される', async ({ request }) => {
    const out = workDir('auto-gis-opts');
    // fbx_nested_in は sub/ の中にのみ FBX+テクスチャを持つ → --recursive 必須
    const job = await apiRunJob(
      request,
      '/api/convert/auto',
      {
        input: path.join(FIXTURES, 'fbx_nested_in'),
        output: out,
        output_format: 'b3dm',
        proj: '+proj=merc +a=6378137 +b=6378137 +lat_ts=0 +lon_0=0 +x_0=0 +y_0=0 +units=m +no_defs',
        geoid: 'EGM96',
        recursive: true,
      },
      240_000,
    );
    expect(job.output).toContain('+proj=merc');
    expect(job.output).toContain('EGM96');
    expect(fs.existsSync(path.join(out, 'tileset.json')), job.output).toBeTruthy();
  });
});
