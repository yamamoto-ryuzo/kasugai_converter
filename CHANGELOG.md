# 更新履歴

## Unreleased

- 3D Tiles ルートの総点検で検出した 3 件を修正
  - `/api/convert` の `input_type` に複合値（`gltf/glb` / `las/laz`）が直送された場合も KML フォールバックしないよう正規化（UI 外からの API 呼び出し対策）
  - Py3DTiles へ入力ディレクトリをそのまま渡していた問題を修正（`convert` はファイル名を位置引数に取るため、ディレクトリ内の las/laz/xyz/ply を展開して渡す）。自動変換の xyz/ply → pnts ルートが実際に動作するようになった（実測: xyz ディレクトリ → 1.0 tileset 生成確認）
  - 自動変換の pnts → gocesiumtiler ルートで `z_offset` / `absolute_altitude` が破棄されていた問題を修正（`--z-offset` へ数値合算で引き継ぎ。実測: 50+10 → `Z-Offset: 60` 確認）
- 自動変換の出力形式セレクトが既定値へスナップバックして変更できない UI バグを修正（E2E 検証で発見。`change` ハンドラ内で `autoOutputFormat` の保持値へ戻す処理を分離）
- `3dtiles-1-1-glb` ルートの 3d-tiles-tools 実行を修正（E2E 検証で発見）
  - サーバーが出力先 `tiles_1_1` を事前作成するため `upgrade` が `already exists` で常に失敗していた問題に `--force` を付与して修正
  - `npx 3d-tiles-tools` の都度インストールが Windows の npm キャッシュロック競合・EPERM・ネイティブビルド失敗で不安定なため、`tools/node-tools/node_modules/3d-tiles-tools/build/src/cli.mjs` を最優先で使用（`node install` スクリプトでも同パッケージを `--ignore-scripts` で常駐インストールするよう拡張）
- 自動変換の既定出力形式を `glb (3D Tiles 1.1)` に変更
  - 出力形式カテゴリの既定タブを `3D タイル` にし、`glb (3D Tiles 1.1)` を既定選択に変更
  - `3dtiles-1-1-glb` の入力可能形式に las / laz を追加（mago で pnts 生成後 1.1 へ upgrade し点群 glb 化）
  - gocesiumtiler タブの 3D Tiles バージョン既定値を `1.0` → `1.1` に変更
- 3D Tiles 系ルートの理論的な入力可能形式を見直し
  - mago-3d-tiler 1.15.4 の `--inputType` が内部で `FormatType.fromExtension` 解釈されることを JAR バイトコード＋実実行で確認し、ヘルプ記載の `gltf/glb` / `las/laz` 値は KML へフォールバックして失敗する問題を修正（`gltf` / `las` 等の有効トークンへ変換）
  - 自動判定を mago の `findInputFormatType` と同じ順序（KML 優先 → 走査順の先頭一致）に変更し、Assimp 系の拡張子（dae / ase / stl / ply / x3d / 3mf / ifc / lxo / lwo / lws / x）を追加。kmz / igml は mago が収集しないため除外
  - `i3dm` の入力可能形式を mago 実装（InstancedProcessFlow: shp / geojson / gpkg / kml ＋ `--instance` モデル）に合わせて修正。自動変換に `extra_args`（追加オプション）を追加し `--instance` を渡せるようにした
  - `pnts` の入力可能形式を las / laz / xyz / ply に修正し、xyz / ply のみの入力は Py3DTiles へ自動分岐
  - `3dtiles-1-1-glb` の入力可能形式を b3dm 系と同一に修正（ifc / cityjson / b3dm の直接入力誤記を解消）
- FBX → 3D Tiles 変換を整理・拡充
  - `自動変換` で入力パスから入力形式を自動判定し、mago-3d-tiler の `--inputType` に渡すように変更
  - `/api/convert/obj-3dtiles11` に `input_type` を追加し、OBJ 以外（FBX / 3DS / glTF / GLB など）→ 3D Tiles 1.1 GLB に対応（未指定時は自動判定）
  - 同ルートの中間出力ディレクトリ名を `tiles_1_0` / `tiles_1_1` に変更（3d-tiles-tools の `-o` 拡張子判定で失敗する問題を修正）
- テクスチャ付きモデルと GIS 座標変換オプションに対応
  - mago-3d-tiler の `--recursive`（サブフォルダ再帰検索）を UI・各 API に追加
  - `座標設定` に PROJ4（`--proj`）・ジオイド（`--geoid`）・地形 GeoTIFF（`--terrain`）・X 軸回転（`--rotateXAxis`）・絶対高度（`--absoluteAltitude`）を追加し、mago-3d-tiler 系ルートへ引き渡し
  - 実ファイル検証済み: 地形/ジオイド GeoTIFF（純 Python 生成の Float32 GeoTIFF）で clampToGround・高さ補正が定量適用されることを確認
  - `absolute_altitude`（絶対高度）: mago 1.15.4 の `--absoluteAltitude` は `--altitudeColumn` 解決に常に上書きされる dead code（実測＋バイトコード確認）のため、垂直オフセットとして実効する `--zOffset` へ加算して渡す代替実装。`z_offset` との合算に対応し、GLB の実測垂直シフトで検証済み

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
