
use std::sync::Arc;

use crate::async_dispatcher::{
    RequiresResources,
    ResourceId,
    Dispatcher,
    AcquiredResourcesLockGuards,
    AcquireResources
};

use legion::query::IntoQuery;

impl<T : legion::Component> RequiresResources for legion::query::Read<T> {
    fn reads() -> Vec<ResourceId>{ vec![ResourceId::new::<T>()] }
    fn writes() -> Vec<ResourceId> { vec![] }
}

impl<T : legion::Component> RequiresResources for legion::query::Write<T> {
    fn reads() -> Vec<ResourceId>{
        vec![]
    }
    fn writes() -> Vec<ResourceId>{ vec![ResourceId::new::<T>()] }
}

pub struct AsyncQuery<T>
    where T : legion::query::DefaultFilter + for<'a> legion::query::View<'a>
{
    lock_guards: AcquiredResourcesLockGuards<T>,
    query: legion::query::QueryDef<T, <T as legion::query::DefaultFilter>::Filter>
}

impl<T> AsyncQuery<T>
    where T : legion::query::DefaultFilter + for<'a> legion::query::View<'a>
{
    pub fn query(&self) -> &legion::query::QueryDef<T, <T as legion::query::DefaultFilter>::Filter> {
        &self.query
    }

    pub fn query_mut(&mut self) -> &mut legion::query::QueryDef<T, <T as legion::query::DefaultFilter>::Filter> {
        &mut self.query
    }
}

pub fn create_query<T>(dispatcher: Arc<Dispatcher>) -> impl futures::future::Future<Item=AsyncQuery<T>, Error=()>
    where
        T : RequiresResources,
        T : legion::query::DefaultFilter + for<'a> legion::query::View<'a>
{
    use crate::async_dispatcher::RequiresResources;

    let required_resources = <T as RequiresResources>::required_resources();
    let query = T::query();

    use futures::future::Future;

    //TODO: Start here, try to make this return an iterator as a future result
    AcquireResources::new(dispatcher, required_resources)
        .map(|lock_guards| {
            AsyncQuery {
                lock_guards,
                query
            }
        })
}