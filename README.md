# Kasuga Converter

自前環境で動作する、GIS データから **2D / 3D タイル** への変換システムです。

## 概要

- `mago-3d-tiler`
- `Py3DTiles`
- `gocesiumtiler`
- `pg2b3dm`
- `GDAL/PDAL 前処理`
- `Cesium Terrain`

などの変換エンジンを、Web UI からタブで切り替えて利用できます。Rust/Axum 製のローカルサーバー + 静的 HTML UI で構成されています。

## ドキュメント

詳細なセットアップ・使い方は **GitHub Pages** で公開しています。

👉 [docs/index.md](docs/index.md)（ローカル表示）

GitHub Pages を有効化する場合は、リポジトリ設定で `docs` フォルダをソースに指定してください。公開後は以下のような URL でアクセスできます。

```
https://<your-username>.github.io/kasuga_converter/
```

## クイックスタート

```powershell
cd server
cargo build
cargo run
```

ブラウザで `http://127.0.0.1:8590/` を開き、各タブから変換を実行します。初回は「データ変換」タブ内の「関連システム」から外部ツールを自動インストールできます。

## 画面構成

画面上部のトップタブから大きく 3 つのモードに切り替えます。

| トップタブ | 用途 |
|---|---|
| `データ取得` | CKAN カタログからデータセットを検索・ダウンロードできます。カテゴリ、検索語、形式フィルターで絞り込み可能です。1回に 100 件を取得し、表示件数は 1〜100 件で入力可能です（デフォルト 20 件）。 |
| `データ変換` | `自動変換` / `個別コンバータ` / `関連システム` の 3 つのタブに分かれます。 |
| `設定` | バージョン確認・更新確認・サーバー停止などを行います。 |

画面上部の `Kasuga Converter` タイトル右側には、処理状態（待機中 / 実行中 / 完了など）が表示されます。

`データ変換` タブ内では、次の 3 つのタブに分かれています。

- **自動変換** … 入力形式に応じて最適なコンバータを自動選択。形式別のルーティングは [docs/usage.md](docs/usage.md#自動変換の形式別ルーティング) を参照。
- **個別コンバータ** … `mago-3d-tiler` / `Py3DTiles` など、エンジンを個別に選択。
- **関連システム** … 外部ツールの状態確認・自動インストール。

## 主な機能

| タブ | エンジン | 用途 |
|------|----------|------|
| `mago-3d-tiler` | [mago-3d-tiler](https://github.com/Gaia3D/mago-3d-tiler) | 3DS/FBX/OBJ/glTF/GLB/LAS/LAZ/CityGML/IndoorGML/SHP/GeoJSON/GPKG → 3D Tiles 1.0（b3dm/i3dm/pnts） |
| `Py3DTiles` | [Py3DTiles](https://py3dtiles.org/) | LAS 点群など → 3D Tiles 1.0（pnts） |
| `gocesiumtiler` | [gocesiumtiler](https://github.com/mfbonfigli/gocesiumtiler) | LAS/LAZ 点群 → 3D Tiles 1.0（pnts） / 1.1（glb） |
| `pg2b3dm` | [pg2b3dm](https://github.com/Geodan/pg2b3dm) | PostGIS 3D ジオメトリ → 3D Tiles 1.0（b3dm） |
| `GDAL/PDAL` | GDAL / PDAL | 再投影、フォーマット変換、点群フィルタなどの前処理 |
| `Cesium Terrain` | Cesium Terrain Builder / tin-terrain | DEM ラスター → quantized-mesh terrain タイル |
| `3D Tiles 1.1` | 3d-tiles-tools | 3D Tiles 1.0 → 1.1 移行、b3dm → glb 変換 |
| `BIM/CIM` | IfcOpenShell / cjio | IFC / CityJSON → glTF/GLB/OBJ/CityGML |
| `2D 画像タイル` | gdal2tiles.py / ctb-tile | ラスター → XYZ/TMS 画像タイル |
| `glTF 最適化` | gltf-pipeline | glTF/GLB の Draco 圧縮・最適化 |

## 自動インストール対応

「データ変換」タブ内の「関連システム」から以下を自動ダウンロード・配置できます。

| ツール | 配置先 |
|--------|--------|
| JDK 21 | `tools/jdk-21` |
| mago-3d-tiler JAR | `tools/mago-3d-tiler.jar` |
| Python 3.12.4 | `tools/python` |
| Py3DTiles | `tools/python/Scripts/py3dtiles.exe` |
| pg2b3dm | `tools/pg2b3dm/pg2b3dm.exe` |
| gocesiumtiler | `tools/gocesiumtiler/gocesiumtiler.exe` |
| IfcOpenShell | `tools/ifcopenshell/IfcConvert.exe` |
| cjio | `tools/python/Scripts/cjio.exe` |

以下は手動でインストールしてください。

- **GDAL/PDAL** … OSGeo4W または conda-forge
- **Cesium Terrain Builder / tin-terrain** … Docker またはソースからビルド
- **gdal2tiles.py / ctb-tile** … GDAL または Cesium Terrain Builder に同梱
- **3d-tiles-tools** … `npm install -g 3d-tiles-tools`（Node.js が必要）
- **gltf-pipeline** … `npm install -g gltf-pipeline`（Node.js が必要）

## リリースビルド

```powershell
python run.py -b        # EXE + ZIP 生成
python run.py -b --installer  # ZIP + NSIS インストーラー生成
```

出力:

- `download/kasugai_converter.zip`
- `download/kasugai_converter_setup.exe`（要 NSIS）
- `download/kasugai_converter_setup.zip`

## ライセンス

本プロジェクト（Rust サーバー・Web UI）自身は **MIT License** で公開します。

なお、本ツールは以下の変換エンジンを自動ダウンロードして利用します。各エンジンのライセンスは該当リポジトリに従います。

| エンジン | ライセンス |
|----------|------------|
| [mago-3d-tiler](https://github.com/Gaia3D/mago-3d-tiler) | [MPL-2.0](https://github.com/Gaia3D/mago-3d-tiler/blob/main/LICENSE) |
| [Py3DTiles](https://py3dtiles.org/) | [Apache-2.0](https://gitlab.com/py3dtiles/py3dtiles/-/blob/master/LICENSE) |
| [pg2b3dm](https://github.com/Geodan/pg2b3dm) | [MIT](https://github.com/Geodan/pg2b3dm/blob/master/LICENSE) |
| [gocesiumtiler](https://github.com/mfbonfigli/gocesiumtiler) | [MPL-2.0](https://github.com/mfbonfigli/gocesiumtiler/blob/master/LICENSE) |
| [IfcOpenShell](https://github.com/IfcOpenShell/IfcOpenShell) | [LGPL-3.0](https://github.com/IfcOpenShell/IfcOpenShell/blob/master/COPYING.LESSER) |
| [cjio](https://github.com/cityjson/cjio) | [MIT](https://github.com/cityjson/cjio/blob/master/LICENSE) |
| [gltf-pipeline](https://github.com/CesiumGS/gltf-pipeline) | [Apache-2.0](https://github.com/CesiumGS/gltf-pipeline/blob/main/LICENSE.md) |

詳細は [docs/license.md](docs/license.md) を参照してください。
