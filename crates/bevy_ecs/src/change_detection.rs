//! Types that detect when their internal data mutate.

use crate::{
    component::{Tick, TickCells},
    ptr::PtrMut,
};
use bevy_ptr::UnsafeCellDeref;
use std::mem;
use std::ops::{Deref, DerefMut};

/// The (arbitrarily chosen) minimum number of world tick increments between `check_tick` scans.
///
/// Change ticks can only be scanned when systems aren't running. Thus, if the threshold is `N`,
/// the maximum is `2 * N - 1` (i.e. the world ticks `N - 1` times, then `N` times).
///
/// If no change is older than `u32::MAX - (2 * N - 1)` following a scan, none of their ages can
/// overflow and cause false positives.
// (518,400,000 = 1000 ticks per frame * 144 frames per second * 3600 seconds per hour)
pub const CHECK_TICK_THRESHOLD: u32 = 518_400_000;

/// The maximum change tick difference that won't overflow before the next `check_tick` scan.
///
/// Changes stop being detected once they become this old.
pub const MAX_CHANGE_AGE: u32 = u32::MAX - (2 * CHECK_TICK_THRESHOLD - 1);

/// Types that can read change detection information.
/// This change detection is controlled by [`DetectChangesMut`] types such as [`ResMut`].
///
/// ## Example
/// Using types that implement [`DetectChanges`], such as [`Res`], provide
/// a way to query if a value has been mutated in another system.
///
/// ```
/// use bevy_ecs::prelude::*;
///
/// #[derive(Resource)]
/// struct MyResource(u32);
///
/// fn my_system(mut resource: Res<MyResource>) {
///     if resource.is_changed() {
///         println!("My component was mutated!");
///     }
/// }
/// ```
pub trait DetectChanges {
    /// Returns `true` if this value was added after the system last ran.
    fn is_added(&self) -> bool;

    /// Returns `true` if this value was added or mutably dereferenced
    /// either since the last time the system ran or, if the system never ran,
    /// since the beginning of the program.
    ///
    /// To check if the value was mutably dereferenced only,
    /// use `this.is_changed() && !this.is_added()`.
    fn is_changed(&self) -> bool;

    /// Returns the change tick recording the time this data was most recently changed.
    ///
    /// Note that components and resources are also marked as changed upon insertion.
    ///
    /// For comparison, the previous change tick of a system can be read using the
    /// [`SystemChangeTick`](crate::system::SystemChangeTick)
    /// [`SystemParam`](crate::system::SystemParam).
    fn last_changed(&self) -> Tick;
}

/// Types that implement reliable change detection.
///
/// ## Example
/// Using types that implement [`DetectChangesMut`], such as [`ResMut`], provide
/// a way to query if a value has been mutated in another system.
/// Normally change detection is triggered by either [`DerefMut`] or [`AsMut`], however
/// it can be manually triggered via [`set_changed`](DetectChangesMut::set_changed).
///
/// To ensure that changes are only triggered when the value actually differs,
/// check if the value would change before assignment, such as by checking that `new != old`.
/// You must be *sure* that you are not mutably dereferencing in this process.
///
/// [`set_if_neq`](DetectChangesMut::set_if_neq) is a helper
/// method for this common functionality.
///
/// ```
/// use bevy_ecs::prelude::*;
///
/// #[derive(Resource)]
/// struct MyResource(u32);
///
/// fn my_system(mut resource: ResMut<MyResource>) {
///     if resource.is_changed() {
///         println!("My resource was mutated!");
///     }
///
///    resource.0 = 42; // triggers change detection via [`DerefMut`]
/// }
/// ```
///
pub trait DetectChangesMut: DetectChanges {
    /// The type contained within this smart pointer
    ///
    /// For example, for `ResMut<T>` this would be `T`.
    type Inner: ?Sized;

    /// Flags this value as having been changed.
    ///
    /// Mutably accessing this smart pointer will automatically flag this value as having been changed.
    /// However, mutation through interior mutability requires manual reporting.
    ///
    /// **Note**: This operation cannot be undone.
    fn set_changed(&mut self);

