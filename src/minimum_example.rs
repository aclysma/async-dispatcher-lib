
use hashbrown::HashMap;
use mopa::Any;

use std::marker::PhantomData;

use std::sync::Arc;

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
    resources: HashMap<ResourceId, Box<dyn Resource>>,
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
        self.resources.insert(id, Box::new(r));
    }

    fn remove_by_id<R>(&mut self, id: ResourceId) -> Option<R>
        where
            R: Resource,
    {
        self.resources
            .remove(&id)
            .map(|x: Box<dyn Resource>| x.downcast())
            .map(|x: Result<Box<R>, _>| x.ok().unwrap())
            .map(|x| *x)
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


// This avoids a 'lifetime on DataRequirement but the DataRequirement can't have a ref to the data
// from world
/*
trait DataRequirement {
    fn fetch<'a>(world: &'a World) -> Self;
}

impl DataRequirement for () {
    fn fetch<'a>(_: &'a World) -> Self {}
}

impl<T : Resource> DataRequirement for Read<T> {
    fn fetch<'a>(_: &'a World) -> Self {
        Read {
            phantom_data: PhantomData
        }
    }
}

impl<T : Resource> DataRequirement for Write<T> {
    fn fetch<'a>(_: &'a World) -> Self {
        Write {
            phantom_data: PhantomData
        }
    }
}

macro_rules! impl_data {
    ( $($ty:ident),* ) => {
        impl<$($ty),*> DataRequirement for ( $( $ty , )* )
            where $( $ty : DataRequirement ),*
            {
                fn fetch<'a>(world: &'a World) -> Self {
                    #![allow(unused_variables)]

                    ( $( <$ty as DataRequirement>::fetch(world), )* )
                }
            }
    };
}
*/



// This has a 'lifetime on data requirement, but then it still exists before we even query anything

pub trait DataRequirement<'a> {
    fn fetch(world: &'a World) -> Self;
}

impl<'a> DataRequirement<'a> for () {
    fn fetch(_: &'a World) -> Self {}
}

impl<'a, T : Resource> DataRequirement<'a> for Read<T> {
    fn fetch(_: &'a World) -> Self {
        Read {
            phantom_data: PhantomData
        }
    }
}

impl<'a, T : Resource> DataRequirement<'a> for Write<T> {
    fn fetch(_: &'a World) -> Self {
        Write {
            phantom_data: PhantomData
        }
    }
}

macro_rules! impl_data {
    ( $($ty:ident),* ) => {
        impl<'a, $($ty),*> DataRequirement<'a> for ( $( $ty , )* )
            where $( $ty : DataRequirement<'a> ),*
            {
                fn fetch(world: &'a World) -> Self {
                    #![allow(unused_variables)]

                    ( $( <$ty as DataRequirement<'a>>::fetch(world), )* )
                }
            }
    };
}


// This uses an Arc and avoids the lifetime stuff
/*
pub trait DataRequirement {
    fn fetch(world: &Arc<World>) -> Self;
}

impl DataRequirement for () {
    fn fetch(_: &Arc<World>) -> Self {}
}

impl<T : Resource> DataRequirement for Read<T> {
    fn fetch(_: &Arc<World>) -> Self {
        Read {
            phantom_data: PhantomData
        }
    }
}

impl<T : Resource> DataRequirement for Write<T> {
    fn fetch(_: &Arc<World>) -> Self {
        Write {
            phantom_data: PhantomData
        }
    }
}

macro_rules! impl_data {
    ( $($ty:ident),* ) => {
        impl<$($ty),*> DataRequirement for ( $( $ty , )* )
            where $( $ty : DataRequirement ),*
            {
                fn fetch(world: &Arc<World>) -> Self {
                    #![allow(unused_variables)]

                    ( $( <$ty as DataRequirement>::fetch(world), )* )
                }
            }
    };
}
*/

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
        F : FnOnce(T),
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


    let reads = <(Read<HelloWorldResourceA>, Write<HelloWorldResourceB>)>::reads();
    let fetched = <(Read<HelloWorldResourceA>, Write<HelloWorldResourceB>)>::fetch(&world);

    dispatcher.enter_game_loop(move |dispatcher| {

        use futures::future::Future;

        acquire_resources::<(Read<HelloWorldResourceA>, Write<HelloWorldResourceB>)>(dispatcher.clone(), world.clone())
            .and_then(move |acquired_resources| {
                acquired_resources.visit(|data| {
                    let (a, mut b) = data;

                    //println!("{} {}", a.value, b.value);

                });
                Ok(())
            })
    })
}