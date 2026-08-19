---
layout: default
title: ライセンス
---

# ライセンス

## 本プロジェクト

Kasuga Converter（Rust サーバー・Web UI）のソースコードは **MIT License** で公開されています。

- [LICENSE](https://github.com/kasuga_converter/blob/main/LICENSE)

## 利用する外部変換エンジン

本ツールは UI から以下の外部エンジンを呼び出します。これらのバイナリは自動ダウンロードにより取得され、各自のライセンスに従います。

| エンジン | リポジトリ | ライセンス |
|----------|------------|------------|
| mago-3d-tiler | [Gaia3D/mago-3d-tiler](https://github.com/Gaia3D/mago-3d-tiler) | **MPL-2.0** |
| Py3DTiles | [py3dtiles / py3dtiles](https://gitlab.com/py3dtiles/py3dtiles) | **Apache-2.0** |
| pg2b3dm | [Geodan/pg2b3dm](https://github.com/Geodan/pg2b3dm) | **MIT** |
| gocesiumtiler | [mfbonfigli/gocesiumtiler](https://github.com/mfbonfigli/gocesiumtiler) | **MPL-2.0** |
| IfcOpenShell | [IfcOpenShell/IfcOpenShell](https://github.com/IfcOpenShell/IfcOpenShell) | **LGPL-3.0** |
| cjio | [cityjson/cjio](https://github.com/cityjson/cjio) | **MIT** |
| gltf-pipeline | [CesiumGS/gltf-pipeline](https://github.com/CesiumGS/gltf-pipeline) | **Apache-2.0** |
|| 3d-tiles-tools | [CesiumGS/3d-tiles-tools](https://github.com/CesiumGS/3d-tiles-tools) | **Apache-2.0** |

## 配布・再配布について

- 本プロジェクトのコード自体は MIT License に基づき商用利用も可能です。
- ただし、上記外部エンジンを再配布する場合は、それぞれのライセンス条件（表示・改変コードの開示など）に従う必要があります。
- MPL-2.0（mago-3d-tiler）を対象とする場合、**MPL のファイルを修正したらそのソースを開示する必要がある**点に注意してください。

## なぜ MIT か

- 本ツールは外部エンジンを **別プロセスとして呼び出すラッパー** です。ソースコードレベルで結合・修正していないため、外部エンジンのライセンスがラッパー本体に影響を与えることはありません。
- MIT License は MPL-2.0、Apache-2.0、MIT すべてと互換性があり、再利用・商用利用ともに制限が少ないです。
- 変換結果の 3D Tiles データはユーザーが所有するデータに帰属します。本ツールのライセンスが出力データに権利を主張することはありません。

## 手動インストールが必要なツール

| ツール | ライセンス |
|--------|------------|
| GDAL | MIT |
| PDAL | BSD-3-Clause |
| Cesium Terrain Builder | Apache-2.0 |
| tin-terrain | Apache-2.0 |
| 
| 