    /// Manually sets the change tick recording the time when this data was last mutated.
    ///
    /// # Warning
    /// This is a complex and error-prone operation, primarily intended for use with rollback networking strategies.
    /// If you merely want to flag this data as changed, use [`set_changed`](DetectChangesMut::set_changed) instead.
    /// If you want to avoid triggering change detection, use [`bypass_change_detection`](DetectChangesMut::bypass_change_detection) instead.
    fn set_last_changed(&mut self, last_changed: Tick);

    /// Manually bypasses change detection, allowing you to mutate the underlying value without updating the change tick.
    ///
    /// # Warning
    /// This is a risky operation, that can have unexpected consequences on any system relying on this code.
    /// However, it can be an essential escape hatch when, for example,
    /// you are trying to synchronize representations using change detection and need to avoid infinite recursion.
    fn bypass_change_detection(&mut self) -> &mut Self::Inner;

    /// Overwrites this smart pointer with the given value, if and only if `*self != value`
    ///
    /// This is useful to ensure change detection is only triggered when the underlying value
    /// changes, instead of every time it is mutably accessed.
    ///
    /// If you need to handle the previous value, use [`replace_if_neq`](DetectChangesMut::replace_if_neq).
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_ecs::{prelude::*, schedule::common_conditions::resource_changed};
    /// #[derive(Resource, PartialEq, Eq)]
    /// pub struct Score(u32);
    ///
    /// fn reset_score(mut score: ResMut<Score>) {
    ///     // Set the score to zero, unless it is already zero.
    ///     score.set_if_neq(Score(0));
    /// }
    /// # let mut world = World::new();
    /// # world.insert_resource(Score(1));
    /// # let mut score_changed = IntoSystem::into_system(resource_changed::<Score>());
    /// # score_changed.initialize(&mut world);
    /// # score_changed.run((), &mut world);
    /// #
    /// # let mut schedule = Schedule::new();
    /// # schedule.add_systems(reset_score);
    /// #
    /// # // first time `reset_score` runs, the score is changed.
    /// # schedule.run(&mut world);
    /// # assert!(score_changed.run((), &mut world));
    /// # // second time `reset_score` runs, the score is not changed.
    /// # schedule.run(&mut world);
    /// # assert!(!score_changed.run((), &mut world));
    /// ```
    #[inline]
    fn set_if_neq(&mut self, value: Self::Inner)
    where
        Self::Inner: Sized + PartialEq,
    {
        let old = self.bypass_change_detection();
        if *old != value {
            *old = value;
            self.set_changed();
        }
    }

