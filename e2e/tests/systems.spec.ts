import { test, expect } from '@playwright/test';
import { openSubTab, openTopTab } from './helpers';

const SYSTEMS = [
  { tab: 'system-jdk', tag: 'jdkStatusTag' },
  { tab: 'system-mago', tag: 'magoStatusTag' },
  { tab: 'system-python', tag: 'pythonStatusTag' },
  { tab: 'system-py3dtiles', tag: 'py3dtilesStatusTag' },
  { tab: 'system-laspy', tag: 'laspyStatusTag' },
  { tab: 'system-pg2b3dm', tag: 'pg2b3dmStatusTag' },
  { tab: 'system-gocesiumtiler', tag: 'gocesiumtilerStatusTag' },
  { tab: 'system-ifcopenshell', tag: 'ifcopenshellStatusTag' },
  { tab: 'system-cjio', tag: 'cjioStatusTag' },
  { tab: 'system-node', tag: 'nodeStatusTag' },
];

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await openTopTab(page, 'data-convert');
  await openSubTab(page, 'systems');
});

test('関連システムの環境が検出される', async ({ page }) => {
  for (const { tab, tag } of SYSTEMS) {
    await openSubTab(page, tab);
    await expect(page.locator(`#${tag}`)).toContainText(/検出済|未検出/, { timeout: 30_000 });
  }
});

// tools/ 配下に同梱済みのツールは「検出済」になるべき
for (const { tab, tag } of SYSTEMS) {
  test(`${tab} が検出済になる`, async ({ page }) => {
    await openSubTab(page, tab);
    await expect(page.locator(`#${tag}`)).toContainText('検出済', { timeout: 30_000 });
  });
}

test('検出済タブのタブボタンが ok クラスになる', async ({ page }) => {
  await expect(page.locator('.tab-btn[data-tab="system-jdk"]')).toHaveClass(/ok/, { timeout: 30_000 });
});
