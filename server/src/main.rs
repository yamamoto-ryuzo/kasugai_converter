#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path as StdPath;
use std::process::Stdio;
use std::sync::Arc;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

mod import;

#[derive(Clone)]
struct AppState {
    jobs: Arc<Mutex<HashMap<String, Job>>>,
    next_id: Arc<Mutex<u64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Job {
    id: String,
    status: JobStatus,
    output: String,
    exit_code: Option<i32>,
    created_at: u64,
}

#[derive(Debug, Deserialize)]
struct ConvertRequest {
    input: String,
    output: String,
    input_type: Option<String>,
    output_type: Option<String>,
    crs: Option<String>,
    x_offset: Option<String>,
    y_offset: Option<String>,
    z_offset: Option<String>,
    longitude: Option<String>,
    latitude: Option<String>,
    java_path: Option<String>,
    jar_path: Option<String>,
    jvm_options: Option<String>,
    extra_args: Option<String>,
}

#[derive(Debug, Serialize)]
struct ConvertResponse {
    job_id: String,
}

#[derive(Debug, Deserialize)]
struct InstallUpdateRequest {
    url: String,
}

#[derive(Debug, Deserialize)]
struct ConvertPy3dtilesRequest {
    input: String,
    output: String,
    srs_in: Option<String>,
    srs_out: Option<String>,
    command: Option<String>,
    extra_args: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConvertPg2b3dmRequest {
    connection: String,
    table: String,
    column: Option<String>,
    output: Option<String>,
    attribute_columns: Option<String>,
    query: Option<String>,
    shader_column: Option<String>,
    geometric_errors: Option<String>,
    command: Option<String>,
    extra_args: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConvertGocesiumtilerRequest {
    input: String,
    output: String,
    epsg: Option<String>,
    resolution: Option<String>,
    depth: Option<String>,
    min_points_per_tile: Option<String>,
    version: Option<String>,
    command: Option<String>,
    extra_args: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PreprocessRequest {
    program: String,
    input: Option<String>,
    output: Option<String>,
    extra_args: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConvertBimcimRequest {
    tool: String,
    input: String,
    output: String,
    output_format: Option<String>,
    command: Option<String>,
    extra_args: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AutoConvertRequest {
    input: String,
    output: String,
    output_format: Option<String>,
    input_format: Option<String>,
    crs: Option<String>,
    x_offset: Option<String>,
    y_offset: Option<String>,
    z_offset: Option<String>,
    longitude: Option<String>,
    latitude: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConvertObjTo3dtiles11Request {
    input: String,
    output: String,
    crs: Option<String>,
    origin: Option<String>,
    x_offset: Option<String>,
    y_offset: Option<String>,
    z_offset: Option<String>,
    longitude: Option<String>,
    latitude: Option<String>,
    output_type: Option<String>,
    java_path: Option<String>,
    jar_path: Option<String>,
    jvm_options: Option<String>,
    mago_extra_args: Option<String>,
    tdt_command: Option<String>,
    tdt_extra_args: Option<String>,
}

#[tokio::main]
async fn main() {
    let open_browser = std::env::args().any(|a| a == "--open-browser");
    let port = 8590;
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    let listener = 'bind: loop {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => break 'bind l,
            Err(e) => {
                let health_url = format!("http://127.0.0.1:{}/health", port);
                let existing = reqwest::get(&health_url)
                    .await
                    .map(|r| r.status().is_success())
                    .unwrap_or(false);
                if !existing {
                    eprintln!("ポート {} で起動できません: {}", port, e);
                    std::process::exit(1);
                }
                println!("既にポート {} で起動しています。古いインスタンスを停止します。", port);
                let stop_url = format!("http://127.0.0.1:{}/api/v1/server/stop", port);
                let _ = reqwest::Client::new().post(&stop_url).send().await;
                for _ in 1..=60 {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    if let Ok(l) = tokio::net::TcpListener::bind(addr).await {
                        println!("ポート {} を確保しました。新しいインスタンスを起動します。", port);
                        break 'bind l;
                    }
                }
                eprintln!("ポート {} の解放を待ちましたが、起動できません: {}", port, e);
                std::process::exit(1);
            }
        }
    };

    let state = AppState {
        jobs: Arc::new(Mutex::new(HashMap::new())),
        next_id: Arc::new(Mutex::new(1)),
    };

    let api = Router::new()
        .route("/v1/server/stop", post(stop_handler))
        .route("/v1/server/version", get(version_handler))
        .route("/v1/server/update-check", get(update_check_handler))
        .route("/v1/server/install-update", post(install_update_handler))
        .route("/convert", post(convert_handler))
        .route("/convert/py3dtiles", post(convert_py3dtiles_handler))
        .route("/convert/pg2b3dm", post(convert_pg2b3dm_handler))
        .route("/convert/gocesiumtiler", post(convert_gocesiumtiler_handler))
        .route("/run/preprocess", post(preprocess_handler))
        .route("/convert/bimcim", post(convert_bimcim_handler))
        .route("/convert/obj-3dtiles11", post(convert_obj_to_3dtiles11_handler))
        .route("/convert/auto", post(convert_auto_handler))
        .route("/open-folder", post(open_folder_handler))
        .route("/install/{name}", post(install_handler))
        .route("/jobs/{id}", get(get_job))
        .route("/jobs", get(list_jobs))
        .route("/env/java", get(get_java_env))
        .route("/env/mago", get(get_mago_env))
        .route("/env/python", get(get_python_env))
        .route("/env/py3dtiles", get(get_py3dtiles_env))
        .route("/env/pg2b3dm", get(get_pg2b3dm_env))
        .route("/env/gocesiumtiler", get(get_gocesiumtiler_env))
        .route("/env/ifcopenshell", get(get_ifcopenshell_env))
        .route("/env/cjio", get(get_cjio_env))
        .route("/env/node", get(get_node_env))
        .route("/import/servers", get(import::get_servers_handler))
        .route("/import/search", post(import::search_handler))
        .route("/import/groups", get(import::groups_handler))
        .route("/import/download", post(import::download_handler))
        .route("/import/dpf/download-urls", post(import::dpf_download_urls_handler))
        .with_state(state);

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/health", get(health_handler))
        .nest("/api", api)
        .fallback_service(ServeDir::new("static"));

    println!("Kasuga converter server listening on http://127.0.0.1:8590");

    if open_browser {
        let open_url = format!("http://127.0.0.1:{}/", port);
        let health_url = format!("http://127.0.0.1:{}/health", port);
        tokio::spawn(async move {
            for _ in 0..60 {
                if let Ok(resp) = reqwest::get(&health_url).await {
                    if resp.status().is_success() {
                        let _ = opener::open(&open_url);
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
    }

    axum::serve(listener, app).await.unwrap();
}

async fn health_handler() -> impl IntoResponse {
    "ok"
}

async fn stop_handler() -> impl IntoResponse {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        std::process::exit(0);
    });
    "shutting down"
}

async fn version_handler() -> impl IntoResponse {
    Json(serde_json::json!({ "version": env!("CARGO_PKG_VERSION") })).into_response()
}

#[derive(Debug, Deserialize)]
struct LatestRelease {
    version: String,
    notes: String,
    platforms: HashMap<String, PlatformRelease>,
}

#[derive(Debug, Deserialize)]
struct PlatformRelease {
    url: String,
}

async fn update_check_handler() -> impl IntoResponse {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let remote =
        "https://raw.githubusercontent.com/yamamoto-ryuzo/kasugai_converter/main/download/latest.json";
    match reqwest::get(remote).await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(text) = resp.text().await {
                if let Ok(release) = serde_json::from_str::<LatestRelease>(&text) {
                    let update_available = release.version != current;
                    let url = release
                        .platforms
                        .get("windows-x86_64")
                        .map(|p| p.url.clone());
                    return Json(serde_json::json!({
                        "current": current,
                        "latest": release.version,
                        "url": url,
                        "update_available": update_available,
                        "notes": release.notes,
                    }))
                    .into_response();
                }
            }
        }
        _ => {}
    }
    (axum::http::StatusCode::SERVICE_UNAVAILABLE, "update check failed").into_response()
}

async fn install_update_handler(Json(req): Json<InstallUpdateRequest>) -> impl IntoResponse {
    let url = req.url.trim();
    if url.is_empty() || !url.starts_with("https://") {
        return (axum::http::StatusCode::BAD_REQUEST, "url must be an https URL").into_response();
    }
    if !url.ends_with(".zip") {
        return (axum::http::StatusCode::BAD_REQUEST, "url must point to a zip file").into_response();
    }

    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("current exe not found: {}", e),
            )
                .into_response();
        }
    };
    let app_dir = match current_exe.parent() {
        Some(d) => d.to_path_buf(),
        None => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "exe has no parent directory",
            )
                .into_response();
        }
    };
    let pid = std::process::id();

