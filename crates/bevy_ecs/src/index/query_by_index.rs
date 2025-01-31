use alloc::vec::Vec;

use crate::{
    archetype::Archetype,
    component::{ComponentId, Tick},
    query::{QueryBuilder, QueryData, QueryFilter, QueryState, With},
    system::{Query, Res, SystemMeta, SystemParam},
    world::{unsafe_world_cell::UnsafeWorldCell, World},
};

use super::{Index, IndexableComponent};

/// This system parameter allows querying by an [indexable component](`IndexableComponent`) value.
///
/// # Examples
///
/// ```rust
/// # use bevy_ecs::prelude::*;
/// # let mut world = World::new();
/// #[derive(Component, PartialEq, Eq, Hash, Clone)]
/// #[component(immutable)]
/// struct Player(u8);
///
/// // Indexing is opt-in through `World::add_index`
/// world.add_index::<Player>();
/// # for i in 0..6 {
/// #   for _ in 0..(i + 1) {
/// #       world.spawn(Player(i));
/// #   }
/// # }
/// #
/// # world.flush();
///
/// fn find_all_player_one_entities(mut query: QueryByIndex<Player, Entity>) {
///     for entity in query.at(&Player(0)).iter() {
///         println!("{entity:?} belongs to Player 1!");
///     }
/// #   assert_eq!((
/// #       query.at(&Player(0)).iter().count(),
/// #       query.at(&Player(1)).iter().count(),
/// #       query.at(&Player(2)).iter().count(),
/// #       query.at(&Player(3)).iter().count(),
/// #       query.at(&Player(4)).iter().count(),
/// #       query.at(&Player(5)).iter().count(),
/// #    ), (1, 2, 3, 4, 5, 6));
/// }
/// # world.run_system_cached(find_all_player_one_entities);
/// ```
pub struct QueryByIndex<
    'world,
    'state,
    C: IndexableComponent,
    D: QueryData + 'static,
    F: QueryFilter + 'static = (),
> {
    world: UnsafeWorldCell<'world>,
    system_param_state: &'state QueryByIndexState<C, D, F>,
    state: Option<QueryState<D, (F, With<C>)>>,
    last_run: Tick,
    this_run: Tick,
    index: Res<'world, Index<C>>,
}

impl<C: IndexableComponent, D: QueryData, F: QueryFilter> QueryByIndex<'_, '_, C, D, F> {
    /// Return a [`Query`] only returning entities with a component `C` of the provided value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_ecs::prelude::*;
    /// # let mut world = World::new();
    /// #[derive(Component, PartialEq, Eq, Hash, Clone)]
    /// #[component(immutable)]
    /// enum FavoriteColor {
    ///     Red,
    ///     Green,
    ///     Blue,
    /// }
    ///
    /// world.add_index::<FavoriteColor>();
    ///
    /// fn find_red_fans(mut query: QueryByIndex<FavoriteColor, Entity>) {
    ///     for entity in query.at(&FavoriteColor::Red).iter() {
    ///         println!("{entity:?} likes the color Red!");
    ///     }
    /// }
    /// ```
    pub fn at(&mut self, value: &C) -> Query<'_, '_, D, (F, With<C>)> {
        self.state = None;

        let Some(&index) = self.index.mapping.get(value) else {
            todo!("make a null query to return");
        };

        for i in 0..self.index.markers.len() {
            if index & (1 << i) > 0 {
                let filter = &self.system_param_state.with_states[i];
                self.state = Some(self.state.as_ref().unwrap_or(&self.system_param_state.primary_query_state).join_filtered(self.world, filter));
            } else {
                let filter = &self.system_param_state.without_states[i];
                self.state = Some(self.state.as_ref().unwrap_or(&self.system_param_state.primary_query_state).join_filtered(self.world, filter));
            }
        }

        // SAFETY: We have registered all of the query's world accesses,
        // so the caller ensures that `world` has permission to access any
        // world data that the query needs.
        unsafe {
            Query::new(
                self.world,
                self.state.as_ref().unwrap_or(&self.system_param_state.primary_query_state),
                self.last_run,
                self.this_run,
            )
        }
    }
}

