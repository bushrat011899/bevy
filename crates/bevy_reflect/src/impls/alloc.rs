#![expect(
    unused_qualifications,
    reason = "Temporary workaround for impl_reflect!(Option/Result false-positive"
)]

use crate::{
    impl_type_path, map_apply, map_partial_eq, map_try_apply,
    prelude::ReflectDefault,
    reflect::impl_full_reflect,
    set_apply, set_partial_eq, set_try_apply,
    utility::{reflect_hasher, GenericTypeInfoCell, GenericTypePathCell, NonGenericTypeInfoCell},
    ApplyError, DynamicMap, FromReflect, FromType, Generics, GetTypeRegistration, List, ListInfo,
    ListIter, Map, MapInfo, MapIter, MaybeTyped, OpaqueInfo, PartialReflect, Reflect,
    ReflectCloneError, ReflectDeserialize, ReflectFromPtr, ReflectFromReflect, ReflectKind,
    ReflectMut, ReflectOwned, ReflectRef, ReflectSerialize, Set, SetInfo, TypeInfo, TypeParamInfo,
    TypePath, TypeRegistration, TypeRegistry, Typed,
};
use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    collections::VecDeque,
    format,
    string::ToString,
    vec::Vec,
};
use bevy_reflect_derive::impl_reflect_opaque;
use core::{
    any::Any,
    fmt,
    hash::{BuildHasher, Hash, Hasher},
};

