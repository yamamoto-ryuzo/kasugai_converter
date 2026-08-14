---
layout: default
title: トップ
---

# Kasuga Converter

自前環境で動作する、GIS データから **OGC 3D Tiles** への変換システムです。

Web ブラウザから複数の変換エンジンを切り替えて利用できます。

## 対応変換エンジン

| エンジン | 用途 |
|----------|------|
| [mago-3d-tiler](https://github.com/Gaia3D/mago-3d-tiler) | 3DS/FBX/OBJ/glTF/GLB/LAS/LAZ/CityGML/IndoorGML/SHP/GeoJSON/GPKG など |
| [Py3DTiles](https://py3dtiles.org/) | LAS 点群などを 3D Tiles へ |
| [pg2b3dm](https://github.com/Geodan/pg2b3dm) | PostGIS 3D ジオメトリを 3D Tiles へ |

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
