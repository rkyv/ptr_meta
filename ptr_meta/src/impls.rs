use core::any::Any;

use crate::{DynMetadata, Pointee};

// SAFETY: The metadata type of `dyn Any` is `DynMetadata<dyn Any>`.
unsafe impl Pointee for dyn Any {
    type Metadata = DynMetadata<dyn Any>;
}

// SAFETY: The metadata type of `dyn Any + Send` is
// `DynMetadata<dyn Any + Send>`.
unsafe impl Pointee for dyn Any + Send {
    type Metadata = DynMetadata<dyn Any + Send>;
}

// SAFETY: The metadata type of `dyn Any + Sync` is
// `DynMetadata<dyn Any + Sync>`.
unsafe impl Pointee for dyn Any + Sync {
    type Metadata = DynMetadata<dyn Any + Sync>;
}

// SAFETY: The metadata type of `dyn Any + Send + Sync` is
// `DynMetadata<dyn Any + Send + Sync>`.
unsafe impl Pointee for dyn Any + Send + Sync {
    type Metadata = DynMetadata<dyn Any + Send + Sync>;
}

#[cfg(feature = "std")]
// SAFETY: The metadata type of `dyn Error` is `DynMetadata<dyn Error>`.
unsafe impl Pointee for dyn std::error::Error {
    type Metadata = DynMetadata<dyn std::error::Error>;
}

#[cfg(feature = "std")]
// SAFETY: The metadata type of `dyn Error + Send` is
// `DynMetadata<dyn Error + Send>`.
unsafe impl Pointee for dyn std::error::Error + Send {
    type Metadata = DynMetadata<dyn std::error::Error + Send>;
}

#[cfg(feature = "std")]
// SAFETY: The metadata type of `dyn Error + Sync` is
// `DynMetadata<dyn Error + Sync>`.
unsafe impl Pointee for dyn std::error::Error + Sync {
    type Metadata = DynMetadata<dyn std::error::Error + Sync>;
}

#[cfg(feature = "std")]
// SAFETY: The metadata type of `dyn Error + Send + Sync` is
// `DynMetadata<dyn Error + Send + Sync>`.
unsafe impl Pointee for dyn std::error::Error + Send + Sync {
    type Metadata = DynMetadata<dyn std::error::Error + Send + Sync>;
}
