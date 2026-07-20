use std::collections::BTreeSet;

use common::origin::OriginExportKey;
use serde::Serialize;

use crate::AttributionAuditReport;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[allow(dead_code)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TraceWorkbenchDisplayStatus {
    SourceExact,
    Generated,
    Context,
    PreparedLinked,
    MissingOptimizedToPrepared,
    MissingDownstreamLineage,
    MissingSourceEvidence,
    SourceOnly,
    CompilerGenerated,
    Unmapped,
    Ambiguous,
    Invalid,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TraceWorkbenchMissingLineageIndex {
    pub(crate) origins: BTreeSet<String>,
}

impl TraceWorkbenchMissingLineageIndex {
    pub(crate) fn contains(&self, key: &OriginExportKey) -> bool {
        self.origins.contains(&key.canonical_storage_key())
    }
}

pub(crate) fn missing_lineage_index(
    attribution_audit: Option<&AttributionAuditReport>,
) -> TraceWorkbenchMissingLineageIndex {
    let mut origins = BTreeSet::new();
    if let Some(attribution_audit) = attribution_audit {
        origins.extend(
            attribution_audit
                .missing_optimized_to_prepared_lineage_pc_keys
                .iter()
                .map(OriginExportKey::canonical_storage_key),
        );
        for gap in &attribution_audit.lineage_gaps {
            origins.insert(gap.bytecode_pc.canonical_storage_key());
            origins.insert(gap.prepared_origin.canonical_storage_key());
        }
    }
    TraceWorkbenchMissingLineageIndex { origins }
}

pub(crate) fn status_for_source_line(_classes: &[String]) -> Option<TraceWorkbenchDisplayStatus> {
    None
}

pub(crate) fn status_for_row(
    key: &OriginExportKey,
    is_instruction: bool,
    classes: &[String],
    selection_groups: &[String],
    exact_groups_with_source_spans: &BTreeSet<String>,
    missing_lineage: &TraceWorkbenchMissingLineageIndex,
) -> Option<TraceWorkbenchDisplayStatus> {
    if missing_lineage.contains(key) {
        return Some(TraceWorkbenchDisplayStatus::MissingOptimizedToPrepared);
    }
    if is_instruction && key.kind() == "bytecode.pc" {
        let has_prepared = classes.iter().any(|class| class.starts_with("prepared-c-"));
        let has_exact = selection_groups
            .iter()
            .any(|class| class.starts_with("exact-c-"));
        let has_source_exact = selection_groups
            .iter()
            .filter(|group| group.starts_with("exact-c-"))
            .any(|group| exact_groups_with_source_spans.contains(group));
        if has_source_exact {
            return Some(TraceWorkbenchDisplayStatus::SourceExact);
        }
        if has_exact {
            return Some(TraceWorkbenchDisplayStatus::MissingSourceEvidence);
        }
        if has_prepared {
            return Some(TraceWorkbenchDisplayStatus::PreparedLinked);
        }
    }
    if origin_key_is_generated(key) || classes.iter().any(|class| class == "origin-generated") {
        return Some(TraceWorkbenchDisplayStatus::Generated);
    }
    None
}

pub(crate) fn origin_key_is_generated(key: &OriginExportKey) -> bool {
    key.owner_key().contains("__synthetic") || key.local_key().contains("__synthetic")
}
