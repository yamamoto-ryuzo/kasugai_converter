---
layout: default
title: 使い方
---

# 使い方

## 1. タブを選択する

画面上部のトップタブから、用途に応じたモードを選びます。

- `データ取得` … CKAN カタログと GraphQL API（国土交通データプラットフォーム/DPF）からデータセットを検索・ダウンロード
  - カタログは CKAN / GraphQL API の2タブで切り替え可能
  - DPF を利用するには「国土交通データプラットフォーム アカウント作成」から取得した API キーを入力
  - カテゴリ、検索語、形式フィルターで絞り込み可能
  - 検索結果は左にデータセット一覧、右に「データセットの説明」「検索されたデータ」「リソースURL」の3ペインで表示
  - 1回に 100 件を取得し、表示件数は 1〜100 件で入力可能（デフォルト 20 件）
  - 100 件を超える場合は次の 100 件を取得
  - 検索条件は「クリア」ボタンで初期化できる
- `前処理` … 変換前のデータ整形・タイル生成・最適化
  - `GDAL/PDAL`
  - `Cesium Terrain`
  - `2D 画像タイル`
  - `glTF 最適化`
  - `XY 反転`
- `データ変換` … 変換関連のタブに切り替え
  - `座標設定` … 共有する EPSG / CRS、原点経度・緯度、X/Y/Z オフセットを一括設定。自動変換・個別コンバータが参照します。
  - `自動変換` … 入力ファイル・ディレクトリを指定すると、形式から最適なコンバータを自動選択
  - `個別コンバータ` … 手動で変換エンジンを選択
    - `mago-3d-tiler`
    - `Py3DTiles`
    - `gocesiumtiler`
    - `pg2b3dm`
    - `3D Tiles 1.1`
    - `BIM/CIM`
  - `関連システム` … 外部ツールの状態確認・自動インストール。タブ単位で概要・変換対応拡張子を確認できます。未検出のタブは赤く表示されます。
- `設定` … バージョン確認・更新確認・サーバー停止

## 自動変換の形式別ルーティング

`自動変換` タブでは、入力の拡張子や構成から対象のコンバータを選択します。`出力形式` は「次元 / 用途」で分類したカテゴリタブから選びます。各選択肢には対応プラットフォーム（Cesium / QGIS / ArcGIS）を併記しています。`output_format` には純粋な形式コード（`geojson`、`gltf`、`b3dm` など）が送信されます。出力形式を選ぶと、その出力に変換可能な入力形式が `入力可能形式` 一覧に表示されます。

`座標設定` タブで設定した EPSG / CRS は、対応するコンバータに自動的に渡されます。

- `2D ベクター` : GeoJSON、KML / KMZ、GeoPackage、Shapefile、File Geodatabase / FGDB (ArcGIS / QGIS)、SpatiaLite
- `2D タイル` : XYZ / TMS、MVT
- `3D モデル` : glTF、OBJ、FBX、DAE
- `3D タイル` : b3dm、i3dm、pnts、glb (3D Tiles 1.1)
- `点群` : pnts、LAZ
- `地形` : quantized-mesh、DEM / GeoTIFF、terrain-rgb

現在実装されている `出力形式` とコンバータの対応は下表の通りです。

| 出力形式 | 選択されるコンバータ | 呼び出しルート | 備考 |
|---|---|---|---|
| `b3dm` | mago-3d-tiler | `/api/convert` | 3D Tiles 1.0 バッチモデル |
| `i3dm` | mago-3d-tiler | `/api/convert` | 3D Tiles 1.0 インスタンスモデル |
| `pnts` | gocesiumtiler | `/api/convert/gocesiumtiler` | 点群 3D Tiles |
| `3dtiles-1-1-glb` | 3D Tiles 1.1 変換 | `/api/convert/obj-3dtiles11` | OBJ → 3D Tiles 1.1 GLB |

`座標設定` タブで設定した EPSG / CRS は、上記の mago-3d-tiler / gocesiumtiler / 3D Tiles 1.1 変換に渡されます。その他の 2D 系・LAZ・3D モデル（gltf / obj / fbx / dae）・地形系の出力形式は、UI 上で選択できますが、現時点では自動変換ルーティングが未対応です。

