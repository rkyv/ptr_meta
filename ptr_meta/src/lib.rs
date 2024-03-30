//! A radioactive stabilization of the [`ptr_meta` RFC][rfc].
//!
//! [rfc]: https://rust-lang.github.io/rfcs/2580-ptr-meta.html
//!
//! ## Usage
//!
//! ### Sized types
//!
//! All `Sized` types have `Pointee` implemented for them with a blanket
//! implementation. You do not need to derive `Pointee` for these types.
//!
//! ### `slice`s and `str`s
//!
//! These core types have implementations provided.
//!
//! ### `CStr` and `OsStr`
//!
//! These std types have implementations provided when the `std` feature is
//! enabled.
//!
//! ### `dyn Any` and `dyn Error`
//!
//! These trait objects have implementations provided.
//!
//! ### Structs with a DST as its last field
//!
//! You can derive `Pointee` for structs with a trailing DST:
//!
//! ```
//! use ptr_meta::Pointee;
//!
//! #[derive(Pointee)]
//! struct Block<H, T> {
//!     header: H,
//!     elements: [T],
//! }
//! ```
//!
//! Note that this will only work when the last field is guaranteed to be a DST.
//! Structs with a generic last field may have a conflicting blanket impl since
//! the generic type may be `Sized`. In these cases, a collection of specific
//! implementations may be required with the generic parameter set to a slice,
//! `str`, or specific trait object.
//!
//! ### Trait objects
//!
//! You can generate a `Pointee` implementation for trait objects:
//!
//! ```
//! use ptr_meta::pointee;
//!
//! // Generates Pointee for dyn Stringy
//! #[pointee]
//! trait Stringy {
//!     fn as_string(&self) -> String;
//! }
//! ```

#![deny(
    future_incompatible,
    missing_docs,
    nonstandard_style,
    unsafe_op_in_unsafe_fn,
    unused,
    warnings,
    clippy::all,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(miri, allow(internal_features), feature(core_intrinsics))]

mod impls;

use core::{
    fmt,
    hash::{Hash, Hasher},
    ptr,
};
pub use ptr_meta_derive::{pointee, Pointee};

mod sealed {
    pub trait Sealed {}
}

/// Provides the pointer metadata type of any pointed-to type.
///
/// # Pointer metadata
///
/// Raw pointer types and reference types in Rust can be thought of as made of
/// two parts: a data pointer that contains the memory address of the value, and
/// some metadata.
///
/// For statically-sized types (that implement the `Sized` traits)
/// as well as for `extern` types,
/// pointers are said to be “thin”: metadata is zero-sized and its type is `()`.
///
/// Pointers to [dynamically-sized types][dst] are said to be “wide” or “fat”,
/// they have non-zero-sized metadata:
///
/// * For structs whose last field is a DST, metadata is the metadata for the
///   last field
/// * For the `str` type, metadata is the length in bytes as `usize`
/// * For slice types like `[T]`, metadata is the length in items as `usize`
/// * For trait objects like `dyn SomeTrait`, metadata is
///   [`DynMetadata<Self>`][DynMetadata] (e.g. `DynMetadata<dyn SomeTrait>`)
///
/// In the future, the Rust language may gain new kinds of types
/// that have different pointer metadata.
///
/// [dst]: https://doc.rust-lang.org/nomicon/exotic-sizes.html#dynamically-sized-types-dsts
///
///
/// # The `Pointee` trait
///
/// The point of this trait is its `Metadata` associated type,
/// which is `()` or `usize` or `DynMetadata<_>` as described above.
/// It is automatically implemented for every type.
/// It can be assumed to be implemented in a generic context, even without a
/// corresponding bound.
///
///
/// # Usage
///
/// Raw pointers can be decomposed into the data address and metadata components
/// with their [`to_raw_parts`] method.
///
/// Alternatively, metadata alone can be extracted with the [`metadata`]
/// function. A reference can be passed to [`metadata`] and implicitly coerced.
///
/// A (possibly-wide) pointer can be put back together from its address and
/// metadata with [`from_raw_parts`] or [`from_raw_parts_mut`].
///
/// [`to_raw_parts`]: PtrExt::to_raw_parts
///
/// # Safety
///
/// The associated `Metadata` type must be the pointer metadata type for the
/// implementing type.
pub unsafe trait Pointee {
    /// The type for metadata in pointers and references to `Self`.
    type Metadata: Copy + Send + Sync + Ord + Hash + Unpin;
}

// SAFETY: Pointers to `Sized` types have no metadata (i.e. their metadata is
// the unit type `()`).
unsafe impl<T> Pointee for T {
    type Metadata = ();
}