    /// Overwrites this smart pointer with the given value, if and only if `*self != value`
    /// returning the previous value if this occurs.
    ///
    /// This is useful to ensure change detection is only triggered when the underlying value
    /// changes, instead of every time it is mutably accessed.
    ///
    /// If you don't need to handle the previous value, use [`set_if_neq`](DetectChangesMut::set_if_neq).
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_ecs::{prelude::*, schedule::common_conditions::{resource_changed, on_event}};
    /// #[derive(Resource, PartialEq, Eq)]
    /// pub struct Score(u32);
    ///
    /// #[derive(Event, PartialEq, Eq)]
    /// pub struct ScoreChanged {
    ///     current: u32,
    ///     previous: u32,
    /// }
    ///
    /// fn reset_score(mut score: ResMut<Score>, mut score_changed: EventWriter<ScoreChanged>) {
    ///     // Set the score to zero, unless it is already zero.
    ///     let new_score = 0;
    ///     if let Some(Score(previous_score)) = score.replace_if_neq(Score(new_score)) {
    ///         // If `score` change, emit a `ScoreChanged` event.
    ///         score_changed.send(ScoreChanged {
    ///             current: new_score,
    ///             previous: previous_score,
    ///         });
    ///     }
    /// }
    /// # let mut world = World::new();
    /// # world.insert_resource(Events::<ScoreChanged>::default());
    /// # world.insert_resource(Score(1));
    /// # let mut score_changed = IntoSystem::into_system(resource_changed::<Score>());
    /// # score_changed.initialize(&mut world);
    /// # score_changed.run((), &mut world);
    /// #
    /// # let mut score_changed_event = IntoSystem::into_system(on_event::<ScoreChanged>());
    /// # score_changed_event.initialize(&mut world);
    /// # score_changed_event.run((), &mut world);
    /// #
    /// # let mut schedule = Schedule::new();
    /// # schedule.add_systems(reset_score);
    /// #
    /// # // first time `reset_score` runs, the score is changed.
    /// # schedule.run(&mut world);
    /// # assert!(score_changed.run((), &mut world));
    /// # assert!(score_changed_event.run((), &mut world));
    /// # // second time `reset_score` runs, the score is not changed.
    /// # schedule.run(&mut world);
    /// # assert!(!score_changed.run((), &mut world));
    /// # assert!(!score_changed_event.run((), &mut world));
    /// ```
    #[inline]
    #[must_use = "If you don't need to handle the previous value, use `set_if_neq` instead."]
    fn replace_if_neq(&mut self, value: Self::Inner) -> Option<Self::Inner>
    where
        Self::Inner: Sized + PartialEq,
    {
        let old = self.bypass_change_detection();
        if *old != value {
            let previous = mem::replace(old, value);
            self.set_changed();
            Some(previous)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub(crate) struct Ticks<TickRef: Deref<Target = Tick>> {
    pub(crate) added: TickRef,
    pub(crate) changed: TickRef,
    pub(crate) last_run: Tick,
    pub(crate) this_run: Tick,
}

impl<'a> Ticks<&'a Tick> {
    /// # Safety
    /// This should never alias the underlying ticks with a mutable one such as `TicksMut`.
    #[inline]
    pub(crate) unsafe fn from_tick_cells(
        cells: TickCells<'a>,
        last_run: Tick,
        this_run: Tick,
    ) -> Self {
        Self {
            added: cells.added.deref(),
            changed: cells.changed.deref(),
            last_run,
            this_run,
        }
    }
}

impl<'a> Ticks<&'a mut Tick> {
    /// # Safety
    /// This should never alias the underlying ticks with a mutable one such as `TicksMut`.
    #[inline]
    pub(crate) unsafe fn from_tick_cells_mut(
        cells: TickCells<'a>,
        last_run: Tick,
        this_run: Tick,
    ) -> Self {
        Self {
            added: cells.added.deref_mut(),
            changed: cells.changed.deref_mut(),
            last_run,
            this_run,
        }
    }
}

impl<'a> From<Ticks<&'a mut Tick>> for Ticks<&'a Tick> {
    fn from(ticks: Ticks<&'a mut Tick>) -> Self {
        Self {
            added: ticks.added,
            changed: ticks.changed,
            last_run: ticks.last_run,
            this_run: ticks.this_run,
        }
    }
}

/// Proxy for a value of type `T`.
pub struct Proxy<TickRef: Deref<Target = Tick>, T> {
    pub(crate) value: T,
    pub(crate) ticks: Ticks<TickRef>,
}

/*impl<'w, T: Copy> Clone for Proxy<&'w Tick, T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            ticks: self.ticks.clone(),
        }
    }
}*/

impl<'w, TickRef, TRef, T> std::fmt::Debug for Proxy<TickRef, TRef>
where
    TickRef: Deref<Target = Tick> + 'w,
    TRef: Deref<Target = T>,
    T: std::fmt::Debug + ?Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Proxy").field(&self.value.deref()).finish()
    }
}

impl<'w, TickRef, TRef, T> Deref for Proxy<TickRef, TRef>
where
    TickRef: Deref<Target = Tick> + 'w,
    TRef: Deref<Target = T>,
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}

impl<'w, TickRef, TRef, T> DerefMut for Proxy<TickRef, TRef>
where
    TickRef: DerefMut<Target = Tick> + 'w,
    TRef: DerefMut<Target = T>,
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.set_changed();
        self.value.deref_mut()
    }
}

