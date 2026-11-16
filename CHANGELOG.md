# 更新履歴

## Unreleased

- FBX → 3D Tiles 変換を整理・拡充
  - `自動変換` で入力パスから入力形式を自動判定し、mago-3d-tiler の `--inputType` に渡すように変更
  - `/api/convert/obj-3dtiles11` に `input_type` を追加し、OBJ 以外（FBX / 3DS / glTF / GLB など）→ 3D Tiles 1.1 GLB に対応（未指定時は自動判定）
  - 同ルートの中間出力ディレクトリ名を `tiles_1_0` / `tiles_1_1` に変更（3d-tiles-tools の `-o` 拡張子判定で失敗する問題を修正）
- テクスチャ付きモデルと GIS 座標変換オプションに対応
  - mago-3d-tiler の `--recursive`（サブフォルダ再帰検索）を UI・各 API に追加
  - `座標設定` に PROJ4（`--proj`）・ジオイド（`--geoid`）・地形 GeoTIFF（`--terrain`）・X 軸回転（`--rotateXAxis`）・絶対高度（`--absoluteAltitude`）を追加し、mago-3d-tiler 系ルートへ引き渡し
  - 実ファイル検証済み: 地形/ジオイド GeoTIFF（純 Python 生成の Float32 GeoTIFF）で clampToGround・高さ補正が定量適用されることを確認。`--absoluteAltitude` は mago 1.15.4 で未適用（`--altitudeColumn` 解決が常に優先される上流実装のため）ことを確認し、ドキュメントに注記

## [0.8.3] - 2026-08-20

- `OBJ 表面処理` タブにファイル選択ダイアログを追加
  - 入力 OBJ ファイルを Windows の「ファイルを開く」ダイアログで選択可能に
  - 初期表示フォルダを自動変換の入力フォルダ（`c:\kasugai\data\input`）に連動

[0.8.3]: https://github.com/yamamoto-ryuzo/kasugai_converter/compare/v0.8.2...v0.8.3

## [0.8.2] - 2026-08-20

- `OBJ 表面処理` タブを追加
  - OBJ ファイルの面の向き（表裏）を反転
  - 法線（vn）を反転
  - テクスチャ座標（vt）の U / V 反転に対応
  - 追加ライブラリ不要。Rust 標準でテキスト処理

[0.8.2]: https://github.com/yamamoto-ryuzo/kasugai_converter/compare/v0.8.1...v0.8.2

## [0.8.1] - 2026-08-20

- 起動時の自動バージョンアップ対応
- Agent.md 仕様への準拠整備（README バージョン表記、CHANGELOG、バージョン整合性チェック）

[0.8.1]: https://github.com/yamamoto-ryuzo/kasugai_converter/compare/v0.8.0...v0.8.1