impl_reflect_opaque!(::alloc::string::String(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(::alloc::collections::BTreeSet<T: Ord + Eq + Clone + Send + Sync>(Clone));
impl_reflect_opaque!(::bevy_platform::sync::Arc<T: Send + Sync + ?Sized>(Clone));
impl_reflect_opaque!(::alloc::collections::BinaryHeap<T: Clone>(Clone));

impl_reflect_for_veclike!(Vec<T>, Vec::insert, Vec::remove, Vec::push, Vec::pop, [T]);
impl_type_path!(::alloc::vec::Vec<T>);
#[cfg(feature = "functions")]
crate::func::macros::impl_function_traits!(Vec<T>; <T: FromReflect + MaybeTyped + TypePath + GetTypeRegistration>);

impl_reflect_for_veclike!(
    VecDeque<T>,
    VecDeque::insert,
    VecDeque::remove,
    VecDeque::push_back,
    VecDeque::pop_back,
    VecDeque::<T>
);
impl_type_path!(::alloc::collections::VecDeque<T>);
#[cfg(feature = "functions")]
crate::func::macros::impl_function_traits!(VecDeque<T>; <T: FromReflect + MaybeTyped + TypePath + GetTypeRegistration>);

impl_reflect_for_hashmap!(bevy_platform::collections::HashMap<K, V, S>);
impl_type_path!(::bevy_platform::collections::HashMap<K, V, S>);
#[cfg(feature = "functions")]
crate::func::macros::impl_function_traits!(::bevy_platform::collections::HashMap<K, V, S>;
    <
        K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Hash,
        V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,
        S: TypePath + BuildHasher + Default + Send + Sync
    >
);

impl_reflect_for_hashset!(::bevy_platform::collections::HashSet<V,S>);
impl_type_path!(::bevy_platform::collections::HashSet<V, S>);
#[cfg(feature = "functions")]
crate::func::macros::impl_function_traits!(::bevy_platform::collections::HashSet<V, S>;
    <
        V: Hash + Eq + FromReflect + TypePath + GetTypeRegistration,
        S: TypePath + BuildHasher + Default + Send + Sync
    >
);

impl<K, V> Map for ::alloc::collections::BTreeMap<K, V>
where
    K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Ord,
    V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,
{
    fn get(&self, key: &dyn PartialReflect) -> Option<&dyn PartialReflect> {
        key.try_downcast_ref::<K>()
            .and_then(|key| Self::get(self, key))
            .map(|value| value as &dyn PartialReflect)
    }

    fn get_mut(&mut self, key: &dyn PartialReflect) -> Option<&mut dyn PartialReflect> {
        key.try_downcast_ref::<K>()
            .and_then(move |key| Self::get_mut(self, key))
            .map(|value| value as &mut dyn PartialReflect)
    }

    fn get_at(&self, index: usize) -> Option<(&dyn PartialReflect, &dyn PartialReflect)> {
        self.iter()
            .nth(index)
            .map(|(key, value)| (key as &dyn PartialReflect, value as &dyn PartialReflect))
    }

    fn get_at_mut(
        &mut self,
        index: usize,
    ) -> Option<(&dyn PartialReflect, &mut dyn PartialReflect)> {
        self.iter_mut()
            .nth(index)
            .map(|(key, value)| (key as &dyn PartialReflect, value as &mut dyn PartialReflect))
    }

    fn len(&self) -> usize {
        Self::len(self)
    }

    fn iter(&self) -> MapIter {
        MapIter::new(self)
    }

    fn drain(&mut self) -> Vec<(Box<dyn PartialReflect>, Box<dyn PartialReflect>)> {
        // BTreeMap doesn't have a `drain` function. See
        // https://github.com/rust-lang/rust/issues/81074. So we have to fake one by popping
        // elements off one at a time.
        let mut result = Vec::with_capacity(self.len());
        while let Some((k, v)) = self.pop_first() {
            result.push((
                Box::new(k) as Box<dyn PartialReflect>,
                Box::new(v) as Box<dyn PartialReflect>,
            ));
        }
        result
    }

    fn clone_dynamic(&self) -> DynamicMap {
        let mut dynamic_map = DynamicMap::default();
        dynamic_map.set_represented_type(self.get_represented_type_info());
        for (k, v) in self {
            let key = K::from_reflect(k).unwrap_or_else(|| {
                panic!(
                    "Attempted to clone invalid key of type {}.",
                    k.reflect_type_path()
                )
            });
            dynamic_map.insert_boxed(Box::new(key), v.to_dynamic());
        }
        dynamic_map
    }

    fn insert_boxed(
        &mut self,
        key: Box<dyn PartialReflect>,
        value: Box<dyn PartialReflect>,
    ) -> Option<Box<dyn PartialReflect>> {
        let key = K::take_from_reflect(key).unwrap_or_else(|key| {
            panic!(
                "Attempted to insert invalid key of type {}.",
                key.reflect_type_path()
            )
        });
        let value = V::take_from_reflect(value).unwrap_or_else(|value| {
            panic!(
                "Attempted to insert invalid value of type {}.",
                value.reflect_type_path()
            )
        });
        self.insert(key, value)
            .map(|old_value| Box::new(old_value) as Box<dyn PartialReflect>)
    }

    fn remove(&mut self, key: &dyn PartialReflect) -> Option<Box<dyn PartialReflect>> {
        let mut from_reflect = None;
        key.try_downcast_ref::<K>()
            .or_else(|| {
                from_reflect = K::from_reflect(key);
                from_reflect.as_ref()
            })
            .and_then(|key| self.remove(key))
            .map(|value| Box::new(value) as Box<dyn PartialReflect>)
    }
}

impl<K, V> PartialReflect for ::alloc::collections::BTreeMap<K, V>
where
    K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Ord,
    V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,
{
    fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
        Some(<Self as Typed>::type_info())
    }
    #[inline]
    fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect> {
        self
    }

    fn as_partial_reflect(&self) -> &dyn PartialReflect {
        self
    }

    fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect {
        self
    }
    #[inline]
    fn try_into_reflect(self: Box<Self>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
        Ok(self)
    }

    fn try_as_reflect(&self) -> Option<&dyn Reflect> {
        Some(self)
    }

    fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {
        Some(self)
    }
    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::Map
    }

    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Map(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Map(self)
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Map(self)
    }

    fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
        let mut map = Self::new();
        for (key, value) in self.iter() {
            let key =
                key.reflect_clone()?
                    .take()
                    .map_err(|_| ReflectCloneError::FailedDowncast {
                        expected: Cow::Borrowed(<K as TypePath>::type_path()),
                        received: Cow::Owned(key.reflect_type_path().to_string()),
                    })?;
            let value =
                value
                    .reflect_clone()?
                    .take()
                    .map_err(|_| ReflectCloneError::FailedDowncast {
                        expected: Cow::Borrowed(<V as TypePath>::type_path()),
                        received: Cow::Owned(value.reflect_type_path().to_string()),
                    })?;
            map.insert(key, value);
        }

        Ok(Box::new(map))
    }

    fn reflect_partial_eq(&self, value: &dyn PartialReflect) -> Option<bool> {
        map_partial_eq(self, value)
    }

    fn apply(&mut self, value: &dyn PartialReflect) {
        map_apply(self, value);
    }

    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
        map_try_apply(self, value)
    }
}

