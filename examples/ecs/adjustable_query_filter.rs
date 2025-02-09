//! This example illustrates how to create a component index using adjustable query filters.

#![expect(
    unsafe_code,
    reason = "this example demonstrates the custom implementation of unsafe traits"
)]

use bevy::prelude::*;
use std::hash::Hash;

use index::{ByIndex, Index, QueryIterIndexExt as _};

// Our goal is to create an index that can integrate with Query.
// First, we will define the component we want to index entities by.

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
// Immutability allows us to ensure observers capture all value changes
#[component(immutable)]
struct Cell(u32, u32);

fn main() {
    App::new()
        .add_systems(Update, (setup, query_for_a_cell).chain())
        .run();
}

fn setup(world: &mut World) {
    // Setup the index for our Cell component
    Index::<Cell>::setup(world);

    // Spawn some cells
    world.spawn_batch((0..500).flat_map(|x| (0..500).map(move |y| Cell(x, y))));
}

fn query_for_a_cell(index: Res<Index<Cell>>, query: Query<Entity, ByIndex<Cell>>) {
    for entity in query.iter().at(&index, &Cell(12, 34)) {
        println!("Entity {entity:?} is in cell (12, 34)");
    }
}

mod index {
    //! We isolate the definition of our index to clearly identify the reuseable
    //! part of this example.

    use bevy::{
        ecs::{
            archetype::Archetype,
            component::{ComponentId, Components, Immutable, Tick},
            entity::hash_set::EntityHashSet,
            query::{
                AdjustableQueryFilter, FilteredAccess, QueryData, QueryFilter, QueryIter,
                WorldQuery,
            },
            storage::{Table, TableRow},
            world::unsafe_world_cell::UnsafeWorldCell,
        },
        platform_support::collections::HashMap,
        prelude::*,
    };
    use std::{hash::Hash, marker::PhantomData};

    /// This index simply maps values of `C` to an [`EntityHashSet`] of all entities
    /// with that value.
    #[derive(Resource)]
    pub struct Index<C: Hash + Eq + Clone + Component<Mutability = Immutable>> {
        mapping: HashMap<C, EntityHashSet>,
    }

    impl<C: Hash + Eq + Clone + Component<Mutability = Immutable>> Index<C> {
        /// Create the index and setup observers for `C`'s lifecycle events.
        pub fn setup(world: &mut World) {
            if world.get_resource::<Self>().is_none() {
                world.insert_resource(Self { mapping: default() });
                world.add_observer(on_replace::<C>);
                world.add_observer(on_insert::<C>);
            }
        }
    }

    /// A [`QueryFilter`] which can be provided a value to find entities by.
    pub struct ByIndex<
        C: Hash + Eq + Clone + Component<Mutability = Immutable>,
        F: QueryFilter = (),
    > {
        _phantom: PhantomData<fn(&C, &F)>,
    }

