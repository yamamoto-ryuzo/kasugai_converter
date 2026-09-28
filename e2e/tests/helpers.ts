import { APIRequestContext, expect, Page } from '@playwright/test';
import fs from 'fs';
import path from 'path';

export const E2E_DIR = path.resolve(__dirname, '..');
export const FIXTURES = path.join(E2E_DIR, 'fixtures');
export const WORK = path.join(E2E_DIR, 'work');

/** Recreate a clean working directory under e2e/work. */
export function workDir(name: string): string {
  const dir = path.join(WORK, name);
  fs.rmSync(dir, { recursive: true, force: true });
  fs.mkdirSync(dir, { recursive: true });
  return dir;
}

export async function openTopTab(page: Page, tab: string) {
  await page.locator(`.top-tab-btn[data-top-tab="${tab}"]`).click();
  await expect(page.locator(`#${tab}`)).toHaveClass(/active/);
}

export async function openSubTab(page: Page, tab: string) {
  await page.locator(`.tab-btn[data-tab="${tab}"]`).click();
  await expect(page.locator(`[id="${tab}"]`)).toHaveClass(/active/);
}

export interface JobResult {
  statusText: string;
  log: string;
  ok: boolean;
}

/**
 * Click a run button and wait for the shared status badge to reach a
 * terminal state (完了 / 失敗 / エラー). Returns the final status + log text.
 */
export async function runJob(page: Page, buttonId: string, timeout = 120_000): Promise<JobResult> {
  await page.locator(`#${buttonId}`).click();
  await page.waitForFunction(
    () => /完了|失敗|エラー|ポーリングエラー/.test(document.getElementById('status')?.textContent ?? ''),
    undefined,
    { timeout },
  );
  const statusText = (await page.locator('#status').textContent()) ?? '';
  const log = (await page.locator('#log').textContent()) ?? '';
  return { statusText, log, ok: statusText.includes('完了') };
}

/**
 * POST a conversion job through the REST API and poll until it finishes.
 * Returns the final job object; throws on failure/timeout.
 */
export async function apiRunJob(
  request: APIRequestContext,
  endpoint: string,
  payload: Record<string, unknown>,
  timeout = 120_000,
) {
  const res = await request.post(endpoint, { data: payload });
  if (!res.ok()) {
    throw new Error(`POST ${endpoint} -> ${res.status()}: ${await res.text()}`);
  }
  const { job_id } = await res.json();
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    const jr = await request.get(`/api/jobs/${job_id}`);
    const job = await jr.json();
    if (job.status === 'completed') return job;
    if (job.status === 'failed') {
      throw new Error(`job ${job_id} failed:\n${job.output}`);
    }
    await new Promise((r) => setTimeout(r, 1_000));
  }
  throw new Error(`job ${job_id} timed out`);
}