impl_full_reflect!(
    <K, V> for ::alloc::collections::BTreeMap<K, V>
    where
        K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Ord,
        V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,
);

impl<K, V> Typed for ::alloc::collections::BTreeMap<K, V>
where
    K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Ord,
    V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,
{
    fn type_info() -> &'static TypeInfo {
        static CELL: GenericTypeInfoCell = GenericTypeInfoCell::new();
        CELL.get_or_insert::<Self, _>(|| {
            TypeInfo::Map(
                MapInfo::new::<Self, K, V>().with_generics(Generics::from_iter([
                    TypeParamInfo::new::<K>("K"),
                    TypeParamInfo::new::<V>("V"),
                ])),
            )
        })
    }
}

impl<K, V> GetTypeRegistration for ::alloc::collections::BTreeMap<K, V>
where
    K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Ord,
    V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,
{
    fn get_type_registration() -> TypeRegistration {
        let mut registration = TypeRegistration::of::<Self>();
        registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
        registration.insert::<ReflectFromReflect>(FromType::<Self>::from_type());
        registration
    }
}

impl<K, V> FromReflect for ::alloc::collections::BTreeMap<K, V>
where
    K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Ord,
    V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,
{
    fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
        let ref_map = reflect.reflect_ref().as_map().ok()?;

        let mut new_map = Self::new();

        for (key, value) in ref_map.iter() {
            let new_key = K::from_reflect(key)?;
            let new_value = V::from_reflect(value)?;
            new_map.insert(new_key, new_value);
        }

        Some(new_map)
    }
}

impl_type_path!(::alloc::collections::BTreeMap<K, V>);
#[cfg(feature = "functions")]
crate::func::macros::impl_function_traits!(::alloc::collections::BTreeMap<K, V>;
    <
        K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Ord,
        V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration
    >
);

impl PartialReflect for Cow<'static, str> {
    fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
        Some(<Self as Typed>::type_info())
    }

    #[inline]
    fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect> {
        self
    }

    fn as_partial_reflect(&self) -> &dyn PartialReflect {
        self
    }

    fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect {
        self
    }

    fn try_into_reflect(self: Box<Self>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
        Ok(self)
    }

    fn try_as_reflect(&self) -> Option<&dyn Reflect> {
        Some(self)
    }

    fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {
        Some(self)
    }

    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::Opaque
    }

    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Opaque(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Opaque(self)
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Opaque(self)
    }

    fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
        Ok(Box::new(self.clone()))
    }

    fn reflect_hash(&self) -> Option<u64> {
        let mut hasher = reflect_hasher();
        Hash::hash(&Any::type_id(self), &mut hasher);
        Hash::hash(self, &mut hasher);
        Some(hasher.finish())
    }

    fn reflect_partial_eq(&self, value: &dyn PartialReflect) -> Option<bool> {
        if let Some(value) = value.try_downcast_ref::<Self>() {
            Some(PartialEq::eq(self, value))
        } else {
            Some(false)
        }
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
        fmt::Debug::fmt(self, f)
    }

    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
        if let Some(value) = value.try_downcast_ref::<Self>() {
            self.clone_from(value);
        } else {
            return Err(ApplyError::MismatchedTypes {
                from_type: value.reflect_type_path().into(),
                // If we invoke the reflect_type_path on self directly the borrow checker complains that the lifetime of self must outlive 'static
                to_type: Self::type_path().into(),
            });
        }
        Ok(())
    }
}

