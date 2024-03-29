// This implements a type system for expressing read/write dependencies.
//
// Lots of inspiration taken from shred for how to create a type system
// to express read/write dependencies

use hashbrown::HashMap;
use mopa::Any;

use std::marker::PhantomData;

mod trust_cell;
use trust_cell::Ref;
use trust_cell::RefMut;
use trust_cell::TrustCell;

//
// ResourceId
//
use crate::async_dispatcher::RequiresResources;
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

//
// Resource
//
pub trait Resource: Any + Send + Sync + 'static {}

mod __resource_mopafy_scope {
    #![allow(clippy::all)]

    use mopa::mopafy;

    use super::Resource;

    mopafy!(Resource);
}

impl<T> Resource for T where T: Any + Send + Sync {}

pub struct WorldBuilder {
    world: World
}

impl WorldBuilder {
    pub fn new() -> Self {
        WorldBuilder {
            world: World::new()
        }
    }

    pub fn with_resource<R>(mut self, r: R) -> Self
        where
            R: Resource
    {
        self.world.insert(r);
        self
    }

    pub fn insert<R>(&mut self, r: R)
        where
            R: Resource
    {
        self.world.insert(r);
    }

    pub fn build(self) -> World {
        self.world
    }
}

//
// World
//
#[derive(Default)]
pub struct World {
    resources: HashMap<ResourceId, TrustCell<Box<dyn Resource>>>,
}

impl World {
    pub fn new() -> Self {
        World {
            resources: HashMap::new(),
        }
    }

    pub fn insert<R>(&mut self, r: R)
    where
        R: Resource,
    {
        self.insert_by_id(ResourceId::new::<R>(), r);
    }

    pub fn remove<R>(&mut self) -> Option<R>
    where
        R: Resource,
    {
        self.remove_by_id(ResourceId::new::<R>())
    }

    fn insert_by_id<R>(&mut self, id: ResourceId, r: R)
    where
        R: Resource,
    {
        self.resources.insert(id, TrustCell::new(Box::new(r)));
    }

    fn remove_by_id<R>(&mut self, id: ResourceId) -> Option<R>
    where
        R: Resource,
    {
        self.resources
            .remove(&id)
            .map(TrustCell::into_inner)
            .map(|x: Box<dyn Resource>| x.downcast())
            .map(|x: Result<Box<R>, _>| x.ok().unwrap())
            .map(|x| *x)
    }

    pub fn fetch<R: Resource>(&self) -> ReadBorrow<R> {
        self.try_fetch().unwrap()
    }

    pub fn try_fetch<R: Resource>(&self) -> Option<ReadBorrow<R>> {
        let res_id = ResourceId::new::<R>();

        self.resources.get(&res_id).map(|r| ReadBorrow {
            inner: Ref::map(r.borrow(), Box::as_ref),
            phantom: PhantomData,
        })
    }

    pub fn fetch_mut<R: Resource>(&self) -> WriteBorrow<R> {
        self.try_fetch_mut().unwrap()
    }

    pub fn try_fetch_mut<R: Resource>(&self) -> Option<WriteBorrow<R>> {
        let res_id = ResourceId::new::<R>();

        self.resources.get(&res_id).map(|r| WriteBorrow::<R> {
            inner: RefMut::map(r.borrow_mut(), Box::as_mut),
            phantom: PhantomData,
        })
    }

    pub fn has_value<R>(&self) -> bool
        where
            R: Resource,
    {
        self.has_value_raw(ResourceId::new::<R>())
    }

    pub fn has_value_raw(&self, id: ResourceId) -> bool {
        self.resources.contains_key(&id)
    }

    pub fn keys(&self) -> impl Iterator<Item=&ResourceId> {
        self.resources.iter().map(|x| x.0)
    }
}

//
// DataRequirement base trait
//
pub trait DataRequirement<'a> {
    type Borrow: DataBorrow;

    fn fetch(world: &'a World) -> Self::Borrow;
}

impl<'a> DataRequirement<'a> for () {
    type Borrow = ();

    fn fetch(_: &'a World) -> Self::Borrow {}
}

//
// Read
//
pub struct Read<T: Resource> {
    phantom_data: PhantomData<T>,
}

impl<'a, T: Resource> DataRequirement<'a> for Read<T> {
    type Borrow = ReadBorrow<'a, T>;

    fn fetch(world: &'a World) -> Self::Borrow {
        world.fetch::<T>()
    }
}

//
// Write
//
pub struct Write<T: Resource> {
    phantom_data: PhantomData<T>,
}

impl<'a, T: Resource> DataRequirement<'a> for Write<T> {
    type Borrow = WriteBorrow<'a, T>;

    fn fetch(world: &'a World) -> Self::Borrow {
        world.fetch_mut::<T>()
    }
}

//
// Borrow base trait
//
pub trait DataBorrow {}

impl DataBorrow for () {}

//
// ReadBorrow
//
pub struct ReadBorrow<'a, T> {
    inner: Ref<'a, dyn Resource>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> DataBorrow for ReadBorrow<'a, T> {}

impl<'a, T> std::ops::Deref for ReadBorrow<'a, T>
where
    T: Resource,
{
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.inner.downcast_ref_unchecked() }
    }
}

impl<'a, T> Clone for ReadBorrow<'a, T> {
    fn clone(&self) -> Self {
        ReadBorrow {
            inner: self.inner.clone(),
            phantom: PhantomData,
        }
    }
}

//
// WriteBorrow
//
pub struct WriteBorrow<'a, T> {
    inner: RefMut<'a, dyn Resource>,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, T> DataBorrow for WriteBorrow<'a, T> {}

impl<'a, T> std::ops::Deref for WriteBorrow<'a, T>
where
    T: Resource,
{
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.inner.downcast_ref_unchecked() }
    }
}

impl<'a, T> std::ops::DerefMut for WriteBorrow<'a, T>
where
    T: Resource,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.inner.downcast_mut_unchecked() }
    }
}

//
// Task
//
pub trait Task {
    type RequiredResources: for<'a> DataRequirement<'a> + RequiresResources<ResourceId> + Send + 'static;

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow);
}

macro_rules! impl_data {
    ( $($ty:ident),* ) => {

        //
        // Make tuples containing DataBorrow types implement DataBorrow
        //
        impl<$($ty),*> DataBorrow for ( $( $ty , )* )
        where $( $ty : DataBorrow ),*
        {

        }

        //
        // Make tuples containing DataRequirement types implement DataBorrow. Additionally implement
        // fetch
        //
        impl<'a, $($ty),*> DataRequirement<'a> for ( $( $ty , )* )
        where $( $ty : DataRequirement<'a> ),*
        {
            type Borrow = ( $( <$ty as DataRequirement<'a>>::Borrow, )* );

            fn fetch(world: &'a World) -> Self::Borrow {
                #![allow(unused_variables)]
                ( $( <$ty as DataRequirement<'a>>::fetch(world), )* )
            }
        }
    };
}

mod impl_data {
    #![cfg_attr(rustfmt, rustfmt_skip)]

    use super::*;

    impl_data!(A);
    impl_data!(A, B);
    impl_data!(A, B, C);
    impl_data!(A, B, C, D);
    impl_data!(A, B, C, D, E);
    impl_data!(A, B, C, D, E, F);
    impl_data!(A, B, C, D, E, F, G);
    impl_data!(A, B, C, D, E, F, G, H);
    impl_data!(A, B, C, D, E, F, G, H, I);
    impl_data!(A, B, C, D, E, F, G, H, I, J);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
}
