import { test, expect } from '@playwright/test';
import { openSubTab, openTopTab } from './helpers';

test('トップページが表示され、初期状態は待機中', async ({ page }) => {
  await page.goto('/');
  await expect(page).toHaveTitle('Kasuga Converter');
  await expect(page.locator('#status')).toContainText('待機中');
  await expect(page.locator('#data-get')).toHaveClass(/active/);
});

test('トップタブが切り替わる', async ({ page }) => {
  await page.goto('/');
  for (const tab of ['data-preprocess', 'data-convert', 'settings', 'data-get']) {
    await openTopTab(page, tab);
  }
});

test('データ変換のサブタブが切り替わる', async ({ page }) => {
  await page.goto('/');
  await openTopTab(page, 'data-convert');
  for (const tab of ['coord', 'individual', 'systems', 'auto']) {
    await openSubTab(page, tab);
  }
});

test('個別コンバータのエンジンタブが切り替わる', async ({ page }) => {
  await page.goto('/');
  await openTopTab(page, 'data-convert');
  await openSubTab(page, 'individual');
  for (const tab of ['py3dtiles', 'pg2b3dm', 'gocesiumtiler', 'tiles11', 'bimcim', 'mago']) {
    await openSubTab(page, tab);
  }
});

test('前処理のサブタブが切り替わる', async ({ page }) => {
  await page.goto('/');
  await openTopTab(page, 'data-preprocess');
  for (const tab of ['terrain', 'imagery', 'gltfopt', 'xyflip', 'obj-surface', 'preprocess']) {
    await openSubTab(page, tab);
  }
});

test('設定タブにバージョンが表示される', async ({ page }) => {
  await page.goto('/');
  await openTopTab(page, 'settings');
  await expect(page.locator('#appVersion')).toContainText(/^\d+\.\d+\.\d+$/);
});

test('API /api/v1/server/version がバージョンを返す', async ({ request }) => {
  const res = await request.get('/api/v1/server/version');
  expect(res.ok()).toBeTruthy();
  const body = await res.json();
  expect(body.version).toMatch(/^\d+\.\d+\.\d+$/);
});
