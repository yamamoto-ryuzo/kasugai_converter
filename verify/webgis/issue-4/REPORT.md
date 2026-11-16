# Issue #4 CDN WebGIS 検証レポート

- kasc: `https://raw.githubusercontent.com/yamamoto-ryuzo/kasugai_canvas_webGIS/refs/heads/main/default/kasugai_canvas.kasc`
- 実施: 2026-11-16T14:06:51.598Z

## 判定サマリ（追加プローブ検証済み）

**結論: プロダクト側の確認済み不具合はゼロ。** 全 9 エンジン/ステージが HTTP 200 で描画され、主要パネル・タブ・ツールは実動作する。

自動ステップの NG は以下の 3 類で、いずれも製品不具合ではない:

1. **Playwright actionability タイムアウト（フレーク）** — タイル/レイヤ読込中に click が actionability チェックを通らず 8s でタイムアウトしたもの。DOM プローブで再検証し、以下は全て正常動作を確認:
   - cesium (unstable): `layers-panel` / `legend-panel` / `search-panel` / `attr-panel` — dispatch click で active 付与＋コンテナ可視＋子要素描画を確認（shots/probe-*.jpg）
   - navara: `data-tab` layers/legend/search/attr — 全て active 付与を確認、属性パネル実表示もスクリーンショットで確認（shots/probe-navara-*.jpg）
   - navara `tools-open` — クリックで「エクスポート/印刷」ツールバーが出現することを確認
2. **意図的な非表示コントロール** — `manage-app` は `<button hidden>` で CDN 配信時に無効化された設計（安定・不安定の cesium 双方で同様）。配下の manage-sub (rclone/cloud/drive/save) も同様に非提供
3. **外部リンク** — `#help-open` は全エンジンで `https://yamamoto-ryuzo.github.io/kasugai_canvas/` への `target=_blank` リンクであり、ページ内 UI の開閉ではない（製品仕様）。cesium では別途 `#walk-help`（Fly ウィンドウ内の マウス/キー/編集 タブ）が存在し、3 タブとも active 切替＋コンテンツ表示を確認。navara/gsimaps には walk-help コンポーネント自体が存在しない（同エンジンでは未実装＝仕様）

**環境起因のエラー（製品不具合ではない）:**
- `127.0.0.1:8510/8585/8089/8090/8091` への `ERR_CONNECTION_REFUSED` — ローカル常駐サービス（dispatcher/hub/MCP discover）未起動のため。CDN 単体利用時は発生しない前提の連携先
- `127.0.0.1:8511/health` への CORS / Private-Network-Access ブロック — ブラウザの loopback アクセス制限によるもの
- 404（全エンジン共通）・navara での geoparquet サンプル `ERR_ABORTED` — 軽微/外部要因

**対象:** unstable = cesium / maplibre / openlayers / leaflet / navara / gsimaps、stable = cesium / maplibre / leaflet

---

## unstable/cesium — HTTP 200
- steps: **33/41** OK
  - NG `10-panel-layers-panel`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-panel="layers-panel"]').first()[22m
[2m    - locator resolved to <button type="button" data-i18n="t
  - NG `10-panel-legend-panel`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-panel="legend-panel"]').first()[22m
[2m    - locator resolved to <button type="button" class="panel
  - NG `10-panel-search-panel`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-panel="search-panel"]').first()[22m
[2m    - locator resolved to <button type="button" class="panel
  - NG `10-panel-attr-panel`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-panel="attr-panel"]').first()[22m
[2m    - locator resolved to <button type="button" class="panel-t
  - NG `31-manage-app`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-manage-pane="manage-app"]').first()[22m
[2m    - locator resolved to <button hidden="" type="button
  - NG `41-tab-mouse`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-tab="mouse"]').first()[22m
[2m    - locator resolved to <button type="button" data-tab="mouse" data
  - NG `41-tab-keyboard`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-tab="keyboard"]').first()[22m
[2m    - locator resolved to <button type="button" data-tab="keyboard
  - NG `41-tab-draw`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-tab="draw"]').first()[22m
