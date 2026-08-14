#!/usr/bin/env python3
"""Kasugai Converter リリースビルドスクリプト"""

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
        print("[Kasugai] ビルドに失敗しました。")
        sys.exit(rc)
    if not TARGET_EXE.exists():
        print(f"[Kasugai] 出力 EXE が見つかりません: {TARGET_EXE}")
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
        print("[Kasugai] warning: static ディレクトリが見つかりません。")

    # ツールは実行時に自動ダウンロードされるが、ディレクトリだけは作っておく
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

    print(f"[Kasugai] リリース ZIP を生成しました: {zip_path}")
    return zip_path


def build_installer():
    """Build NSIS installer if the .nsi file and makensis are available."""
    nsi = PROJECT_ROOT / "installer" / f"{APP_NAME}.nsi"
    if not nsi.exists():
        print(f"[Kasugai] インストーラースクリプトが見つかりません: {nsi}")
        return 1
    if shutil.which("makensis") is None:
        print("[Kasugai] makensis が見つかりません。NSIS をインストールしてください。")
        return 1
    return run_command(["makensis", str(nsi)], cwd=PROJECT_ROOT)


def run_release():
    """Run the release EXE."""
    if not TARGET_EXE.exists():
        print(f"[Kasugai] リリース EXE が見つかりません: {TARGET_EXE}")
        print("[Kasugai] 先に `python run.py -b` でビルドしてください。")
        sys.exit(1)
    sys.exit(run_command([str(TARGET_EXE)], cwd=SERVER_DIR))


def main():
    parser = argparse.ArgumentParser(description="Kasugai Converter リリースビルドスクリプト")
    parser.add_argument(
        "cmd",
        nargs="?",
        choices=["b", "B"],
        help="`b` または `B` でリリースビルドを実行します。",
    )
    parser.add_argument("-b", "-B", "--build", action="store_true", help="リリースビルド・ZIP 化")
    parser.add_argument("--installer", action="store_true", help="NSIS インストーラーを作成")
    parser.add_argument("--release", action="store_true", help="リリース EXE を起動")
    args = parser.parse_args()

    if args.build or args.cmd in ("b", "B"):
        build_release()
        package_zip()
        if args.installer:
            build_installer()
    elif args.installer:
        build_installer()
    elif args.release:
        run_release()
    else:
        # 引数なし: 開発モード起動
        sys.exit(run_command(["cargo", "run"], cwd=SERVER_DIR))


if __name__ == "__main__":
    main()