    let tmp_dir = std::env::temp_dir().join("kasugai_update");
    if tmp_dir.exists() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
    if let Err(e) = std::fs::create_dir_all(&tmp_dir) {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to create temp dir: {}", e),
        )
            .into_response();
    }

    let script = r#"
$ErrorActionPreference = "Stop"
$appDir = $args[0]
$url = $args[1]
$oldPid = $args[2]
$tmp = Join-Path $appDir "update_tmp"
if (Test-Path $tmp) { Remove-Item $tmp -Recurse -Force }
New-Item -ItemType Directory -Path $tmp -Force | Out-Null
$zip = Join-Path $tmp "kasugai_converter.zip"
Write-Output "Downloading update..."
curl.exe -s -S -L --fail -o $zip $url
if ($LASTEXITCODE -ne 0) { throw "download failed" }
Write-Output "Extracting update..."
Expand-Archive -Path $zip -DestinationPath $tmp -Force
Write-Output "Waiting for old process to exit..."
while (Get-Process -Id $oldPid -ErrorAction SilentlyContinue) { Start-Sleep -Milliseconds 200 }
$src = Join-Path $tmp "kasugai_converter.exe"
if (-not (Test-Path $src)) { throw "kasugai_converter.exe not found in archive" }
$srcStatic = Join-Path $tmp "static"
$srcResources = Join-Path $tmp "resources"
$srcTools = Join-Path $tmp "tools"
Copy-Item $src -Destination (Join-Path $appDir "kasugai_converter.exe") -Force
if (Test-Path $srcStatic) { Copy-Item $srcStatic -Destination $appDir -Recurse -Force }
if (Test-Path $srcResources) { Copy-Item $srcResources -Destination $appDir -Recurse -Force }
if (Test-Path $srcTools) { Copy-Item $srcTools -Destination $appDir -Recurse -Force }
Remove-Item $tmp -Recurse -Force
Write-Output "Starting new version..."
Start-Process -FilePath (Join-Path $appDir "kasugai_converter.exe") -WorkingDirectory $appDir -ArgumentList "--open-browser"
"#;

    let script_path = tmp_dir.join("updater.ps1");
    if let Err(e) = std::fs::write(&script_path, script) {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to write updater script: {}", e),
        )
            .into_response();
    }

    let app_dir_str = match app_dir.to_str() {
        Some(s) => s.to_string(),
        None => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "app dir path is not valid UTF-8",
            )
                .into_response();
        }
    };
    let script_path_str = match script_path.to_str() {
        Some(s) => s.to_string(),
        None => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "script path is not valid UTF-8",
            )
                .into_response();
        }
    };

    match std::process::Command::new("powershell")
        .arg("-WindowStyle")
        .arg("Hidden")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(&script_path_str)
        .arg(&app_dir_str)
        .arg(url)
        .arg(pid.to_string())
        .spawn()
    {
        Ok(_) => {
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(2000)).await;
                std::process::exit(0);
            });
            Json(serde_json::json!({
                "status": "updating",
                "message": "更新を適用し、再起動します。しばらくお待ちください。"
            }))
            .into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to spawn updater: {}", e),
        )
            .into_response(),
    }
}

async fn serve_index() -> impl IntoResponse {
    let html = match tokio::fs::read_to_string("static/index.html").await {
        Ok(s) => s,
        Err(_) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                axum::http::header::HeaderMap::new(),
                "index.html not found",
            )
                .into_response();
        }
    };
    let html = html.replace("<version>", env!("CARGO_PKG_VERSION"));
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        "text/html; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        axum::http::header::CACHE_CONTROL,
        "no-cache".parse().unwrap(),
    );
    (headers, html).into_response()
}

