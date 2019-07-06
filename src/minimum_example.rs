
use std::sync::Arc;

use crate::async_dispatcher::{
    ResourceId,
    Dispatcher,
    DispatcherBuilder,
    RequiresResources,
    AcquireResources,
    AcquiredResourcesLockGuards
};

use crate::minimum::{
    World,
    DataRequirement,
    Read,
    Write,
    Resource
};

impl<T : Resource> RequiresResources for Read<T> {
    fn reads() -> Vec<ResourceId> { vec![ResourceId::new::<T>()] }
    fn writes() -> Vec<ResourceId> { vec![] }
}

impl<T : Resource> RequiresResources for Write<T> {
    fn reads() -> Vec<ResourceId> { vec![] }
    fn writes() -> Vec<ResourceId> { vec![ResourceId::new::<T>()] }
}


struct HelloWorldResourceA {
    value: i32,
}

struct HelloWorldResourceB {
    value: i32,
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