#[doc(hidden)]
pub struct QueryByIndexState<
    C: IndexableComponent,
    D: QueryData + 'static,
    F: QueryFilter + 'static,
> {
    primary_query_state: QueryState<D, (F, With<C>)>,
    index_state: ComponentId,

    // TODO: THERE MUST BE A BETTER WAY
    without_states: Vec<QueryState<(), With<C>>>, // No, With<C> is not a typo
    with_states: Vec<QueryState<(), With<C>>>,
}

impl<C: IndexableComponent, D: QueryData + 'static, F: QueryFilter + 'static>
    QueryByIndexState<C, D, F>
{
    fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        let primary_query_state =
            <Query<D, (F, With<C>)> as SystemParam>::init_state(world, system_meta);
        let index_state = <Res<Index<C>> as SystemParam>::init_state(world, system_meta);

        let ids = world.resource::<Index<C>>().markers.clone();

        let with_states = ids
            .iter()
            .map(|&id| {
                let mut builder = QueryBuilder::<(), With<C>>::new(world);
                builder.with_id(id);
                builder.build()
            })
            .collect::<Vec<_>>();

        let without_states = ids
            .iter()
            .map(|&id| {
                let mut builder = QueryBuilder::<(), With<C>>::new(world);
                builder.without_id(id);
                builder.build()
            })
            .collect::<Vec<_>>();

        Self {
            primary_query_state,
            index_state,
            without_states,
            with_states,
        }
    }

    unsafe fn new_archetype(&mut self, archetype: &Archetype, system_meta: &mut SystemMeta) {
        <Query<D, (F, With<C>)> as SystemParam>::new_archetype(
            &mut self.primary_query_state,
            archetype,
            system_meta,
        );

        for state in self
            .with_states
            .iter_mut()
            .chain(self.without_states.iter_mut())
        {
            <Query<(), With<C>> as SystemParam>::new_archetype(state, archetype, system_meta);
        }
    }

    #[inline]
    unsafe fn validate_param(&self, system_meta: &SystemMeta, world: UnsafeWorldCell) -> bool {
        let mut valid = true;

        valid &= <Query<D, (F, With<C>)> as SystemParam>::validate_param(
            &self.primary_query_state,
            system_meta,
            world,
        );
        valid &=
            <Res<Index<C>> as SystemParam>::validate_param(&self.index_state, system_meta, world);

        for state in self.with_states.iter().chain(self.without_states.iter()) {
            valid &= <Query<(), With<C>> as SystemParam>::validate_param(state, system_meta, world);
        }

        valid
    }
}

// SAFETY: We rely on the known-safe implementations of `SystemParam` for `Res` and `Query`.
unsafe impl<C: IndexableComponent, D: QueryData + 'static, F: QueryFilter + 'static> SystemParam
    for QueryByIndex<'_, '_, C, D, F>
{
    type State = QueryByIndexState<C, D, F>;
    type Item<'w, 's> = QueryByIndex<'w, 's, C, D, F>;

    fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
        Self::State::init_state(world, system_meta)
    }

    unsafe fn new_archetype(
        state: &mut Self::State,
        archetype: &Archetype,
        system_meta: &mut SystemMeta,
    ) {
        Self::State::new_archetype(state, archetype, system_meta);
    }

    #[inline]
    unsafe fn validate_param(
        state: &Self::State,
        system_meta: &SystemMeta,
        world: UnsafeWorldCell,
    ) -> bool {
        Self::State::validate_param(state, system_meta, world)
    }

    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        system_meta: &SystemMeta,
        world: UnsafeWorldCell<'world>,
        change_tick: Tick,
    ) -> Self::Item<'world, 'state> {
        state.primary_query_state.validate_world(world.id());

        let index = <Res<Index<C>> as SystemParam>::get_param(
            &mut state.index_state,
            system_meta,
            world,
            change_tick,
        );

        QueryByIndex {
            world,
            system_param_state: state,
            state: None,
            last_run: system_meta.last_run,
            this_run: change_tick,
            index,
        }
    }
}
