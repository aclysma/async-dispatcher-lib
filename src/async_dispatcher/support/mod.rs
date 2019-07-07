
// This module holds glue-code to ECS systems we integrate with

#[cfg(feature = "minimum")]
pub mod minimum;

#[cfg(feature = "legion")]
pub mod legion;

#[cfg(feature = "shred")]
pub mod shred;