impl<'w, TickRef, TRef, T> DetectChanges for Proxy<TickRef, TRef>
where
    TickRef: Deref<Target = Tick> + 'w,
    TRef: Deref<Target = T>,
    T: ?Sized,
{
    #[inline]
    fn is_added(&self) -> bool {
        self.ticks
            .added
            .is_newer_than(self.ticks.last_run, self.ticks.this_run)
    }

    #[inline]
    fn is_changed(&self) -> bool {
        self.ticks
            .changed
            .is_newer_than(self.ticks.last_run, self.ticks.this_run)
    }

    #[inline]
    fn last_changed(&self) -> Tick {
        *self.ticks.changed
    }
}

impl<'w, TickRef, TRef, T> DetectChangesMut for Proxy<TickRef, TRef>
where
    TickRef: DerefMut<Target = Tick> + 'w,
    TRef: DerefMut<Target = T>,
    T: ?Sized,
{
    type Inner = T;

    #[inline]
    fn set_changed(&mut self) {
        *self.ticks.changed = self.ticks.this_run;
    }

    #[inline]
    fn set_last_changed(&mut self, last_changed: Tick) {
        *self.ticks.changed = last_changed;
    }

    #[inline]
    fn bypass_change_detection(&mut self) -> &mut Self::Inner {
        self.value.deref_mut()
    }
}

impl<'w, TickRef, TRef> AsRef<<Self as Deref>::Target> for Proxy<TickRef, TRef>
where
    TickRef: Deref<Target = Tick> + 'w,
    Self: Deref,
{
    #[inline]
    fn as_ref(&self) -> &<Self as Deref>::Target {
        self.deref()
    }
}

impl<'w, TickRef, TRef> AsMut<<Self as Deref>::Target> for Proxy<TickRef, TRef>
where
    TickRef: Deref<Target = Tick> + 'w,
    Self: DerefMut,
{
    #[inline]
    fn as_mut(&mut self) -> &mut <Self as Deref>::Target {
        self.deref_mut()
    }
}

impl<'w, T: ?Sized> From<Proxy<&'w mut Tick, &'w mut T>> for Proxy<&'w Tick, &'w T> {
    fn from(proxy: Proxy<&'w mut Tick, &'w mut T>) -> Self {
        Self {
            value: &*proxy.value,
            ticks: proxy.ticks.into(),
        }
    }
}

impl<'w, T: ?Sized> Proxy<&'w Tick, &'w T> {
    /// Returns the reference wrapped by this type. The reference is allowed to outlive `self`,
    /// which makes this method more flexible than simply borrowing `self`.
    #[inline]
    pub fn into_inner(self) -> &'w T {
        self.value
    }
}

impl<'w, T: ?Sized> Proxy<&'w mut Tick, &'w mut T> {
    /// Consume `self` and return a mutable reference to the
    /// contained value while marking `self` as "changed".
    #[inline]
    pub fn into_inner(mut self) -> &'w mut T {
        self.set_changed();
        self.value
    }

    /// Returns a `Proxy<>` with a smaller lifetime.
    /// This is useful if you have `&mut Proxy`,
    /// but you need a `Proxy<T>`.
    pub fn reborrow<'a: 'b, 'b>(&'a mut self) -> Proxy<&'b mut Tick, &'b mut T>
    where
        'w: 'a,
    {
        Proxy {
            value: self.value,
            ticks: Ticks {
                added: self.ticks.added,
                changed: self.ticks.changed,
                last_run: self.ticks.last_run,
                this_run: self.ticks.this_run,
            },
        }
    }
}

impl<'w, TickRef: Deref<Target = Tick> + 'w, TRef> Proxy<TickRef, TRef> {
    /// Create a new `Proxy` using provided values.
    ///
    /// This is an advanced feature, `Proxy`s are designed to be _created_ by
    /// engine-internal code and _consumed_ by end-user code.
    ///
    /// - `value` - The value wrapped by `Proxy`.
    /// - `added` - A [`Tick`] that stores the tick when the wrapped value was created.
    /// - `changed` - A [`Tick`] that stores the last time the wrapped value was changed.
    /// - `last_run` - A [`Tick`], occurring before `this_run`, which is used
    ///    as a reference to determine whether the wrapped value is newly added or changed.
    /// - `this_run` - A [`Tick`] corresponding to the current point in time -- "now".
    pub fn new(
        value: TRef,
        added: TickRef,
        changed: TickRef,
        last_run: Tick,
        this_run: Tick,
    ) -> Self {
        Self {
            value,
            ticks: Ticks {
                added,
                changed,
                last_run,
                this_run,
            },
        }
    }

