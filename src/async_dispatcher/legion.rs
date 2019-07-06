
use crate::async_dispatcher::RequiresResources;
use crate::async_dispatcher::ResourceId;

impl<T : legion::EntityData> RequiresResources for legion::query::Read<T> {
    fn reads() -> Vec<ResourceId>{ vec![ResourceId::new::<T>()] }
    fn writes() -> Vec<ResourceId> { vec![] }
}

impl<T : legion::EntityData> RequiresResources for legion::query::Write<T> {
    fn reads() -> Vec<ResourceId>{
        vec![]
    }
    fn writes() -> Vec<ResourceId>{ vec![ResourceId::new::<T>()] }
}
