---
layout: default
title: 使い方
---

# 使い方

## 1. タブを選択する

画面上部のトップタブから、用途に応じたモードを選びます。

- `データ取得` … CKAN カタログからデータセットを検索・ダウンロード
- `データ変換` … 変換関連のタブに切り替え
  - `自動変換` … 入力ファイル・ディレクトリを指定すると、形式から最適なコンバータを自動選択
  - `個別コンバータ` … 手動で変換エンジンを選択
    - `mago-3d-tiler`
    - `Py3DTiles`
    - `gocesiumtiler`
    - `pg2b3dm`
    - `GDAL/PDAL`
    - `Cesium Terrain`
    - `3D Tiles 1.1`
    - `BIM/CIM`
    - `2D 画像タイル`
    - `glTF 最適化`
  - `関連システム` … 外部ツールの状態確認・自動インストール
- `設定` … バージョン確認・更新確認・サーバー停止

## 自動変換の形式別ルーティング

> **注意**: 現時点で `/api/convert/auto` のバックエンド処理は未実装です。実行すると `auto converter not yet implemented` が返されます。以下のマッピングは将来の実装を見据えた想定です。

`自動変換` タブでは、入力の拡張子や構成から対象のコンバータを選択します。`出力形式` は「次元 / 用途」で分類したカテゴリタブから選びます。各選択肢には対応プラットフォーム（Cesium / QGIS / ArcGIS）を併記しています。`output_format` には純粋な形式コード（`geojson`、`gltf`、`b3dm` など）が送信されます。出力形式を選ぶと、その出力に変換可能な入力形式が `入力可能形式` 一覧に表示されます。

- `2D ベクター` : GeoJSON、KML / KMZ、GeoPackage、Shapefile、File Geodatabase / FGDB (ArcGIS / QGIS)、SpatiaLite
- `2D タイル` : XYZ / TMS、MVT
- `3D モデル` : glTF、OBJ、FBX、DAE
- `3D タイル` : b3dm、i3dm、pnts、glb (3D Tiles 1.1)
- `点群` : pnts、LAZ
- `地形` : quantized-mesh、DEM / GeoTIFF、terrain-rgb

MVT、OBJ、FBX、DAE、LAZ、terrain-rgb などは現状のコンバーターでは未対応ですが、将来の標準拡張として UI 上に配置しています。空欄の場合は自動判定されます。現時点では下表のマッピングを想定しています。

| 入力形式 | 拡張子・目印 | 選択されるコンバータ | 呼び出しルート |
|---|---|---|---|
| Shapefile | `.shp` | mago-3d-tiler | `/api/convert` |
| GeoJSON | `.geojson` | mago-3d-tiler | `/api/convert` |
| GeoPackage | `.gpkg` | mago-3d-tiler | `/api/convert` |
| KML | `.kml` | mago-3d-tiler | `/api/convert` |
| CityGML | `.gml`, `.citygml` | mago-3d-tiler | `/api/convert` |
| IndoorGML | `.igml` | mago-3d-tiler | `/api/convert` |
| OBJ | `.obj` | mago-3d-tiler | `/api/convert` |
| FBX | `.fbx` | mago-3d-tiler | `/api/convert` |
| 3DS | `.3ds` | mago-3d-tiler | `/api/convert` |
| glTF / GLB | `.gltf`, `.glb` | mago-3d-tiler | `/api/convert` |
| LAS / LAZ | `.las`, `.laz` | Py3DTiles / gocesiumtiler | `/api/convert/py3dtiles` または `/api/convert/gocesiumtiler` |
| PostGIS | PostgreSQL 接続文字列 | pg2b3dm | `/api/convert/pg2b3dm` |
| DEM GeoTIFF | `.tif` など（標高） | Cesium Terrain | `/api/run/preprocess` |
| ラスター画像 | `.tif`, `.png`, `.jpg` など | 2D 画像タイル | `/api/run/preprocess` |
| IFC | `.ifc` | BIM/CIM（IfcConvert） | `/api/convert/bimcim` |
| CityJSON | `.json`（CityJSON） | BIM/CIM（cjio） | `/api/convert/bimcim` |
| glTF 最適化 | `.gltf`, `.glb` | glTF 最適化 | `/api/run/preprocess` |

## 2. 必要ツールをインストールする

初回は「データ変換」タブ内の「関連システム」から、各エンジンに必要なツールを自動インストールしてください。

自動インストールに対応しているもの:

- JDK 21
- mago-3d-tiler JAR
- Python 3.12.4
- Py3DTiles
- pg2b3dm
- gocesiumtiler
- IfcOpenShell
- cjio