async fn convert_handler(
    State(state): State<AppState>,
    Json(req): Json<ConvertRequest>,
) -> impl IntoResponse {
    if req.input.trim().is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "input is required").into_response();
    }
    if req.output.trim().is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "output is required").into_response();
    }

    let java_path = req
        .java_path
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| std::env::var("MAGO_JAVA_PATH").unwrap_or_else(|_| "java".to_string()));

    let jar_path = req
        .jar_path
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            std::env::var("MAGO_JAR_PATH")
                .unwrap_or_else(|_| "tools/mago-3d-tiler.jar".to_string())
        });

    if !StdPath::new(&jar_path).exists() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            format!("jar not found: {}", jar_path),
        )
            .into_response();
    }

    let job_id = {
        let mut counter = state.next_id.lock().await;
        let id = counter.to_string();
        *counter += 1;
        id
    };
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    {
        let mut jobs = state.jobs.lock().await;
        jobs.insert(
            job_id.clone(),
            Job {
                id: job_id.clone(),
                status: JobStatus::Pending,
                output: String::new(),
                exit_code: None,
                created_at,
            },
        );
    }

    let mut args: Vec<String> = Vec::new();

    if let Some(jvm) = req.jvm_options.as_ref() {
        let trimmed = jvm.trim();
        if !trimmed.is_empty() {
            for part in trimmed.split_whitespace() {
                args.push(part.to_string());
            }
        }
    }

    args.push("-jar".to_string());
    args.push(jar_path);
    args.push("--input".to_string());
    args.push(req.input.trim().to_string());
    args.push("--output".to_string());
    args.push(req.output.trim().to_string());

    if let Some(it) = req.input_type.as_ref() {
        let trimmed = it.trim();
        if !trimmed.is_empty() {
            args.push("--inputType".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(ot) = req.output_type.as_ref() {
        let trimmed = ot.trim();
        if !trimmed.is_empty() {
            args.push("--outputType".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(crs) = req.crs.as_ref() {
        let trimmed = crs.trim();
        if !trimmed.is_empty() {
            args.push("--crs".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(v) = req.x_offset.as_ref() {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            args.push("--xOffset".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(v) = req.y_offset.as_ref() {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            args.push("--yOffset".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(v) = req.z_offset.as_ref() {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            args.push("--zOffset".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(v) = req.longitude.as_ref() {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            args.push("--longitude".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(v) = req.latitude.as_ref() {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            args.push("--latitude".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(extra) = req.extra_args.as_ref() {
        let trimmed = extra.trim();
        if !trimmed.is_empty() {
            for part in trimmed.split_whitespace() {
                args.push(part.to_string());
            }
        }
    }

    let response_id = job_id.clone();
    let state_spawn = state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_conversion(state_spawn, job_id.clone(), java_path, args).await {
            let mut jobs = state.jobs.lock().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = JobStatus::Failed;
                job.output.push_str(&format!("\n[system error] {}\n", e));
            }
        }
    });

    Json(ConvertResponse { job_id: response_id }).into_response()
}

async fn convert_py3dtiles_handler(
    State(state): State<AppState>,
    Json(req): Json<ConvertPy3dtilesRequest>,
) -> impl IntoResponse {
    if req.input.trim().is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "input is required").into_response();
    }
    if req.output.trim().is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "output is required").into_response();
    }

    let program = req
        .command
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "py3dtiles".to_string());

    let job_id = {
        let mut counter = state.next_id.lock().await;
        let id = counter.to_string();
        *counter += 1;
        id
    };
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    {
        let mut jobs = state.jobs.lock().await;
        jobs.insert(
            job_id.clone(),
            Job {
                id: job_id.clone(),
                status: JobStatus::Pending,
                output: String::new(),
                exit_code: None,
                created_at,
            },
        );
    }

    let mut args: Vec<String> = vec![
        "convert".to_string(),
        req.input.trim().to_string(),
        "--out".to_string(),
        req.output.trim().to_string(),
    ];

    if let Some(srs_in) = req.srs_in.as_ref() {
        let trimmed = srs_in.trim();
        if !trimmed.is_empty() {
            args.push("--srs-in".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(srs_out) = req.srs_out.as_ref() {
        let trimmed = srs_out.trim();
        if !trimmed.is_empty() {
            args.push("--srs-out".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(extra) = req.extra_args.as_ref() {
        let trimmed = extra.trim();
        if !trimmed.is_empty() {
            for part in trimmed.split_whitespace() {
                args.push(part.to_string());
            }
        }
    }

    let response_id = job_id.clone();
    let state_spawn = state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_conversion(state_spawn, job_id.clone(), program, args).await {
            let mut jobs = state.jobs.lock().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = JobStatus::Failed;
                job.output.push_str(&format!("\n[system error] {}\n", e));
            }
        }
    });

    Json(ConvertResponse { job_id: response_id }).into_response()
}

async fn convert_pg2b3dm_handler(
    State(state): State<AppState>,
    Json(req): Json<ConvertPg2b3dmRequest>,
) -> impl IntoResponse {
    if req.connection.trim().is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "connection is required").into_response();
    }
    if req.table.trim().is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "table is required").into_response();
    }

    let program = req
        .command
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "pg2b3dm".to_string());

    let job_id = {
        let mut counter = state.next_id.lock().await;
        let id = counter.to_string();
        *counter += 1;
        id
    };
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    {
        let mut jobs = state.jobs.lock().await;
        jobs.insert(
            job_id.clone(),
            Job {
                id: job_id.clone(),
                status: JobStatus::Pending,
                output: String::new(),
                exit_code: None,
                created_at,
            },
        );
    }

    let mut args: Vec<String> = vec![
        "--connection".to_string(),
        req.connection.trim().to_string(),
        "-t".to_string(),
        req.table.trim().to_string(),
    ];

    if let Some(col) = req.column.as_ref() {
        let trimmed = col.trim();
        if !trimmed.is_empty() {
            args.push("-c".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(out) = req.output.as_ref() {
        let trimmed = out.trim();
        if !trimmed.is_empty() {
            args.push("-o".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(attrs) = req.attribute_columns.as_ref() {
        let trimmed = attrs.trim();
        if !trimmed.is_empty() {
            args.push("-a".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(query) = req.query.as_ref() {
        let trimmed = query.trim();
        if !trimmed.is_empty() {
            args.push("-q".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(shader) = req.shader_column.as_ref() {
        let trimmed = shader.trim();
        if !trimmed.is_empty() {
            args.push("--shaderscolumn".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(errs) = req.geometric_errors.as_ref() {
        let trimmed = errs.trim();
        if !trimmed.is_empty() {
            args.push("-g".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(extra) = req.extra_args.as_ref() {
        let trimmed = extra.trim();
        if !trimmed.is_empty() {
            for part in trimmed.split_whitespace() {
                args.push(part.to_string());
            }
        }
    }

    let response_id = job_id.clone();
    let state_spawn = state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_conversion(state_spawn, job_id.clone(), program, args).await {
            let mut jobs = state.jobs.lock().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = JobStatus::Failed;
                job.output.push_str(&format!("\n[system error] {}\n", e));
            }
        }
    });

    Json(ConvertResponse { job_id: response_id }).into_response()
}

async fn convert_gocesiumtiler_handler(
    State(state): State<AppState>,
    Json(req): Json<ConvertGocesiumtilerRequest>,
) -> impl IntoResponse {
    if req.input.trim().is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "input is required").into_response();
    }
    if req.output.trim().is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "output is required").into_response();
    }

    let program = req
        .command
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "tools/gocesiumtiler/gocesiumtiler.exe".to_string());

    let job_id = {
        let mut counter = state.next_id.lock().await;
        let id = counter.to_string();
        *counter += 1;
        id
    };
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    {
        let mut jobs = state.jobs.lock().await;
        jobs.insert(
            job_id.clone(),
            Job {
                id: job_id.clone(),
                status: JobStatus::Pending,
                output: String::new(),
                exit_code: None,
                created_at,
            },
        );
    }

    let mut args: Vec<String> = vec!["file".to_string()];

    if let Some(epsg) = req.epsg.as_ref() {
        let trimmed = epsg.trim();
        if !trimmed.is_empty() {
            args.push("--epsg".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(res) = req.resolution.as_ref() {
        let trimmed = res.trim();
        if !trimmed.is_empty() {
            args.push("--resolution".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(depth) = req.depth.as_ref() {
        let trimmed = depth.trim();
        if !trimmed.is_empty() {
            args.push("--depth".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(min) = req.min_points_per_tile.as_ref() {
        let trimmed = min.trim();
        if !trimmed.is_empty() {
            args.push("--min-points-per-tile".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(version) = req.version.as_ref() {
        let trimmed = version.trim();
        if !trimmed.is_empty() {
            args.push("--version".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(extra) = req.extra_args.as_ref() {
        let trimmed = extra.trim();
        if !trimmed.is_empty() {
            for part in trimmed.split_whitespace() {
                args.push(part.to_string());
            }
        }
    }

    args.push("--out".to_string());
    args.push(req.output.trim().to_string());
    args.push(req.input.trim().to_string());

    let response_id = job_id.clone();
    let state_spawn = state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_conversion(state_spawn, job_id.clone(), program, args).await {
            let mut jobs = state.jobs.lock().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = JobStatus::Failed;
                job.output.push_str(&format!("\n[system error] {}\n", e));
            }
        }
    });

    Json(ConvertResponse { job_id: response_id }).into_response()
}

async fn preprocess_handler(
    State(state): State<AppState>,
    Json(req): Json<PreprocessRequest>,
) -> impl IntoResponse {
    let trimmed = req.program.trim().to_string();
    let mut parts: Vec<String> = trimmed
        .split_whitespace()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "program is required").into_response();
    }
    let program = parts.remove(0);

    let job_id = {
        let mut counter = state.next_id.lock().await;
        let id = counter.to_string();
        *counter += 1;
        id
    };
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    {
        let mut jobs = state.jobs.lock().await;
        jobs.insert(
            job_id.clone(),
            Job {
                id: job_id.clone(),
                status: JobStatus::Pending,
                output: String::new(),
                exit_code: None,
                created_at,
            },
        );
    }

    let mut args = parts;
    if let Some(extra) = req.extra_args.as_ref() {
        let trimmed = extra.trim();
        if !trimmed.is_empty() {
            for part in trimmed.split_whitespace() {
                args.push(part.to_string());
            }
        }
    }

    if let Some(input) = req.input.as_ref() {
        let trimmed = input.trim();
        if !trimmed.is_empty() {
            args.push(trimmed.to_string());
        }
    }

    if let Some(output) = req.output.as_ref() {
        let trimmed = output.trim();
        if !trimmed.is_empty() {
            args.push(trimmed.to_string());
        }
    }

    let response_id = job_id.clone();
    let state_spawn = state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_conversion(state_spawn, job_id.clone(), program, args).await {
            let mut jobs = state.jobs.lock().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = JobStatus::Failed;
                job.output.push_str(&format!("\n[system error] {}\n", e));
            }
        }
    });

    Json(ConvertResponse { job_id: response_id }).into_response()
}

async fn convert_bimcim_handler(
    State(state): State<AppState>,
    Json(req): Json<ConvertBimcimRequest>,
) -> impl IntoResponse {
    let tool = req.tool.trim().to_string();
    if tool.is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "tool is required").into_response();
    }
    if req.input.trim().is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "input is required").into_response();
    }
    if req.output.trim().is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "output is required").into_response();
    }

    let (program, mut args) = match tool.as_str() {
        "ifcconvert" => {
            let exe = req
                .command
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "tools/ifcopenshell/IfcConvert.exe".to_string());
            let script = format!(
                "cd (Split-Path -Parent '{}'); & (Split-Path -Leaf '{}') '{}' '{}'",
                exe, exe, req.input.trim().replace("'", "''"), req.output.trim().replace("'", "''")
            );
            ("powershell".to_string(), vec!["-Command".to_string(), script])
        }
        "cjio" => {
            let command = req
                .command
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "tools/python/Scripts/cjio.exe".to_string());
            let mut a = vec![req.input.trim().to_string(), "export".to_string()];
            if let Some(fmt) = req.output_format.as_ref() {
                let trimmed = fmt.trim();
                if !trimmed.is_empty() {
                    a.push(trimmed.to_string());
                }
            }
            a.push(req.output.trim().to_string());
            (command, a)
        }
        _ => {
            return (axum::http::StatusCode::BAD_REQUEST, "unknown tool").into_response();
        }
    };

    if let Some(extra) = req.extra_args.as_ref() {
        let trimmed = extra.trim();
        if !trimmed.is_empty() {
            for part in trimmed.split_whitespace() {
                args.push(part.to_string());
            }
        }
    }

    let job_id = {
        let mut counter = state.next_id.lock().await;
        let id = counter.to_string();
        *counter += 1;
        id
    };
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    {
        let mut jobs = state.jobs.lock().await;
        jobs.insert(
            job_id.clone(),
            Job {
                id: job_id.clone(),
                status: JobStatus::Pending,
                output: String::new(),
                exit_code: None,
                created_at,
            },
        );
    }

    let response_id = job_id.clone();
    let state_spawn = state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_conversion(state_spawn, job_id.clone(), program, args).await {
            let mut jobs = state.jobs.lock().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = JobStatus::Failed;
                job.output.push_str(&format!("\n[system error] {}\n", e));
            }
        }
    });

    Json(ConvertResponse { job_id: response_id }).into_response()
}

async fn convert_auto_handler(
    State(state): State<AppState>,
    Json(req): Json<AutoConvertRequest>,
) -> impl IntoResponse {
    if req.input.trim().is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "input is required").into_response();
    }
    if req.output.trim().is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "output is required").into_response();
    }

    let job_id = {
        let mut counter = state.next_id.lock().await;
        let id = counter.to_string();
        *counter += 1;
        id
    };
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    {
        let mut jobs = state.jobs.lock().await;
        jobs.insert(
            job_id.clone(),
            Job {
                id: job_id.clone(),
                status: JobStatus::Pending,
                output: String::new(),
                exit_code: None,
                created_at,
            },
        );
    }

    let response_id = job_id.clone();
    let input = req.input.trim().to_string();
    let output_format = req
        .output_format
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let input_format = req
        .input_format
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let crs = req
        .crs
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let x_offset = req
        .x_offset
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let y_offset = req
        .y_offset
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let z_offset = req
        .z_offset
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let longitude = req
        .longitude
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let latitude = req
        .latitude
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if let Some(fmt) = output_format.as_ref() {
        match fmt.as_str() {
            "b3dm" | "i3dm" => {
                let convert_req = ConvertRequest {
                    input: req.input.trim().to_string(),
                    output: req.output.trim().to_string(),
                    input_type: input_format,
                    output_type: Some(fmt.clone()),
                    crs,
                    x_offset,
                    y_offset,
                    z_offset,
                    longitude,
                    latitude,
                    java_path: None,
                    jar_path: None,
                    jvm_options: None,
                    extra_args: None,
                };
                return convert_handler(State(state.clone()), Json(convert_req)).await.into_response();
            }
            "pnts" => {
                let convert_req = ConvertGocesiumtilerRequest {
                    input: req.input.trim().to_string(),
                    output: req.output.trim().to_string(),
                    epsg: crs,
                    resolution: None,
                    depth: None,
                    min_points_per_tile: None,
                    version: None,
                    command: None,
                    extra_args: None,
                };
                return convert_gocesiumtiler_handler(State(state.clone()), Json(convert_req)).await.into_response();
            }
            "3dtiles-1-1-glb" => {
                let convert_req = ConvertObjTo3dtiles11Request {
                    input: req.input.trim().to_string(),
                    output: req.output.trim().to_string(),
                    crs,
                    origin: None,
                    x_offset,
                    y_offset,
                    z_offset,
                    longitude,
                    latitude,
                    output_type: None,
                    java_path: None,
                    jar_path: None,
                    jvm_options: None,
                    mago_extra_args: None,
                    tdt_command: None,
                    tdt_extra_args: None,
                };
                return convert_obj_to_3dtiles11_handler(State(state.clone()), Json(convert_req)).await.into_response();
            }
            _ => {}
        }
    }

    let state_spawn = state.clone();
    tokio::spawn(async move {
        {
            let mut jobs = state_spawn.jobs.lock().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = JobStatus::Running;
            }
        }

        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut message = format!("auto converter not yet implemented: {}", input);
        if let Some(fmt) = output_format {
            message.push_str(&format!(", output_format: {}", fmt));
        }
        if let Some(fmt) = input_format {
            message.push_str(&format!(", input_format: {}", fmt));
        }
        let mut jobs = state_spawn.jobs.lock().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            job.status = JobStatus::Completed;
            job.exit_code = Some(0);
            job.output = message;
        }
    });

    Json(ConvertResponse { job_id: response_id }).into_response()
}

async fn convert_obj_to_3dtiles11_handler(
    State(state): State<AppState>,
    Json(req): Json<ConvertObjTo3dtiles11Request>,
) -> impl IntoResponse {
    if req.input.trim().is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "input is required").into_response();
    }
    if req.output.trim().is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "output is required").into_response();
    }

    let input = req.input.trim();
    if !StdPath::new(input).exists() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            format!("input not found: {}", input),
        )
            .into_response();
    }

    let java_path = req
        .java_path
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| std::env::var("MAGO_JAVA_PATH").unwrap_or_else(|_| "java".to_string()));

    let jar_path = req
        .jar_path
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            std::env::var("MAGO_JAR_PATH")
                .unwrap_or_else(|_| "tools/mago-3d-tiler.jar".to_string())
        });

    if !StdPath::new(&jar_path).exists() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            format!("jar not found: {}", jar_path),
        )
            .into_response();
    }

    let tdt_command = req
        .tdt_command
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            std::env::var("TDT_COMMAND")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| {
                    let bundled_node = "tools/node/node.exe";
                    let bundled_npx_cli = "tools/node/node_modules/npm/bin/npx-cli.js";
                    if StdPath::new(bundled_node).exists() && StdPath::new(bundled_npx_cli).exists() {
                        format!("{} {} 3d-tiles-tools", bundled_node, bundled_npx_cli)
                    } else {
                        "npx 3d-tiles-tools".to_string()
                    }
                })
        });

    let job_id = {
        let mut counter = state.next_id.lock().await;
        let id = counter.to_string();
        *counter += 1;
        id
    };
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    {
        let mut jobs = state.jobs.lock().await;
        jobs.insert(
            job_id.clone(),
            Job {
                id: job_id.clone(),
                status: JobStatus::Pending,
                output: String::new(),
                exit_code: None,
                created_at,
            },
        );
    }

    let input = input.to_string();
    let output_base = req.output.trim().to_string();
    let mut crs = req
        .crs
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if crs.is_none() {
        if let Some(o) = req.origin.as_ref() {
            let o = o.trim().to_lowercase();
            if !o.is_empty() {
                crs = Some(match o.as_str() {
                    "jgd2011" => "6668".to_string(),
                    "jgd2000" => "4612".to_string(),
                    "tokyo" => "4301".to_string(),
                    _ => {
                        return (
                            axum::http::StatusCode::BAD_REQUEST,
                            format!("unknown geodetic origin: {}", o),
                        )
                            .into_response();
                    }
                });
            }
        }
    }

    let x_offset = req
        .x_offset
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let y_offset = req
        .y_offset
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let z_offset = req
        .z_offset
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let longitude = req
        .longitude
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let latitude = req
        .latitude
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let output_type = req
        .output_type
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "b3dm".to_string());
    let jvm_options = req
        .jvm_options
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let mago_extra_args = req
        .mago_extra_args
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let tdt_extra_args = req
        .tdt_extra_args
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "--targetVersion 1.1".to_string());

    let state_spawn = state.clone();
    let response_id = job_id.clone();

    tokio::spawn(async move {
        {
            let mut jobs = state_spawn.jobs.lock().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = JobStatus::Running;
            }
        }

        let work_dir = format!("{}/{}", output_base, job_id);
        let step10 = format!("{}/1.0", work_dir);
        let step11 = format!("{}/1.1", work_dir);
        let tileset_path = format!("{}/tileset.json", step10);

        let mut combined = String::new();
        combined.push_str(&format!("working directory: {}\n", work_dir));

        if let Err(e) = std::fs::create_dir_all(step10.as_str()) {
            let mut jobs = state_spawn.jobs.lock().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = JobStatus::Failed;
                job.output = format!("{}\nfailed to create 1.0 dir: {}", combined, e);
            }
            return;
        }
        if let Err(e) = std::fs::create_dir_all(step11.as_str()) {
            let mut jobs = state_spawn.jobs.lock().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = JobStatus::Failed;
                job.output = format!("{}\nfailed to create 1.1 dir: {}", combined, e);
            }
            return;
        }

        // mago-3d-tiler: OBJ -> 3D Tiles 1.0
        let mut mago_args: Vec<String> = Vec::new();
        if let Some(jvm) = jvm_options {
            for part in jvm.split_whitespace() {
                mago_args.push(part.to_string());
            }
        }
        mago_args.push("-jar".to_string());
        mago_args.push(jar_path);
        mago_args.push("--input".to_string());
        mago_args.push(input);
        mago_args.push("--output".to_string());
        mago_args.push(step10.clone());
        mago_args.push("--inputType".to_string());
        mago_args.push("obj".to_string());
        mago_args.push("--outputType".to_string());
        mago_args.push(output_type);
        if let Some(c) = crs {
            mago_args.push("--crs".to_string());
            mago_args.push(c);
        }
        if let Some(v) = x_offset {
            mago_args.push("--xOffset".to_string());
            mago_args.push(v);
        }
        if let Some(v) = y_offset {
            mago_args.push("--yOffset".to_string());
            mago_args.push(v);
        }
        if let Some(v) = z_offset {
            mago_args.push("--zOffset".to_string());
            mago_args.push(v);
        }
        if let Some(v) = longitude {
            mago_args.push("--longitude".to_string());
            mago_args.push(v);
        }
        if let Some(v) = latitude {
            mago_args.push("--latitude".to_string());
            mago_args.push(v);
        }
        if let Some(extra) = mago_extra_args {
            for part in extra.split_whitespace() {
                mago_args.push(part.to_string());
            }
        }

        match run_command(&java_path, &mago_args).await {
            Ok((code, out)) => {
                combined.push_str(&format!("[mago-3d-tiler] exit_code={}\n{}\n", code, out));
                if code != 0 {
                    let mut jobs = state_spawn.jobs.lock().await;
                    if let Some(job) = jobs.get_mut(&job_id) {
                        job.status = JobStatus::Failed;
                        job.exit_code = Some(code);
                        job.output = combined;
                    }
                    return;
                }
            }
            Err(e) => {
                let mut jobs = state_spawn.jobs.lock().await;
                if let Some(job) = jobs.get_mut(&job_id) {
                    job.status = JobStatus::Failed;
                    job.output = format!("{}\n[mago-3d-tiler] failed to start: {}", combined, e);
                }
                return;
            }
        }

        // 3d-tiles-tools: 3D Tiles 1.0 -> 1.1
        let mut tdt_cmd_parts: Vec<String> = tdt_command
            .split_whitespace()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if tdt_cmd_parts.is_empty() {
            tdt_cmd_parts.push("3d-tiles-tools".to_string());
        }
        let tdt_program = tdt_cmd_parts.remove(0);
        let mut tdt_args = tdt_cmd_parts;
        tdt_args.push("upgrade".to_string());
        tdt_args.push("-i".to_string());
        tdt_args.push(tileset_path);
        tdt_args.push("-o".to_string());
        tdt_args.push(step11.clone());
        if !tdt_extra_args.is_empty() {
            for part in tdt_extra_args.split_whitespace() {
                tdt_args.push(part.to_string());
            }
        }

        match run_command(&tdt_program, &tdt_args).await {
            Ok((code, out)) => {
                combined.push_str(&format!("[3d-tiles-tools] exit_code={}\n{}\n", code, out));
                let mut jobs = state_spawn.jobs.lock().await;
                if let Some(job) = jobs.get_mut(&job_id) {
                    job.exit_code = Some(code);
                    job.output = format!(
                        "{}\n1.0 output: {}\n1.1 output: {}",
                        combined, step10, step11
                    );
                    job.status = if code == 0 {
                        JobStatus::Completed
                    } else {
                        JobStatus::Failed
                    };
                }
            }
            Err(e) => {
                let mut jobs = state_spawn.jobs.lock().await;
                if let Some(job) = jobs.get_mut(&job_id) {
                    job.status = JobStatus::Failed;
                    job.output = format!(
                        "{}\n[3d-tiles-tools] failed to start: {}\n1.0 output remains at: {}",
                        combined, e, step10
                    );
                }
            }
        }
    });

    Json(ConvertResponse { job_id: response_id }).into_response()
}

