#![expect(
    unused_qualifications,
    reason = "Temporary workaround for impl_reflect!(Option/Result false-positive"
)]

use crate::{
    impl_type_path,
    prelude::ReflectDefault,
    reflect::impl_full_reflect,
    utility::{reflect_hasher, GenericTypeInfoCell, GenericTypePathCell, NonGenericTypeInfoCell},
    ApplyError, Array, ArrayInfo, ArrayIter, DynamicTypePath, FromReflect, FromType,
    GetTypeRegistration, MaybeTyped, OpaqueInfo, PartialReflect, Reflect, ReflectCloneError,
    ReflectDeserialize, ReflectFromPtr, ReflectFromReflect, ReflectKind, ReflectMut, ReflectOwned,
    ReflectRef, ReflectSerialize, TypeInfo, TypePath, TypeRegistration, TypeRegistry, Typed,
};
use alloc::{boxed::Box, format, vec::Vec};
use bevy_reflect_derive::{impl_reflect, impl_reflect_opaque};
use core::{
    any::Any,
    fmt,
    hash::{Hash, Hasher},
    panic::Location,
};

macro_rules! impl_reflect_for_atomic {
    ($ty:ty, $ordering:expr) => {
        impl_type_path!($ty);

        const _: () = {
            #[cfg(feature = "functions")]
            crate::func::macros::impl_function_traits!($ty);

            impl GetTypeRegistration for $ty
            where
                $ty: Any + Send + Sync,
            {
                fn get_type_registration() -> TypeRegistration {
                    let mut registration = TypeRegistration::of::<Self>();
                    registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
                    registration.insert::<ReflectFromReflect>(FromType::<Self>::from_type());
                    registration.insert::<ReflectDefault>(FromType::<Self>::from_type());

                    // Serde only supports atomic types when the "std" feature is enabled
                    crate::cfg::std! {
                        registration.insert::<ReflectSerialize>(FromType::<Self>::from_type());
                        registration.insert::<ReflectDeserialize>(FromType::<Self>::from_type());
                    }

                    registration
                }
            }

            impl Typed for $ty
            where
                $ty: Any + Send + Sync,
            {
                fn type_info() -> &'static TypeInfo {
                    static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
                    CELL.get_or_set(|| {
                        let info = OpaqueInfo::new::<Self>();
                        TypeInfo::Opaque(info)
                    })
                }
            }

            impl PartialReflect for $ty
            where
                $ty: Any + Send + Sync,
            {
                #[inline]
                fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
                    Some(<Self as Typed>::type_info())
                }
                #[inline]
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
                #[inline]
                fn try_into_reflect(
                    self: Box<Self>,
                ) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
                    Ok(self)
                }
                #[inline]
                fn try_as_reflect(&self) -> Option<&dyn Reflect> {
                    Some(self)
                }
                #[inline]
                fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {
                    Some(self)
                }

                #[inline]
                fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
                    Ok(Box::new(<$ty>::new(self.load($ordering))))
                }

                #[inline]
                fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
                    if let Some(value) = value.try_downcast_ref::<Self>() {
                        *self = <$ty>::new(value.load($ordering));
                    } else {
                        return Err(ApplyError::MismatchedTypes {
                            from_type: Into::into(DynamicTypePath::reflect_type_path(value)),
                            to_type: Into::into(<Self as TypePath>::type_path()),
                        });
                    }
                    Ok(())
                }
                #[inline]
                fn reflect_kind(&self) -> ReflectKind {
                    ReflectKind::Opaque
                }
                #[inline]
                fn reflect_ref(&self) -> ReflectRef {
                    ReflectRef::Opaque(self)
                }
                #[inline]
                fn reflect_mut(&mut self) -> ReflectMut {
                    ReflectMut::Opaque(self)
                }
                #[inline]
                fn reflect_owned(self: Box<Self>) -> ReflectOwned {
                    ReflectOwned::Opaque(self)
                }
                fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    fmt::Debug::fmt(self, f)
                }
            }

            impl FromReflect for $ty
            where
                $ty: Any + Send + Sync,
            {
                fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
                    Some(<$ty>::new(
                        reflect.try_downcast_ref::<$ty>()?.load($ordering),
                    ))
                }
            }
        };

        impl_full_reflect!(for $ty where $ty: Any + Send + Sync);
    };
}