手動でインストールが必要なもの:

- `GDAL/PDAL` … OSGeo4W / conda-forge
- `Cesium Terrain Builder / tin-terrain` … Docker / ソースからビルド
- `gdal2tiles.py / ctb-tile` … GDAL / Cesium Terrain Builder に同梱
- `3d-tiles-tools` … `npm install -g 3d-tiles-tools`
- `gltf-pipeline` … `npm install -g gltf-pipeline`

## 3. 変換パラメータを入力する

各タブに応じた入力を行います。

### mago-3d-tiler

- 入力ディレクトリ
- 出力ディレクトリ
- 入力・出力形式
- CRS
- Java パス / JAR パス

### Py3DTiles

- 入力ファイル（例: `sample.las`）
- 出力ディレクトリ
- 入力 / 出力 SRS
- コマンドパス

### gocesiumtiler

- 入力ファイル（例: `sample.las`）
- 出力ディレクトリ
- EPSG / CRS
- **3D Tiles バージョン（`1.0` または `1.1`）**
- コマンドパス

### pg2b3dm

- PostgreSQL 接続文字列
- テーブル名、ジオメトリ列
- 属性列、出力ディレクトリ
- コマンドパス

### GDAL/PDAL

- プログラム（例: `gdalwarp`、`pdal translate`）
- 入力、出力
- 追加オプション

### Cesium Terrain

- コマンド（例: `ctb-tile`、`tin-terrain`）
- 入力 DEM
- 出力ディレクトリ
- 出力形式、プロファイル、ズーム範囲

### 3D Tiles 1.1

3d-tiles-tools を使って 3D Tiles 1.0 を 1.1 化します。

- コマンド（例: `3d-tiles-tools`、`npx 3d-tiles-tools`）
- 処理（例: `upgrade`、`b3dmToGlb`、`convertB3dmToGlb`、`optimizeGlb`）
- 入力ファイルまたは `tileset.json`
- 出力ディレクトリ
- 追加オプション（例: `--targetVersion 1.1`）

Node.js がインストール済みの環境で `npm install -g 3d-tiles-tools` してください。

### BIM/CIM

IFC や CityJSON を glTF/GLB/OBJ/CityGML に変換します。

- ツール: `IfcConvert` または `cjio`
- 入力ファイル: `C:/data/building.ifc` または `C:/data/city.json`
- 出力ファイル: `C:/data/building.glb`
- 出力形式（cjio）: `glb`、`obj`、`citygml` など

変換結果は `mago-3d-tiler` タブへ入力して 3D Tiles 化できます。

### 2D 画像タイル

衛星写真や航空写真を Cesium 用の XYZ/TMS 2D 画像タイルに変換します。

- コマンド: `gdal2tiles.py` または `ctb-tile`
- 入力: `C:/data/ortho.tif`
- 出力ディレクトリ: `C:/data/imagery`
- プロファイル: `mercator` または `geodetic`
- ズーム範囲: `5-18`
- 画像形式: `png` / `jpg`

GDAL/ctb-tile は OSGeo4W などから事前にインストールしてください。

### glTF 最適化

gltf-pipeline を使って glTF/GLB を最適化します。

- コマンド: `gltf-pipeline`
- 入力: `C:/data/input.glb`
- 出力: `C:/data/output.glb`
- Draco 圧縮: 有効 / 無効
- 追加: `--keepUnusedElements`、`--textureCompression etc1s` など

Node.js 環境で `npm install -g gltf-pipeline` してください。

## 4. 実行

`実行` ボタンを押すと、バックエンドでジョブが作成されます。
下部のログエリアで進捗と結果を確認できます。

## API

バックエンドは REST API としても利用可能です。

```bash
# 自動変換（output_format は省略可）
curl -X POST http://127.0.0.1:8590/api/convert/auto \
  -H "Content-Type: application/json" \
  -d '{"input":"C:/data/input","output":"C:/out","output_format":"geojson"}'

# 変換ジョブの開始
curl -X POST http://127.0.0.1:8590/api/convert/py3dtiles \
  -H "Content-Type: application/json" \
  -d '{"input":"C:/data/sample.las","output":"C:/out/tiles"}'

# 前処理ジョブの開始
curl -X POST http://127.0.0.1:8590/api/run/preprocess \
  -H "Content-Type: application/json" \
  -d '{"program":"gdalwarp","input":"C:/data/in.tif","output":"C:/data/out.tif","extra_args":"-t_srs EPSG:4326"}'

# ジョブ状態の確認
curl http://127.0.0.1:8590/api/jobs/1
```
