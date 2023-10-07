use crate::{DynMetadata, Pointee};
use core::any::Any;

// SAFETY: The metadata type of `dyn Any` is `DynMetadata<dyn Any>`.
unsafe impl Pointee for dyn Any {
    type Metadata = DynMetadata<dyn Any>;
}