impl_reflect_opaque!(bool(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(char(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(u8(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(u16(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(u32(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(u64(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(u128(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(usize(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(i8(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(i16(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(i32(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(i64(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(i128(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(isize(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(f32(
    Clone,
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(f64(
    Clone,
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_type_path!(str);

impl_reflect_opaque!(::core::any::TypeId(Clone, Debug, Hash, PartialEq,));
impl_reflect_opaque!(::core::ops::Range<T: Clone + Send + Sync>(Clone));
impl_reflect_opaque!(::core::ops::RangeInclusive<T: Clone + Send + Sync>(Clone));
impl_reflect_opaque!(::core::ops::RangeFrom<T: Clone + Send + Sync>(Clone));
impl_reflect_opaque!(::core::ops::RangeTo<T: Clone + Send + Sync>(Clone));
impl_reflect_opaque!(::core::ops::RangeToInclusive<T: Clone + Send + Sync>(Clone));
impl_reflect_opaque!(::core::ops::RangeFull(Clone));
impl_reflect_opaque!(::core::ops::Bound<T: Clone + Send + Sync>(Clone));
impl_reflect_opaque!(::core::time::Duration(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Default
));
impl_reflect_opaque!(::bevy_platform::time::Instant(
    Clone, Debug, Hash, PartialEq
));
impl_reflect_opaque!(::core::num::NonZeroI128(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize
));
impl_reflect_opaque!(::core::num::NonZeroU128(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize
));
impl_reflect_opaque!(::core::num::NonZeroIsize(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize
));
impl_reflect_opaque!(::core::num::NonZeroUsize(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize
));
impl_reflect_opaque!(::core::num::NonZeroI64(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize
));
impl_reflect_opaque!(::core::num::NonZeroU64(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize
));
impl_reflect_opaque!(::core::num::NonZeroU32(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize
));
impl_reflect_opaque!(::core::num::NonZeroI32(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize
));
impl_reflect_opaque!(::core::num::NonZeroI16(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize
));
impl_reflect_opaque!(::core::num::NonZeroU16(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize
));
impl_reflect_opaque!(::core::num::NonZeroU8(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize
));
impl_reflect_opaque!(::core::num::NonZeroI8(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize
));
impl_reflect_opaque!(::core::num::Wrapping<T: Clone + Send + Sync>(Clone));
impl_reflect_opaque!(::core::num::Saturating<T: Clone + Send + Sync>(Clone));

impl_reflect_for_atomic!(
    ::core::sync::atomic::AtomicIsize,
    ::core::sync::atomic::Ordering::SeqCst
);
impl_reflect_for_atomic!(
    ::core::sync::atomic::AtomicUsize,
    ::core::sync::atomic::Ordering::SeqCst
);
#[cfg(target_has_atomic = "64")]
impl_reflect_for_atomic!(
    ::core::sync::atomic::AtomicI64,
    ::core::sync::atomic::Ordering::SeqCst
);
#[cfg(target_has_atomic = "64")]
impl_reflect_for_atomic!(
    ::core::sync::atomic::AtomicU64,
    ::core::sync::atomic::Ordering::SeqCst
);
impl_reflect_for_atomic!(
    ::core::sync::atomic::AtomicI32,
    ::core::sync::atomic::Ordering::SeqCst
);
impl_reflect_for_atomic!(
    ::core::sync::atomic::AtomicU32,
    ::core::sync::atomic::Ordering::SeqCst
);
impl_reflect_for_atomic!(
    ::core::sync::atomic::AtomicI16,
    ::core::sync::atomic::Ordering::SeqCst
);
impl_reflect_for_atomic!(
    ::core::sync::atomic::AtomicU16,
    ::core::sync::atomic::Ordering::SeqCst
);
impl_reflect_for_atomic!(
    ::core::sync::atomic::AtomicI8,
    ::core::sync::atomic::Ordering::SeqCst
);
impl_reflect_for_atomic!(
    ::core::sync::atomic::AtomicU8,
    ::core::sync::atomic::Ordering::SeqCst
);
impl_reflect_for_atomic!(
    ::core::sync::atomic::AtomicBool,
    ::core::sync::atomic::Ordering::SeqCst
);
impl_type_path!(::core::hash::BuildHasherDefault<H>);

impl_type_path!(::bevy_platform::hash::NoOpHash);
impl_type_path!(::bevy_platform::hash::FixedHasher);
impl_reflect_opaque!(::core::net::SocketAddr(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Serialize,
    Deserialize
));

impl<T: Reflect + MaybeTyped + TypePath + GetTypeRegistration, const N: usize> Array for [T; N] {
    #[inline]
    fn get(&self, index: usize) -> Option<&dyn PartialReflect> {
        <[T]>::get(self, index).map(|value| value as &dyn PartialReflect)
    }

    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect> {
        <[T]>::get_mut(self, index).map(|value| value as &mut dyn PartialReflect)
    }

    #[inline]
    fn len(&self) -> usize {
        N
    }

    #[inline]
    fn iter(&self) -> ArrayIter {
        ArrayIter::new(self)
    }

    #[inline]
    fn drain(self: Box<Self>) -> Vec<Box<dyn PartialReflect>> {
        self.into_iter()
            .map(|value| Box::new(value) as Box<dyn PartialReflect>)
            .collect()
    }
}

impl<T: Reflect + MaybeTyped + TypePath + GetTypeRegistration, const N: usize> PartialReflect
    for [T; N]
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

    #[inline]
    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::Array
    }

    #[inline]
    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Array(self)
    }

    #[inline]
    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Array(self)
    }

    #[inline]
    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Array(self)
    }

    #[inline]
    fn reflect_hash(&self) -> Option<u64> {
        crate::array_hash(self)
    }

    #[inline]
    fn reflect_partial_eq(&self, value: &dyn PartialReflect) -> Option<bool> {
        crate::array_partial_eq(self, value)
    }

    fn apply(&mut self, value: &dyn PartialReflect) {
        crate::array_apply(self, value);
    }

    #[inline]
    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
        crate::array_try_apply(self, value)
    }
}

impl<T: Reflect + MaybeTyped + TypePath + GetTypeRegistration, const N: usize> Reflect for [T; N] {
    #[inline]
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
        self
    }

    #[inline]
    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    #[inline]
    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }

    #[inline]
    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        *self = value.take()?;
        Ok(())
    }
}

impl<T: FromReflect + MaybeTyped + TypePath + GetTypeRegistration, const N: usize> FromReflect
    for [T; N]
{
    fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
        let ref_array = reflect.reflect_ref().as_array().ok()?;

        let mut temp_vec = Vec::with_capacity(ref_array.len());

        for field in ref_array.iter() {
            temp_vec.push(T::from_reflect(field)?);
        }

        temp_vec.try_into().ok()
    }
}

impl<T: Reflect + MaybeTyped + TypePath + GetTypeRegistration, const N: usize> Typed for [T; N] {
    fn type_info() -> &'static TypeInfo {
        static CELL: GenericTypeInfoCell = GenericTypeInfoCell::new();
        CELL.get_or_insert::<Self, _>(|| TypeInfo::Array(ArrayInfo::new::<Self, T>(N)))
    }
}

impl<T: TypePath, const N: usize> TypePath for [T; N] {
    fn type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| format!("[{t}; {N}]", t = T::type_path()))
    }

    fn short_type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| format!("[{t}; {N}]", t = T::short_type_path()))
    }
}

impl<T: Reflect + MaybeTyped + TypePath + GetTypeRegistration, const N: usize> GetTypeRegistration
    for [T; N]
{
    fn get_type_registration() -> TypeRegistration {
        TypeRegistration::of::<[T; N]>()
    }

    fn register_type_dependencies(registry: &mut TypeRegistry) {
        registry.register::<T>();
    }
}

#[cfg(feature = "functions")]
crate::func::macros::impl_function_traits!([T; N]; <T: Reflect + MaybeTyped + TypePath + GetTypeRegistration> [const N: usize]);

impl_reflect! {
    #[type_path = "core::option"]
    enum Option<T> {
        None,
        Some(T),
    }
}

impl_reflect! {
    #[type_path = "core::result"]
    enum Result<T, E> {
        Ok(T),
        Err(E),
    }
}

impl<T: TypePath + ?Sized> TypePath for &'static T {
    fn type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| format!("&{}", T::type_path()))
    }

    fn short_type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| format!("&{}", T::short_type_path()))
    }
}

impl<T: TypePath + ?Sized> TypePath for &'static mut T {
    fn type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| format!("&mut {}", T::type_path()))
    }

    fn short_type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| format!("&mut {}", T::short_type_path()))
    }
}

impl PartialReflect for &'static str {
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
        Ok(Box::new(*self))
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

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }

    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
        if let Some(value) = value.try_downcast_ref::<Self>() {
            self.clone_from(value);
        } else {
            return Err(ApplyError::MismatchedTypes {
                from_type: value.reflect_type_path().into(),
                to_type: Self::type_path().into(),
            });
        }
        Ok(())
    }
}

impl Reflect for &'static str {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }

    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        *self = value.take()?;
        Ok(())
    }
}

impl Typed for &'static str {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_set(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl GetTypeRegistration for &'static str {
    fn get_type_registration() -> TypeRegistration {
        let mut registration = TypeRegistration::of::<Self>();
        registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
        registration.insert::<ReflectFromReflect>(FromType::<Self>::from_type());
        registration
    }
}

impl FromReflect for &'static str {
    fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
        reflect.try_downcast_ref::<Self>().copied()
    }
}

