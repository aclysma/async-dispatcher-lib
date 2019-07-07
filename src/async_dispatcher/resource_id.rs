use std::any::TypeId;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResourceId {
    type_id: TypeId,
}

impl ResourceId {
    /// Creates a new resource id from a given type.
    #[inline]
    pub fn new<T: 'static>() -> Self {
        ResourceId {
            type_id: std::any::TypeId::of::<T>(),
        }
    }
}
