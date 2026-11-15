use axum::extract::Query;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::path::{Path, PathBuf};

pub async fn inspect_handler(Query(params): Query<HashMap<String, String>>) -> Response {
    let Some(path) = params.get("path") else {
        return (axum::http::StatusCode::BAD_REQUEST, "path is required").into_response();
    };
    let p = Path::new(path.trim());
    if !p.exists() {
        return (axum::http::StatusCode::NOT_FOUND, "path not found").into_response();
    }
    Json(if p.is_dir() {
        inspect_dir(p)
    } else {
        inspect_file(p)
    })
    .into_response()
}

fn inspect_dir(dir: &Path) -> Value {
    let mut exts: BTreeMap<String, u64> = BTreeMap::new();
    let mut file_count: u64 = 0;
    let mut total_size: u64 = 0;
    let mut tileset: Option<PathBuf> = None;
    let mut stack: Vec<(PathBuf, u32)> = vec![(dir.to_path_buf(), 0)];
    while let Some((d, depth)) = stack.pop() {
        if depth > 8 || file_count > 100_000 {
            break;
        }
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push((p, depth + 1));
            } else {
                file_count += 1;
                total_size += e.metadata().map(|m| m.len()).unwrap_or(0);
                let ext = p
                    .extension()
                    .and_then(|x| x.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                *exts.entry(ext).or_insert(0) += 1;
                if tileset.is_none() && p.file_name().and_then(|n| n.to_str()) == Some("tileset.json")
                {
                    tileset = Some(p);
                }
            }
        }
    }
    let tile_count: u64 = ["b3dm", "i3dm", "pnts", "glb", "cmpt"]
        .iter()
        .map(|k| exts.get(*k).copied().unwrap_or(0))
        .sum();
    let mut m = Map::new();
    m.insert("kind".into(), "directory".into());
    m.insert("path".into(), dir.to_string_lossy().into());
    m.insert("file_count".into(), file_count.into());
    m.insert("total_size".into(), total_size.into());
    m.insert("extensions".into(), json!(exts));
    m.insert("tile_count".into(), tile_count.into());
    let mut warnings: Vec<String> = Vec::new();
    match tileset {
        Some(ts) => {
            m.insert("tileset_json".into(), ts.to_string_lossy().into());
            match std::fs::read_to_string(&ts)
                .ok()
                .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            {
                Some(v) => {
                    if let Some(bv) = v.get("root").and_then(|r| r.get("boundingVolume")) {
                        m.insert("bounding_volume".into(), bv.clone());
                    } else {
                        warnings.push("tileset.json に root.boundingVolume がありません".into());
                    }
                }
                None => warnings.push("tileset.json の解析に失敗しました".into()),
            }
        }
        None => warnings.push("tileset.json が見つかりません".into()),
    }
    m.insert("warnings".into(), json!(warnings));
    Value::Object(m)
}

fn driver_for(ext: &str) -> &'static str {
    match ext {
        "geojson" | "json" => "GeoJSON",
        "shp" => "ESRI Shapefile",
        "gpkg" => "GeoPackage",
        "las" => "LAS point cloud",
        "laz" => "LAZ point cloud",
        "tif" | "tiff" => "GeoTIFF",
        "gml" | "xml" => "CityGML/IndoorGML",
        "ifc" => "IFC",
        "obj" => "OBJ",
        "glb" | "gltf" => "glTF",
        "fbx" => "FBX",
        "3ds" => "3DS",
        "b3dm" | "i3dm" | "pnts" | "cmpt" => "3D Tiles tile",
        "terrain" | "terraindb" => "Terrain tile",
        "cityjson" => "CityJSON",
        _ => "unknown",
    }
}

fn inspect_file(p: &Path) -> Value {
    let size = p.metadata().map(|m| m.len()).unwrap_or(0);
    let ext = p
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mut warnings: Vec<String> = Vec::new();
    let mut m = Map::new();
    m.insert("kind".into(), "file".into());
    m.insert("path".into(), p.to_string_lossy().into());
    m.insert("size".into(), size.into());
    m.insert("driver".into(), driver_for(&ext).into());
    match ext.as_str() {
        "geojson" | "json" | "cityjson" => {
            if size > 512 * 1024 * 1024 {
                warnings.push("ファイルが大きすぎるため詳細解析を省略しました".into());
            } else if let Ok(text) = std::fs::read_to_string(p) {
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    inspect_geojson(&v, &mut m, &mut warnings);
                } else {
                    warnings.push("JSON の解析に失敗しました".into());
                }
            } else {
                warnings.push("ファイルの読み込みに失敗しました".into());
            }
        }
        "las" | "laz" => inspect_las(p, &mut m, &mut warnings),
        _ => warnings.push("この形式の詳細サマリには未対応です".into()),
    }
    m.insert("warnings".into(), json!(warnings));
    Value::Object(m)
}

fn walk_coords(v: &Value, bbox: &mut [f64; 4]) {
    if let Some(arr) = v.as_array() {
        if arr.len() >= 2 && arr[0].is_f64() && arr[1].is_f64() {
            let x = arr[0].as_f64().unwrap();
            let y = arr[1].as_f64().unwrap();
            if x < bbox[0] {
                bbox[0] = x;
            }
            if y < bbox[1] {
                bbox[1] = y;
            }
            if x > bbox[2] {
                bbox[2] = x;
            }
            if y > bbox[3] {
                bbox[3] = y;
            }
        }
        for a in arr {
            if a.is_array() || a.is_object() {
                walk_coords(a, bbox);
            }
        }
    }
}

