#[doc(hidden)]
#[macro_export]
macro_rules! impl_reflect_for_veclike {
    ($ty:ty, $insert:expr, $remove:expr, $push:expr, $pop:expr, $sub:ty) => {
        impl<T: FromReflect + MaybeTyped + TypePath + GetTypeRegistration> List for $ty {
            #[inline]
            fn get(&self, index: usize) -> Option<&dyn PartialReflect> {
                <$sub>::get(self, index).map(|value| value as &dyn PartialReflect)
            }

            #[inline]
            fn get_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect> {
                <$sub>::get_mut(self, index).map(|value| value as &mut dyn PartialReflect)
            }

            fn insert(&mut self, index: usize, value: Box<dyn PartialReflect>) {
                let value = value.try_take::<T>().unwrap_or_else(|value| {
                    T::from_reflect(&*value).unwrap_or_else(|| {
                        panic!(
                            "Attempted to insert invalid value of type {}.",
                            value.reflect_type_path()
                        )
                    })
                });
                $insert(self, index, value);
            }

            fn remove(&mut self, index: usize) -> Box<dyn PartialReflect> {
                Box::new($remove(self, index))
            }

            fn push(&mut self, value: Box<dyn PartialReflect>) {
                let value = T::take_from_reflect(value).unwrap_or_else(|value| {
                    panic!(
                        "Attempted to push invalid value of type {}.",
                        value.reflect_type_path()
                    )
                });
                $push(self, value);
            }

            fn pop(&mut self) -> Option<Box<dyn PartialReflect>> {
                $pop(self).map(|value| Box::new(value) as Box<dyn PartialReflect>)
            }

            #[inline]
            fn len(&self) -> usize {
                <$sub>::len(self)
            }

            #[inline]
            fn iter(&self) -> ListIter {
                ListIter::new(self)
            }

            #[inline]
            fn drain(&mut self) -> Vec<Box<dyn PartialReflect>> {
                self.drain(..)
                    .map(|value| Box::new(value) as Box<dyn PartialReflect>)
                    .collect()
            }
        }

        impl<T: FromReflect + MaybeTyped + TypePath + GetTypeRegistration> PartialReflect for $ty {
            #[inline]
            fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
                Some(<Self as Typed>::type_info())
            }

            fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect> {
                self
            }

            #[inline]
            fn as_partial_reflect(&self) -> &dyn PartialReflect {
                self
            }

            #[inline]
            fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect {
                self
            }

            fn try_into_reflect(
                self: Box<Self>,
            ) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
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
                Ok(Box::new(
                    self.iter()
                        .map(|value| {
                            value.reflect_clone()?.take().map_err(|_| {
                                ReflectCloneError::FailedDowncast {
                                    expected: Cow::Borrowed(<T as TypePath>::type_path()),
                                    received: Cow::Owned(value.reflect_type_path().to_string()),
                                }
                            })
                        })
                        .collect::<Result<Self, ReflectCloneError>>()?,
                ))
            }

            fn reflect_hash(&self) -> Option<u64> {
                $crate::list_hash(self)
            }

            fn reflect_partial_eq(&self, value: &dyn PartialReflect) -> Option<bool> {
                $crate::list_partial_eq(self, value)
            }

            fn apply(&mut self, value: &dyn PartialReflect) {
                $crate::list_apply(self, value);
            }

            fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
                $crate::list_try_apply(self, value)
            }
        }

        impl_full_reflect!(<T> for $ty where T: FromReflect + MaybeTyped + TypePath + GetTypeRegistration);

        impl<T: FromReflect + MaybeTyped + TypePath + GetTypeRegistration> Typed for $ty {
            fn type_info() -> &'static TypeInfo {
                static CELL: GenericTypeInfoCell = GenericTypeInfoCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    TypeInfo::List(
                        ListInfo::new::<Self, T>().with_generics(Generics::from_iter([
                            TypeParamInfo::new::<T>("T")
                        ]))
                    )
                })
            }
        }

        impl<T: FromReflect + MaybeTyped + TypePath + GetTypeRegistration> GetTypeRegistration
            for $ty
        {
            fn get_type_registration() -> TypeRegistration {
                let mut registration = TypeRegistration::of::<$ty>();
                registration.insert::<ReflectFromPtr>(FromType::<$ty>::from_type());
                registration.insert::<ReflectFromReflect>(FromType::<$ty>::from_type());
                registration
            }

            fn register_type_dependencies(registry: &mut TypeRegistry) {
                registry.register::<T>();
            }
        }

        impl<T: FromReflect + MaybeTyped + TypePath + GetTypeRegistration> FromReflect for $ty {
            fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
                let ref_list = reflect.reflect_ref().as_list().ok()?;

                let mut new_list = Self::with_capacity(ref_list.len());

                for field in ref_list.iter() {
                    $push(&mut new_list, T::from_reflect(field)?);
                }

                Some(new_list)
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_reflect_for_hashmap {
    ($ty:path) => {
        impl<K, V, S> Map for $ty
        where
            K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Hash,
            V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,
            S: TypePath + BuildHasher + Default + Send + Sync,
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
                self.iter_mut().nth(index).map(|(key, value)| {
                    (key as &dyn PartialReflect, value as &mut dyn PartialReflect)
                })
            }

            fn len(&self) -> usize {
                Self::len(self)
            }

            fn iter(&self) -> MapIter {
                MapIter::new(self)
            }

            fn drain(&mut self) -> Vec<(Box<dyn PartialReflect>, Box<dyn PartialReflect>)> {
                self.drain()
                    .map(|(key, value)| {
                        (
                            Box::new(key) as Box<dyn PartialReflect>,
                            Box::new(value) as Box<dyn PartialReflect>,
                        )
                    })
                    .collect()
            }

            fn to_dynamic_map(&self) -> DynamicMap {
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

        impl<K, V, S> PartialReflect for $ty
        where
            K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Hash,
            V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,
            S: TypePath + BuildHasher + Default + Send + Sync,
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

            fn try_into_reflect(
                self: Box<Self>,
            ) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
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
                let mut map = Self::with_capacity_and_hasher(self.len(), S::default());
                for (key, value) in self.iter() {
                    let key = key.reflect_clone()?.take().map_err(|_| {
                        ReflectCloneError::FailedDowncast {
                            expected: Cow::Borrowed(<K as TypePath>::type_path()),
                            received: Cow::Owned(key.reflect_type_path().to_string()),
                        }
                    })?;
                    let value = value.reflect_clone()?.take().map_err(|_| {
                        ReflectCloneError::FailedDowncast {
                            expected: Cow::Borrowed(<V as TypePath>::type_path()),
                            received: Cow::Owned(value.reflect_type_path().to_string()),
                        }
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
            <K, V, S> for $ty
            where
                K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Hash,
                V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,
                S: TypePath + BuildHasher + Default + Send + Sync,
        );

        impl<K, V, S> Typed for $ty
        where
            K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Hash,
            V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,
            S: TypePath + BuildHasher + Default + Send + Sync,
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

        impl<K, V, S> GetTypeRegistration for $ty
        where
            K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Hash,
            V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,
            S: TypePath + BuildHasher + Default + Send + Sync + Default,
        {
            fn get_type_registration() -> TypeRegistration {
                let mut registration = TypeRegistration::of::<Self>();
                registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
                registration.insert::<ReflectFromReflect>(FromType::<Self>::from_type());
                registration
            }

            fn register_type_dependencies(registry: &mut TypeRegistry) {
                registry.register::<K>();
                registry.register::<V>();
            }
        }

        impl<K, V, S> FromReflect for $ty
        where
            K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Hash,
            V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,
            S: TypePath + BuildHasher + Default + Send + Sync,
        {
            fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
                let ref_map = reflect.reflect_ref().as_map().ok()?;

                let mut new_map = Self::with_capacity_and_hasher(ref_map.len(), S::default());

                for (key, value) in ref_map.iter() {
                    let new_key = K::from_reflect(key)?;
                    let new_value = V::from_reflect(value)?;
                    new_map.insert(new_key, new_value);
                }

                Some(new_map)
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_reflect_for_hashset {
    ($ty:path) => {
        impl<V, S> Set for $ty
        where
            V: FromReflect + TypePath + GetTypeRegistration + Eq + Hash,
            S: TypePath + BuildHasher + Default + Send + Sync,
        {
            fn get(&self, value: &dyn PartialReflect) -> Option<&dyn PartialReflect> {
                value
                    .try_downcast_ref::<V>()
                    .and_then(|value| Self::get(self, value))
                    .map(|value| value as &dyn PartialReflect)
            }

            fn len(&self) -> usize {
                Self::len(self)
            }

            fn iter(&self) -> Box<dyn Iterator<Item = &dyn PartialReflect> + '_> {
                let iter = self.iter().map(|v| v as &dyn PartialReflect);
                Box::new(iter)
            }

            fn drain(&mut self) -> Vec<Box<dyn PartialReflect>> {
                self.drain()
                    .map(|value| Box::new(value) as Box<dyn PartialReflect>)
                    .collect()
            }

            fn insert_boxed(&mut self, value: Box<dyn PartialReflect>) -> bool {
                let value = V::take_from_reflect(value).unwrap_or_else(|value| {
                    panic!(
                        "Attempted to insert invalid value of type {}.",
                        value.reflect_type_path()
                    )
                });
                self.insert(value)
            }

            fn remove(&mut self, value: &dyn PartialReflect) -> bool {
                let mut from_reflect = None;
                value
                    .try_downcast_ref::<V>()
                    .or_else(|| {
                        from_reflect = V::from_reflect(value);
                        from_reflect.as_ref()
                    })
                    .is_some_and(|value| self.remove(value))
            }

            fn contains(&self, value: &dyn PartialReflect) -> bool {
                let mut from_reflect = None;
                value
                    .try_downcast_ref::<V>()
                    .or_else(|| {
                        from_reflect = V::from_reflect(value);
                        from_reflect.as_ref()
                    })
                    .is_some_and(|value| self.contains(value))
            }
        }

        impl<V, S> PartialReflect for $ty
        where
            V: FromReflect + TypePath + GetTypeRegistration + Eq + Hash,
            S: TypePath + BuildHasher + Default + Send + Sync,
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
            fn try_into_reflect(
                self: Box<Self>,
            ) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
                Ok(self)
            }

            fn try_as_reflect(&self) -> Option<&dyn Reflect> {
                Some(self)
            }

            fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {
                Some(self)
            }

            fn apply(&mut self, value: &dyn PartialReflect) {
                set_apply(self, value);
            }

            fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
                set_try_apply(self, value)
            }

            fn reflect_kind(&self) -> ReflectKind {
                ReflectKind::Set
            }

            fn reflect_ref(&self) -> ReflectRef {
                ReflectRef::Set(self)
            }

            fn reflect_mut(&mut self) -> ReflectMut {
                ReflectMut::Set(self)
            }

            fn reflect_owned(self: Box<Self>) -> ReflectOwned {
                ReflectOwned::Set(self)
            }

            fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
                let mut set = Self::with_capacity_and_hasher(self.len(), S::default());
                for value in self.iter() {
                    let value = value.reflect_clone()?.take().map_err(|_| {
                        ReflectCloneError::FailedDowncast {
                            expected: Cow::Borrowed(<V as TypePath>::type_path()),
                            received: Cow::Owned(value.reflect_type_path().to_string()),
                        }
                    })?;
                    set.insert(value);
                }

                Ok(Box::new(set))
            }

            fn reflect_partial_eq(&self, value: &dyn PartialReflect) -> Option<bool> {
                set_partial_eq(self, value)
            }
        }

        impl<V, S> Typed for $ty
        where
            V: FromReflect + TypePath + GetTypeRegistration + Eq + Hash,
            S: TypePath + BuildHasher + Default + Send + Sync,
        {
            fn type_info() -> &'static TypeInfo {
                static CELL: GenericTypeInfoCell = GenericTypeInfoCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    TypeInfo::Set(
                        SetInfo::new::<Self, V>().with_generics(Generics::from_iter([
                            TypeParamInfo::new::<V>("V")
                        ]))
                    )
                })
            }
        }

        impl<V, S> GetTypeRegistration for $ty
        where
            V: FromReflect + TypePath + GetTypeRegistration + Eq + Hash,
            S: TypePath + BuildHasher + Default + Send + Sync + Default,
        {
            fn get_type_registration() -> TypeRegistration {
                let mut registration = TypeRegistration::of::<Self>();
                registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
                registration.insert::<ReflectFromReflect>(FromType::<Self>::from_type());
                registration
            }

            fn register_type_dependencies(registry: &mut TypeRegistry) {
                registry.register::<V>();
            }
        }

        impl_full_reflect!(
            <V, S> for $ty
            where
                V: FromReflect + TypePath + GetTypeRegistration + Eq + Hash,
                S: TypePath + BuildHasher + Default + Send + Sync,
        );

        impl<V, S> FromReflect for $ty
        where
            V: FromReflect + TypePath + GetTypeRegistration + Eq + Hash,
            S: TypePath + BuildHasher + Default + Send + Sync,
        {
            fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
                let ref_set = reflect.reflect_ref().as_set().ok()?;

                let mut new_set = Self::with_capacity_and_hasher(ref_set.len(), S::default());

                for value in ref_set.iter() {
                    let new_value = V::from_reflect(value)?;
                    new_set.insert(new_value);
                }

                Some(new_set)
            }
        }
    };
}

mod alloc;
mod core;
mod foldhash;

crate::cfg::std! {
    mod std;
}

#[cfg(feature = "glam")]
mod glam;
#[cfg(feature = "petgraph")]
mod petgraph;
#[cfg(feature = "smallvec")]
mod smallvec;
#[cfg(feature = "smol_str")]
mod smol_str;
#[cfg(feature = "uuid")]
mod uuid;
#[cfg(feature = "wgpu-types")]
mod wgpu_types;