impl_full_reflect!(for Cow<'static, str>);

impl Typed for Cow<'static, str> {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_set(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl GetTypeRegistration for Cow<'static, str> {
    fn get_type_registration() -> TypeRegistration {
        let mut registration = TypeRegistration::of::<Cow<'static, str>>();
        registration.insert::<ReflectDeserialize>(FromType::<Cow<'static, str>>::from_type());
        registration.insert::<ReflectFromPtr>(FromType::<Cow<'static, str>>::from_type());
        registration.insert::<ReflectFromReflect>(FromType::<Cow<'static, str>>::from_type());
        registration.insert::<ReflectSerialize>(FromType::<Cow<'static, str>>::from_type());
        registration
    }
}

impl FromReflect for Cow<'static, str> {
    fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
        Some(reflect.try_downcast_ref::<Cow<'static, str>>()?.clone())
    }
}

#[cfg(feature = "functions")]
crate::func::macros::impl_function_traits!(Cow<'static, str>);

impl<T: TypePath> TypePath for [T]
where
    [T]: ToOwned,
{
    fn type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| format!("[{}]", <T>::type_path()))
    }

    fn short_type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| format!("[{}]", <T>::short_type_path()))
    }
}

impl<T: FromReflect + MaybeTyped + Clone + TypePath + GetTypeRegistration> List
    for Cow<'static, [T]>
{
    fn get(&self, index: usize) -> Option<&dyn PartialReflect> {
        self.as_ref().get(index).map(|x| x as &dyn PartialReflect)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect> {
        self.to_mut()
            .get_mut(index)
            .map(|x| x as &mut dyn PartialReflect)
    }

    fn insert(&mut self, index: usize, element: Box<dyn PartialReflect>) {
        let value = T::take_from_reflect(element).unwrap_or_else(|value| {
            panic!(
                "Attempted to insert invalid value of type {}.",
                value.reflect_type_path()
            );
        });
        self.to_mut().insert(index, value);
    }

    fn remove(&mut self, index: usize) -> Box<dyn PartialReflect> {
        Box::new(self.to_mut().remove(index))
    }

    fn push(&mut self, value: Box<dyn PartialReflect>) {
        let value = T::take_from_reflect(value).unwrap_or_else(|value| {
            panic!(
                "Attempted to push invalid value of type {}.",
                value.reflect_type_path()
            )
        });
        self.to_mut().push(value);
    }

    fn pop(&mut self) -> Option<Box<dyn PartialReflect>> {
        self.to_mut()
            .pop()
            .map(|value| Box::new(value) as Box<dyn PartialReflect>)
    }

    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn iter(&self) -> ListIter {
        ListIter::new(self)
    }

    fn drain(&mut self) -> Vec<Box<dyn PartialReflect>> {
        self.to_mut()
            .drain(..)
            .map(|value| Box::new(value) as Box<dyn PartialReflect>)
            .collect()
    }
}

impl<T: FromReflect + MaybeTyped + Clone + TypePath + GetTypeRegistration> PartialReflect
    for Cow<'static, [T]>
{
    fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
        Some(<Self as Typed>::type_info())
    }

    #[inline]
    fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect> {
        self
    }

    fn as_partial_reflect(&self) -> &dyn PartialReflect {
        self
    }

    fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect {
        self
    }

    fn try_into_reflect(self: Box<Self>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
        Ok(self)
    }

    fn try_as_reflect(&self) -> Option<&dyn Reflect> {
        Some(self)
    }

    fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {
        Some(self)
    }

    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::List
    }

    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::List(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::List(self)
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::List(self)
    }

    fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
        Ok(Box::new(self.clone()))
    }

    fn reflect_hash(&self) -> Option<u64> {
        crate::list_hash(self)
    }

    fn reflect_partial_eq(&self, value: &dyn PartialReflect) -> Option<bool> {
        crate::list_partial_eq(self, value)
    }

    fn apply(&mut self, value: &dyn PartialReflect) {
        crate::list_apply(self, value);
    }

    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
        crate::list_try_apply(self, value)
    }
}