// SAFETY: Pointers to slices have a `usize` representing the length of the
// slice in elements as their metadata.
unsafe impl<T> Pointee for [T] {
    type Metadata = usize;
}

// SAFETY: String slice pointers have a `usize` representing the length of the
// string slice in bytes as their metadata.
unsafe impl Pointee for str {
    type Metadata = usize;
}

#[cfg(feature = "std")]
// SAFETY: `CStr` pointers have a `usize` representing the length of the
// C-string slice in bytes (nul included) as their metadata.
unsafe impl Pointee for ::std::ffi::CStr {
    type Metadata = usize;
}

#[cfg(feature = "std")]
// SAFETY: `OsStr` pointers have a `usize` representing the length of the
// string in bytes as their metadata.
unsafe impl Pointee for ::std::ffi::OsStr {
    type Metadata = usize;
}

/// Extract the metadata component of a pointer.
///
/// Values of type `*mut T`, `&T`, or `&mut T` can be passed directly to this
/// function as they implicitly coerce to `*const T`.
///
/// # Example
///
/// ```
/// use ptr_meta::metadata;
///
/// assert_eq!(metadata("foo"), 3_usize);
/// ```
#[inline]
pub const fn metadata<T: Pointee + ?Sized>(
    ptr: *const T,
) -> <T as Pointee>::Metadata {
    // SAFETY: Accessing the value from the `PtrRepr` union is safe since
    // *const T and PtrComponents<T> have the same memory layouts. Only std can
    // make this guarantee.
    unsafe { PtrRepr { const_ptr: ptr }.components.metadata }
}

/// Forms a (possibly-wide) raw pointer from a data address and metadata.
///
/// This function is safe but the returned pointer is not necessarily safe to
/// dereference. For slices, see the documentation of [`slice::from_raw_parts`]
/// for safety requirements. For trait objects, the metadata must come from a
/// pointer to the same underlying erased type.
///
/// [`slice::from_raw_parts`]: core::slice::from_raw_parts
#[inline]
pub const fn from_raw_parts<T: Pointee + ?Sized>(
    data_address: *const (),
    metadata: <T as Pointee>::Metadata,
) -> *const T {
    // SAFETY: Accessing the value from the `PtrRepr` union is safe since
    // *const T and PtrComponents<T> have the same memory layouts. Only std can
    // make this guarantee.
    unsafe {
        PtrRepr {
            components: PtrComponents {
                data_address,
                metadata,
            },
        }
        .const_ptr
    }
}

/// Performs the same functionality as [`from_raw_parts`], except that a
/// raw `*mut` pointer is returned, as opposed to a raw `*const` pointer.
///
/// See the documentation of [`from_raw_parts`] for more details.
#[inline]
pub const fn from_raw_parts_mut<T: Pointee + ?Sized>(
    data_address: *mut (),
    metadata: <T as Pointee>::Metadata,
) -> *mut T {
    // SAFETY: Accessing the value from the `PtrRepr` union is safe since
    // *const T and PtrComponents<T> have the same memory layouts. Only std can
    // make this guarantee.
    unsafe {
        PtrRepr {
            components: PtrComponents {
                data_address,
                metadata,
            },
        }
        .mut_ptr
    }
}

/// Extension methods for pointers.
pub trait PtrExt<T: Pointee + ?Sized>: sealed::Sealed {
    /// The type's raw pointer (`*const ()` or `*mut ()`).
    type Raw;

    /// Decompose a (possibly wide) pointer into its address and metadata
    /// components.
    ///
    /// The pointer can be later reconstructed with [`from_raw_parts`].
    fn to_raw_parts(self) -> (Self::Raw, <T as Pointee>::Metadata);
}

impl<T: Pointee + ?Sized> sealed::Sealed for *const T {}

impl<T: Pointee + ?Sized> PtrExt<T> for *const T {
    type Raw = *const ();

    fn to_raw_parts(self) -> (Self::Raw, <T as Pointee>::Metadata) {
        (self as Self::Raw, metadata(self))
    }
}

impl<T: Pointee + ?Sized> sealed::Sealed for *mut T {}

impl<T: Pointee + ?Sized> PtrExt<T> for *mut T {
    type Raw = *mut ();

    fn to_raw_parts(self) -> (Self::Raw, <T as Pointee>::Metadata) {
        (self as Self::Raw, metadata(self))
    }
}

/// Extension methods for [`NonNull`](core::ptr::NonNull).
pub trait NonNullExt<T: Pointee + ?Sized>: PtrExt<T> {
    /// Creates a new non-null pointer from its raw parts.
    fn from_raw_parts(
        raw: ptr::NonNull<()>,
        meta: <T as Pointee>::Metadata,
    ) -> Self;
}

