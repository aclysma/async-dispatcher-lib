
use std::sync::Arc;

use crate::async_dispatcher::{
    RequiresResources,
    ResourceId,
    Dispatcher,
    AcquiredResourcesLockGuards,
    AcquireResources
};

use crate::minimum::{
    World,
    Read,
    Write,
    Resource,
    DataRequirement
};

impl<T : Resource> RequiresResources for Read<T> {
    fn reads() -> Vec<ResourceId> { vec![ResourceId::new::<T>()] }
    fn writes() -> Vec<ResourceId> { vec![] }
}

impl<T : Resource> RequiresResources for Write<T> {
    fn reads() -> Vec<ResourceId> { vec![] }
    fn writes() -> Vec<ResourceId> { vec![ResourceId::new::<T>()] }
}


pub struct AcquiredResources<T>
    where T : RequiresResources + 'static + Send
{
    lock_guards: AcquiredResourcesLockGuards<T>,
    world: Arc<World>
}

impl<T> AcquiredResources<T>
    where T : RequiresResources + 'static + Send {

    pub fn visit<'a, F>(&'a self, f : F)
        where
            F : FnOnce(T::Borrow),
            T : DataRequirement<'a>
    {
        let fetched = T::fetch(&self.world);
        (f)(fetched);
    }
}

pub fn acquire_resources<T>(dispatcher: Arc<Dispatcher>, world: Arc<World>) -> impl futures::future::Future<Item=AcquiredResources<T>, Error=()>
    where T : RequiresResources + 'static + Send
{
    use futures::future::Future;

    Box::new(AcquireResources::new(dispatcher, T::required_resources())
        .map(move |lock_guards| {
            AcquiredResources {
                lock_guards,
                world
            }
        }))
}