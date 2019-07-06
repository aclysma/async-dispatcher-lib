
use crate::async_dispatcher::{RequiresResources, Dispatcher};
use crate::async_dispatcher::ResourceId;
use crate::async_dispatcher::DispatcherBuilder;

use std::sync::atomic::Ordering;
use std::sync::Arc;

impl<'a, T : shred::Resource> RequiresResources for shred::ReadExpect<'a, T> {
    fn reads() -> Vec<ResourceId> { vec![ResourceId::new::<T>()] }
    fn writes() -> Vec<ResourceId> { vec![] }
}

impl<'a, T : shred::Resource> RequiresResources for shred::WriteExpect<'a, T> {
    fn reads() -> Vec<ResourceId> { vec![] }
    fn writes() -> Vec<ResourceId> { vec![ResourceId::new::<T>()] }
}

pub struct ShredDispatcherBuilder {
    dispatcher_builder: DispatcherBuilder,
    world: shred::World
}

impl ShredDispatcherBuilder {
    // Create an empty dispatcher builder
    pub fn new() -> Self {
        ShredDispatcherBuilder {
            dispatcher_builder: DispatcherBuilder::new(),
            world: shred::World::empty(),
        }
    }

    pub fn insert<T : shred::Resource>(mut self, resource: T) -> Self {
        self.world.insert(resource);
        self.dispatcher_builder = self.dispatcher_builder.register_resource::<T>();

        self
    }

    // Create the dispatcher
    pub fn build(self) -> ShredDispatcher {
        let dispatcher =self.dispatcher_builder.build();
        let world = Arc::new(self.world);

        ShredDispatcher {
            dispatcher,
            world
        }
    }
}

pub struct ShredDispatcherContext {
    dispatcher: Arc<Dispatcher>,
    world: Arc<shred::World>
}

impl ShredDispatcherContext {
    pub fn end_game_loop(&self) {
        self.dispatcher.end_game_loop();
    }

    pub fn dispatcher(&self) -> Arc<Dispatcher> {
        self.dispatcher.clone()
    }

    pub fn world(&self) -> Arc<shred::World> {
        self.world.clone()
    }

    pub fn run_shred_system<T>(
        &self,
        mut system: T
    ) -> impl futures::future::Future<Item=(), Error=()>
        where
            T : shred::System<'static> + 'static,
            <T as shred::System<'static>>::SystemData: RequiresResources
    {
        use futures::future::Future;

        let required_resources = <<T as shred::System<'static>>::SystemData as RequiresResources>::required_resources();

        //let dispatcher = self.dispatcher.clone();
        let world = self.world.clone();

        crate::async_dispatcher::AcquireResources::new(self.dispatcher.clone(), required_resources)
            .and_then(move |guards| {

                //SAFETY:
                // We now have exclusive ownership of the system, and an Arc to the world. Therefore,
                // both will live for the lifetime of this function. Additionally, the mutable system
                // is not returned, so it can't accidentally hold a raw reference to anything from the world
                let world = world;
                let mut system = system;

                unsafe {
                    let sys : &mut T = &mut system;
                    let sys : &'static mut T = std::mem::transmute(sys);

                    let world : &shred::World = &world;
                    let world : &'static shred::World = std::mem::transmute(world);

                    use shred::RunNow;
                    sys.run_now(&world);
                }

                Ok(())
            })
    }
}

pub struct ShredDispatcher {
    dispatcher: Dispatcher,
    world: Arc<shred::World>
}

impl ShredDispatcher {

    // Call this to kick off processing.
    pub fn enter_game_loop<F, FutureT>(self, f: F) -> shred::World
        where
            F: Fn(Arc<ShredDispatcherContext>) -> FutureT + Send + Sync + 'static,
            FutureT: futures::future::Future<Item = (), Error = ()> + Send + 'static,
    {
        let world = self.world.clone();

        self.dispatcher.enter_game_loop(move |dispatcher| {
            let ctx = Arc::new(ShredDispatcherContext {
                dispatcher: dispatcher.clone(),
                world: world.clone()
            });

            (f)(ctx)
        });

        // Then unwrap the world inside it
        let world = Arc::try_unwrap(self.world).unwrap_or_else(|_| {
            unreachable!();
        });

        // Return the world
        world
    }

//    pub fn run_system<T>(&self, mut system: T) -> T
//    where
//        T: for<'b> shred::System<'b> + Send + 'static,
//    {
//        use shred::RunNow;
//        system.run_now(&self.world);
//        system
//    }

//    // Queues up a system to run. This code will acquire the appropriate resources first, then
//    // run the given system
//    pub fn create_future_with_result<T>(
//        dispatcher: &Arc<Dispatcher>,
//        system: T,
//    ) -> Box<impl futures::Future<Item = T, Error = ()>>
//    where
//        T: for<'b> shred::System<'b> + Send + 'static,
//    {
//        let dispatcher = dispatcher.clone();
//        let required_resources = super::RequiredResources::from_system(&system);
//        use futures::Future;
//        Box::new(
//            super::AcquireResources::<T>::new(dispatcher.clone(), required_resources).and_then(
//                move |_result| {
//                    let system = dispatcher.run_system(system);
//                    Ok(system)
//                },
//            ),
//        )
//    }
//
//    // Queues up a system to run. This code will acquire the appropriate resources first, then
//    // run the given system
//    pub fn create_future<T>(
//        dispatcher: &Arc<Dispatcher>,
//        system: T,
//    ) -> Box<impl futures::Future<Item = (), Error = ()>>
//    where
//        T: for<'b> shred::System<'b> + Send + 'static,
//    {
//        use futures::future::Future;
//        Box::new(Dispatcher::create_future_with_result(dispatcher, system).map(|_| ()))
//    }
}