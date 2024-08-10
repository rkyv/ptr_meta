# ptr_meta &emsp; [![Latest Version]][crates.io] [![License]][license path]

[Latest Version]: https://img.shields.io/crates/v/ptr_meta.svg
[crates.io]: https://crates.io/crates/ptr_meta
[License]: https://img.shields.io/badge/license-MIT-blue.svg
[license path]: https://github.com/djkoloski/ptr_meta/blob/master/LICENSE

A radioactive stabilization of the [`ptr_meta` RFC][rfc].

[rfc]: https://rust-lang.github.io/rfcs/2580-ptr-meta.html

Along with the core `Pointee` trait, `PtrExt` extension trait, and
helper functions, `ptr_meta` also provides inherent implementations for a
common builtin types:

## Sized types

All `Sized` types have `Pointee` implemented for them with a blanket
implementation. You cannot write or derive `Pointee` implementations for these
types.

## `slice`s and `str`s

These core types have implementations provided.

## `CStr` and `OsStr`

These std types have implementations provided when the `std` feature is enabled.

## `dyn Any` (`+ Send`) (`+ Sync`)

`dyn Any`, optionally with `+ Send` and/or `+ Sync`, have implementations
provided.

## `dyn Error` (`+ Send`) (`+ Sync`)

`dyn Error`, optionally with `+ Send` and/or `+ Sync`, have implementations
provided when the `std` feature is enabled.

## Structs with trailing DSTs

You can derive `Pointee` for structs with trailing DSTs:

```rust
use ptr_meta::Pointee;

#[derive(Pointee)]
struct Block<H, T> {
    header: H,
    elements: [T],
}
```

Note that the last field is required to be a DST. Structs with a generic type as
the last field may have conflicting blanket implementations, as the generic type
may be `Sized`. A collection of specific implementations may be required in
these cases, with the generic parameter set (for example) a slice, `str`, or
specific trait object.

## Trait objects

You can generate `Pointee` implementations for trait objects:

```rust
use ptr_meta::pointee;

// Generates Pointee for dyn Stringy
#[ptr_meta::pointee]
trait Stringy {
    fn as_string(&self) -> String;
}
```

Note that this will not produce implementations for `Trait + Send + Sync`.
