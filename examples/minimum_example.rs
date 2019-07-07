
use std::sync::Arc;

use async_dispatcher::minimum::{
    Read,
    Write,
    DataRequirement,
    Task
};

use async_dispatcher::support::minimum::{
    AcquiredResources,
    MinimumDispatcherBuilder,
    MinimumDispatcherContext
};

use async_dispatcher::ExecuteSequential;

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
        let (_a, mut b) = r;
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

fn main() {

    let dispatcher = MinimumDispatcherBuilder::new()
        .insert(HelloWorldResourceA { value: 5 } )
        .insert(HelloWorldResourceB { value: 10 } )
        .build();

    let _world = dispatcher.enter_game_loop(move |ctx| {
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
                        b.value += a.value;
                        println!("a {}", a.value);
                    })
                }
            ),

            ctx.run_task(ExampleTask { ctx: ctx.clone() })
        ])
    });
}