    /// Maps to an inner value by applying a function to the contained reference, without flagging a change.
    ///
    /// You should never modify the argument passed to the closure -- if you want to modify the data
    /// without flagging a change, consider using [`DetectChangesMut::bypass_change_detection`] to make your intent explicit.
    ///
    /// ```rust
    /// # use bevy_ecs::prelude::*;
    /// # #[derive(PartialEq)] pub struct Vec2;
    /// # impl Vec2 { pub const ZERO: Self = Self; }
    /// # #[derive(Component)] pub struct Transform { translation: Vec2 }
    /// // When run, zeroes the translation of every entity.
    /// fn reset_positions(mut transforms: Query<&mut Transform>) {
    ///     for transform in &mut transforms {
    ///         // We pinky promise not to modify `t` within the closure.
    ///         // Breaking this promise will result in logic errors, but will never cause undefined behavior.
    ///         let mut translation = transform.map_unchanged(|t| &mut t.translation);
    ///         // Only reset the translation if it isn't already zero;
    ///         translation.set_if_neq(Vec2::ZERO);
    ///     }
    /// }
    /// # bevy_ecs::system::assert_is_system(reset_positions);
    /// ```
    pub fn map_unchanged<U>(self, f: impl FnOnce(TRef) -> U) -> Proxy<TickRef, U> {
        Proxy {
            value: f(self.value),
            ticks: self.ticks,
        }
    }
}

impl<'w, TickRef: DerefMut<Target = Tick> + 'w> Proxy<TickRef, PtrMut<'w>> {
    /// Transforms this [`Proxy`] on a [`PtrMut`] into a [`Proxy`] over the type `T` with the same lifetime.
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this [`PtrMut`].
    pub unsafe fn with_type<T>(self) -> Proxy<TickRef, &'w mut T> {
        Proxy {
            value: self.value.deref_mut(),
            ticks: self.ticks,
        }
    }

    /// Reset the changed state of this [`Proxy`]
    #[inline]
    pub fn set_changed(&mut self) {
        *self.ticks.changed = self.ticks.this_run;
    }

    /// Override the changed state of this [`Proxy`]
    #[inline]
    pub fn set_last_changed(&mut self, last_changed: Tick) {
        *self.ticks.changed = last_changed;
    }

    /// Consume `self` and return a mutable reference to the
    /// contained value while marking `self` as "changed".
    #[inline]
    pub fn into_inner(mut self) -> PtrMut<'w> {
        self.set_changed();
        self.value
    }
}

/// TODO
pub type Ref<'w, T> = Proxy<&'w Tick, &'w T>;

/// TODO
pub type Mut<'w, T> = Proxy<&'w mut Tick, &'w mut T>;

/// TODO
pub type MutUntyped<'w> = Proxy<&'w mut Tick, PtrMut<'w>>;

/// TODO
pub type Res<'w, T> = Ref<'w, T>;

/// TODO
pub type ResMut<'w, T> = Mut<'w, T>;

/// TODO
#[derive(Debug)]
pub struct NonSendMut<'w, T: ?Sized>(pub(crate) Proxy<&'w mut Tick, &'w mut T>);

impl<'w, T: ?Sized> From<NonSendMut<'w, T>> for Mut<'w, T> {
    fn from(value: NonSendMut<'w, T>) -> Self {
        value.0
    }
}

impl<'w, T: ?Sized> Deref for NonSendMut<'w, T> {
    type Target = Mut<'w, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'w, T: ?Sized> DerefMut for NonSendMut<'w, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs_macros::Resource;
    use bevy_ptr::PtrMut;
    use bevy_reflect::{FromType, ReflectFromPtr};