#[derive(Debug, Deserialize)]
struct OpenFolderRequest {
    path: String,
}

async fn open_folder_handler(Json(req): Json<OpenFolderRequest>) -> impl IntoResponse {
    let path = req.path.trim();
    if path.is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "path is required").into_response();
    }
    match opener::open(path) {
        Ok(_) => (axum::http::StatusCode::OK, "opened").into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn install_handler(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let (program, args): (String, Vec<String>) = match name.as_str() {
        "python" => {
            let script = r#"
$ErrorActionPreference = "Stop"
$tools = "tools"
$tmp = "$tools/temp-python"
Remove-Item "$tools/python" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $tmp -Force
Write-Output "Downloading Python 3.12.4..."
curl.exe -s -S -L --fail -o "$tmp/python.zip" "https://www.python.org/ftp/python/3.12.4/python-3.12.4-embed-amd64.zip"
if ($LASTEXITCODE -ne 0) { throw "Python download failed" }
Write-Output "Extracting Python..."
Expand-Archive -Path "$tmp/python.zip" -DestinationPath "$tools/python" -Force
Remove-Item $tmp -Recurse -Force
$pth = Get-ChildItem "$tools/python" -Filter "python*._pth" | Select-Object -First 1
if ($pth) {
    $content = Get-Content $pth.FullName
    $content = $content -replace '^#import site', 'import site'
    $content | Set-Content $pth.FullName
}
$python = "$tools/python/python.exe"
Write-Output "Checking pip..."
& $python -m pip --version
if ($LASTEXITCODE -ne 0) {
    Write-Output "Bootstrapping pip..."
    curl.exe -s -S -L --fail -o "$tools/python/get-pip.py" "https://bootstrap.pypa.io/get-pip.py"
    if ($LASTEXITCODE -ne 0) { throw "get-pip.py download failed" }
    & $python "$tools/python/get-pip.py" --no-setuptools --no-wheel
    if ($LASTEXITCODE -ne 0) { throw "pip bootstrap failed" }
    Remove-Item "$tools/python/get-pip.py"
}
Write-Output "Upgrading pip..."
& $python -m pip install --upgrade pip --no-warn-script-location --progress-bar off
if ($LASTEXITCODE -ne 0) { throw "pip upgrade failed" }
Write-Output "Python installed to $tools/python"
"#;
            ("powershell".to_string(), vec!["-Command".to_string(), script.to_string()])
        }
        "jdk" => {
            let script = r#"
$ErrorActionPreference = "Stop"
$tools = "tools"
$tmp = "$tools/temp-jdk"
Remove-Item "$tools/jdk-21" -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $tmp -Force
Write-Host "Downloading JDK 21..."
curl.exe -s -S -L --fail -o "$tmp/jdk.zip" "https://api.adoptium.net/v3/binary/latest/21/ga/windows/x64/jdk/hotspot/normal/eclipse?project=jdk"
if ($LASTEXITCODE -ne 0) { throw "JDK download failed" }
Write-Host "Extracting JDK..."
Expand-Archive -Path "$tmp/jdk.zip" -DestinationPath $tmp -Force
$dir = Get-ChildItem $tmp -Directory | Select-Object -First 1
if (-not $dir) { throw "No extracted directory found" }
$dst = "$tools/jdk-21"
Move-Item $dir.FullName $dst -Force
Remove-Item $tmp -Recurse -Force
Write-Host "JDK installed to $dst"
"#;
            ("powershell".to_string(), vec!["-Command".to_string(), script.to_string()])
        }
        "mago" => {
            let script = r#"
$ErrorActionPreference = "Stop"
$tools = "tools"
New-Item -ItemType Directory -Path $tools -Force
Write-Host "Downloading mago-3d-tiler JAR..."
curl.exe -s -S -L --fail -o "$tools/mago-3d-tiler.jar" "https://github.com/Gaia3D/mago-3d-tiler/releases/download/v1.15.4/mago-3d-tiler-1.15.4.jar"
if ($LASTEXITCODE -ne 0) { throw "mago-3d-tiler download failed" }
Write-Host "JAR saved to $tools/mago-3d-tiler.jar"
"#;
            ("powershell".to_string(), vec!["-Command".to_string(), script.to_string()])
        }
        "py3dtiles" => {
            let script = r#"
$ErrorActionPreference = "Stop"
$tools = "tools"
$python = if (Test-Path "$tools/python/python.exe") { "$tools/python/python.exe" } else { "python" }
Write-Output "Upgrading pip..."
& $python -m pip install --upgrade pip --no-warn-script-location --progress-bar off
if ($LASTEXITCODE -ne 0) { throw "pip upgrade failed" }
Write-Output "Installing py3dtiles..."
& $python -m pip install --no-warn-script-location --progress-bar off py3dtiles
if ($LASTEXITCODE -ne 0) { throw "py3dtiles install failed" }
Write-Output "py3dtiles installed"
"#;
            ("powershell".to_string(), vec!["-Command".to_string(), script.to_string()])
        }
        "pg2b3dm" => {
            let script = r#"
$ErrorActionPreference = "Stop"
$tools = "tools"
$tmp = "$tools/temp-pg2b3dm"
$dst = "$tools/pg2b3dm"
Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item $dst -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $tmp -Force
Write-Host "Downloading pg2b3dm..."
curl.exe -s -S -L --fail -o "$tmp/pg2b3dm.zip" "https://github.com/Geodan/pg2b3dm/releases/download/v2.27.0/pg2b3dm-win-x64.zip"
if ($LASTEXITCODE -ne 0) { throw "pg2b3dm download failed" }
Write-Host "Extracting pg2b3dm..."
Expand-Archive -Path "$tmp/pg2b3dm.zip" -DestinationPath $tmp -Force
$exe = Get-ChildItem $tmp -Recurse -Filter "pg2b3dm.exe" | Select-Object -First 1
if (-not $exe) { throw "pg2b3dm.exe not found in archive" }
$dir = Split-Path $exe.FullName
New-Item -ItemType Directory -Path $dst -Force
Move-Item "$dir/*" $dst -Force -ErrorAction SilentlyContinue
Remove-Item $tmp -Recurse -Force
Write-Host "pg2b3dm installed to $dst"
"#;
            ("powershell".to_string(), vec!["-Command".to_string(), script.to_string()])
        }
        "gocesiumtiler" => {
            let script = r#"
$ErrorActionPreference = "Stop"
$tools = "tools"
$tmp = "$tools/temp-gocesiumtiler"
$dst = "$tools/gocesiumtiler"
Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item $dst -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $tmp -Force
Write-Output "Downloading gocesiumtiler..."
curl.exe -s -S -L --fail -o "$tmp/gocesiumtiler.zip" "https://github.com/mfbonfigli/gocesiumtiler/releases/download/v2.0.1/gocesiumtiler-v2.0.1.zip"
if ($LASTEXITCODE -ne 0) { throw "gocesiumtiler download failed" }
Write-Output "Extracting gocesiumtiler..."
Expand-Archive -Path "$tmp/gocesiumtiler.zip" -DestinationPath $tmp -Force
$exe = Get-ChildItem $tmp -Recurse -Filter "gocesiumtiler-win-x64.exe" | Select-Object -First 1
if (-not $exe) { throw "gocesiumtiler-win-x64.exe not found in archive" }
$dir = Split-Path $exe.FullName
New-Item -ItemType Directory -Path $dst -Force
Move-Item "$dir/*" $dst -Force -ErrorAction SilentlyContinue
if (Test-Path "$dst/gocesiumtiler.exe") { Remove-Item "$dst/gocesiumtiler.exe" -Force }
Rename-Item "$dst/gocesiumtiler-win-x64.exe" "gocesiumtiler.exe"
Remove-Item $tmp -Recurse -Force
Write-Output "gocesiumtiler installed to $dst"
"#;
            ("powershell".to_string(), vec!["-Command".to_string(), script.to_string()])
        }
        "ifcopenshell" => {
            let script = r#"
$ErrorActionPreference = "Stop"
$tools = "tools"
$tmp = "$tools/temp-ifcopenshell"
$dst = "$tools/ifcopenshell"
Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item $dst -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $tmp -Force
Write-Output "Downloading IfcOpenShell..."
curl.exe -s -S -L --fail -o "$tmp/ifcopenshell.zip" "https://github.com/IfcOpenShell/IfcOpenShell/releases/download/ifcconvert-0.8.4/ifcconvert-0.8.4-win64.zip"
if ($LASTEXITCODE -ne 0) { throw "ifcopenshell download failed" }
Write-Output "Extracting IfcOpenShell..."
Expand-Archive -Path "$tmp/ifcopenshell.zip" -DestinationPath $tmp -Force
$exe = Get-ChildItem $tmp -Recurse -Filter "IfcConvert.exe" | Select-Object -First 1
if (-not $exe) { throw "IfcConvert.exe not found in archive" }
$dir = Split-Path $exe.FullName
New-Item -ItemType Directory -Path $dst -Force
Move-Item "$dir/*" $dst -Force -ErrorAction SilentlyContinue
Remove-Item $tmp -Recurse -Force
Write-Output "ifcopenshell installed to $dst"
"#;
            ("powershell".to_string(), vec!["-Command".to_string(), script.to_string()])
        }
        "cjio" => {
            let script = r#"
$ErrorActionPreference = "Stop"
$tools = "tools"
$python = if (Test-Path "$tools/python/python.exe") { "$tools/python/python.exe" } else { "python" }
Write-Output "Installing cjio..."
& $python -m pip install --no-warn-script-location --progress-bar off cjio
if ($LASTEXITCODE -ne 0) { throw "cjio install failed" }
Write-Output "cjio installed"
"#;
            ("powershell".to_string(), vec!["-Command".to_string(), script.to_string()])
        }
        "node" => {
            let version = std::env::var("NODE_VERSION")
                .unwrap_or_else(|_| "v24.18.0".to_string());
            let script = r#"
$ErrorActionPreference = "Stop"
$tools = "tools"
$tmp = "$tools/temp-node"
$dst = "$tools/node"
Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item $dst -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $tmp -Force
Write-Output "Downloading Node.js {version}..."
curl.exe -s -S -L --fail -o "$tmp/node.zip" "https://nodejs.org/dist/{version}/node-{version}-win-x64.zip"
if ($LASTEXITCODE -ne 0) { throw "node download failed" }
Write-Output "Extracting Node.js..."
Expand-Archive -Path "$tmp/node.zip" -DestinationPath $tmp -Force
$dir = Get-ChildItem $tmp -Directory | Select-Object -First 1
if (-not $dir) { throw "node directory not found in archive" }
New-Item -ItemType Directory -Path $dst -Force
Move-Item "$($dir.FullName)/*" $dst -Force -ErrorAction SilentlyContinue
Remove-Item $tmp -Recurse -Force
Write-Output "node installed to $dst"
"#;
            let script = script.replace("{version}", &version);
            ("powershell".to_string(), vec!["-Command".to_string(), script])
        }
        _ => {
            return (axum::http::StatusCode::NOT_FOUND, "install target not found").into_response();
        }
    };

    let job_id = {
        let mut counter = state.next_id.lock().await;
        let id = counter.to_string();
        *counter += 1;
        id
    };
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    {
        let mut jobs = state.jobs.lock().await;
        jobs.insert(
            job_id.clone(),
            Job {
                id: job_id.clone(),
                status: JobStatus::Pending,
                output: String::new(),
                exit_code: None,
                created_at,
            },
        );
    }

    let response_id = job_id.clone();
    let state_spawn = state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_conversion(state_spawn, job_id.clone(), program, args).await {
            let mut jobs = state.jobs.lock().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = JobStatus::Failed;
                job.output.push_str(&format!("\n[system error] {}\n", e));
            }
        }
    });

    Json(ConvertResponse { job_id: response_id }).into_response()
}

