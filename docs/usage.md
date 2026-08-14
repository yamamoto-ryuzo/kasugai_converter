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

### Py3DTiles / gocesiumtiler

- 入力ファイル（例: `sample.las`）
- 出力ディレクトリ
- EPSG / CRS
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
