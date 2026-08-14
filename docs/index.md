---
layout: default
title: トップ
---

# Kasuga Converter

自前環境で動作する、GIS データから **OGC 3D Tiles** への変換システムです。

Web ブラウザから複数の変換エンジンをタブで切り替えて利用できます。

## 対応変換エンジン

| タブ | エンジン | 用途 |
|------|----------|------|
| `mago-3d-tiler` | [mago-3d-tiler](https://github.com/Gaia3D/mago-3d-tiler) | 3DS/FBX/OBJ/glTF/GLB/LAS/LAZ/CityGML/IndoorGML/SHP/GeoJSON/GPKG → 3D Tiles 1.0 |
| `Py3DTiles` | [Py3DTiles](https://py3dtiles.org/) | LAS 点群を 3D Tiles 1.0（pnts）へ |
| `gocesiumtiler` | [gocesiumtiler](https://github.com/mfbonfigli/gocesiumtiler) | LAS/LAZ 点群を 3D Tiles 1.0 / 1.1（pnts / glb）へ |
| `pg2b3dm` | [pg2b3dm](https://github.com/Geodan/pg2b3dm) | PostGIS 3D ジオメトリを 3D Tiles 1.0（b3dm）へ |
| `GDAL/PDAL` | GDAL / PDAL | 再投影、フォーマット変換、点群フィルタなどの前処理 |
| `Cesium Terrain` | Cesium Terrain Builder / tin-terrain | DEM ラスターを quantized-mesh terrain タイルへ |
| `3D Tiles 1.1` | 3d-tiles-tools | 3D Tiles 1.0 から 1.1 GLB への移行・変換 |

## 自動インストール

「関連システム」タブから以下を自動ダウンロード・配置できます。

| ツール | 配置先 |
|--------|--------|
| JDK 21 | `tools/jdk-21` |
| mago-3d-tiler JAR | `tools/mago-3d-tiler.jar` |
| Python 3.12.4 | `tools/python` |
| Py3DTiles | `tools/python/Scripts/py3dtiles.exe` |
| pg2b3dm | `tools/pg2b3dm/pg2b3dm.exe` |
| gocesiumtiler | `tools/gocesiumtiler/gocesiumtiler.exe` |

## クイックスタート

```powershell
cd server
cargo build
cargo run
```

ブラウザで `http://127.0.0.1:8590/` を開きます。

## 詳細

- [セットアップ](setup)
- [使い方](usage)
- [ライセンス](license)