#[cfg(feature = "functions")]
crate::func::macros::impl_function_traits!(&'static str);

impl TypePath for &'static Location<'static> {
    fn type_path() -> &'static str {
        "core::panic::Location"
    }

    fn short_type_path() -> &'static str {
        "Location"
    }
}

impl PartialReflect for &'static Location<'static> {
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
        Ok(Box::new(*self))
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

    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
        if let Some(value) = value.try_downcast_ref::<Self>() {
            self.clone_from(value);
            Ok(())
        } else {
            Err(ApplyError::MismatchedTypes {
                from_type: value.reflect_type_path().into(),
                to_type: <Self as DynamicTypePath>::reflect_type_path(self).into(),
            })
        }
    }
}

impl Reflect for &'static Location<'static> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }

    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        *self = value.take()?;
        Ok(())
    }
}

impl Typed for &'static Location<'static> {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_set(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl GetTypeRegistration for &'static Location<'static> {
    fn get_type_registration() -> TypeRegistration {
        let mut registration = TypeRegistration::of::<Self>();
        registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
        registration.insert::<ReflectFromReflect>(FromType::<Self>::from_type());
        registration
    }
}

impl FromReflect for &'static Location<'static> {
    fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
        reflect.try_downcast_ref::<Self>().copied()
    }
}

#[cfg(feature = "functions")]
crate::func::macros::impl_function_traits!(&'static Location<'static>);

#[cfg(test)]
mod tests {
    use crate::{
        Enum, FromReflect, PartialReflect, Reflect, ReflectSerialize, TypeInfo, TypeRegistry,
        Typed, VariantInfo, VariantType,
    };
    use bevy_platform::time::Instant;
    use core::{
        f32::consts::{PI, TAU},
        time::Duration,
    };
    use static_assertions::assert_impl_all;

