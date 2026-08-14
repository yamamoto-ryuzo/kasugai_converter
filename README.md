# Kasuga Converter

自前環境で動作する、GIS データから OGC 3D Tiles への変換システムです。

## 概要

- `mago-3d-tiler`
- `Py3DTiles`
- `pg2b3dm`

などの変換エンジンを、Web UI からひとつのタブで切り替えて利用できます。Rust/Axum 製のローカルサーバー + 静的 HTML UI で構成されています。

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

ブラウザで `http://127.0.0.1:8590/` を開き、「関連システム」タブから必要なツールを自動インストールします。

## 主な機能

| 機能 | 対象 |
|------|------|
| mago-3d-tiler | 3DS/FBX/OBJ/glTF/GLB/LAS/LAZ/CityGML/IndoorGML/SHP/GeoJSON/GPKG → b3dm/i3dm/pnts |
| Py3DTiles | LAS 点群など → 3D Tiles |
| pg2b3dm | PostGIS 3D ジオメトリ → 3D Tiles |

## ライセンス

本プロジェクト（Rust サーバー・Web UI）自身は **MIT License** で公開します。

なお、本ツールは以下の変換エンジンを自動ダウンロードして利用します。各エンジンのライセンスは該当リポジトリに従います。

| エンジン | ライセンス |
|----------|------------|
| [mago-3d-tiler](https://github.com/Gaia3D/mago-3d-tiler) | [MPL-2.0](https://github.com/Gaia3D/mago-3d-tiler/blob/main/LICENSE) |
| [Py3DTiles](https://py3dtiles.org/) | [Apache-2.0](https://gitlab.com/py3dtiles/py3dtiles/-/blob/master/LICENSE) |
| [pg2b3dm](https://github.com/Geodan/pg2b3dm) | [MIT](https://github.com/Geodan/pg2b3dm/blob/master/LICENSE) |

詳細は [docs/license.md](docs/license.md) を参照してください。