    /// This extension trait makes working with our index more ergonomic.
    pub trait QueryIterIndexExt<'a, C: Hash + Eq + Clone + Component<Mutability = Immutable>> {
        fn at(&mut self, index: &'a Index<C>, value: &'a C) -> &mut Self;
    }

    impl<
            'a,
            's,
            C: Hash + Eq + Clone + Component<Mutability = Immutable>,
            D: QueryData,
            F: QueryFilter,
        > QueryIterIndexExt<'a, C> for QueryIter<'a, 's, D, ByIndex<C, F>>
    {
        fn at(&mut self, index: &'a Index<C>, value: &'a C) -> &mut Self {
            self.provide_filter((index, value))
        }
    }

    pub struct ByIndexState<
        C: Hash + Eq + Clone + Component<Mutability = Immutable>,
        F: QueryFilter,
    > {
        inner: <(With<C>, F) as WorldQuery>::State,
    }

    pub struct ByIndexFetch<
        'a,
        C: Hash + Eq + Clone + Component<Mutability = Immutable>,
        F: QueryFilter,
    > {
        inner: <(With<C>, F) as WorldQuery>::Fetch<'a>,
        index: Option<&'a EntityHashSet>,
    }

    impl<C: Hash + Eq + Clone + Component<Mutability = Immutable>, F: QueryFilter> Clone
        for ByIndexFetch<'_, C, F>
    {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
                index: self.index.clone(),
            }
        }
    }

    unsafe impl<C: Hash + Eq + Clone + Component<Mutability = Immutable>, F: QueryFilter> WorldQuery
        for ByIndex<C, F>
    {
        type Fetch<'a> = ByIndexFetch<'a, C, F>;

        type State = ByIndexState<C, F>;

        fn shrink_fetch<'wlong: 'wshort, 'wshort>(
            fetch: Self::Fetch<'wlong>,
        ) -> Self::Fetch<'wshort> {
            Self::Fetch::<'wshort> {
                inner: <(With<C>, F) as WorldQuery>::shrink_fetch(fetch.inner),
                index: fetch.index,
            }
        }

        unsafe fn init_fetch<'w>(
            world: UnsafeWorldCell<'w>,
            state: &Self::State,
            last_run: Tick,
            this_run: Tick,
        ) -> Self::Fetch<'w> {
            let inner = unsafe {
                <(With<C>, F) as WorldQuery>::init_fetch(world, &state.inner, last_run, this_run)
            };

            Self::Fetch { inner, index: None }
        }

        const IS_DENSE: bool = false;

        unsafe fn set_archetype<'w>(
            fetch: &mut Self::Fetch<'w>,
            state: &Self::State,
            archetype: &'w Archetype,
            table: &'w Table,
        ) {
            unsafe {
                <(With<C>, F) as WorldQuery>::set_archetype(
                    &mut fetch.inner,
                    &state.inner,
                    archetype,
                    table,
                );
            }
        }

        unsafe fn set_table<'w>(
            fetch: &mut Self::Fetch<'w>,
            state: &Self::State,
            table: &'w Table,
        ) {
            unsafe {
                <(With<C>, F) as WorldQuery>::set_table(&mut fetch.inner, &state.inner, table);
            }
        }

        fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
            <(With<C>, F) as WorldQuery>::update_component_access(&state.inner, access);
        }

        fn init_state(world: &mut World) -> Self::State {
            Self::State {
                inner: <(With<C>, F) as WorldQuery>::init_state(world),
            }
        }

        fn get_state(components: &Components) -> Option<Self::State> {
            let inner_filter_state = <(With<C>, F) as WorldQuery>::get_state(components)?;

            Some(Self::State {
                inner: inner_filter_state,
            })
        }

        fn matches_component_set(
            state: &Self::State,
            set_contains_id: &impl Fn(ComponentId) -> bool,
        ) -> bool {
            <(With<C>, F) as WorldQuery>::matches_component_set(&state.inner, set_contains_id)
        }
    }

    unsafe impl<C: Hash + Eq + Clone + Component<Mutability = Immutable>, F: QueryFilter>
        QueryFilter for ByIndex<C, F>
    {
        const IS_ARCHETYPAL: bool = false;

        unsafe fn filter_fetch(
            fetch: &mut Self::Fetch<'_>,
            entity: Entity,
            table_row: TableRow,
        ) -> bool {
            let inner_filter_fetch = unsafe {
                <(With<C>, F) as QueryFilter>::filter_fetch(&mut fetch.inner, entity, table_row)
            };
            let matches_index = fetch.index.is_none_or(|index| index.contains(&entity));
            inner_filter_fetch && matches_index
        }
    }

    unsafe impl<C: Hash + Eq + Clone + Component<Mutability = Immutable>, F: QueryFilter>
        AdjustableQueryFilter for ByIndex<C, F>
    {
        type Input<'a> = (&'a Index<C>, &'a C);

        unsafe fn adjust_filter<'a>(
            state: &mut <Self as WorldQuery>::Fetch<'a>,
            (index, value): Self::Input<'a>,
        ) {
            state.index = index.mapping.get(value);
        }
    }

    fn on_replace<C: Hash + Eq + Clone + Component<Mutability = Immutable>>(
        trigger: Trigger<OnReplace, C>,
        query: Query<&C>,
        mut index: ResMut<Index<C>>,
    ) {
        let entity = trigger.target();

        let value = query.get(entity).unwrap();

        index.mapping.get_mut(value).unwrap().remove(&entity);
    }

    fn on_insert<C: Hash + Eq + Clone + Component<Mutability = Immutable>>(
        trigger: Trigger<OnInsert, C>,
        query: Query<&C>,
        mut index: ResMut<Index<C>>,
    ) {
        let entity = trigger.target();

        let value = query.get(entity).unwrap();

        match index.mapping.get_mut(value) {
            Some(slot) => {
                slot.insert(entity);
            }
            None => {
                let mut set = EntityHashSet::new();
                set.insert(entity);
                index.mapping.insert(value.clone(), set);
            }
        }
    }
}
