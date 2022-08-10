use crate::{DynMetadata, Pointee};
use core::any::Any;

impl Pointee for dyn Any {
    type Metadata = DynMetadata<dyn Any>;
}