async fn get_job(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let jobs = state.jobs.lock().await;
    match jobs.get(&id) {
        Some(job) => Json(job.clone()).into_response(),
        None => (axum::http::StatusCode::NOT_FOUND, "job not found").into_response(),
    }
}

async fn list_jobs(State(state): State<AppState>) -> impl IntoResponse {
    let jobs = state.jobs.lock().await;
    let values: Vec<Job> = jobs.values().cloned().collect();
    Json(values).into_response()
}

async fn run_conversion(
    state: AppState,
    job_id: String,
    java_path: String,
    args: Vec<String>,
) -> Result<(), String> {
    {
        let mut jobs = state.jobs.lock().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            job.status = JobStatus::Running;
        }
    }

    let mut child = Command::new(&java_path)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("failed to start {}: {}", java_path, e))?;

    let log = Arc::new(Mutex::new(String::new()));
    let out_log = log.clone();
    let err_log = log.clone();

    let stdout = child.stdout.take().ok_or("stdout not piped")?;
    let stderr = child.stderr.take().ok_or("stderr not piped")?;

    let out_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let mut l = out_log.lock().await;
            l.push_str(&line);
            l.push('\n');
        }
    });

    let err_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let mut l = err_log.lock().await;
            l.push_str(&line);
            l.push('\n');
        }
    });

    let status = child
        .wait()
        .await
        .map_err(|e| format!("process error: {}", e))?;

    out_task.await.unwrap_or(());
    err_task.await.unwrap_or(());

    let output = log.lock().await.clone();
    let mut jobs = state.jobs.lock().await;
    if let Some(job) = jobs.get_mut(&job_id) {
        job.exit_code = status.code();
        job.output = output;
        job.status = if status.success() {
            JobStatus::Completed
        } else {
            JobStatus::Failed
        };
    }

    Ok(())
}

