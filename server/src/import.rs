use axum::{extract::Query, response::IntoResponse, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::time::timeout;

const DEFAULT_IMPORT_DIR: &str = r"C:\kasugai\data\import";
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

fn is_dpf_url(url: &str) -> bool {
    let u = url.to_lowercase();
    u.contains("data-platform.mlit.go.jp") || u.contains("mlit-data.jp")
}

const INSTANCE_SOURCES: &[&str] = &[
    "https://raw.githubusercontent.com/yamamoto-ryuzo/geo_import/main/resources/instances/instances.json",
    "https://raw.githubusercontent.com/ckan/ckan-instances/gh-pages/config/instances.json",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogInstance {
    pub title: String,
    pub url: String,
    #[serde(rename = "url-api")]
    pub url_api: String,
    #[serde(rename = "type")]
    pub catalog_type: String,
    pub country: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub api_url: String,
    pub query: String,
    pub rows: Option<i64>,
    pub start: Option<i64>,
    pub groups: Option<Vec<String>>,
    pub formats: Option<String>,
    #[serde(default)]
    pub catalog_type: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub count: i64,
    pub start: i64,
    pub results: Vec<Value>,
    pub search_url: String,
}

#[derive(Debug, Deserialize)]
pub struct GroupsRequest {
    pub api_url: String,
    #[serde(default)]
    pub catalog_type: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GroupInfo {
    pub name: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct DownloadItem {
    pub url: String,
    pub filename: Option<String>,
    pub catalog_title: Option<String>,
    pub dataset_title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DownloadRequest {
    pub items: Vec<DownloadItem>,
    pub import_dir: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DownloadResult {
    pub url: String,
    pub path: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct DownloadResponse {
    pub imported_dir: String,
    pub results: Vec<DownloadResult>,
}

fn http_client() -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "User-Agent",
        reqwest::header::HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/50.0.2661.102 Safari/537.36",
        ),
    );
    headers.insert(
        "Accept-Language",
        reqwest::header::HeaderValue::from_static("en-US,en;q=0.8"),
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::limited(10))
        .timeout(REQUEST_TIMEOUT)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

fn import_dir() -> PathBuf {
    if let Ok(v) = std::env::var("KASUGAI_IMPORT_DIR") {
        PathBuf::from(v)
    } else {
        PathBuf::from(DEFAULT_IMPORT_DIR)
    }
}

fn validate_ckan_url(api_url: &str) -> Result<String, String> {
    let mut url = api_url.trim().to_string();
    url = url.replace('\r', "").replace('\n', "");
    if !url.ends_with('/') {
        url.push('/');
    }

    if url.ends_with("backend/api/") {
        return Ok(url);
    }

    if !url.ends_with("3/") {
        return Err(format!("未対応の CKAN API URL です: {}", url));
    }

    Ok(url)
}

async fn resolve_canonical_url(api_url: &str) -> Result<String, String> {
    let client = http_client();
    let check_url = format!("{}action/package_search", api_url);
    let res = timeout(
        CONNECTION_TIMEOUT,
        client.get(&check_url).query(&[("rows", "0")]).send(),
    )
    .await;
    match res {
        Ok(Ok(resp)) if resp.status().is_success() => {
            let mut final_url = resp.url().clone();
            let path = final_url.path();
            if let Some(base_path) = path.strip_suffix("/action/package_search") {
                let new_path = format!("{}/", base_path);
                final_url.set_path(&new_path);
                final_url.set_query(None);
                return Ok(final_url.as_str().to_string());
            }
            Ok(api_url.to_string())
        }
        Ok(Ok(resp)) => Err(format!(
            "接続先が応答しません ({}): {}",
            resp.status(),
            api_url
        )),
        Ok(Err(e)) => Err(format!("接続エラー: {} ({})", e, api_url)),
        Err(_) => Err(format!("接続タイムアウト: {}", api_url)),
    }
}

pub async fn get_servers_handler() -> impl IntoResponse {
    match load_instances().await {
        Ok(servers) => Json(servers).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

async fn load_instances() -> Result<Vec<CatalogInstance>, String> {
    let client = http_client();
    let mut servers = Vec::new();

    for &url in INSTANCE_SOURCES {
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(text) = resp.text().await {
                    if let Ok(mut list) = serde_json::from_str::<Vec<CatalogInstance>>(&text) {
                        servers.append(&mut list);
                    }
                }
            }
            _ => {}
        }
    }

    let local = std::path::Path::new("resources/instances/instances.json");
    if local.exists() {
        if let Ok(text) = tokio::fs::read_to_string(local).await {
            if let Ok(mut list) = serde_json::from_str::<Vec<CatalogInstance>>(&text) {
                servers.append(&mut list);
            }
        }
    }

    if !servers.iter().any(|s| s.catalog_type == "dpf") {
        servers.push(CatalogInstance {
            title: "国土交通データプラットフォーム".to_string(),
            url: "https://data-platform.mlit.go.jp/".to_string(),
            url_api: "https://data-platform.mlit.go.jp/api/v1/".to_string(),
            catalog_type: "dpf".to_string(),
            country: Some("JP".to_string()),
            description: Some("国土交通省 DPF（GraphQL）".to_string()),
        });
    }

    if servers.is_empty() {
        return Err("CKAN カタログ一覧を取得できませんでした".to_string());
    }

    Ok(servers)
}

fn dpf_api_key(req_key: Option<&str>) -> Result<String, String> {
    if let Some(k) = req_key {
        let k = k.trim();
        if !k.is_empty() {
            return Ok(k.to_string());
        }
    }
    std::env::var("MLIT_DPF_API_KEY")
        .map(|v| v.trim().to_string())
        .map_err(|_| "MLIT_DPF_API_KEY 環境変数が必要です".to_string())
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DpfAdapter {
    name: String,
    #[serde(rename = "type")]
    adapter_type: String,
    endpoint: String,
    search_query: String,
    search_fields: String,
    count_json_path: String,
    results_json_path: String,
    field_map: HashMap<String, String>,
    download_query: String,
    download_url_json_path: String,
}

async fn load_dpf_adapter() -> Result<DpfAdapter, String> {
    let path = std::path::Path::new("resources/adapters/dpf.json");
    let text = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("DPF adapter 設定を読めません: {}", e))?;
    serde_json::from_str::<DpfAdapter>(&text)
        .map_err(|e| format!("DPF adapter JSON パースエラー: {}", e))
}

fn to_json_pointer(path: &str) -> String {
    let parts: Vec<String> = path
        .split('.')
        .map(|s| s.replace('~', "~0").replace('/', "~1"))
        .collect();
    format!("/{}", parts.join("/"))
}

fn get_json_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    value.pointer(&to_json_pointer(path))
}

fn map_dpf_result(result: &Value, field_map: &HashMap<String, String>) -> Value {
    let mut out = serde_json::Map::new();
    for (target, source) in field_map {
        if let Some(v) = result.get(source) {
            out.insert(target.clone(), v.clone());
        }
    }
    Value::Object(out)
}

fn dpf_attribute_filter(groups: &[String]) -> String {
    if groups.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = groups
        .iter()
        .map(|id| {
            let escaped = id.replace('"', "\\\"");
            format!(r#"{{ attributeName: "DPF:catalog_id", is: "{escaped}" }}"#)
        })
        .collect();
    format!(", attributeFilter: {{ OR: [ {} ] }}",
        parts.join(", ")
    )
}

async fn dpf_search(req: SearchRequest) -> impl IntoResponse {
    let adapter = match load_dpf_adapter().await {
        Ok(a) => a,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    let mut api_url = req.api_url.trim().to_string();
    if api_url.is_empty() {
        api_url = adapter.endpoint.clone();
    }
    if !api_url.ends_with('/') {
        api_url.push('/');
    }

    let api_key = match dpf_api_key(req.api_key.as_deref()) {
        Ok(k) => k,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    let rows = req.rows.unwrap_or(100).clamp(1, 1000);
    let start = req.start.unwrap_or(0).max(0);
    let term = req.query.trim();
    let escaped_term = term.replace('"', "\\\"");

    let attr_filter = dpf_attribute_filter(req.groups.as_deref().unwrap_or(&[]));
    let selected_formats: std::collections::HashSet<String> = req
        .formats
        .as_ref()
        .map(|f| {
            f.split(|c: char| c == ',' || c == ' ' || c == '　')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let graph_query = adapter
        .search_query
        .replace("{{term}}", &escaped_term)
        .replace("{{start}}", &start.to_string())
        .replace("{{rows}}", &rows.to_string())
        .replace("{{attribute_filter}}", &attr_filter)
        .replace("{{fields}}", &adapter.search_fields);
    let payload = serde_json::json!({ "query": graph_query });

    let client = http_client();
    let result = client
        .post(&api_url)
        .header("apikey", &api_key)
        .header("Accept", "application/json")
        .json(&payload)
        .send()
        .await;

    match result {
        Ok(resp) if resp.status().is_success() => match resp.json::<Value>().await {
            Ok(json) => {
                if let Some(errors) = json.get("errors") {
                    return (StatusCode::BAD_GATEWAY, format!("DPF GraphQL エラー: {}", errors)).into_response();
                }
                let count = get_json_path(&json, &adapter.count_json_path)
                    .and_then(Value::as_i64)
                    .unwrap_or(0);
                let results = get_json_path(&json, &adapter.results_json_path)
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let filtered_results: Vec<Value> = results
                    .into_iter()
                    .filter_map(|mut r| {
                        if selected_formats.is_empty() {
                            return Some(r);
                        }
                        let files = r
                            .get("files")
                            .and_then(Value::as_array)
                            .cloned()
                            .unwrap_or_default();
                        let filtered_files: Vec<Value> = files
                            .into_iter()
                            .filter(|f| {
                                let path = f
                                    .get("original_path")
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                                    .to_lowercase();
                                path.rsplit_once('.')
                                    .map(|(_, ext)| ext)
                                    .map_or(false, |ext| selected_formats.contains(ext))
                            })
                            .collect();
                        if filtered_files.is_empty() {
                            None
                        } else {
                            if let Some(obj) = r.as_object_mut() {
                                obj.insert("files".to_string(), Value::Array(filtered_files));
                            }
                            Some(r)
                        }
                    })
                    .collect();
                let mapped: Vec<Value> = filtered_results
                    .iter()
                    .map(|r| map_dpf_result(r, &adapter.field_map))
                    .collect();
                Json(SearchResponse {
                    count,
                    start,
                    results: mapped,
                    search_url: api_url,
                })
                .into_response()
            }
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                format!("JSON パースエラー: {} (URL: {})", e, api_url),
            )
                .into_response(),
        },
        Ok(resp) => (
            StatusCode::BAD_GATEWAY,
            format!("DPF サーバーエラー: {} (URL: {})", resp.status(), api_url),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            format!("リクエストエラー: {} (URL: {})", e, api_url),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct DpfFile {
    pub id: String,
    pub original_path: String,
}

#[derive(Debug, Deserialize)]
pub struct DpfDownloadUrlsRequest {
    pub api_url: String,
    pub api_key: String,
    pub files: Vec<DpfFile>,
}

pub async fn dpf_download_urls_handler(Json(req): Json<DpfDownloadUrlsRequest>) -> impl IntoResponse {
    let adapter = match load_dpf_adapter().await {
        Ok(a) => a,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    let mut api_url = req.api_url.trim().to_string();
    if api_url.is_empty() {
        api_url = adapter.endpoint.clone();
    }
    if !api_url.ends_with('/') {
        api_url.push('/');
    }
    let api_key = if !req.api_key.trim().is_empty() {
        req.api_key.trim().to_string()
    } else {
        match dpf_api_key(None) {
            Ok(k) => k,
            Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
        }
    };
    if req.files.is_empty() {
        return Json(serde_json::json!([])).into_response();
    }

    let items: Vec<String> = req
        .files
        .iter()
        .map(|f| {
            let id = f.id.replace('"', "\\\"");
            let path = f.original_path.replace('"', "\\\"");
            format!("{{ id: \"{}\", original_path: \"{}\" }}", id, path)
        })
        .collect();

    let graph_query = adapter
        .download_query
        .replace("{{files}}", &items.join(", "));
    let payload = serde_json::json!({ "query": graph_query });

    let client = http_client();
    match client
        .post(&api_url)
        .header("apikey", &api_key)
        .header("Accept", "application/json")
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.json::<Value>().await {
            Ok(json) => {
                if let Some(errors) = json.get("errors") {
                    return (
                        StatusCode::BAD_GATEWAY,
                        format!("DPF GraphQL エラー: {}", errors),
                    )
                        .into_response();
                }
                let urls = get_json_path(&json, &adapter.download_url_json_path)
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!([]));
                Json(urls).into_response()
            }
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                format!("JSON パースエラー: {} (URL: {})", e, api_url),
            )
                .into_response(),
        },
        Ok(resp) => (
            StatusCode::BAD_GATEWAY,
            format!("DPF サーバーエラー: {} (URL: {})", resp.status(), api_url),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            format!("リクエストエラー: {} (URL: {})", e, api_url),
        )
            .into_response(),
    }
}

pub async fn search_handler(Json(req): Json<SearchRequest>) -> axum::response::Response {
    if req.catalog_type == "dpf" {
        dpf_search(req).await.into_response()
    } else {
        ckan_search(req).await.into_response()
    }
}

async fn ckan_search(req: SearchRequest) -> impl IntoResponse {
    let api_url = match validate_ckan_url(&req.api_url) {
        Ok(u) => u,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    let api_url = match resolve_canonical_url(&api_url).await {
        Ok(u) => u,
        Err(e) => return (StatusCode::BAD_GATEWAY, e).into_response(),
    };

    let rows = req.rows.unwrap_or(100).clamp(1, 1000);
    let start = req.start.unwrap_or(0).max(0);
    let q = if req.query.trim().is_empty() {
        "*:*".to_string()
    } else {
        req.query.trim().to_string()
    };
    let search_url = format!("{}action/package_search", api_url);

    let client = http_client();

    let mut fq_terms = Vec::new();

    if let Some(groups) = &req.groups {
        if !groups.is_empty() {
            fq_terms.push(format!("groups:({})", groups.join(" OR ")));
        }
    }

    let selected_formats: std::collections::HashSet<String> = req
        .formats
        .as_ref()
        .map(|f| {
            f.split(|c: char| c == ',' || c == ' ' || c == '　')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let mut body = serde_json::Map::new();
    body.insert("q".to_string(), serde_json::Value::String(q.clone()));
    body.insert(
        "rows".to_string(),
        serde_json::Value::Number(serde_json::Number::from(rows)),
    );
    body.insert(
        "start".to_string(),
        serde_json::Value::Number(serde_json::Number::from(start)),
    );
    if !fq_terms.is_empty() {
        body.insert(
            "fq".to_string(),
            serde_json::Value::String(fq_terms.join(" AND ")),
        );
    }
    let body = serde_json::Value::Object(body);

    let result = client
        .post(&search_url)
        .json(&body)
        .send()
        .await;

    match result {
        Ok(resp) if resp.status().is_success() => match resp.text().await {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(json) => {
                    let count = json
                        .get("result")
                        .and_then(|r| r.get("count"))
                        .and_then(Value::as_i64)
                        .unwrap_or(0);
                    let results = json
                        .get("result")
                        .and_then(|r| r.get("results"))
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default();

                    // データ単位で形式フィルタのみ
                    let results: Vec<Value> = results
                        .into_iter()
                        .filter_map(|mut pkg| {
                            let resources = pkg
                                .get("resources")
                                .and_then(Value::as_array)
                                .cloned()
                                .unwrap_or_default();
                            let filtered: Vec<Value> = resources
                                .into_iter()
                                .filter(|r| {
                                    if !selected_formats.is_empty() {
                                        let fmt = r
                                            .get("format")
                                            .and_then(Value::as_str)
                                            .unwrap_or("")
                                            .to_lowercase();
                                        if !selected_formats.contains(&fmt) {
                                            // format フィールドが不一致の場合、name/url の拡張子を確認
                                            let name = r
                                                .get("name")
                                                .and_then(Value::as_str)
                                                .unwrap_or("")
                                                .to_lowercase();
                                            let url = r
                                                .get("url")
                                                .and_then(Value::as_str)
                                                .unwrap_or("")
                                                .to_lowercase();
                                            let url_filename = url
                                                .rsplit_once('/')
                                                .map(|(_, s)| s)
                                                .unwrap_or(&url)
                                                .split('?')
                                                .next()
                                                .unwrap_or("");
                                            let mut exts = Vec::new();
                                            if let Some((_, ext)) = name.rsplit_once('.') {
                                                exts.push(ext.to_string());
                                            }
                                            if let Some((_, ext)) = url_filename.rsplit_once('.') {
                                                exts.push(ext.to_string());
                                            }
                                            if !exts.iter().any(|e| selected_formats.contains(e)) {
                                                return false;
                                            }
                                        }
                                    }
                                    true
                                })
                                .collect();
                            if filtered.is_empty() {
                                None
                            } else {
                                if let Some(obj) = pkg.as_object_mut() {
                                    obj.insert("resources".to_string(), Value::Array(filtered));
                                }
                                Some(pkg)
                            }
                        })
                        .collect();

                    Json(SearchResponse {
                        count,
                        start,
                        results,
                        search_url,
                    })
                    .into_response()
                }
                Err(e) => (
                    StatusCode::BAD_GATEWAY,
                    format!("JSON パースエラー: {} (URL: {})", e, search_url),
                )
                    .into_response(),
            },
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                format!("テキスト取得エラー: {} (URL: {})", e, search_url),
            )
                .into_response(),
        },
        Ok(resp) => (
            StatusCode::BAD_GATEWAY,
            format!(
                "CKAN サーバーエラー: {} (URL: {})",
                resp.status(),
                search_url
            ),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            format!("リクエストエラー: {} (URL: {})", e, search_url),
        )
            .into_response(),
    }
}

async fn dpf_catalogs(api_url: &str, api_key: Option<&str>) -> Result<Vec<GroupInfo>, String> {
    let api_key = dpf_api_key(api_key)?;
    let mut url = api_url.trim().to_string();
    if url.is_empty() {
        url = "https://data-platform.mlit.go.jp/api/v1/".to_string();
    }
    if !url.ends_with('/') {
        url.push('/');
    }

    let client = http_client();
    let query = r#"query { dataCatalog(IDs: null) { id title } }"#;
    let payload = serde_json::json!({ "query": query });

    match client
        .post(&url)
        .header("apikey", &api_key)
        .header("Accept", "application/json")
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.json::<Value>().await {
            Ok(json) => {
                if let Some(errors) = json.get("errors") {
                    return Err(format!("DPF GraphQL エラー: {}", errors));
                }
                let list = json
                    .get("data")
                    .and_then(|d| d.get("dataCatalog"))
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let groups: Vec<GroupInfo> = list
                    .iter()
                    .filter_map(|v| {
                        let name = v.get("id")?.as_str()?.to_string();
                        let title = v
                            .get("title")
                            .and_then(Value::as_str)
                            .unwrap_or(&name)
                            .to_string();
                        Some(GroupInfo { name, title })
                    })
                    .collect();
                Ok(groups)
            }
            Err(e) => Err(format!("JSON パースエラー: {}", e)),
        },
        Ok(resp) => Err(format!("DPF サーバーエラー: {}", resp.status())),
        Err(e) => Err(format!("リクエストエラー: {}", e)),
    }
}

pub async fn groups_handler(Query(req): Query<GroupsRequest>) -> impl IntoResponse {
    if is_dpf_url(&req.api_url) || req.catalog_type == "dpf" {
        return match dpf_catalogs(&req.api_url, req.api_key.as_deref()).await {
            Ok(groups) => Json(groups).into_response(),
            Err(e) => (StatusCode::BAD_GATEWAY, e).into_response(),
        };
    }
    let api_url = match validate_ckan_url(&req.api_url) {
        Ok(u) => u,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    let api_url = match resolve_canonical_url(&api_url).await {
        Ok(u) => u,
        Err(e) => return (StatusCode::BAD_GATEWAY, e).into_response(),
    };

    let list_url = format!("{}action/group_list", api_url);
    let client = http_client();
    match client
        .get(&list_url)
        .query(&[("all_fields", "true"), ("include_dataset_count", "false")])
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.text().await {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(json) => {
                    let result = json
                        .get("result")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default();
                    let groups: Vec<GroupInfo> = result
                        .iter()
                        .filter_map(|g| {
                            let name = g.get("name")?.as_str()?.to_string();
                            let title = g
                                .get("display_name")
                                .or(g.get("title"))
                                .and_then(Value::as_str)
                                .unwrap_or(&name)
                                .to_string();
                            Some(GroupInfo { name, title })
                        })
                        .collect();
                    Json(groups).into_response()
                }
                Err(e) => (
                    StatusCode::BAD_GATEWAY,
                    format!("JSON パースエラー: {} (URL: {})", e, list_url),
                )
                    .into_response(),
            },
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                format!("テキスト取得エラー: {} (URL: {})", e, list_url),
            )
                .into_response(),
        },
        Ok(resp) => (
            StatusCode::BAD_GATEWAY,
            format!(
                "CKAN サーバーエラー: {} (URL: {})",
                resp.status(),
                list_url
            ),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            format!("リクエストエラー: {} (URL: {})", e, list_url),
        )
            .into_response(),
    }
}

pub async fn download_handler(Json(req): Json<DownloadRequest>) -> impl IntoResponse {
    let base_dir = req.import_dir.map(PathBuf::from).unwrap_or_else(import_dir);
    if let Err(e) = tokio::fs::create_dir_all(&base_dir).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("保存先を作成できません: {}", e),
        )
            .into_response();
    }

    let client = http_client();
    let mut results = Vec::new();

    for (idx, item) in req.items.iter().enumerate() {
        let url = item.url.trim();
        if url.is_empty() {
            continue;
        }

        let filename = item
            .filename
            .as_ref()
            .and_then(|f| {
                let t = f.trim();
                if t.is_empty() { None } else { Some(t.to_string()) }
            })
            .unwrap_or_else(|| filename_from_url(url, idx));
        let safe = sanitize_filename(&filename);

        let catalog = item
            .catalog_title
            .as_deref()
            .map(sanitize_filename)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "default".to_string());
        let dataset = item
            .dataset_title
            .as_deref()
            .map(sanitize_filename)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "default".to_string());
        let dir = base_dir.join(&catalog).join(&dataset);
        if let Err(e) = tokio::fs::create_dir_all(&dir).await {
            results.push(DownloadResult {
                url: url.to_string(),
                path: "".to_string(),
                status: format!("フォルダ作成エラー: {}", e),
            });
            continue;
        }
        let path = dir.join(&safe);

        let status = match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => match resp.bytes().await {
                Ok(bytes) => match tokio::fs::File::create(&path).await {
                    Ok(mut file) => match file.write_all(&bytes).await {
                        Ok(_) => "ok".to_string(),
                        Err(e) => format!("書き込みエラー: {}", e),
                    },
                    Err(e) => format!("ファイル作成エラー: {}", e),
                },
                Err(e) => format!("ダウンロードエラー: {}", e),
            },
            Ok(resp) => format!("HTTP {}", resp.status()),
            Err(e) => format!("リクエストエラー: {}", e),
        };

        results.push(DownloadResult {
            url: url.to_string(),
            path: path.to_string_lossy().to_string(),
            status,
        });
    }

    Json(DownloadResponse {
        imported_dir: base_dir.to_string_lossy().to_string(),
        results,
    })
    .into_response()
}

fn filename_from_url(url: &str, idx: usize) -> String {
    let before_q = url.split('?').next().unwrap_or(url);
    let name = before_q
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("resource");
    if name.is_empty() {
        format!("resource_{}", idx)
    } else {
        name.to_string()
    }
}

fn sanitize_filename(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | '%' | '#' => out.push('_'),
            _ => out.push(c),
        }
    }

    if out.trim().is_empty() {
        out = "resource".to_string();
    }

    if out.len() > 200 {
        out.truncate(200);
    }

    out
}
