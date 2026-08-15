---
layout: default
title: セットアップ
---

# セットアップ

## 前提

- Windows 10/11
- Rust + Cargo
- PowerShell

## サーバーの起動

```powershell
cd server
cargo build
cargo run
```

`http://127.0.0.1:8590/` で UI にアクセスできます。

## 必要ツールの自動インストール

UI の「データ変換」タブ内の「関連システム」から、以下を自動ダウンロード・配置できます。

| ツール | ボタン | 配置先 |
|--------|--------|--------|
| JDK 21 | 自動インストール | `tools/jdk-21` |
| mago-3d-tiler JAR | 自動ダウンロード | `tools/mago-3d-tiler.jar` |
| Python 3.12.4 | 自動ダウンロード | `tools/python` |
| Py3DTiles | 自動インストール | `tools/python/Scripts/py3dtiles.exe` |
| pg2b3dm | 自動ダウンロード | `tools/pg2b3dm/pg2b3dm.exe` |
| gocesiumtiler | 自動ダウンロード | `tools/gocesiumtiler/gocesiumtiler.exe` |
| IfcOpenShell | 自動ダウンロード | `tools/ifcopenshell/IfcConvert.exe` |
| cjio | 自動インストール | `tools/python/Scripts/cjio.exe` |

## 手動でパスを指定する場合

環境変数を利用できます。

```powershell
$env:MAGO_JAVA_PATH = "C:\tools\jdk-21\bin\java.exe"
$env:MAGO_JAR_PATH = "C:\tools\mago-3d-tiler.jar"
$env:PY3DTILES_PATH = "C:\tools\py3dtiles-venv\Scripts\py3dtiles.exe"
$env:PG2B3DM_PATH = "C:\tools\pg2b3dm\pg2b3dm.exe"
$env:GOCESIUMTILER_PATH = "C:\tools\gocesiumtiler\gocesiumtiler.exe"
$env:IFCCONVERT_PATH = "C:\tools\ifcopenshell\IfcConvert.exe"
$env:CJIO_PATH = "C:\tools\python\Scripts\cjio.exe"
```

## GDAL/PDAL・Cesium Terrain ツールのインストール

| 用途 | 推奨方法 |
|------|----------|
| GDAL/PDAL | OSGeo4W または conda-forge |
| Cesium Terrain Builder | Docker、または[ quantized-mesh 対応フォーク](https://github.com/tum-gis/cesium-terrain-builder-docker) |
| tin-terrain | Docker、またはソースからビルド |
| gdal2tiles.py / ctb-tile | GDAL / Cesium Terrain Builder に同梱 |
| 3d-tiles-tools | `npm install -g 3d-tiles-tools`（Node.js が必要） |
| gltf-pipeline | `npm install -g gltf-pipeline`（Node.js が必要） |

## リリースビルド

```powershell
python run.py -b              # EXE + ZIP 生成
python run.py -b --installer  # ZIP + NSIS インストーラー生成
python run.py                 # 開発モード起動
```

生成物:

- `download/kasugai_converter.zip`
- `download/kasugai_converter_setup.exe`（NSIS インストーラー）
- `download/kasugai_converter_setup.zip`