## 2. 必要ツールをインストールする

初回は「データ変換」タブ内の「関連システム」から、各エンジンに必要なツールを自動インストールしてください。

自動インストールに対応しているもの:

- JDK 21
- mago-3d-tiler JAR
- Python 3.12.4
- Py3DTiles
- laspy（+ lazrs、XY 反転で使用）
- pg2b3dm
- gocesiumtiler
- IfcOpenShell
- cjio

手動でインストールが必要なもの:

- `GDAL/PDAL` … OSGeo4W / conda-forge
- `Cesium Terrain Builder / tin-terrain` … Docker / ソースからビルド
- `gdal2tiles.py / ctb-tile` … GDAL / Cesium Terrain Builder に同梱



## 3. 変換パラメータを入力する

各タブに応じた入力を行います。

### 座標設定

`座標設定` タブは、`自動変換` および `個別コンバータ` の該当エンジンで共有される座標系パラメータを一括設定します。

- **EPSG / SRS** … 使用する座標参照系を入力します。日本の平面直角座標系を含む主要な EPSG コードは入力候補に表示されます。
- **X / Y / Z オフセット** … モデルの原点に対する補正値（mago-3d-tiler / 3D Tiles 1.1 に渡されます）。
- **経度 / 緯度** … モデルの原点が置かれる地理座標（mago-3d-tiler / 3D Tiles 1.1 に渡されます）。平面直角座標系を EPSG で選ぶと、対応する原点経度・緯度が自動入力されます。

以下のエンジンが `座標設定` の値を参照します。

- mago-3d-tiler（`crs`、`xOffset`、`yOffset`、`zOffset`、`longitude`、`latitude`）
- Py3DTiles（`srs_in` / `srs_out`）
- gocesiumtiler（`epsg`）
- 3D Tiles 1.1 自動変換（`crs`）

### mago-3d-tiler

- 入力ディレクトリ
- 出力ディレクトリ
- 入力・出力形式
- 座標系 / オフセット / 経度・緯度（`座標設定` タブ）
- Java パス / JAR パス

### Py3DTiles

- 入力ファイル（例: `sample.las`）
- 出力ディレクトリ
- 入力 / 出力 SRS（`座標設定` タブ）
- コマンドパス

### gocesiumtiler

- 入力ファイル（例: `sample.las`）
- 出力ディレクトリ
- EPSG / CRS（`座標設定` タブ）
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

Node.js は「関連システム」から自動インストール可能です。`npx 3d-tiles-tools` を指定する場合、個別の `npm install` は不要です。

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

- コマンド: `npx gltf-pipeline`
- 入力: `C:/data/input.glb`
- 出力: `C:/data/output.glb`
- Draco 圧縮: 有効 / 無効
- 追加: `--keepUnusedElements`、`--textureCompression etc1s` など

Node.js は「関連システム」から自動インストール可能です。`npx gltf-pipeline` を指定する場合、個別の `npm install` は不要です。

### XY 反転

laspy を使って LAS/LAZ 点群の X 座標と Y 座標を入れ替えます。座標軸の取り違えがあるデータの補正に使用します。

- 入力点群ファイル: `C:/data/input.las`（`.las` / `.laz`）
- 出力点群ファイル: `C:/data/output.las`（`.las` / `.laz`。出力拡張子で LAS↔LAZ 変換も可能）
- 入力拡張子 / 出力拡張子: `自動判定` のままで通常は問題ありません
- Python パス: 省略時は `tools/python/python.exe` → `python` の順で自動選択

laspy（+ LAZ 用の lazrs）は「関連システム」の `laspy` タブから自動インストールできます。

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

# XY 反転ジョブの開始
curl -X POST http://127.0.0.1:8590/api/convert/xy-flip \
  -H "Content-Type: application/json" \
  -d '{"input":"C:/data/input.las","output":"C:/data/output.las"}'

# ジョブ状態の確認
curl http://127.0.0.1:8590/api/jobs/1
```
