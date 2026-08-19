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
| `データ取得` | CKAN カタログと GraphQL API（国土交通データプラットフォーム/DPF）からデータセットを検索・ダウンロードできます。カテゴリ、検索語、形式フィルターで絞り込み可能です。検索結果は左にデータセット一覧、右に「データセットの説明」「検索されたデータ」「リソースURL」の3ペインで表示されます。1回に 100 件を取得し、表示件数は 1〜100 件で入力可能です（デフォルト 20 件）。 |
| `データ変換` | `自動変換` / `個別コンバータ` / `関連システム` の 3 つのタブに分かれます。 |
| `設定` | バージョン確認・更新確認・サーバー停止などを行います。 |

画面上部の `Kasuga Converter` タイトル右側には、処理状態（待機中 / 実行中 / 完了など）が表示されます。

`データ変換` タブ内では、次の 4 つのタブに分かれています。

- **座標設定** … 共有する EPSG / CRS と原点座標（経度・緯度）、および X/Y/Z オフセットを一括設定。`自動変換` および `個別コンバータ` の該当エンジンに反映されます。
- **自動変換** … 入力形式に応じて最適なコンバータを自動選択。形式別のルーティングは [docs/usage.md](docs/usage.md#自動変換の形式別ルーティング) を参照。
- **個別コンバータ** … `mago-3d-tiler` / `Py3DTiles` / `gocesiumtiler` など、エンジンを個別に選択。
- **関連システム** … 外部ツールの状態確認・自動インストール。各ツールをタブで切り替え、概要と変換対応拡張子を確認できます。未検出のタブは赤く表示されます。

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

「データ変換」タブ内の「関連システム」から、タブ単位で以下を自動ダウンロード・配置できます。各タブには概要・変換対応拡張子が表示されます。

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
| Node.js（LTS） | 自動ダウンロード | `tools/node` |

以下は手動でインストールしてください。

※ `3d-tiles-tools` と `gltf-pipeline` は Node.js 自動インストール後に `npx` で利用できるため、個別の手動インストールは不要です。

- **GDAL/PDAL** … OSGeo4W または conda-forge
- **Cesium Terrain Builder / tin-terrain** … Docker またはソースからビルド
- **gdal2tiles.py / ctb-tile** … GDAL または Cesium Terrain Builder に同梱



## 更新履歴

### v0.7.0

- `データ変換` タブ内に `座標設定` タブを追加
  - 日本測地系（JGD2011 / JGD2000 / Tokyo）の平面直角座標系 1 〜 19 系を EPSG コードで選択可能
  - 選択した EPSG に対応する原点経度・緯度を自動入力
  - X/Y/Z オフセット、経度、緯度を mago-3d-tiler / 3D Tiles 1.1 変換に渡せるよう対応
- `自動変換` バックエンドを実装
  - `b3dm` / `i3dm` → mago-3d-tiler
  - `pnts` → gocesiumtiler
  - `3dtiles-1-1-glb` → 3D Tiles 1.1 変換
- `個別コンバータ` の mago / Py3DTiles / gocesiumtiler から座標系入力欄を集約し、`座標設定` を参照するように変更

### v0.6.0

- 関連システムをタブ化し、各ツールの概要・変換対応拡張子を表示
- 未検出の関連システムタブを赤く色分け表示

### v0.5.0

- OBJ → 3D Tiles 1.1 変換ルート（`/api/convert/obj-3dtiles11`）を追加
- Node.js（LTS）の自動インストール対応
- `3d-tiles-tools` / `gltf-pipeline` を npx 化
- `/api/run/preprocess` のコマンドを空白区切り対応

### v0.4.0

- データ取得の検索対象をデータ単位に統一
  - DPF 形式フィルターを `files[].original_path` の拡張子で後絞り
  - CKAN 形式フィルターを `resources[].format` / `name` / `url` の拡張子でデータ単位に後絞り
  - CKAN 検索語を `package_search` のデータセット単位検索のみに変更
- DPF 検索を `phraseMatch: true` に変更し、フレーズ全体での一致に絞り込み
- カタログサイト一覧を `server/resources/instances/instances.json` に追加
  - 日本の主要 CKAN カタログ 13 件を同梱
- 「ダウンロード先を開く」を、データセット保存フォルダ（`base_dir/catalog/dataset`）を開くように変更

### v0.3.0

- DPF（国土交通データプラットフォーム）検索で `Catalog ID` をカテゴリで絞り込めるようになりました
- 選択リソースのダウンロード先を `C:\kasugai\data\import\カタログ名\データセット名\` の階層で保存するようになりました
- ダウンロード後に保存先フォルダーをエクスプローラーで開くボタンを追加
- DPF API キーの前後空白をトリムするよう修正

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
