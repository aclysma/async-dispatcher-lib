use std::sync::Arc;

use async_dispatcher::ExecuteSequential;

use async_dispatcher::support::shred::{ShredDispatcherBuilder, ShredDispatcherContext};

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

fn main() {
    let dispatcher = ShredDispatcherBuilder::new()
        .insert(HelloWorldResourceA { value: 5 })
        .insert(HelloWorldResourceB { value: 10 })
        .build();

    let _world = dispatcher.enter_game_loop(move |ctx| {
        ExecuteSequential::new(vec![
            ctx.run_shred_system(HelloWorldSystem {
                dispatcher: ctx.clone(),
            }),
            ctx.run_shred_system(HelloWorldSystem {
                dispatcher: ctx.clone(),
            }),
        ])
    });
}
