
use hashbrown::HashMap;
use mopa::Any;

use std::marker::PhantomData;

use std::sync::{Arc, RwLockReadGuard};
use shred::cell::TrustCell;
use shred::cell::Ref;
use shred::cell::RefMut;

use crate::async_dispatcher::{ResourceId, Dispatcher, DispatcherBuilder, RequiresResources, AcquireResources, AcquiredResourcesLockGuards};

pub trait Resource: Any + Send + Sync + 'static {}

mod __resource_mopafy_scope {
    #![allow(clippy::all)]

    use mopa::mopafy;

    use super::Resource;

    mopafy!(Resource);
}

impl<T> Resource for T where T: Any + Send + Sync {}

#[derive(Default)]
pub struct World {
    resources: HashMap<ResourceId, TrustCell<Box<dyn Resource>>>,
}

impl World {
    pub fn new() -> Self {
        World {
            resources: HashMap::new()
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

    fn fetch<R : Resource>(&self) -> ReadBorrow<R> {
        self.try_fetch().unwrap()
    }

    fn try_fetch<R : Resource>(&self) -> Option<ReadBorrow<R>> {
        let res_id = ResourceId::new::<R>();

        self.resources.get(&res_id).map(|r| ReadBorrow {
            inner: Ref::map(r.borrow(), Box::as_ref),
            phantom: PhantomData,
        })
    }

    fn fetch_mut<R : Resource>(&self) -> WriteBorrow<R> {
        self.try_fetch_mut().unwrap()
    }

    fn try_fetch_mut<R : Resource>(&self) -> Option<WriteBorrow<R>> {
        let res_id = ResourceId::new::<R>();

        self.resources.get(&res_id).map(|r| WriteBorrow::<R> {
            inner: RefMut::map(r.borrow_mut(), Box::as_mut),
            phantom: PhantomData,
        })
    }
}




struct HelloWorldResourceA {
    value: i32,
}

struct HelloWorldResourceB {
    value: i32,
}

struct Read<T : Resource> {
    phantom_data: PhantomData<T>
}

struct Write<T : Resource> {
    phantom_data: PhantomData<T>
}

impl<T : Resource> RequiresResources for Read<T> {
    fn reads() -> Vec<ResourceId> { vec![ResourceId::new::<T>()] }
    fn writes() -> Vec<ResourceId> { vec![] }
}

impl<T : Resource> RequiresResources for Write<T> {
    fn reads() -> Vec<ResourceId> { vec![] }
    fn writes() -> Vec<ResourceId> { vec![ResourceId::new::<T>()] }
}

pub trait DataBorrow {

}

impl DataBorrow for () {

}

pub struct ReadBorrow<'a, T> {
    inner: Ref<'a, dyn Resource>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> DataBorrow for ReadBorrow<'a, T> {

}

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

pub struct WriteBorrow<'a, T> {
    inner: RefMut<'a, dyn Resource>,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, T> DataBorrow for WriteBorrow<'a, T> {

}

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

pub trait DataRequirement<'a> {
    type Borrow : DataBorrow;

    fn fetch(world: &'a World) -> Self::Borrow;
}

impl<'a> DataRequirement<'a> for () {
    type Borrow = ();

    fn fetch(_: &'a World) -> Self::Borrow { }
}

impl<'a, T : Resource> DataRequirement<'a> for Read<T> {
    type Borrow = ReadBorrow<'a, T>;

    fn fetch(world: &'a World) -> Self::Borrow {
        world.fetch::<T>()
    }
}

impl<'a, T : Resource> DataRequirement<'a> for Write<T> {
    type Borrow = WriteBorrow<'a, T>;

    fn fetch(world: &'a World) -> Self::Borrow {
        world.fetch_mut::<T>()
    }
}

macro_rules! impl_data {
    ( $($ty:ident),* ) => {

        impl<$($ty),*> DataBorrow for ( $( $ty , )* )

            where $( $ty : DataBorrow ),*
            {

            }



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
    //TODO: More of these
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


pub fn minimum_example() {

    let mut world = World::new();

    world.insert(HelloWorldResourceA { value: 5 } );
    world.insert(HelloWorldResourceB { value: 10 } );

    let world = Arc::new(world);

    let dispatcher = DispatcherBuilder::new()
        .register_resource::<HelloWorldResourceA>()
        .register_resource::<HelloWorldResourceB>()
        .build();

    dispatcher.enter_game_loop(move |dispatcher| {

        use futures::future::Future;

        acquire_resources::<(Read<HelloWorldResourceA>, Write<HelloWorldResourceB>)>(dispatcher.clone(), world.clone())
            .and_then(move |acquired_resources| {
                acquired_resources.visit(|data| {
                    let (a, mut b) = data;
                    println!("value {}", a.value);
                    println!("value {}", b.value);
                    b.value += 5;
                });
                Ok(())
            })
    })
}