use nako_core::{LibraryItemState, MediaItemId, MediaKind};
use nako_naming::{ParsedMediaKind, ParsedName};

use super::types::{
    LocalInferencePlan, MediaItemResolution, ProvisionalAncestorPlan, ProvisionalAncestorRole,
    ProvisionalHierarchyPlan, ProvisionalItemPlan,
};

pub(super) fn plan_hierarchy(
    primary_item_id: MediaItemId,
    parsed_name: &ParsedName,
) -> ProvisionalHierarchyPlan {
    let required_ancestors = if parsed_name.kind_hint == ParsedMediaKind::Episode
        && parsed_name.season_number.is_some()
    {
        vec![
            ProvisionalAncestorPlan {
                role: ProvisionalAncestorRole::Series,
                kind: MediaKind::Series,
                title: parsed_name.title.clone(),
                release_year: None,
            },
            ProvisionalAncestorPlan {
                role: ProvisionalAncestorRole::Season,
                kind: MediaKind::Season,
                title: format!("Season {}", parsed_name.season_number.unwrap()),
                release_year: None,
            },
        ]
    } else {
        Vec::new()
    };

    ProvisionalHierarchyPlan {
        primary_item_id,
        primary_provisional: true,
        required_ancestors,
    }
}

pub(crate) fn resolve_local_inference_plan(
    plan: LocalInferencePlan,
    supporting_items: Vec<ProvisionalItemPlan>,
) -> MediaItemResolution {
    let mut item = plan.media_item;

    if item.kind == MediaKind::Episode {
        item.parent_id = supporting_items
            .iter()
            .rev()
            .find(|candidate| candidate.item.kind == MediaKind::Season)
            .map(|candidate| candidate.item.id);
    }

    let mut resolved_supporting_items = Vec::new();
    let mut supporting_library_item_states = Vec::new();
    for supporting_item in supporting_items {
        if supporting_item.created {
            supporting_library_item_states.push(LibraryItemState {
                library_id: plan.source_state.library_id,
                item_id: supporting_item.item.id,
                provisional: true,
            });
            resolved_supporting_items.push(supporting_item.item);
        }
    }

    MediaItemResolution {
        item,
        provisional: plan.hierarchy.primary_provisional,
        supporting_items: resolved_supporting_items,
        supporting_library_item_states,
    }
}
