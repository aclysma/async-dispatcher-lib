#[macro_use]
extern crate log;

pub mod async_dispatcher;

#[cfg(feature = "minimum")]
pub mod minimum;

pub use async_dispatcher::support;
pub use async_dispatcher::AcquireResources;
pub use async_dispatcher::AcquiredResourcesLockGuards;
pub use async_dispatcher::Dispatcher;
pub use async_dispatcher::DispatcherBuilder;
pub use async_dispatcher::ExecuteParallel;
pub use async_dispatcher::ExecuteSequential;
pub use async_dispatcher::RequiredResources;
pub use async_dispatcher::RequiresResources;
pub use async_dispatcher::ResourceId;
