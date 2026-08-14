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
        .route("/jobs/{id}", get(get_job))
        .route("/jobs", get(list_jobs))
        .route("/env/java", get(get_java_env))
        .route("/env/mago", get(get_mago_env))
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
