
use std::sync::Arc;

use crate::minimum::{
    Read,
    Write,
    DataRequirement,
    Task
};

use crate::async_dispatcher::minimum::{
    AcquiredResources,
    MinimumDispatcherBuilder,
    MinimumDispatcherContext
};

use crate::async_dispatcher::ExecuteSequential;

struct HelloWorldResourceA {
    value: i32,
}

struct HelloWorldResourceB {
    value: i32,
}

// Functions can be declared like this
fn example_inline(resources: AcquiredResources<(
    Read<HelloWorldResourceA>,
    Write<HelloWorldResourceB>
)>) {
    resources.visit(|r| {
        let (a, mut b) = r;
        b.value += 1;
    })
}

struct ExampleTask {
    ctx: Arc<MinimumDispatcherContext>,
}

impl Task for ExampleTask {
    type RequiredResources = (
        Read<HelloWorldResourceA>,
        Write<HelloWorldResourceB>,
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (a, mut b) = data;
        println!("Hello World a: {:?} b: {:?}", a.value, b.value);
        b.value += 1;

        if b.value > 200 {
            self.ctx.end_game_loop();
        }
    }
}

pub fn minimum_example() {

    let dispatcher = MinimumDispatcherBuilder::new()
        .insert(HelloWorldResourceA { value: 5 } )
        .insert(HelloWorldResourceB { value: 10 } )
        .build();

    use futures::future::Future;

    let world = dispatcher.enter_game_loop(move |ctx| {
        ExecuteSequential::new(vec![
            // Demo of three different styles
            ctx.run_fn(example_inline),

            // It's possible to use callbacks as well
            ctx.run_fn(
                |resources: AcquiredResources<(
                    Read<HelloWorldResourceA>,
                    Write<HelloWorldResourceB>
                )>| {
                    resources.visit(|res| {
                        let (a, mut b) = res;
                        println!("a {}", a.value);
                    })
                }
            ),

            ctx.run_task(ExampleTask { ctx: ctx.clone() })
        ])
    });
}