async fn run_command(program: &str, args: &[String]) -> Result<(i32, String), String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("failed to start {}: {}", program, e))?;

    let log = Arc::new(Mutex::new(String::new()));
    let out_log = log.clone();
    let err_log = log.clone();

    let stdout = child.stdout.take().ok_or("stdout not piped")?;
    let stderr = child.stderr.take().ok_or("stderr not piped")?;

    let out_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let mut l = out_log.lock().await;
            l.push_str(&line);
            l.push('\n');
        }
    });

    let err_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let mut l = err_log.lock().await;
            l.push_str(&line);
            l.push('\n');
        }
    });

    let status = child
        .wait()
        .await
        .map_err(|e| format!("process error: {}", e))?;

    out_task.await.unwrap_or(());
    err_task.await.unwrap_or(());

    let output = log.lock().await.clone();
    let exit_code = status.code().unwrap_or(-1);
    Ok((exit_code, output))
}

#[derive(Debug, Serialize)]
struct JavaEnv {
    found: bool,
    path: Option<String>,
    version_output: String,
}

#[derive(Debug, Serialize)]
struct MagoEnv {
    found: bool,
    path: String,
    size: Option<u64>,
}

async fn get_java_env() -> impl IntoResponse {
    let mut path: Option<String> = None;

    if let Ok(p) = std::env::var("MAGO_JAVA_PATH") {
        let trimmed = p.trim().to_string();
        if !trimmed.is_empty() && StdPath::new(&trimmed).exists() {
            path = Some(trimmed);
        }
    }

    if path.is_none() {
        let tools_java = "tools/jdk-21/bin/java.exe";
        if StdPath::new(tools_java).exists() {
            path = Some(tools_java.to_string());
        }
    }

    if path.is_none() {
        match Command::new("where").arg("java").output().await {
            Ok(out) if out.status.success() => {
                let first = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !first.is_empty() {
                    path = Some(first);
                }
            }
            _ => {}
        }
    }

    let mut found = false;
    let mut version_output = String::new();
    if let Some(p) = path.as_ref() {
        match Command::new(p).arg("-version").output().await {
            Ok(out) => {
                version_output = String::from_utf8_lossy(&out.stderr).to_string();
                found = out.status.success();
            }
            _ => {}
        }
    }

    Json(JavaEnv {
        found,
        path,
        version_output,
    })
}

