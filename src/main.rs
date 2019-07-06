
#[macro_use]
extern crate log;

mod requires_resources_legion;
mod requires_resources_shred;
mod async_dispatcher;

use std::sync::Arc;
use crate::async_dispatcher::RequiresResources;
use crate::requires_resources_shred::{ShredDispatcher, ShredDispatcherContext};


#[derive(Clone, Copy, Debug, PartialEq)]
struct Pos(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct Vel(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct ResA(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct ResB(f32, f32, f32);




struct HelloWorldResourceA {
    value: i32,
}

struct HelloWorldResourceB {
    value: i32,
}

struct HelloWorldSystem {
    dispatcher: Arc<ShredDispatcherContext>,
}

impl<'a> shred::System<'a> for HelloWorldSystem {
    type SystemData = (
        shred::ReadExpect<'a, HelloWorldResourceA>,
        shred::WriteExpect<'a, HelloWorldResourceB>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (a, mut b) = data;

        println!("Hello World a: {:?} b: {:?}", a.value, b.value);
        b.value += 1;

        if b.value > 20 {
            self.dispatcher.end_game_loop();
        }
    }
}




fn test_fn<T>(value: i32) {
    println!("{}", value);
}

fn main_legion() {
    use legion::prelude::*;
    use async_dispatcher::RequiresResources;
    let reads = <(Write<Pos>, Read<Vel>, Write<ResA>, Write<ResB>)>::reads();
    let writes = <(Write<Pos>, Read<Vel>, Write<ResA>, Write<ResB>)>::writes();

    println!("legion reads: {:?}", reads);
    println!("legion writes: {:?}", writes);
}
/*
pub fn run_system<T>(world: Arc<shred::World>, mut system: T) -> T
    where
        T: for<'b> shred::System<'b> + Send + 'static,
{
    use shred::RunNow;
    system.run_now(&world);
    system
}

fn run_shred_system<T>(
    dispatcher: Arc<async_dispatcher::Dispatcher>,
    world: Arc<shred::World>,
    system: T
) -> impl futures::future::Future<Item=(), Error=()>
where
    T : shred::System<'static>,
    <T as shred::System<'static>>::SystemData: async_dispatcher::RequiresResources
{
    use futures::future::Future;

    let required_resources = <<T as shred::System<'static>>::SystemData as RequiresResources>::required_resources();

    async_dispatcher::AcquireResources::new(dispatcher.clone(), required_resources)
        .and_then(move |guards| {
//            let world = world.clone();
//            use shred::RunNow;
//            system.run_now(&world);

            run_system(world.clone(), system);
            Ok(())
        })
}
*/

/*
fn main_raw() {
    main_shred();
    main_legion();

    let dispatcher = async_dispatcher::DispatcherBuilder::new()
        .register_resource::<Pos>()
        .register_resource::<Vel>()
        .register_resource::<ResA>()
        .register_resource::<ResB>()
        .register_resource::<HelloWorldResourceA>()
        .register_resource::<HelloWorldResourceB>()
        .build();

    let mut world = shred::World::empty();
    world.insert(ResA);
    world.insert(ResB);
    world.insert(HelloWorldResourceA { value: 5 } );
    world.insert(HelloWorldResourceB { value: 10 } );

    let world = Arc::new(world);

    use futures::future::Future;
    use async_dispatcher::RequiresResources;

    dispatcher.enter_game_loop(move |dispatcher| {

        //let w = Arc::clone(&world);
        let world = Arc::clone(&world);

        run_shred_system(
            dispatcher.clone(),
            world.clone(),
            HelloWorldSystem { dispatcher: dispatcher.clone() }
        )
    })
}
*/

fn main_shred() {

    use requires_resources_shred::ShredDispatcher;
    use requires_resources_shred::ShredDispatcherBuilder;

    let dispatcher = ShredDispatcherBuilder::new()
        .insert(ResA)
        .insert(ResB)
        .insert(HelloWorldResourceA { value: 5 } )
        .insert(HelloWorldResourceB { value: 10 } )
        .build();

    use futures::future::Future;
    use async_dispatcher::RequiresResources;

    use requires_resources_shred::ShredDispatcherContext;

    let world = dispatcher.enter_game_loop(move |ctx| {

        //let w = Arc::clone(&world);
        //let world = Arc::clone(&world);

        ctx.run_shred_system(
            HelloWorldSystem {
                dispatcher: ctx.clone()
            }
        )

            /*
        run_shred_system(
            dispatcher.clone(),
            world.clone(),
            HelloWorldSystem { dispatcher: dispatcher.clone() }
        )
        */
    });
}

fn main() {
    main_shred();
}