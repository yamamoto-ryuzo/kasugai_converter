---
layout: default
title: トップ
---

# Kasuga Converter

自前環境で動作する、GIS データから **2D / 3D タイル** への変換システムです。

Web ブラウザから複数の変換エンジンをタブで切り替えて利用できます。

## 画面構成

画面上部のトップタブで、大きく次の 4 つのモードに切り替えます。

| トップタブ | 用途 |
|---|---|
| `データ取得` | CKAN カタログと GraphQL API（国土交通データプラットフォーム/DPF）からデータセットを検索・ダウンロードできます。カテゴリ、検索語、形式フィルターで絞り込み可能です。検索結果は左にデータセット一覧、右に「データセットの説明」「検索されたデータ」「リソースURL」の3ペインで表示されます。1回に 100 件を取得し、表示件数は 1〜100 件で入力可能です（デフォルト 20 件）。検索条件は「クリア」ボタンで初期化できます。 |
| `前処理` | `GDAL/PDAL` / `Cesium Terrain` / `2D 画像タイル` / `glTF 最適化` / `XY 反転` / `OBJ 表面処理` の 6 つのタブに分かれます。変換前のデータ整形・タイル生成・最適化を行います。 |
| `データ変換` | `座標設定` / `自動変換` / `個別コンバータ` / `関連システム` の 4 つのタブに分かれます。 |
| `設定` | バージョン確認・更新確認・サーバー停止などを行います。 |

画面上部の `Kasuga Converter` タイトル右側には、処理状態（待機中 / 実行中 / 完了など）が表示されます。

`データ変換` タブ内では、次の 4 つのタブに分かれています。

| タブ | 用途 |
|---|---|
| `座標設定` | 共有する EPSG / CRS、原点経度・緯度、X/Y/Z オフセットを一括設定。自動変換・個別コンバータが参照します。 |
| `自動変換` | 入力形式から最適なコンバータを自動選択。形式別のルーティングは [usage.md](usage#自動変換の形式別ルーティング) を参照。 |
| `個別コンバータ` | mago-3d-tiler など、変換エンジンを個別に選択。 |
| `関連システム` | 外部ツールの状態確認・自動インストール。各ツールをタブで切り替え、概要と変換対応拡張子を確認できます。未検出のタブは赤く表示されます。 |

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
| `BIM/CIM` | IfcOpenShell / cjio | IFC / CityJSON から glTF/GLB/OBJ/CityGML へ |
| `2D 画像タイル` | gdal2tiles.py / ctb-tile | ラスターから XYZ/TMS 画像タイルへ |
| `glTF 最適化` | gltf-pipeline | glTF/GLB の Draco 圧縮・最適化 |
| `XY 反転` | laspy | LAS/LAZ 点群の X 座標と Y 座標を入れ替え |
| `OBJ 表面処理` | なし | OBJ ファイルの面の向き、法線、U/V UV 座標を反転 |

## 自動インストール

「データ変換」タブ内の「関連システム」から、タブ単位で以下を自動ダウンロード・配置できます。各タブには概要・変換対応拡張子が表示されます。

| ツール | 配置先 |
|--------|--------|
| JDK 21 | `tools/jdk-21` |
| mago-3d-tiler JAR | `tools/mago-3d-tiler.jar` |
| Python 3.12.4 | `tools/python` |
| Py3DTiles | `tools/python/Scripts/py3dtiles.exe` |
| laspy（+ lazrs） | `tools/python` 内の Python パッケージ |
| pg2b3dm | `tools/pg2b3dm/pg2b3dm.exe` |
| gocesiumtiler | `tools/gocesiumtiler/gocesiumtiler.exe` |
| IfcOpenShell | `tools/ifcopenshell/IfcConvert.exe` |
| cjio | `tools/python/Scripts/cjio.exe` |
| Node.js（LTS） | `tools/node` |

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