fn inspect_geojson(v: &Value, m: &mut Map<String, Value>, warnings: &mut Vec<String>) {
    if let Some(t) = v.get("type").and_then(|t| t.as_str()) {
        m.insert("geojson_type".into(), t.into());
    }
    match v.get("crs") {
        Some(c) => {
            m.insert("crs".into(), c.clone());
        }
        None => warnings.push("CRS メンバーがありません（GeoJSON 既定は WGS84）".into()),
    }
    let empty: Vec<Value> = Vec::new();
    let feats: Vec<&Value> = match v.get("type").and_then(|t| t.as_str()) {
        Some("FeatureCollection") => v
            .get("features")
            .and_then(|f| f.as_array())
            .unwrap_or(&empty)
            .iter()
            .collect(),
        Some("Feature") => vec![v],
        _ => Vec::new(),
    };
    let mut gtypes: BTreeMap<String, u64> = BTreeMap::new();
    let mut attrs: BTreeMap<String, u64> = BTreeMap::new();
    let mut bbox = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
    for f in &feats {
        if let Some(g) = f.get("geometry") {
            if let Some(t) = g.get("type").and_then(|t| t.as_str()) {
                *gtypes.entry(t.to_string()).or_insert(0) += 1;
            }
            if let Some(c) = g.get("coordinates") {
                walk_coords(c, &mut bbox);
            }
        }
        if attrs.len() < 100 {
            if let Some(obj) = f.get("properties").and_then(|p| p.as_object()) {
                for k in obj.keys() {
                    *attrs.entry(k.clone()).or_insert(0) += 1;
                }
            }
        }
    }
    if !feats.is_empty() {
        m.insert("feature_count".into(), feats.len().into());
        m.insert("geometry_types".into(), json!(gtypes));
        m.insert(
            "attributes".into(),
            json!(attrs.keys().collect::<Vec<_>>()),
        );
        if bbox[0] <= bbox[2] {
            m.insert("extent".into(), json!(bbox));
        }
    }
}

fn ru16(b: &[u8], o: usize) -> Option<u16> {
    b.get(o..o + 2).map(|x| u16::from_le_bytes([x[0], x[1]]))
}
fn ru32(b: &[u8], o: usize) -> Option<u32> {
    b.get(o..o + 4)
        .map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
}
fn ru64(b: &[u8], o: usize) -> Option<u64> {
    b.get(o..o + 8)
        .map(|x| u64::from_le_bytes(x.try_into().unwrap()))
}
fn rf64(b: &[u8], o: usize) -> Option<f64> {
    b.get(o..o + 8)
        .map(|x| f64::from_le_bytes(x.try_into().unwrap()))
}

fn inspect_las(p: &Path, m: &mut Map<String, Value>, warnings: &mut Vec<String>) {
    let buf = std::fs::File::open(p)
        .ok()
        .map(|mut f| {
            let mut b = vec![0u8; 1 << 16];
            let n = f.read(&mut b).unwrap_or(0);
            b.truncate(n);
            b
        })
        .unwrap_or_default();
    if buf.len() < 227 || &buf[0..4] != b"LASF" {
        warnings.push("LAS ヘッダを読み取れません".into());
        return;
    }
    let major = buf[24];
    let minor = buf[25];
    m.insert("las_version".into(), format!("{}.{}", major, minor).into());
    m.insert("point_format".into(), (buf[104] & 0x3f).into());
    let mut count = ru32(&buf, 107).unwrap_or(0) as u64;
    if major == 1 && minor >= 4 {
        if let Some(c) = ru64(&buf, 247) {
            if c > 0 {
                count = c;
            }
        }
    }
    m.insert("point_count".into(), count.into());
    m.insert(
        "extent".into(),
        json!({
            "x": [rf64(&buf, 187), rf64(&buf, 179)],
            "y": [rf64(&buf, 203), rf64(&buf, 195)],
            "z": [rf64(&buf, 219), rf64(&buf, 211)],
        }),
    );
    let header_size = ru16(&buf, 94).unwrap_or(0) as usize;
    let num_vlrs = ru32(&buf, 100).unwrap_or(0) as usize;
    let point_off = ru32(&buf, 96).unwrap_or(u32::MAX) as usize;
    let mut crs: Option<String> = None;
    let mut off = header_size;
    for _ in 0..num_vlrs {
        if off + 54 > buf.len() || off >= point_off {
            break;
        }
        let uid = String::from_utf8_lossy(&buf[off + 2..off + 18])
            .trim_matches('\0')
            .to_string();
        let rid = ru16(&buf, off + 18).unwrap_or(0);
        let len = ru16(&buf, off + 20).unwrap_or(0) as usize;
        let end = (off + 54 + len).min(buf.len()).min(point_off);
        let data = &buf[(off + 54).min(end)..end];
        if uid == "LASF_Projection" {
            if rid == 21112 {
                let wkt = String::from_utf8_lossy(data).trim_matches('\0').to_string();
                if !wkt.is_empty() {
                    crs = Some(wkt.chars().take(500).collect());
                }
            } else if rid == 34735 && data.len() >= 8 {
                let n = ru16(data, 6).unwrap_or(0) as usize;
                for i in 0..n {
                    let e = 8 + i * 8;
                    if e + 8 > data.len() {
                        break;
                    }
                    let key = ru16(data, e).unwrap_or(0);
                    let loc = ru16(data, e + 2).unwrap_or(0);
                    if (key == 2048 || key == 3072) && loc == 0 {
                        crs = Some(format!("EPSG:{}", ru16(data, e + 6).unwrap_or(0)));
                    }
                }
            }
        }
        off += 54 + len;
    }
    match crs {
        Some(c) => {
            m.insert("crs".into(), c.into());
        }
        None => warnings.push("CRS を特定できません（VLR に投影情報がありません）".into()),
    }
}