async fn get_mago_env() -> impl IntoResponse {
    let default_path = "tools/mago-3d-tiler.jar".to_string();
    let env_path = std::env::var("MAGO_JAR_PATH").ok();
    let path = match env_path {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ if StdPath::new(&default_path).exists() => default_path.clone(),
        _ => default_path,
    };

    let (found, size) = match std::fs::metadata(&path) {
        Ok(meta) => (true, Some(meta.len())),
        Err(_) => (false, None),
    };

    Json(MagoEnv { found, path, size })
}

#[derive(Debug, Serialize)]
struct PythonEnv {
    found: bool,
    path: Option<String>,
    version_output: String,
}

#[derive(Debug, Serialize)]
struct Py3dtilesEnv {
    found: bool,
    path: String,
    version_output: String,
}

#[derive(Debug, Serialize)]
struct Pg2b3dmEnv {
    found: bool,
    path: String,
    version_output: String,
}

async fn get_python_env() -> impl IntoResponse {
    let mut path: Option<String> = None;

    if let Ok(p) = std::env::var("MAGO_PYTHON_PATH") {
        let trimmed = p.trim().to_string();
        if !trimmed.is_empty() && StdPath::new(&trimmed).exists() {
            path = Some(trimmed);
        }
    }

    if path.is_none() {
        let tools_python = "tools/python/python.exe";
        if StdPath::new(tools_python).exists() {
            path = Some(tools_python.to_string());
        }
    }

    if path.is_none() {
        match Command::new("where").arg("python").output().await {
            Ok(out) if out.status.success() => {
                let first = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !first.is_empty() {
                    path = Some(first);
                }
            }
            _ => {}
        }
    }

    let mut found = false;
    let mut version_output = String::new();
    if let Some(p) = path.as_ref() {
        match Command::new(p).arg("--version").output().await {
            Ok(out) => {
                version_output = String::from_utf8_lossy(&out.stdout).to_string();
                version_output.push_str(&String::from_utf8_lossy(&out.stderr));
                found = out.status.success();
            }
            _ => {}
        }
    }

    Json(PythonEnv {
        found,
        path,
        version_output,
    })
}

