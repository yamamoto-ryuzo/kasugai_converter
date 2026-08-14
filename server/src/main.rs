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
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

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
    command: Option<String>,
    extra_args: Option<String>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        jobs: Arc::new(Mutex::new(HashMap::new())),
        next_id: Arc::new(Mutex::new(1)),
    };

    let api = Router::new()
        .route("/convert", post(convert_handler))
        .route("/convert/py3dtiles", post(convert_py3dtiles_handler))
        .route("/convert/pg2b3dm", post(convert_pg2b3dm_handler))
        .route("/convert/gocesiumtiler", post(convert_gocesiumtiler_handler))
        .route("/install/{name}", post(install_handler))
        .route("/jobs/{id}", get(get_job))
        .route("/jobs", get(list_jobs))
        .route("/env/java", get(get_java_env))
        .route("/env/mago", get(get_mago_env))
        .route("/env/python", get(get_python_env))
        .route("/env/py3dtiles", get(get_py3dtiles_env))
        .route("/env/pg2b3dm", get(get_pg2b3dm_env))
        .route("/env/gocesiumtiler", get(get_gocesiumtiler_env))
        .with_state(state);

    let app = Router::new()
        .nest("/api", api)
        .fallback_service(ServeDir::new("static"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8590").await.unwrap();
    println!("Kasuga converter server listening on http://127.0.0.1:8590");
    axum::serve(listener, app).await.unwrap();
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
        match Command::new(&path).arg("-h").output().await {
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
