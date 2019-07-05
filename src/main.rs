
#[macro_use]
extern crate log;

mod requires_resources_legion;
mod requires_resources_shred;
mod async_dispatcher;

use std::sync::Arc;



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
    dispatcher: Arc<async_dispatcher::Dispatcher>,
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

fn main_shred() {
    use shred::ReadExpect;
    use shred::WriteExpect;
    use async_dispatcher::RequiresResources;
    let reads = <(WriteExpect<Pos>, ReadExpect<Vel>, WriteExpect<ResA>, WriteExpect<ResB>)>::reads();
    let writes = <(WriteExpect<Pos>, ReadExpect<Vel>, WriteExpect<ResA>, WriteExpect<ResB>)>::writes();

    let reads2 = ExampleSystemData::required_resources();

    println!("shred reads: {:?}", reads);
    println!("shred writes: {:?}", writes);
}

type ExampleSystemData<'a> = (shred::WriteExpect<'a, Pos>, shred::ReadExpect<'a, Vel>, shred::WriteExpect<'a, ResA>, shred::WriteExpect<'a, ResB>);


fn run_system() {

}

fn main() {
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

        println!("hi");

        async_dispatcher::AcquireResources::new(dispatcher.clone(), <HelloWorldSystem as shred::System>::SystemData::required_resources())
            .map_err(|_| ())
            .and_then(move |guards| {

                let world = world.clone();
                let mut system = HelloWorldSystem { dispatcher: dispatcher.clone() };
                use shred::RunNow;
                system.run_now(&world);


                Ok(())
            })
    })




}
