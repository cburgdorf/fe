use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceViewManifest {
    pub revision: u64,
    pub root_digest: String,
    pub summary_digest: String,
    pub metadata_digest: String,
    pub source_digest: String,
    pub indexes_digest: String,
    pub rail_components_digest: String,
    pub panes: BTreeMap<String, String>,
    pub reports: BTreeMap<String, String>,
}

pub fn trace_workbench_manifest(projection: &serde_json::Value) -> TraceViewManifest {
    let revision = projection
        .get("revision")
        .and_then(|revision| revision.get("id"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    let metadata = projection
        .get("metadata")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let source = projection
        .get("source")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let indexes = projection
        .get("indexes")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let summary = trace_workbench_summary_chunk(projection);
    let reports = [
        (
            "attribution".to_string(),
            projection.get("attribution_audit"),
        ),
        (
            "static_analysis".to_string(),
            projection.get("static_analysis"),
        ),
        ("closure_audit".to_string(), projection.get("audit")),
        (
            "duplicate_shapes".to_string(),
            projection.get("duplicate_shapes"),
        ),
    ]
    .into_iter()
    .filter_map(|(name, value)| {
        value.and_then(|value| (!value.is_null()).then(|| (name, digest_json(value))))
    })
    .collect();
    let panes = projection
        .get("panels")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|pane| {
            let id = pane
                .get("id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)?;
            Some((id, digest_json(pane)))
        })
        .collect();
    TraceViewManifest {
        revision,
        root_digest: digest_json(projection),
        summary_digest: digest_json(&summary),
        metadata_digest: digest_json(&metadata),
        source_digest: digest_json(&source),
        indexes_digest: digest_json(&indexes),
        rail_components_digest: digest_json(
            projection
                .get("rail_components")
                .unwrap_or(&serde_json::Value::Null),
        ),
        panes,
        reports,
    }
}

pub fn trace_workbench_summary_chunk(projection: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "revision": projection.get("revision").cloned().unwrap_or(serde_json::Value::Null),
        "metadata": projection.get("metadata").cloned().unwrap_or(serde_json::Value::Null),
        "provenance": projection.get("provenance").cloned().unwrap_or(serde_json::Value::Null),
        "counts": projection.get("counts").cloned().unwrap_or(serde_json::Value::Null),
        "salsa": projection.get("salsa").cloned().unwrap_or(serde_json::Value::Null),
        "bytecode_count": projection.get("bytecode_count").cloned().unwrap_or(serde_json::Value::Null),
        "selection_remap": projection.get("selection_remap").cloned().unwrap_or(serde_json::Value::Null),
        "notes": projection.get("notes").cloned().unwrap_or(serde_json::Value::Null),
    })
}

fn digest_json(value: &serde_json::Value) -> String {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    format!("blake3:{}", blake3::hash(&bytes).to_hex())
}