impl<T: Pointee + ?Sized> sealed::Sealed for ptr::NonNull<T> {}

impl<T: Pointee + ?Sized> PtrExt<T> for ptr::NonNull<T> {
    type Raw = ptr::NonNull<()>;

    fn to_raw_parts(self) -> (Self::Raw, <T as Pointee>::Metadata) {
        let (data_address, metadata) = PtrExt::to_raw_parts(self.as_ptr());
        // SAFETY: `self` is non-null, and so the data pointer returned from
        // `to_raw_parts` is also non-null.
        unsafe { (ptr::NonNull::new_unchecked(data_address), metadata) }
    }
}

impl<T: Pointee + ?Sized> NonNullExt<T> for ptr::NonNull<T> {
    fn from_raw_parts(
        raw: ptr::NonNull<()>,
        meta: <T as Pointee>::Metadata,
    ) -> Self {
        // SAFETY: `raw` is non-null, and so the data pointer returned from
        // `from_raw_parts_mut` is also non-null.
        unsafe { Self::new_unchecked(from_raw_parts_mut(raw.as_ptr(), meta)) }
    }
}

#[repr(C)]
union PtrRepr<T: Pointee + ?Sized> {
    const_ptr: *const T,
    mut_ptr: *mut T,
    components: PtrComponents<T>,
}

#[repr(C)]
struct PtrComponents<T: Pointee + ?Sized> {
    data_address: *const (),
    metadata: <T as Pointee>::Metadata,
}

// Manual impl needed to avoid `T: Copy` bound.
impl<T: Pointee + ?Sized> Copy for PtrComponents<T> {}

// Manual impl needed to avoid `T: Clone` bound.
impl<T: Pointee + ?Sized> Clone for PtrComponents<T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// The metadata for a `Dyn = dyn SomeTrait` trait object type.
///
/// It is a pointer to a vtable (virtual call table)
/// that represents all the necessary information
/// to manipulate the concrete type stored inside a trait object.
/// The vtable notably it contains:
///
/// * type size
/// * type alignment
/// * a pointer to the type’s `drop_in_place` impl (may be a no-op for
///   plain-old-data)
/// * pointers to all the methods for the type’s implementation of the trait
///
/// Note that the first three are special because they’re necessary to allocate,
/// drop, and deallocate any trait object.
///
/// It is possible to name this struct with a type parameter that is not a `dyn`
/// trait object (for example `DynMetadata<u64>`) but not to obtain a meaningful
/// value of that struct.
pub struct DynMetadata<Dyn: ?Sized> {
    vtable_ptr: &'static VTable,
    phantom: core::marker::PhantomData<Dyn>,
}

// Extern types are not stable, so we substitute a ZST. This is not a perfect
// substitute but since it's not exposed anywhere, it's close enough.
struct VTable;

impl<Dyn: ?Sized> DynMetadata<Dyn> {
    /// Returns the size of the type associated with this vtable.
    #[inline]
    pub fn size_of(self) -> usize {
        #[cfg(miri)]
        {
            // Note that "size stored in vtable" is *not* the same as "result of size_of_val_raw".
            // Consider a reference like `&(i32, dyn Send)`: the vtable will only store the size of the
            // `Send` part!
            // SAFETY: DynMetadata always contains a valid vtable pointer
            return unsafe {
                core::intrinsics::vtable_size(
                    self.vtable_ptr as *const VTable as *const (),
                )
            };
        }
        #[cfg(not(miri))]
        {
            // SAFETY: This happens to be true. It may not always be true. The
            // location of the size for vtables is based on the implementation
            // of the vtable_size intrinsic.
            unsafe {
                (self.vtable_ptr as *const VTable as *const usize)
                    .add(1)
                    .read()
            }
        }
    }

    /// Returns the alignment of the type associated with this vtable.
    #[inline]
    pub fn align_of(self) -> usize {
        #[cfg(miri)]
        {
            // SAFETY: DynMetadata always contains a valid vtable pointer
            return unsafe {
                core::intrinsics::vtable_align(
                    self.vtable_ptr as *const VTable as *const (),
                )
            };
        }
        #[cfg(not(miri))]
        {
            // SAFETY: This happens to be true. It may not always be true. The
            // location of the alignment for vtables is based on the
            // implementation of the vtable_align intrinsic.
            unsafe {
                (self.vtable_ptr as *const VTable as *const usize)
                    .add(2)
                    .read()
            }
        }
    }

