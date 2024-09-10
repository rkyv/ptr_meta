# 0.3.0

- Implement `Pointee` for `dyn core::error::Error`
- Add impls for trait objects plus `Send` and `Sync`
- Replace `PtrExt` and `NonNullExt` with free functions
- Fix `DynMetadata` implementation under MIRI
- Add `crate = ..` derive argument
- Update syn to 2
- Set MSRV to 1.81