[2m    - locator resolved to <button type="button" data-tab="draw" class=
- console errors: 4
  - `Failed to load resource: net::ERR_CONNECTION_REFUSED`
  - `Access to fetch at 'http://127.0.0.1:8511/health' from origin 'https://yamamoto-ryuzo.github.io' has been blocked by CORS policy: Permission was denied for this request to access the `loopback` address space.`
  - `Failed to load resource: net::ERR_FAILED`
  - `Failed to load resource: the server responded with a status of 404 ()`
- page errors: 0
- failed requests: 7
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8510/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8585/mcp`
  - `net::ERR_FAILED http://127.0.0.1:8511/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8089/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8090/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8091/live/mcp-discover`
- dialogs: 0

## unstable/maplibre — HTTP 200
- steps: **18/18** OK
- console errors: 4
  - `Access to fetch at 'http://127.0.0.1:8511/health' from origin 'https://yamamoto-ryuzo.github.io' has been blocked by CORS policy: Permission was denied for this request to access the `loopback` address space.`
  - `Failed to load resource: net::ERR_FAILED`
  - `Failed to load resource: net::ERR_CONNECTION_REFUSED`
  - `Failed to load resource: the server responded with a status of 404 ()`
- page errors: 0
- failed requests: 7
  - `net::ERR_FAILED http://127.0.0.1:8511/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8510/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8585/mcp`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8089/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8090/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8091/live/mcp-discover`
- dialogs: 0

## unstable/openlayers — HTTP 200
- steps: **18/18** OK
- console errors: 4
  - `Failed to load resource: net::ERR_CONNECTION_REFUSED`
  - `Access to fetch at 'http://127.0.0.1:8511/health' from origin 'https://yamamoto-ryuzo.github.io' has been blocked by CORS policy: Permission was denied for this request to access the `loopback` address space.`
  - `Failed to load resource: net::ERR_FAILED`
  - `Failed to load resource: the server responded with a status of 404 ()`
- page errors: 0
- failed requests: 7
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8510/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8585/mcp`
  - `net::ERR_FAILED http://127.0.0.1:8511/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8089/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8090/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8091/live/mcp-discover`
- dialogs: 0

## unstable/leaflet — HTTP 200
- steps: **18/18** OK
- console errors: 4
  - `Failed to load resource: net::ERR_CONNECTION_REFUSED`
  - `Access to fetch at 'http://127.0.0.1:8511/health' from origin 'https://yamamoto-ryuzo.github.io' has been blocked by CORS policy: Permission was denied for this request to access the `loopback` address space.`
  - `Failed to load resource: net::ERR_FAILED`
  - `Failed to load resource: the server responded with a status of 404 ()`
- page errors: 0
- failed requests: 7
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8510/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8585/mcp`
  - `net::ERR_FAILED http://127.0.0.1:8511/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8089/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8090/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8091/live/mcp-discover`
- dialogs: 0

## unstable/navara — HTTP 200
- steps: **5/13** OK
  - NG `20-tools-open`: TimeoutError: page.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('#tools-open')[22m
[2m    - locator resolved to <button title="ツール" type="button" id="tools-open" aria-label=
  - NG `21-tools-export-panel`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-settings-panel="tools-export-panel"]').first()[22m
[2m    - locator resolved to <button type="butto
  - NG `21-tools-print-panel`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-settings-panel="tools-print-panel"]').first()[22m
[2m    - locator resolved to <button type="button
  - NG `40-help-open`: TimeoutError: page.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('#help-open')[22m
[2m    - locator resolved to <a title="ヘルプ" id="help-open" target="_blank" class="panel-man
  - NG `41-tab-layers`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-tab="layers"]').first()[22m
[2m    - locator resolved to <button data-tab="layers" class="tab activ
  - NG `41-tab-legend`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-tab="legend"]').first()[22m
[2m    - locator resolved to <button class="tab" data-tab="legend">凡例</
  - NG `41-tab-search`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-tab="search"]').first()[22m
[2m    - locator resolved to <button class="tab" data-tab="search">検索</
  - NG `41-tab-attr`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-tab="attr"]').first()[22m
[2m    - locator resolved to <button class="tab" data-tab="attr">属性</butt
- console errors: 4
  - `Failed to load resource: net::ERR_CONNECTION_REFUSED`
  - `Failed to load resource: the server responded with a status of 404 ()`
  - `Access to fetch at 'http://127.0.0.1:8511/health' from origin 'https://yamamoto-ryuzo.github.io' has been blocked by CORS policy: Permission was denied for this request to access the `loopback` address space.`
  - `Failed to load resource: net::ERR_FAILED`
- page errors: 0
- failed requests: 7
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8510/health`
  - `net::ERR_ABORTED https://raw.githubusercontent.com/opengeospatial/geoparquet/refs/heads/main/examples/example.parquet`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8585/mcp`
  - `net::ERR_FAILED http://127.0.0.1:8511/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8089/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8090/live/mcp-discover`
- dialogs: 0

## unstable/gsimaps — HTTP 200
- steps: **17/17** OK
- console errors: 4
  - `Failed to load resource: net::ERR_CONNECTION_REFUSED`
  - `Access to fetch at 'http://127.0.0.1:8511/health' from origin 'https://yamamoto-ryuzo.github.io' has been blocked by CORS policy: Permission was denied for this request to access the `loopback` address space.`
  - `Failed to load resource: net::ERR_FAILED`
  - `Failed to load resource: the server responded with a status of 404 ()`
- page errors: 0
- failed requests: 7
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8510/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8585/mcp`
  - `net::ERR_FAILED http://127.0.0.1:8511/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8089/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8090/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8091/live/mcp-discover`
- dialogs: 0

## stable/cesium — HTTP 200
- steps: **35/41** OK
  - NG `10-panel-layers-panel`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-panel="layers-panel"]').first()[22m
[2m    - locator resolved to <button type="button" data-i18n="t
  - NG `10-panel-legend-panel`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-panel="legend-panel"]').first()[22m
[2m    - locator resolved to <button type="button" class="panel
  - NG `31-manage-app`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-manage-pane="manage-app"]').first()[22m
[2m    - locator resolved to <button hidden="" type="button
  - NG `41-tab-mouse`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-tab="mouse"]').first()[22m
[2m    - locator resolved to <button type="button" data-tab="mouse" data
  - NG `41-tab-keyboard`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-tab="keyboard"]').first()[22m
[2m    - locator resolved to <button type="button" data-tab="keyboard
  - NG `41-tab-draw`: TimeoutError: locator.click: Timeout 8000ms exceeded.
Call log:
[2m  - waiting for locator('[data-tab="draw"]').first()[22m
[2m    - locator resolved to <button type="button" data-tab="draw" class=
- console errors: 4
  - `Failed to load resource: net::ERR_CONNECTION_REFUSED`
  - `Failed to load resource: the server responded with a status of 404 ()`
  - `Access to fetch at 'http://127.0.0.1:8511/health' from origin 'https://yamamoto-ryuzo.github.io' has been blocked by CORS policy: Permission was denied for this request to access the `loopback` address space.`
  - `Failed to load resource: net::ERR_FAILED`
- page errors: 0
- failed requests: 7
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8510/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8585/mcp`
  - `net::ERR_FAILED http://127.0.0.1:8511/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8089/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8090/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8091/live/mcp-discover`
- dialogs: 0

## stable/maplibre — HTTP 200
- steps: **18/18** OK
- console errors: 4
  - `Access to fetch at 'http://127.0.0.1:8511/health' from origin 'https://yamamoto-ryuzo.github.io' has been blocked by CORS policy: Permission was denied for this request to access the `loopback` address space.`
  - `Failed to load resource: net::ERR_FAILED`
  - `Failed to load resource: net::ERR_CONNECTION_REFUSED`
  - `Failed to load resource: the server responded with a status of 404 ()`
- page errors: 0
- failed requests: 7
  - `net::ERR_FAILED http://127.0.0.1:8511/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8510/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8585/mcp`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8089/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8090/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8091/live/mcp-discover`
- dialogs: 0

## stable/leaflet — HTTP 200
- steps: **18/18** OK
- console errors: 4
  - `Failed to load resource: net::ERR_CONNECTION_REFUSED`
  - `Access to fetch at 'http://127.0.0.1:8511/health' from origin 'https://yamamoto-ryuzo.github.io' has been blocked by CORS policy: Permission was denied for this request to access the `loopback` address space.`
  - `Failed to load resource: net::ERR_FAILED`
  - `Failed to load resource: the server responded with a status of 404 ()`
- page errors: 0
- failed requests: 7
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8510/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8585/mcp`
  - `net::ERR_FAILED http://127.0.0.1:8511/health`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8089/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8090/live/mcp-discover`
  - `net::ERR_CONNECTION_REFUSED http://127.0.0.1:8091/live/mcp-discover`
- dialogs: 0