    /// Returns the size and alignment together as a `Layout`
    #[inline]
    pub fn layout(self) -> core::alloc::Layout {
        // SAFETY: the compiler emitted this vtable for a concrete Rust type which
        // is known to have a valid layout. Same rationale as in `Layout::for_value`.
        unsafe {
            core::alloc::Layout::from_size_align_unchecked(
                self.size_of(),
                self.align_of(),
            )
        }
    }
}

// SAFETY: References to trait object vtables are guaranteed to be `Send`.
unsafe impl<Dyn: ?Sized> Send for DynMetadata<Dyn> {}
// SAFETY: References to trait object vtables are guaranteed to be `Sync`.
unsafe impl<Dyn: ?Sized> Sync for DynMetadata<Dyn> {}

impl<Dyn: ?Sized> fmt::Debug for DynMetadata<Dyn> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DynMetadata")
            .field(&(self.vtable_ptr as *const VTable))
            .finish()
    }
}

// Manual impls needed to avoid `Dyn: $Trait` bounds.

impl<Dyn: ?Sized> Unpin for DynMetadata<Dyn> {}

impl<Dyn: ?Sized> Copy for DynMetadata<Dyn> {}

impl<Dyn: ?Sized> Clone for DynMetadata<Dyn> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Dyn: ?Sized> Eq for DynMetadata<Dyn> {}

impl<Dyn: ?Sized> PartialEq for DynMetadata<Dyn> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq::<VTable>(self.vtable_ptr, other.vtable_ptr)
    }
}

impl<Dyn: ?Sized> Ord for DynMetadata<Dyn> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (self.vtable_ptr as *const VTable)
            .cmp(&(other.vtable_ptr as *const VTable))
    }
}

impl<Dyn: ?Sized> PartialOrd for DynMetadata<Dyn> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Dyn: ?Sized> Hash for DynMetadata<Dyn> {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        core::ptr::hash::<VTable, _>(self.vtable_ptr, hasher)
    }
}

#[cfg(test)]
mod tests {
    use super::{from_raw_parts, pointee, Pointee, PtrExt};
    use crate as ptr_meta;

    fn test_pointee<T: Pointee + ?Sized>(value: &T) {
        let ptr = value as *const T;
        let (raw, meta) = PtrExt::to_raw_parts(ptr);
        let re_ptr = from_raw_parts::<T>(raw, meta);
        assert_eq!(ptr, re_ptr);
    }

    #[test]
    fn sized_types() {
        test_pointee(&());
        test_pointee(&42);
        test_pointee(&true);
        test_pointee(&[1, 2, 3, 4]);

        struct TestUnit;

        test_pointee(&TestUnit);

        #[allow(dead_code)]
        struct TestStruct {
            a: (),
            b: i32,
            c: bool,
        }

        test_pointee(&TestStruct {
            a: (),
            b: 42,
            c: true,
        });

        #[allow(dead_code)]
        struct TestTuple((), i32, bool);

        test_pointee(&TestTuple((), 42, true));

        struct TestGeneric<T>(T);

        test_pointee(&TestGeneric(42));
    }

    #[test]
    fn unsized_types() {
        test_pointee("hello world");
        test_pointee(&[1, 2, 3, 4] as &[i32]);
    }

    #[test]
    fn trait_objects() {
        #[pointee]
        trait TestTrait {
            #[allow(dead_code)]
            fn foo(&self);
        }

        struct A;

        impl TestTrait for A {
            fn foo(&self) {}
        }

        let trait_object = &A as &dyn TestTrait;

        test_pointee(trait_object);

        #[allow(dead_code)]
        struct B(i32);

        impl TestTrait for B {
            fn foo(&self) {}
        }

        let b = B(42);
        let trait_object = &b as &dyn TestTrait;

        test_pointee(trait_object);
    }

    #[test]
    fn last_field_dst() {
        #[allow(dead_code)]
        #[derive(Pointee)]
        struct Test<H, T> {
            head: H,
            tail: [T],
        }

        #[allow(dead_code)]
        #[derive(Pointee)]
        struct TestDyn {
            tail: dyn core::any::Any,
        }

        #[pointee]
        trait TestTrait {}

        #[allow(dead_code)]
        #[derive(Pointee)]
        struct TestCustomDyn {
            tail: dyn TestTrait,
        }
    }

    #[test]
    fn generic_trait() {
        #[allow(dead_code)]
        #[pointee]
        trait TestTrait<T: ?Sized> {}

        impl<T: ?Sized> TestTrait<T> for () {}

        test_pointee(&() as &dyn TestTrait<u32>);
    }
}
