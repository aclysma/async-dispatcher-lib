
use std::sync::Arc;

use crate::async_dispatcher::{
    RequiresResources,
    ResourceId,
    Dispatcher,
    DispatcherBuilder,
    AcquiredResourcesLockGuards,
    AcquireResources
};

use crate::minimum;
use crate::minimum::DataRequirement;

//
// Hook up Read/Write to the resource system
//
impl<T : minimum::Resource> RequiresResources for minimum::Read<T> {
    fn reads() -> Vec<ResourceId> { vec![ResourceId::new::<T>()] }
    fn writes() -> Vec<ResourceId> { vec![] }
}

impl<T : minimum::Resource> RequiresResources for minimum::Write<T> {
    fn reads() -> Vec<ResourceId> { vec![] }
    fn writes() -> Vec<ResourceId> { vec![ResourceId::new::<T>()] }
}

//
// Helper that holds the locks and provides a method to fetch the data
//
pub struct AcquiredResources<T>
    where T : RequiresResources + 'static + Send
{
    lock_guards: AcquiredResourcesLockGuards<T>,
    world: Arc<minimum::World>
}

impl<T> AcquiredResources<T>
    where T : RequiresResources + 'static + Send {

    pub fn visit<'a, F>(&'a self, f : F)
        where
            F : FnOnce(T::Borrow),
            T : minimum::DataRequirement<'a>
    {
        let fetched = T::fetch(&self.world);
        (f)(fetched);
    }

    //TODO: Try a normal fetch API
}

//
// Creates a future to acquire the resources needed
//
pub fn acquire_resources<T>(dispatcher: Arc<Dispatcher>, world: Arc<minimum::World>) -> impl futures::future::Future<Item=AcquiredResources<T>, Error=()>
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


pub struct MinimumDispatcherBuilder {
    dispatcher_builder: DispatcherBuilder,
    world: minimum::World
}

impl MinimumDispatcherBuilder {
    // Create an empty dispatcher builder
    pub fn new() -> Self {
        MinimumDispatcherBuilder {
            dispatcher_builder: DispatcherBuilder::new(),
            world: minimum::World::new(),
        }
    }

    pub fn insert<T : minimum::Resource>(mut self, resource: T) -> Self {
        self.world.insert(resource);
        self.dispatcher_builder = self.dispatcher_builder.register_resource::<T>();

        self
    }

    // Create the dispatcher
    pub fn build(self) -> MinimumDispatcher {
        let dispatcher =self.dispatcher_builder.build();
        let world = Arc::new(self.world);

        MinimumDispatcher {
            dispatcher,
            world
        }
    }
}

pub struct MinimumDispatcher {
    dispatcher: Dispatcher,
    world: Arc<minimum::World>
}

impl MinimumDispatcher {

    // Call this to kick off processing.
    pub fn enter_game_loop<F, FutureT>(self, f: F) -> minimum::World
        where
            F: Fn(Arc<MinimumDispatcherContext>) -> FutureT + Send + Sync + 'static,
            FutureT: futures::future::Future<Item = (), Error = ()> + Send + 'static,
    {
        let world = self.world.clone();

        self.dispatcher.enter_game_loop(move |dispatcher| {
            let ctx = Arc::new(MinimumDispatcherContext {
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
}

pub struct MinimumDispatcherContext {
    dispatcher: Arc<Dispatcher>,
    world: Arc<minimum::World>
}

impl MinimumDispatcherContext {
    pub fn end_game_loop(&self) {
        self.dispatcher.end_game_loop();
    }

    pub fn dispatcher(&self) -> Arc<Dispatcher> {
        self.dispatcher.clone()
    }

    pub fn world(&self) -> Arc<minimum::World> {
        self.world.clone()
    }

    pub fn run_fn<RequirementT, F>(
        &self,
        f: F
    ) -> Box<impl futures::future::Future<Item=(), Error=()>>
        where
            RequirementT: RequiresResources + 'static + Send,
            F : Fn(AcquiredResources<RequirementT>) + 'static,
    {
        use futures::future::Future;

        Box::new(
            acquire_resources::<RequirementT>(self.dispatcher.clone(), self.world.clone())
                .map(move |acquired_resources| {
                    (f)(acquired_resources);
                })
        )
    }

    pub fn run_task<T>(
        &self,
        mut task: T
    ) -> Box<impl futures::future::Future<Item=(), Error=()>>
        where
            T: minimum::Task,
    {
        use futures::future::Future;

        Box::new(
            acquire_resources::<T::RequiredResources>(self.dispatcher.clone(), self.world.clone())
                .map(move |acquired_resources| {
                    acquired_resources.visit(move |resources| {
                        task.run(resources);
                    });
                })
        )
    }
}