    use crate::{
        self as bevy_ecs,
        change_detection::{
            Mut, NonSendMut, Proxy, Ref, ResMut, Ticks, CHECK_TICK_THRESHOLD, MAX_CHANGE_AGE,
        },
        component::{Component, ComponentTicks, Tick},
        system::{IntoSystem, Query, System},
        world::World,
    };

    use super::{DetectChanges, DetectChangesMut, MutUntyped};

    #[derive(Component, PartialEq)]
    struct C;

    #[derive(Resource)]
    struct R;

    #[derive(Resource, PartialEq)]
    struct R2(u8);

    #[test]
    fn change_expiration() {
        fn change_detected(query: Query<Ref<C>>) -> bool {
            query.single().is_changed()
        }

        fn change_expired(query: Query<Ref<C>>) -> bool {
            query.single().is_changed()
        }

        let mut world = World::new();

        // component added: 1, changed: 1
        world.spawn(C);

        let mut change_detected_system = IntoSystem::into_system(change_detected);
        let mut change_expired_system = IntoSystem::into_system(change_expired);
        change_detected_system.initialize(&mut world);
        change_expired_system.initialize(&mut world);

        // world: 1, system last ran: 0, component changed: 1
        // The spawn will be detected since it happened after the system "last ran".
        assert!(change_detected_system.run((), &mut world));

        // world: 1 + MAX_CHANGE_AGE
        let change_tick = world.change_tick.get_mut();
        *change_tick = change_tick.wrapping_add(MAX_CHANGE_AGE);

        // Both the system and component appeared `MAX_CHANGE_AGE` ticks ago.
        // Since we clamp things to `MAX_CHANGE_AGE` for determinism,
        // `ComponentTicks::is_changed` will now see `MAX_CHANGE_AGE > MAX_CHANGE_AGE`
        // and return `false`.
        assert!(!change_expired_system.run((), &mut world));
    }

    #[test]
    fn change_tick_wraparound() {
        let mut world = World::new();
        world.last_change_tick = Tick::new(u32::MAX);
        *world.change_tick.get_mut() = 0;

        // component added: 0, changed: 0
        world.spawn(C);

        world.increment_change_tick();

        // Since the world is always ahead, as long as changes can't get older than `u32::MAX` (which we ensure),
        // the wrapping difference will always be positive, so wraparound doesn't matter.
        let mut query = world.query::<Ref<C>>();
        assert!(query.single(&world).is_changed());
    }

    #[test]
    fn change_tick_scan() {
        let mut world = World::new();

        // component added: 1, changed: 1
        world.spawn(C);

        // a bunch of stuff happens, the component is now older than `MAX_CHANGE_AGE`
        *world.change_tick.get_mut() += MAX_CHANGE_AGE + CHECK_TICK_THRESHOLD;
        let change_tick = world.change_tick();

        let mut query = world.query::<Ref<C>>();
        for tracker in query.iter(&world) {
            let ticks_since_insert = change_tick.relative_to(*tracker.ticks.added).get();
            let ticks_since_change = change_tick.relative_to(*tracker.ticks.changed).get();
            assert!(ticks_since_insert > MAX_CHANGE_AGE);
            assert!(ticks_since_change > MAX_CHANGE_AGE);
        }

        // scan change ticks and clamp those at risk of overflow
        world.check_change_ticks();

        for tracker in query.iter(&world) {
            let ticks_since_insert = change_tick.relative_to(*tracker.ticks.added).get();
            let ticks_since_change = change_tick.relative_to(*tracker.ticks.changed).get();
            assert!(ticks_since_insert == MAX_CHANGE_AGE);
            assert!(ticks_since_change == MAX_CHANGE_AGE);
        }
    }

    #[test]
    fn mut_from_res_mut() {
        let mut component_ticks = ComponentTicks {
            added: Tick::new(1),
            changed: Tick::new(2),
        };
        let ticks = Ticks {
            added: &mut component_ticks.added,
            changed: &mut component_ticks.changed,
            last_run: Tick::new(3),
            this_run: Tick::new(4),
        };
        let mut res = R {};
        let res_mut = ResMut {
            value: &mut res,
            ticks,
        };

        let into_mut: Mut<R> = res_mut;
        assert_eq!(1, into_mut.ticks.added.get());
        assert_eq!(2, into_mut.ticks.changed.get());
        assert_eq!(3, into_mut.ticks.last_run.get());
        assert_eq!(4, into_mut.ticks.this_run.get());
    }

