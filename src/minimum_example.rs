
use std::sync::Arc;

use crate::minimum::{
    World,
    Read,
    Write
};

use crate::async_dispatcher::{
    DispatcherBuilder,
    minimum::acquire_resources
};

struct HelloWorldResourceA {
    value: i32,
}

struct HelloWorldResourceB {
    value: i32,
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

                    if b.value > 10000 {
                        dispatcher.end_game_loop();
                    }
                });
                Ok(())
            })
    })
}