import { execFileSync } from 'child_process';
import fs from 'fs';
import path from 'path';

const E2E_DIR = __dirname;
const FIXTURES = path.join(E2E_DIR, 'fixtures');
const EMBEDDED_PYTHON = path.resolve(E2E_DIR, '..', 'server', 'tools', 'python', 'python.exe');

/** Generate binary fixtures (LAS) and per-converter input dirs before tests run. */
export default function globalSetup() {
  const las = path.join(FIXTURES, 'sample.las');
  if (!fs.existsSync(las)) {
    execFileSync(EMBEDDED_PYTHON, [path.join(FIXTURES, 'make_sample_las.py'), las], {
      stdio: 'inherit',
    });
  }

  const glb = path.join(FIXTURES, 'sample.glb');
  if (!fs.existsSync(glb)) {
    execFileSync(process.execPath, [path.join(FIXTURES, 'make_sample_glb.js'), glb], {
      stdio: 'inherit',
    });
  }

  const texDir = path.join(FIXTURES, 'fbx_tex_in');
  if (!fs.existsSync(path.join(texDir, 'sample_tex.fbx'))) {
    execFileSync(process.execPath, [path.join(FIXTURES, 'make_sample_fbx_tex.js'), texDir], {
      stdio: 'inherit',
    });
  }
  const nestedDir = path.join(FIXTURES, 'fbx_nested_in', 'sub');
  if (!fs.existsSync(path.join(nestedDir, 'sample_tex.fbx'))) {
    execFileSync(process.execPath, [path.join(FIXTURES, 'make_sample_fbx_tex.js'), nestedDir], {
      stdio: 'inherit',
    });
  }

  const copies: Array<[string, string]> = [
    ['mago_in', 'sample.obj'],
    ['auto_in', 'sample.las'],
    ['fbx_in', 'sample.fbx'],
    ['auto_fbx_in', 'sample.fbx'],
  ];
  for (const [dir, src] of copies) {
    const dirPath = path.join(FIXTURES, dir);
    fs.mkdirSync(dirPath, { recursive: true });
    const dest = path.join(dirPath, src);
    if (!fs.existsSync(dest)) {
      fs.copyFileSync(path.join(FIXTURES, src), dest);
    }
  }
}
