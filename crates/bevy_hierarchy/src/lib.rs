#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![forbid(unsafe_code)]
#![doc(
    html_logo_url = "https://bevyengine.org/assets/icon.png",
    html_favicon_url = "https://bevyengine.org/assets/icon.png"
)]

//! Relationships for Bevy entities.
//!
//! You should use the tools in this crate
//! whenever you want to organize your entities in a hierarchical fashion,
//! to make groups of entities more manageable,
//! or to propagate properties throughout the entity hierarchy.
//!
//! This crate introduces various tools, including a [plugin]
//! for managing parent-child relationships between entities.
//! It provides two components, [`Parent`] and [`Children`],
//! to store references to related entities.
//! It also provides [command and world] API extensions
//! to set and clear those relationships.
//!
//! More advanced users may also appreciate
//! [query extension methods] to traverse hierarchies,
//! and [events] to notify hierarchical changes.
//! There is also a [diagnostic plugin] to validate property propagation.
//!
//! # Hierarchy management
//!
//! The methods defined in this crate fully manage
//! the components responsible for defining the entity hierarchy.
//! Mutating these components manually may result in hierarchy invalidation.
//!
//! Hierarchical relationships are always managed symmetrically.
//! For example, assigning a child to an entity
//! will always set the parent in the other,
//! and vice versa.
//! Similarly, unassigning a child in the parent
//! will always unassign the parent in the child.
//!
//! These components will use component hooks to ensure both members of a relationship
//! have appropriate data for that relationship.
//!
//! Note that to maintain the invariants of a relationship, you must not mutate the component
//! in-place using methods like [`swap`](core::mem::swap), as these bypass the currently
//! available component hooks. To mutate a relationship, instead replace the component with an
//! updated value.
//!
//! ## Despawning entities
//!
//! The commands and methods provided by `bevy_ecs` to despawn entities
//! are not capable of automatically despawning hierarchies of entities.
//! In most cases, these operations will invalidate the hierarchy.
//! Instead, you should use the provided [hierarchical despawn extension methods].
//!
//! [command and world]: BuildChildren
//! [diagnostic plugin]: ValidParentCheckPlugin
//! [events]: HierarchyEvent
//! [hierarchical despawn extension methods]: DespawnRecursiveExt
//! [plugin]: HierarchyPlugin
//! [query extension methods]: HierarchyQueryExt

extern crate alloc;

pub(crate) mod many_to_one;
pub(crate) mod one_to_many;
pub(crate) mod relationship;

mod family;
pub use family::*;

mod despawn_recursive;
pub use despawn_recursive::*;

mod child_builder;
pub use child_builder::*;

mod valid_parent_check_plugin;
pub use valid_parent_check_plugin::*;

mod query_extension;
pub use query_extension::*;

/// The hierarchy prelude.
///
/// This includes the most common types in this crate, re-exported for your convenience.
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{child_builder::*, despawn_recursive::*, family::*, query_extension::*};

    #[doc(hidden)]
    #[cfg(feature = "bevy_app")]
    pub use crate::{HierarchyPlugin, ValidParentCheckPlugin};
}

#[cfg(feature = "bevy_app")]
use bevy_app::prelude::*;

/// Provides hierarchy functionality to a Bevy app.
///
/// Check the [crate-level documentation] for all the features.
///
/// [crate-level documentation]: crate
#[cfg(feature = "bevy_app")]
#[derive(Default)]
pub struct HierarchyPlugin;

#[cfg(feature = "bevy_app")]
impl Plugin for HierarchyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Children>()
            .register_type::<Parent>()
            .add_event::<HierarchyEvent>();
    }
}
