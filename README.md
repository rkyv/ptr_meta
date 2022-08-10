# ptr_meta &emsp; [![Latest Version]][crates.io] [![License]][license path] [![requires: rustc 1.47+]][Rust 1.47]

[Latest Version]: https://img.shields.io/crates/v/ptr_meta.svg
[crates.io]: https://crates.io/crates/ptr_meta
[License]: https://img.shields.io/badge/license-MIT-blue.svg
[license path]: https://github.com/djkoloski/ptr_meta/blob/master/LICENSE
[requires: rustc 1.47+]: https://img.shields.io/badge/rustc-1.47+-lightgray.svg
[Rust 1.47]: https://blog.rust-lang.org/2020/10/08/Rust-1.47.html

# ptr_meta

A radioactive stabilization of the [`ptr_meta` RFC][rfc].

[rfc]: https://rust-lang.github.io/rfcs/2580-ptr-meta.html

# Usage

## Sized types

All `Sized` types have `Pointee` implemented for them with a blanket implementation. You do not
need to derive `Pointee` for these types.

## `slice`s and `str`s

These core types have implementations built in.

# `dyn Any`

The trait object for this standard library type comes with an implementation built in.

## Structs with a DST as its last field

You can derive `Pointee` for structs with a trailing DST:

```rust
use ptr_meta::Pointee;

#[derive(Pointee)]
struct Block<H, T> {
    header: H,
    elements: [T],
}
```

Note that this will only work when the last field is guaranteed to be a DST. Structs with a
generic last field may have a conflicting blanket impl since the generic type may be `Sized`. In
these cases, a collection of specific implementations may be required with the generic parameter
set to a slice, `str`, or specific trait object.

## Trait objects

You can generate a `Pointee` implementation for trait objects:

```rust
use ptr_meta::pointee;

// Generates Pointee for dyn Stringy
#[pointee]
trait Stringy {
    fn as_string(&self) -> String;
}
```