impl_full_reflect!(
    <T> for Cow<'static, [T]>
    where
        T: FromReflect + Clone + MaybeTyped + TypePath + GetTypeRegistration,
);

impl<T: FromReflect + MaybeTyped + Clone + TypePath + GetTypeRegistration> Typed
    for Cow<'static, [T]>
{
    fn type_info() -> &'static TypeInfo {
        static CELL: GenericTypeInfoCell = GenericTypeInfoCell::new();
        CELL.get_or_insert::<Self, _>(|| TypeInfo::List(ListInfo::new::<Self, T>()))
    }
}

impl<T: FromReflect + MaybeTyped + Clone + TypePath + GetTypeRegistration> GetTypeRegistration
    for Cow<'static, [T]>
{
    fn get_type_registration() -> TypeRegistration {
        TypeRegistration::of::<Cow<'static, [T]>>()
    }

    fn register_type_dependencies(registry: &mut TypeRegistry) {
        registry.register::<T>();
    }
}

impl<T: FromReflect + MaybeTyped + Clone + TypePath + GetTypeRegistration> FromReflect
    for Cow<'static, [T]>
{
    fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
        let ref_list = reflect.reflect_ref().as_list().ok()?;

        let mut temp_vec = Vec::with_capacity(ref_list.len());

        for field in ref_list.iter() {
            temp_vec.push(T::from_reflect(field)?);
        }

        Some(temp_vec.into())
    }
}

#[cfg(feature = "functions")]
crate::func::macros::impl_function_traits!(Cow<'static, [T]>; <T: FromReflect + MaybeTyped + Clone + TypePath + GetTypeRegistration>);

impl_type_path!(::alloc::borrow::Cow<'a: 'static, T: ToOwned + ?Sized>);

#[cfg(test)]
mod tests {
    use crate::{PartialReflect, Reflect};
    use alloc::{collections::BTreeMap, string::String, vec};
    use bevy_platform::collections::HashMap;

    #[test]
    fn should_partial_eq_string() {
        let a: &dyn PartialReflect = &String::from("Hello");
        let b: &dyn PartialReflect = &String::from("Hello");
        let c: &dyn PartialReflect = &String::from("World");
        assert!(a.reflect_partial_eq(b).unwrap_or_default());
        assert!(!a.reflect_partial_eq(c).unwrap_or_default());
    }

    #[test]
    fn should_partial_eq_vec() {
        let a: &dyn PartialReflect = &vec![1, 2, 3];
        let b: &dyn PartialReflect = &vec![1, 2, 3];
        let c: &dyn PartialReflect = &vec![3, 2, 1];
        assert!(a.reflect_partial_eq(b).unwrap_or_default());
        assert!(!a.reflect_partial_eq(c).unwrap_or_default());
    }

    #[test]
    fn should_partial_eq_hash_map() {
        let mut a = <HashMap<_, _>>::default();
        a.insert(0usize, 1.23_f64);
        let b = a.clone();
        let mut c = <HashMap<_, _>>::default();
        c.insert(0usize, 3.21_f64);

        let a: &dyn PartialReflect = &a;
        let b: &dyn PartialReflect = &b;
        let c: &dyn PartialReflect = &c;
        assert!(a.reflect_partial_eq(b).unwrap_or_default());
        assert!(!a.reflect_partial_eq(c).unwrap_or_default());
    }

    #[test]
    fn should_partial_eq_btree_map() {
        let mut a = BTreeMap::new();
        a.insert(0usize, 1.23_f64);
        let b = a.clone();
        let mut c = BTreeMap::new();
        c.insert(0usize, 3.21_f64);

        let a: &dyn Reflect = &a;
        let b: &dyn Reflect = &b;
        let c: &dyn Reflect = &c;
        assert!(a
            .reflect_partial_eq(b.as_partial_reflect())
            .unwrap_or_default());
        assert!(!a
            .reflect_partial_eq(c.as_partial_reflect())
            .unwrap_or_default());
    }
}
