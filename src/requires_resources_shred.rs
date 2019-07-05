
use crate::async_dispatcher::RequiresResources;
use crate::async_dispatcher::ResourceId;

impl<'a, T : legion::EntityData> RequiresResources for shred::ReadExpect<'a, T> {
    fn reads() -> Vec<ResourceId>{ vec![ResourceId::new::<T>()] }
    fn writes() -> Vec<ResourceId> { vec![] }
}

impl<'a, T : legion::EntityData> RequiresResources for shred::WriteExpect<'a, T> {
    fn reads() -> Vec<ResourceId>{ vec![] }
    fn writes() -> Vec<ResourceId>{ vec![ResourceId::new::<T>()] }
}

