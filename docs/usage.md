---
layout: default
title: 使い方
---

# 使い方

## 1. タブを選択する

ブラウザで開いた画面上部のタブから、使いたい変換エンジンを選びます。

- `mago-3d-tiler`
- `Py3DTiles`
- `gocesiumtiler`
- `pg2b3dm`
- `GDAL/PDAL`
- `Cesium Terrain`
- `関連システム`

## 2. 必要ツールをインストールする

初回は「関連システム」タブから、各エンジンに必要なツールを自動インストールしてください。

自動インストールに対応しているもの:

- JDK 21
- mago-3d-tiler JAR
- Python 3.12.4
- Py3DTiles
- pg2b3dm
- gocesiumtiler

`GDAL/PDAL`、`Cesium Terrain` は各自で OSGeo4W、conda-forge、Docker などからツールをインストールしてください。

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

### Imagery

衛星写真や航空写真を Cesium 用の画像タイルに変換します。

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
