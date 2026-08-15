use axum::{extract::Query, response::IntoResponse, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::time::timeout;

const DEFAULT_IMPORT_DIR: &str = r"C:\kasugai\data\import";
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

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
        "Accept",
        reqwest::header::HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
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

async fn check_connection(api_url: &str) -> Result<(), String> {
    let client = http_client();
    let check_url = format!("{}action/package_search", api_url);
    let res = timeout(
        CONNECTION_TIMEOUT,
        client.get(&check_url).query(&[("rows", "0")]).send(),
    )
    .await;
    match res {
        Ok(Ok(resp)) if resp.status().is_success() => Ok(()),
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

    if servers.is_empty() {
        return Err("CKAN カタログ一覧を取得できませんでした".to_string());
    }

    Ok(servers)
}

pub async fn search_handler(Json(req): Json<SearchRequest>) -> impl IntoResponse {
    let api_url = match validate_ckan_url(&req.api_url) {
        Ok(u) => u,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    if let Err(e) = check_connection(&api_url).await {
        return (StatusCode::BAD_GATEWAY, e).into_response();
    }

    let rows = req.rows.unwrap_or(100).clamp(1, 1000);
    let start = req.start.unwrap_or(0).max(0);
    let q = if req.query.trim().is_empty() {
        "*:*".to_string()
    } else {
        req.query.trim().to_string()
    };
    let search_url = format!("{}action/package_search", api_url);

    let client = http_client();
    let mut query_params: Vec<(&str, String)> = vec![("q", q), ("rows", rows.to_string()), ("start", start.to_string())];
    if let Some(groups) = &req.groups {
        if !groups.is_empty() {
            query_params.push(("fq", format!("groups:({})", groups.join(" OR "))));
        }
    }

    let result = client
        .get(&search_url)
        .query(&query_params)
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

pub async fn groups_handler(Query(req): Query<GroupsRequest>) -> impl IntoResponse {
    let api_url = match validate_ckan_url(&req.api_url) {
        Ok(u) => u,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    if let Err(e) = check_connection(&api_url).await {
        return (StatusCode::BAD_GATEWAY, e).into_response();
    }

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
        let path = base_dir.join(&safe);

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