async fn get_py3dtiles_env() -> impl IntoResponse {
    let default_path = "tools/py3dtiles-venv/Scripts/py3dtiles.exe".to_string();
    let alt_path = "tools/python/Scripts/py3dtiles.exe".to_string();
    let env_path = std::env::var("PY3DTILES_PATH").ok();
    let path = match env_path {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ if StdPath::new(&default_path).exists() => default_path.clone(),
        _ if StdPath::new(&alt_path).exists() => alt_path.clone(),
        _ => default_path,
    };

    let mut found = false;
    let mut version_output = String::new();
    if StdPath::new(&path).exists() {
        match Command::new(&path).arg("--help").output().await {
            Ok(out) => {
                version_output = String::from_utf8_lossy(&out.stdout).to_string();
                version_output.push_str(&String::from_utf8_lossy(&out.stderr));
                found = out.status.success();
            }
            _ => {}
        }
    }

    Json(Py3dtilesEnv {
        found,
        path,
        version_output,
    })
}

async fn get_pg2b3dm_env() -> impl IntoResponse {
    let default_path = "tools/pg2b3dm/pg2b3dm.exe".to_string();
    let env_path = std::env::var("PG2B3DM_PATH").ok();
    let path = match env_path {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ if StdPath::new(&default_path).exists() => default_path.clone(),
        _ => default_path,
    };

    let mut found = false;
    let mut version_output = String::new();
    if StdPath::new(&path).exists() {
        match Command::new(&path).arg("--version").output().await {
            Ok(out) => {
                version_output = String::from_utf8_lossy(&out.stdout).to_string();
                version_output.push_str(&String::from_utf8_lossy(&out.stderr));
                found = out.status.success();
            }
            _ => {}
        }
    }

    Json(Pg2b3dmEnv {
        found,
        path,
        version_output,
    })
}

#[derive(Debug, Serialize)]
struct GocesiumtilerEnv {
    found: bool,
    path: String,
    version_output: String,
}

async fn get_gocesiumtiler_env() -> impl IntoResponse {
    let default_path = "tools/gocesiumtiler/gocesiumtiler.exe".to_string();
    let env_path = std::env::var("GOCESIUMTILER_PATH").ok();
    let path = match env_path {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ if StdPath::new(&default_path).exists() => default_path.clone(),
        _ => default_path,
    };

    let mut found = false;
    let mut version_output = String::new();
    if StdPath::new(&path).exists() {
        match Command::new(&path).arg("--help").output().await {
            Ok(out) => {
                version_output = String::from_utf8_lossy(&out.stdout).to_string();
                version_output.push_str(&String::from_utf8_lossy(&out.stderr));
                found = out.status.success();
            }
            _ => {}
        }
    }

    Json(GocesiumtilerEnv {
        found,
        path,
        version_output,
    })
}

#[derive(Debug, Serialize)]
struct IfcOpenShellEnv {
    found: bool,
    path: String,
    version_output: String,
}

async fn get_ifcopenshell_env() -> impl IntoResponse {
    let default_path = "tools/ifcopenshell/IfcConvert.exe".to_string();
    let env_path = std::env::var("IFCCONVERT_PATH").ok();
    let path = match env_path {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ if StdPath::new(&default_path).exists() => default_path.clone(),
        _ => default_path,
    };

    let mut found = false;
    let mut version_output = String::new();
    if StdPath::new(&path).exists() {
        match Command::new(&path).arg("--version").output().await {
            Ok(out) => {
                version_output = String::from_utf8_lossy(&out.stdout).to_string();
                version_output.push_str(&String::from_utf8_lossy(&out.stderr));
                found = out.status.success();
            }
            _ => {}
        }
    }

    Json(IfcOpenShellEnv {
        found,
        path,
        version_output,
    })
}

#[derive(Debug, Serialize)]
struct CjioEnv {
    found: bool,
    path: String,
    version_output: String,
}

async fn get_cjio_env() -> impl IntoResponse {
    let default_path = "tools/python/Scripts/cjio.exe".to_string();
    let env_path = std::env::var("CJIO_PATH").ok();
    let path = match env_path {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ if StdPath::new(&default_path).exists() => default_path.clone(),
        _ => default_path,
    };

    let mut found = false;
    let mut version_output = String::new();
    if StdPath::new(&path).exists() {
        match Command::new(&path).arg("--help").output().await {
            Ok(out) => {
                version_output = String::from_utf8_lossy(&out.stdout).to_string();
                version_output.push_str(&String::from_utf8_lossy(&out.stderr));
                found = out.status.success();
            }
            _ => {}
        }
    }

    Json(CjioEnv {
        found,
        path,
        version_output,
    })
}

#[derive(Debug, Serialize)]
struct NodeEnv {
    found: bool,
    path: Option<String>,
    version_output: String,
}

async fn get_node_env() -> impl IntoResponse {
    let mut path: Option<String> = None;

    if let Ok(p) = std::env::var("NODE_PATH") {
        let trimmed = p.trim().to_string();
        if !trimmed.is_empty() && StdPath::new(&trimmed).exists() {
            path = Some(trimmed);
        }
    }

    if path.is_none() {
        let tools_node = "tools/node/node.exe";
        if StdPath::new(tools_node).exists() {
            path = Some(tools_node.to_string());
        }
    }

    if path.is_none() {
        match Command::new("where").arg("node").output().await {
            Ok(out) if out.status.success() => {
                let first = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !first.is_empty() {
                    path = Some(first);
                }
            }
            _ => {}
        }
    }

    let mut found = false;
    let mut version_output = String::new();
    if let Some(p) = path.as_ref() {
        match Command::new(p).arg("--version").output().await {
            Ok(out) => {
                version_output = String::from_utf8_lossy(&out.stdout).to_string();
                version_output.push_str(&String::from_utf8_lossy(&out.stderr));
                found = out.status.success();
            }
            _ => {}
        }
    }

    Json(NodeEnv {
        found,
        path,
        version_output,
    })
}
