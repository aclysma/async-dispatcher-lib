use std::sync::Arc;

use crate::async_dispatcher::{
    AcquireResources, AcquiredResourcesLockGuards, Dispatcher, RequiresResources, DispatcherBuilder
};

use legion::query::IntoQuery;

use std::any::TypeId;

impl crate::async_dispatcher::ResourceIdTrait for TypeId {

}

impl<T: legion::Component> RequiresResources<TypeId> for legion::query::Read<T> {
    fn reads() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }
    fn writes() -> Vec<TypeId> {
        vec![]
    }
}

impl<T: legion::Component> RequiresResources<TypeId> for legion::query::Write<T> {
    fn reads() -> Vec<TypeId> {
        vec![]
    }
    fn writes() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }
}

pub struct AsyncQuery<T>
where
    T: legion::query::DefaultFilter + for<'a> legion::query::View<'a>,
{
    _lock_guards: AcquiredResourcesLockGuards<T>,
    query: legion::query::QueryDef<T, <T as legion::query::DefaultFilter>::Filter>,
}

impl<T> AsyncQuery<T>
where
    T: legion::query::DefaultFilter + for<'a> legion::query::View<'a>,
{
    pub fn query(
        &self,
    ) -> &legion::query::QueryDef<T, <T as legion::query::DefaultFilter>::Filter> {
        &self.query
    }

    pub fn query_mut(
        &mut self,
    ) -> &mut legion::query::QueryDef<T, <T as legion::query::DefaultFilter>::Filter> {
        &mut self.query
    }
}

pub fn create_query<T>(
    dispatcher: Arc<Dispatcher<TypeId>>,
) -> impl futures::future::Future<Item = AsyncQuery<T>, Error = ()>
where
    T: RequiresResources<TypeId>,
    T: legion::query::DefaultFilter + for<'a> legion::query::View<'a>,
{
    let required_resources = <T as RequiresResources<TypeId>>::required_resources();
    let query = T::query();

    use futures::future::Future;

    //TODO: Try to make this return an iterator as a future result
    AcquireResources::new(dispatcher, required_resources).map(|lock_guards| AsyncQuery {
        _lock_guards: lock_guards,
        query,
    })
}

pub struct LegionDispatcherBuilder {
    dispatcher_builder: DispatcherBuilder<TypeId>
}

impl LegionDispatcherBuilder {
    // Create an empty dispatcher builder
    pub fn new() -> Self {
        LegionDispatcherBuilder {
            dispatcher_builder: DispatcherBuilder::<TypeId>::new()
        }
    }

    pub fn with_resource_type<T : 'static>(mut self) -> Self {
        self.insert_resource_type::<T>();
        self
    }

    pub fn insert_resource_type<T : 'static>(&mut self) {
        self.dispatcher_builder.register_resource_id(TypeId::of::<T>());
    }

    // Create the dispatcher
    pub fn build(self) -> Dispatcher<TypeId> {
        self.dispatcher_builder.build()
    }
}