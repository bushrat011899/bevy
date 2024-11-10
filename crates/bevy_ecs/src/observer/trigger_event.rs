use bevy_utils::all_tuples;

use crate::{
    component::ComponentId,
    entity::Entity,
    event::Event,
    world::{Command, DeferredWorld, World},
};

/// A [`Command`] that emits a given trigger for a given set of targets.
pub struct TriggerEvent<E, Targets: TriggerTargets = ()> {
    /// The event to trigger.
    pub event: E,

    /// The targets to trigger the event for.
    pub targets: Targets,
}

impl<E: Event, Targets: TriggerTargets> TriggerEvent<E, Targets> {
    pub(super) fn trigger(mut self, world: &mut World) {
        let event_type = world.register_component::<E>();
        trigger_event(world, event_type, &mut self.event, self.targets);
    }
}

impl<E: Event, Targets: TriggerTargets> TriggerEvent<&mut E, Targets> {
    pub(super) fn trigger_ref(self, world: &mut World) {
        let event_type = world.register_component::<E>();
        trigger_event(world, event_type, self.event, self.targets);
    }
}

impl<E: Event, Targets: TriggerTargets + Send + Sync + 'static> Command
    for TriggerEvent<E, Targets>
{
    fn apply(self, world: &mut World) {
        self.trigger(world);
    }
}

/// Emit a trigger for a dynamic component id. This is unsafe and must be verified manually.
pub struct EmitDynamicTrigger<T, Targets: TriggerTargets = ()> {
    event_type: ComponentId,
    event_data: T,
    targets: Targets,
}

impl<E, Targets: TriggerTargets> EmitDynamicTrigger<E, Targets> {
    /// Sets the event type of the resulting trigger, used for dynamic triggers
    /// # Safety
    /// Caller must ensure that the component associated with `event_type` is accessible as E
    pub unsafe fn new_with_id(event_type: ComponentId, event_data: E, targets: Targets) -> Self {
        Self {
            event_type,
            event_data,
            targets,
        }
    }
}

impl<E: Event, Targets: TriggerTargets + Send + Sync + 'static> Command
    for EmitDynamicTrigger<E, Targets>
{
    fn apply(mut self, world: &mut World) {
        trigger_event(world, self.event_type, &mut self.event_data, self.targets);
    }
}

#[inline]
fn trigger_event<E: Event, Targets: TriggerTargets>(
    world: &mut World,
    event_type: ComponentId,
    event_data: &mut E,
    targets: Targets,
) {
    let mut world = DeferredWorld::from(world);
    let mut entity_targets = targets.entities().peekable();
    if entity_targets.peek().is_none() {
        // SAFETY: T is accessible as the type represented by self.trigger, ensured in `Self::new`
        unsafe {
            world.trigger_observers_with_data::<_, E::Traversal>(
                event_type,
                Entity::PLACEHOLDER,
                targets.components(),
                event_data,
                false,
            );
        };
    } else {
        for target_entity in entity_targets {
            // SAFETY: T is accessible as the type represented by self.trigger, ensured in `Self::new`
            unsafe {
                world.trigger_observers_with_data::<_, E::Traversal>(
                    event_type,
                    target_entity,
                    targets.components(),
                    event_data,
                    E::AUTO_PROPAGATE,
                );
            };
        }
    }
}

/// Represents a collection of targets for a specific [`Trigger`] of an [`Event`]. Targets can be of type [`Entity`] or [`ComponentId`].
///
/// When a trigger occurs for a given event and [`TriggerTargets`], any [`Observer`] that watches for that specific event-target combination
/// will run.
///
/// [`Trigger`]: crate::observer::Trigger
/// [`Observer`]: crate::observer::Observer
pub trait TriggerTargets {
    /// The components the trigger should target.
    fn components(&self) -> impl ExactSizeIterator<Item = ComponentId> + Clone;

    /// The entities the trigger should target.
    fn entities(&self) -> impl ExactSizeIterator<Item = Entity> + Clone;
}

impl TriggerTargets for Entity {
    fn components(&self) -> impl ExactSizeIterator<Item = ComponentId> + Clone {
        [].into_iter()
    }

    fn entities(&self) -> impl ExactSizeIterator<Item = Entity> + Clone {
        core::iter::once(*self)
    }
}

impl TriggerTargets for ComponentId {
    fn components(&self) -> impl ExactSizeIterator<Item = ComponentId> + Clone {
        core::iter::once(*self)
    }

    fn entities(&self) -> impl ExactSizeIterator<Item = Entity> + Clone {
        [].into_iter()
    }
}

impl<T: TriggerTargets> TriggerTargets for Vec<T> {
    fn components(&self) -> impl ExactSizeIterator<Item = ComponentId> + Clone {
        self.iter().flat_map(T::components).collect::<Vec<_>>().into_iter()
    }

    fn entities(&self) -> impl ExactSizeIterator<Item = Entity> + Clone {
        self.iter().flat_map(T::entities).collect::<Vec<_>>().into_iter()
    }
}

impl<const N: usize, T: TriggerTargets> TriggerTargets for [T; N] {
    fn components(&self) -> impl ExactSizeIterator<Item = ComponentId> + Clone {
        self.iter().flat_map(T::components).collect::<Vec<_>>().into_iter()
    }

    fn entities(&self) -> impl ExactSizeIterator<Item = Entity> + Clone {
        self.iter().flat_map(T::entities).collect::<Vec<_>>().into_iter()
    }
}

impl<T: TriggerTargets> TriggerTargets for &[T] {
    fn components(&self) -> impl ExactSizeIterator<Item = ComponentId> + Clone {
        self.iter().flat_map(T::components).collect::<Vec<_>>().into_iter()
    }

    fn entities(&self) -> impl ExactSizeIterator<Item = Entity> + Clone {
        self.iter().flat_map(T::entities).collect::<Vec<_>>().into_iter()
    }
}

macro_rules! impl_trigger_targets_tuples {
    ($(#[$meta:meta])* $($trigger_targets: ident),*) => {
        #[allow(non_snake_case, reason = "`all_tuples!()` generates non-snake-case variable names.")]
        $(#[$meta])*
        impl<$($trigger_targets: TriggerTargets),*> TriggerTargets for ($($trigger_targets,)*)
        {
            fn components(&self) -> impl ExactSizeIterator<Item = ComponentId> + Clone {
                let iter = [].into_iter();
                let ($($trigger_targets,)*) = self;
                $(
                    let iter = iter.chain($trigger_targets.components());
                )*
                iter.collect::<Vec<_>>().into_iter()
            }

            fn entities(&self) -> impl ExactSizeIterator<Item = Entity> + Clone {
                let iter = [].into_iter();
                let ($($trigger_targets,)*) = self;
                $(
                    let iter = iter.chain($trigger_targets.entities());
                )*
                iter.collect::<Vec<_>>().into_iter()
            }
        }
    }
}

all_tuples!(
    #[doc(fake_variadic)]
    impl_trigger_targets_tuples,
    0,
    15,
    T
);