    #[test]
    fn mut_new() {
        let mut component_ticks = ComponentTicks {
            added: Tick::new(1),
            changed: Tick::new(3),
        };
        let mut res = R {};

        let val = Mut::new(
            &mut res,
            &mut component_ticks.added,
            &mut component_ticks.changed,
            Tick::new(2), // last_run
            Tick::new(4), // this_run
        );

        assert!(!val.is_added());
        assert!(val.is_changed());
    }

    #[test]
    fn mut_from_non_send_mut() {
        let mut component_ticks = ComponentTicks {
            added: Tick::new(1),
            changed: Tick::new(2),
        };
        let ticks = Ticks {
            added: &mut component_ticks.added,
            changed: &mut component_ticks.changed,
            last_run: Tick::new(3),
            this_run: Tick::new(4),
        };
        let mut res = R {};
        let non_send_mut = NonSendMut(Proxy {
            value: &mut res,
            ticks,
        });

        let into_mut: Mut<R> = non_send_mut.into();
        assert_eq!(1, into_mut.ticks.added.get());
        assert_eq!(2, into_mut.ticks.changed.get());
        assert_eq!(3, into_mut.ticks.last_run.get());
        assert_eq!(4, into_mut.ticks.this_run.get());
    }

    #[test]
    fn map_mut() {
        use super::*;
        struct Outer(i64);

        let last_run = Tick::new(2);
        let this_run = Tick::new(3);
        let mut component_ticks = ComponentTicks {
            added: Tick::new(1),
            changed: Tick::new(2),
        };
        let ticks = Ticks {
            added: &mut component_ticks.added,
            changed: &mut component_ticks.changed,
            last_run,
            this_run,
        };

        let mut outer = Outer(0);
        let ptr = Mut {
            value: &mut outer,
            ticks,
        };
        assert!(!ptr.is_changed());

        // Perform a mapping operation.
        let mut inner = ptr.map_unchanged(|x| &mut x.0);
        assert!(!inner.is_changed());

        // Mutate the inner value.
        *inner = 64;
        assert!(inner.is_changed());
        // Modifying one field of a component should flag a change for the entire component.
        assert!(component_ticks.is_changed(last_run, this_run));
    }

    #[test]
    fn set_if_neq() {
        let mut world = World::new();

        world.insert_resource(R2(0));
        // Resources are Changed when first added
        world.increment_change_tick();
        // This is required to update world::last_change_tick
        world.clear_trackers();

        let mut r = world.resource_mut::<R2>();
        assert!(!r.is_changed(), "Resource must begin unchanged.");

        r.set_if_neq(R2(0));
        assert!(
            !r.is_changed(),
            "Resource must not be changed after setting to the same value."
        );

        r.set_if_neq(R2(3));
        assert!(
            r.is_changed(),
            "Resource must be changed after setting to a different value."
        );
    }

    #[test]
    fn mut_untyped_to_reflect() {
        let last_run = Tick::new(2);
        let this_run = Tick::new(3);
        let mut component_ticks = ComponentTicks {
            added: Tick::new(1),
            changed: Tick::new(2),
        };
        let ticks = Ticks {
            added: &mut component_ticks.added,
            changed: &mut component_ticks.changed,
            last_run,
            this_run,
        };

        let mut value: i32 = 5;
        let value = MutUntyped {
            value: PtrMut::from(&mut value),
            ticks,
        };

        let reflect_from_ptr = <ReflectFromPtr as FromType<i32>>::from_type();

        let mut new = value.map_unchanged(|ptr| {
            // SAFETY: The underlying type of `ptr` matches `reflect_from_ptr`.
            let value = unsafe { reflect_from_ptr.as_reflect_ptr_mut(ptr) };
            value
        });

        assert!(!new.is_changed());

        new.reflect_mut();

        assert!(new.is_changed());
    }
}
