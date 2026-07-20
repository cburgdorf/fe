use std::collections::{BTreeMap, BTreeSet, VecDeque};

use common::origin::OriginExportKey;
use trace_facts::{OriginEdgeFact, OriginEdgeTraversalClass, TraceSnapshot};

use crate::{
    origin_closure::OriginClosureIndex, trace_index::origin_edge_satisfies_phase_contract,
};

pub fn component_classes_by_origin_key(snapshot: &TraceSnapshot) -> BTreeMap<String, Vec<String>> {
    let index = OriginClosureIndex::new(snapshot);
    component_classes_for_index(&index)
}

pub(crate) fn component_classes_for_index(
    index: &OriginClosureIndex<'_>,
) -> BTreeMap<String, Vec<String>> {
    let mut classes = BTreeMap::<String, BTreeSet<String>>::new();
    for rail in OriginComponentRail::ALL {
        for (ordinal, component) in connected_components_for_rail(index, rail)
            .into_iter()
            .enumerate()
        {
            if suppress_row_level_component(rail, &component) {
                continue;
            }
            let class_name = component_class_name(rail, ordinal, &component);
            for key in component {
                classes
                    .entry(key.canonical_storage_key())
                    .or_default()
                    .insert(class_name.clone());
            }
        }
    }
    classes
        .into_iter()
        .map(|(key, value)| (key, value.into_iter().collect()))
        .collect()
}

#[derive(Clone, Copy, Debug)]
enum OriginComponentRail {
    Exact,
    Generated,
    Prepared,
    Contextual,
    Structural,
}

impl OriginComponentRail {
    const ALL: [Self; 5] = [
        Self::Exact,
        Self::Generated,
        Self::Prepared,
        Self::Contextual,
        Self::Structural,
    ];

    const fn class_prefix(self) -> &'static str {
        match self {
            Self::Exact => "exact-c",
            Self::Generated => "generated-c",
            Self::Prepared => "prepared-c",
            Self::Contextual => "context-c",
            Self::Structural => "structural-c",
        }
    }

    fn allows_edge(self, edge: &OriginEdgeFact, index: &OriginClosureIndex<'_>) -> bool {
        if !origin_edge_satisfies_phase_contract(edge, &index.prepared_lineage_events) {
            return false;
        }
        let class = edge.traversal_class();
        let exact = matches!(
            class,
            OriginEdgeTraversalClass::ExactAttribution | OriginEdgeTraversalClass::SnapshotAlias
        );
        match self {
            Self::Exact => exact && !is_prepared_codegen_connectivity_edge(edge),
            Self::Generated => matches!(class, OriginEdgeTraversalClass::Synthetic),
            Self::Prepared => is_prepared_codegen_connectivity_edge(edge),
            Self::Contextual => {
                matches!(class, OriginEdgeTraversalClass::Contextual)
                    && !is_prepared_codegen_connectivity_edge(edge)
            }
            Self::Structural => matches!(class, OriginEdgeTraversalClass::Structural),
        }
    }
}

fn connected_components_for_rail(
    index: &OriginClosureIndex<'_>,
    rail: OriginComponentRail,
) -> Vec<BTreeSet<OriginExportKey>> {
    let mut candidates = BTreeSet::<OriginExportKey>::new();
    for edges in index.edges_by_from.values() {
        for edge in edges {
            if rail.allows_edge(edge, index) {
                candidates.insert(edge.from.clone());
                candidates.insert(edge.to.clone());
            }
        }
    }

    let mut visited = BTreeSet::<OriginExportKey>::new();
    let mut components = Vec::<BTreeSet<OriginExportKey>>::new();
    for root in candidates.iter().cloned().collect::<Vec<_>>() {
        if visited.contains(&root) {
            continue;
        }
        let mut component = BTreeSet::new();
        let mut queue = VecDeque::from([root.clone()]);
        visited.insert(root);
        while let Some(key) = queue.pop_front() {
            component.insert(key.clone());
            let outgoing = index.edges_by_from.get(&key).into_iter().flatten().copied();
            let incoming = index.edges_by_to.get(&key).into_iter().flatten().copied();
            for edge in outgoing.chain(incoming) {
                if !rail.allows_edge(edge, index) {
                    continue;
                }
                let other = if edge.from == key {
                    &edge.to
                } else {
                    &edge.from
                };
                if visited.insert(other.clone()) {
                    queue.push_back(other.clone());
                }
            }
        }
        if component.len() > 1 {
            components.push(component);
        }
    }
    components.sort_by(|left, right| {
        component_sort_key(left)
            .unwrap_or_default()
            .cmp(&component_sort_key(right).unwrap_or_default())
    });
    components
}

fn suppress_row_level_component(
    rail: OriginComponentRail,
    component: &BTreeSet<OriginExportKey>,
) -> bool {
    match rail {
        OriginComponentRail::Exact => component.iter().any(is_component_hub_origin),
        OriginComponentRail::Generated | OriginComponentRail::Contextual => {
            component.len() > 128
                || component.iter().any(is_component_hub_origin)
                || component
                    .iter()
                    .filter(|key| is_source_like_origin_kind(key.kind()))
                    .count()
                    > 8
                || component
                    .iter()
                    .filter(|key| key.kind().starts_with("bytecode."))
                    .count()
                    > 64
        }
        OriginComponentRail::Prepared | OriginComponentRail::Structural => false,
    }
}

fn is_component_hub_origin(key: &OriginExportKey) -> bool {
    let kind = key.kind();
    kind == "source.file"
        || kind == "code.object"
        || kind == "package"
        || kind == "module"
        || kind.ends_with(".module")
        || kind.ends_with(".contract")
        || kind.ends_with(".function")
        || kind.ends_with(".body")
}

fn is_source_like_origin_kind(kind: &str) -> bool {
    kind.starts_with("source.") || kind.starts_with("hir.")
}

fn component_sort_key(component: &BTreeSet<OriginExportKey>) -> Option<String> {
    component
        .iter()
        .next()
        .map(OriginExportKey::canonical_storage_key)
}

fn component_class_name(
    rail: OriginComponentRail,
    fallback_ordinal: usize,
    component: &BTreeSet<OriginExportKey>,
) -> String {
    let mut hash = 2166136261u32;
    for byte in rail.class_prefix().bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16777619);
    }
    for key in component {
        for byte in key.canonical_storage_key().bytes() {
            hash ^= u32::from(byte);
            hash = hash.wrapping_mul(16777619);
        }
    }
    if hash == 0 {
        format!("{}-{fallback_ordinal}", rail.class_prefix())
    } else {
        format!("{}-{hash:08x}", rail.class_prefix())
    }
}

fn is_prepared_codegen_connectivity_edge(edge: &OriginEdgeFact) -> bool {
    let from_prepared = is_prepared_origin_kind(edge.from.kind());
    let to_prepared = is_prepared_origin_kind(edge.to.kind());
    let from_bytecode = edge.from.kind().starts_with("bytecode.");
    let to_bytecode = edge.to.kind().starts_with("bytecode.");
    (from_bytecode && to_prepared)
        || (to_bytecode && from_prepared)
        || (from_prepared && to_prepared)
}

fn is_prepared_origin_kind(kind: &str) -> bool {
    kind.starts_with("sonatina.evm.prepared.")
        || kind.starts_with("sonatina.codegen.")
        || kind.starts_with("evm.vcode.")
        || kind.starts_with("vcode.")
}