    #[test]
    fn can_serialize_duration() {
        let mut type_registry = TypeRegistry::default();
        type_registry.register::<Duration>();

        let reflect_serialize = type_registry
            .get_type_data::<ReflectSerialize>(core::any::TypeId::of::<Duration>())
            .unwrap();
        let _serializable = reflect_serialize.get_serializable(&Duration::ZERO);
    }

    #[test]
    fn should_partial_eq_char() {
        let a: &dyn PartialReflect = &'x';
        let b: &dyn PartialReflect = &'x';
        let c: &dyn PartialReflect = &'o';
        assert!(a.reflect_partial_eq(b).unwrap_or_default());
        assert!(!a.reflect_partial_eq(c).unwrap_or_default());
    }

    #[test]
    fn should_partial_eq_i32() {
        let a: &dyn PartialReflect = &123_i32;
        let b: &dyn PartialReflect = &123_i32;
        let c: &dyn PartialReflect = &321_i32;
        assert!(a.reflect_partial_eq(b).unwrap_or_default());
        assert!(!a.reflect_partial_eq(c).unwrap_or_default());
    }

    #[test]
    fn should_partial_eq_f32() {
        let a: &dyn PartialReflect = &PI;
        let b: &dyn PartialReflect = &PI;
        let c: &dyn PartialReflect = &TAU;
        assert!(a.reflect_partial_eq(b).unwrap_or_default());
        assert!(!a.reflect_partial_eq(c).unwrap_or_default());
    }

    #[test]
    fn should_partial_eq_option() {
        let a: &dyn PartialReflect = &Some(123);
        let b: &dyn PartialReflect = &Some(123);
        assert_eq!(Some(true), a.reflect_partial_eq(b));
    }

