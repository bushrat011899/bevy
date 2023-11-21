//! Provides an [`Index`] system parameter, allowing a user to lookup an [`Entity`]
//! based on the value of one of its [`Components`][`Component`].

use crate as bevy_ecs;
use bevy_ecs::{
    component::{Component, Tick},
    prelude::{Changed, Entity, Query, Ref, RemovedComponents, ResMut},
    query::ReadOnlyWorldQuery,
    system::{SystemChangeTick, SystemParam},
};

use bevy_ecs_macros::Resource;

use bevy_utils::{default, EntityHashMap, EntityHashSet, HashMap};

use std::{hash::Hash, marker::PhantomData};

/// Describes how to transform a [`Component`] `Input` into an `Index` suitable for an [`Index`].
pub trait Indexer {
    /// The input [`Component`] to index against.
    type Input: Component;

    /// A type suitable for indexing the [`Component`] `Input`
    type Index: Hash + Eq + Clone + Sync + Send + 'static;

    /// Generate an `Index` from the provided `Input`
    fn index(input: &Self::Input) -> Self::Index;
}

/// A basic [`Indexer`] which directly uses the [`Component`] `T`'s value.
pub struct SimpleIndexer<T>(PhantomData<T>);

impl<T> Indexer for SimpleIndexer<T>
where
    T: Component + Hash + Eq + Clone,
{
    type Input = T;

    type Index = T;

    fn index(input: &Self::Input) -> Self::Index {
        input.clone()
    }
}

/// Stored data required for an [`Index`].
#[derive(Resource)]
pub struct IndexBacking<T, F = (), I = SimpleIndexer<T>>
where
    I: Indexer,
{
    forward: HashMap<I::Index, EntityHashSet<Entity>>,
    reverse: EntityHashMap<Entity, I::Index>,
    last_this_run: Option<Tick>,
    _phantom: PhantomData<fn(T, F, I)>,
    /// Used to return an empty `impl Iterator` from `get` on the `None` branch
    empty: EntityHashSet<Entity>,
}

impl<T, F, I> Default for IndexBacking<T, F, I>
where
    I: Indexer,
{
    fn default() -> Self {
        Self {
            forward: default(),
            reverse: default(),
            last_this_run: default(),
            _phantom: PhantomData,
            empty: default(),
        }
    }
}

impl<T, F, I> IndexBacking<T, F, I>
where
    I: Indexer<Input = T>,
{
    fn update(&mut self, entity: Entity, value: Option<&T>) -> Option<I::Index> {
        let value = value.map(|value| I::index(value));

        let old = if let Some(ref value) = value {
            self.reverse.insert(entity, value.clone())
        } else {
            self.reverse.remove(&entity)
        };

        if let Some(ref old) = old {
            if let Some(set) = self.forward.get_mut(old) {
                set.remove(&entity);

                if set.is_empty() {
                    self.forward.remove(old);
                }
            }
        }

        if let Some(value) = value {
            self.forward.entry(value).or_default().insert(entity);
        };

        old
    }

    fn get(&self, value: &T) -> impl Iterator<Item = Entity> + '_ {
        self.forward
            .get(&I::index(value))
            .unwrap_or(&self.empty)
            .iter()
            .copied()
    }
}

/// Allows for lookup of an [`Entity`] based on the [`Component`] `T`'s value.
/// `F` allows this [`Index`] to only target a subset of all [entities](`Entity`) using a
/// [`ReadOnlyWorldQuery`].
/// `I` controls how the [`Component`] `T` will be used to create an indexable value using the [`Indexer`] trait.
#[derive(SystemParam)]
pub struct Index<'w, 's, T, F = (), I = SimpleIndexer<T>>
where
    T: Component,
    I: Indexer + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    changed: Query<'w, 's, (Entity, Ref<'static, T>), (Changed<T>, F)>,
    removed: RemovedComponents<'w, 's, T>,
    index: ResMut<'w, IndexBacking<T, F, I>>,
    this_run: SystemChangeTick,
}

impl<'w, 's, T, F, I> Index<'w, 's, T, F, I>
where
    T: Component,
    I: Indexer<Input = T> + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    fn update_index_internal(&mut self) {
        let this_run = self.this_run.this_run();

        // Remove old entires
        for entity in self.removed.read() {
            self.index.update(entity, None);
        }

        // Update new and existing entries
        for (entity, component) in self.changed.iter() {
            self.index.update(entity, Some(component.as_ref()));
        }

        self.index.last_this_run = Some(this_run);
    }

    /// System to keep [`Index`] coarsely updated every frame
    pub fn update_index(mut index: Index<T, F, I>) {
        index.update_index_internal();
    }

    fn ensure_updated(&mut self) {
        let this_run = self.this_run.this_run();

        if self.index.last_this_run != Some(this_run) {
            self.update_index_internal();
        }
    }

    /// Get
    pub fn get(&mut self, value: &T) -> impl Iterator<Item = Entity> + '_ {
        self.ensure_updated();

        self.index.get(value)
    }

    /// Iterate over [entities](`Entity`) grouped by their [Index](`Indexer::Index`)
    pub fn iter(
        &mut self,
    ) -> impl Iterator<Item = (&I::Index, impl Iterator<Item = Entity> + '_)> + '_ {
        self.ensure_updated();

        self.index
            .forward
            .iter()
            .map(|(index, entities)| (index, entities.iter().copied()))
    }
}
