#!/usr/bin/env python3
"""Kasugai Converter release build script."""

import argparse
import os
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

APP_NAME = "kasugai_converter"
PROJECT_ROOT = Path(__file__).resolve().parent
SERVER_DIR = PROJECT_ROOT / "server"
TARGET_EXE = SERVER_DIR / "target" / "release" / f"{APP_NAME}.exe"
DOWNLOAD_DIR = PROJECT_ROOT / "download"
DIST_DIR = PROJECT_ROOT / "dist"


def run_command(cmd, cwd=None):
    """Run a command and return its return code."""
    print(f"[Kasugai] {' '.join(map(str, cmd))}")
    return subprocess.run(cmd, cwd=cwd, check=False).returncode


def build_release():
    """Build the Rust server in release mode."""
    rc = run_command(["cargo", "build", "--release"], cwd=SERVER_DIR)
    if rc != 0:
        print("[Kasugai] Build failed.")
        sys.exit(rc)
    if not TARGET_EXE.exists():
        print(f"[Kasugai] Output EXE not found: {TARGET_EXE}")
        sys.exit(1)


def package_zip():
    """Package release EXE and static assets into download/<app>.zip."""
    if DIST_DIR.exists():
        shutil.rmtree(DIST_DIR)
    DIST_DIR.mkdir(parents=True, exist_ok=True)
    DOWNLOAD_DIR.mkdir(parents=True, exist_ok=True)

    shutil.copy(TARGET_EXE, DIST_DIR / f"{APP_NAME}.exe")
    static_src = SERVER_DIR / "static"
    if static_src.exists():
        shutil.copytree(static_src, DIST_DIR / "static")
    else:
        print("[Kasugai] warning: static directory not found.")

    resources_src = SERVER_DIR / "resources"
    if resources_src.exists():
        shutil.copytree(resources_src, DIST_DIR / "resources")
    else:
        print("[Kasugai] warning: resources directory not found.")

    (DIST_DIR / "tools").mkdir(exist_ok=True)

    zip_path = DOWNLOAD_DIR / f"{APP_NAME}.zip"
    if zip_path.exists():
        zip_path.unlink()

    with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as zf:
        for root, _, files in os.walk(DIST_DIR):
            for file in files:
                file_path = Path(root) / file
                arcname = str(file_path.relative_to(DIST_DIR))
                zf.write(file_path, arcname)

    print(f"[Kasugai] Release ZIP created: {zip_path}")
    return zip_path


def find_makensis():
    """Find makensis.exe either in PATH or common install locations."""
    found = shutil.which("makensis")
    if found:
        return found
    for p in [
        r"C:\Program Files (x86)\NSIS\makensis.exe",
        r"C:\Program Files\NSIS\makensis.exe",
        r"C:\nsis\makensis.exe",
        r"C:\Tools\NSIS\makensis.exe",
    ]:
        if Path(p).exists():
            return p
    return None


def build_installer():
    """Build NSIS installer and wrap it into a ZIP."""
    nsi = PROJECT_ROOT / "installer" / f"{APP_NAME}.nsi"
    if not nsi.exists():
        print(f"[Kasugai] Installer script not found: {nsi}")
        return 1
    makensis = find_makensis()
    if makensis is None:
        print("[Kasugai] makensis not found. Install NSIS to build the installer.")
        return 1

    rc = run_command([makensis, "-INPUTCHARSET", "UTF8", str(nsi)], cwd=PROJECT_ROOT / "installer")
    if rc != 0:
        return rc

    installer_exe = DOWNLOAD_DIR / f"{APP_NAME}_setup.exe"
    if not installer_exe.exists():
        print(f"[Kasugai] Installer EXE not found: {installer_exe}")
        return 1

    dest_zip = DOWNLOAD_DIR / f"{APP_NAME}_setup.zip"
    if dest_zip.exists():
        dest_zip.unlink()

    with zipfile.ZipFile(dest_zip, "w", zipfile.ZIP_DEFLATED) as zf:
        zf.write(installer_exe, installer_exe.name)

    print(f"[Kasugai] Installer EXE created: {installer_exe}")
    print(f"[Kasugai] Installer ZIP created: {dest_zip}")
    return 0


def run_release():
    """Run the release EXE."""
    if not TARGET_EXE.exists():
        print(f"[Kasugai] Release EXE not found: {TARGET_EXE}")
        print("[Kasugai] Run `python run.py -b` first.")
        sys.exit(1)
    sys.exit(run_command([str(TARGET_EXE)], cwd=SERVER_DIR))


def main():
    parser = argparse.ArgumentParser(description="Kasugai Converter release build script")
    parser.add_argument(
        "cmd",
        nargs="?",
        choices=["b", "B"],
        help="`b` or `B` to run release build.",
    )
    parser.add_argument("-b", "-B", "--build", action="store_true", help="Release build and ZIP packaging")
    parser.add_argument("--installer", action="store_true", help="Build NSIS installer after packaging")
    parser.add_argument("--release", action="store_true", help="Run the release EXE")
    args = parser.parse_args()

    wants_installer = args.installer
    build_requested = args.build or args.cmd in ("b", "B")

    if args.release:
        run_release()
    elif build_requested or wants_installer:
        build_release()
        package_zip()
        if wants_installer:
            rc = build_installer()
            if rc != 0:
                sys.exit(rc)
    else:
        # 引数なし: 開発モード起動
        sys.exit(run_command(["cargo", "run"], cwd=SERVER_DIR))


if __name__ == "__main__":
    main()