    #[test]
    fn option_should_impl_enum() {
        assert_impl_all!(Option<()>: Enum);

        let mut value = Some(123usize);

        assert!(value
            .reflect_partial_eq(&Some(123usize))
            .unwrap_or_default());
        assert!(!value
            .reflect_partial_eq(&Some(321usize))
            .unwrap_or_default());

        assert_eq!("Some", value.variant_name());
        assert_eq!("core::option::Option<usize>::Some", value.variant_path());

        if value.is_variant(VariantType::Tuple) {
            if let Some(field) = value
                .field_at_mut(0)
                .and_then(|field| field.try_downcast_mut::<usize>())
            {
                *field = 321;
            }
        } else {
            panic!("expected `VariantType::Tuple`");
        }

        assert_eq!(Some(321), value);
    }

    #[test]
    fn option_should_from_reflect() {
        #[derive(Reflect, PartialEq, Debug)]
        struct Foo(usize);

        let expected = Some(Foo(123));
        let output = <Option<Foo> as FromReflect>::from_reflect(&expected).unwrap();

        assert_eq!(expected, output);
    }

    #[test]
    fn option_should_apply() {
        #[derive(Reflect, PartialEq, Debug)]
        struct Foo(usize);

        // === None on None === //
        let patch = None::<Foo>;
        let mut value = None::<Foo>;
        PartialReflect::apply(&mut value, &patch);

        assert_eq!(patch, value, "None apply onto None");

        // === Some on None === //
        let patch = Some(Foo(123));
        let mut value = None::<Foo>;
        PartialReflect::apply(&mut value, &patch);

        assert_eq!(patch, value, "Some apply onto None");

        // === None on Some === //
        let patch = None::<Foo>;
        let mut value = Some(Foo(321));
        PartialReflect::apply(&mut value, &patch);

        assert_eq!(patch, value, "None apply onto Some");

        // === Some on Some === //
        let patch = Some(Foo(123));
        let mut value = Some(Foo(321));
        PartialReflect::apply(&mut value, &patch);

        assert_eq!(patch, value, "Some apply onto Some");
    }

    #[test]
    fn option_should_impl_typed() {
        assert_impl_all!(Option<()>: Typed);

        type MyOption = Option<i32>;
        let info = MyOption::type_info();
        if let TypeInfo::Enum(info) = info {
            assert_eq!(
                "None",
                info.variant_at(0).unwrap().name(),
                "Expected `None` to be variant at index `0`"
            );
            assert_eq!(
                "Some",
                info.variant_at(1).unwrap().name(),
                "Expected `Some` to be variant at index `1`"
            );
            assert_eq!("Some", info.variant("Some").unwrap().name());
            if let VariantInfo::Tuple(variant) = info.variant("Some").unwrap() {
                assert!(
                    variant.field_at(0).unwrap().is::<i32>(),
                    "Expected `Some` variant to contain `i32`"
                );
                assert!(
                    variant.field_at(1).is_none(),
                    "Expected `Some` variant to only contain 1 field"
                );
            } else {
                panic!("Expected `VariantInfo::Tuple`");
            }
        } else {
            panic!("Expected `TypeInfo::Enum`");
        }
    }

    #[test]
    fn nonzero_usize_impl_reflect_from_reflect() {
        let a: &dyn PartialReflect = &core::num::NonZero::<usize>::new(42).unwrap();
        let b: &dyn PartialReflect = &core::num::NonZero::<usize>::new(42).unwrap();
        assert!(a.reflect_partial_eq(b).unwrap_or_default());
        let forty_two: core::num::NonZero<usize> = FromReflect::from_reflect(a).unwrap();
        assert_eq!(forty_two, core::num::NonZero::<usize>::new(42).unwrap());
    }

    #[test]
    fn instant_should_from_reflect() {
        let expected = Instant::now();
        let output = <Instant as FromReflect>::from_reflect(&expected).unwrap();
        assert_eq!(expected, output);
    }

    #[test]
    fn type_id_should_from_reflect() {
        let type_id = core::any::TypeId::of::<usize>();
        let output = <core::any::TypeId as FromReflect>::from_reflect(&type_id).unwrap();
        assert_eq!(type_id, output);
    }

    #[test]
    fn static_str_should_from_reflect() {
        let expected = "Hello, World!";
        let output = <&'static str as FromReflect>::from_reflect(&expected).unwrap();
        assert_eq!(expected, output);
    }
}
