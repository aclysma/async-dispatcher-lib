
use std::sync::Arc;

use crate::minimum::{
    World,
    Read,
    Write,
    DataBorrow,
    DataRequirement,
    ReadBorrow,
    WriteBorrow,
    Task
};

use crate::async_dispatcher::{
    ExecuteSequential,
    RequiresResources,
    minimum
};

use crate::async_dispatcher::minimum::{AcquiredResources, MinimumDispatcherContext};

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

// It may be more convenient to pull the args out seperately
type ExampleTypedefArgs = (Read<HelloWorldResourceA>, Write<HelloWorldResourceB>);
fn example_typedef(resources: AcquiredResources<ExampleTypedefArgs>) {
    resources.visit(|r| {
        let (a, mut b) = r;
        b.value += 1;
    })
}

// If you only need a single resource, you don't need to use a tuple
fn example_single_resource(resources: AcquiredResources<Write<HelloWorldResourceB>>) {
    resources.visit(|mut b| {
        println!("hi");
        b.value += 1;
    })
}

fn example_inline2(resources: <(
    Read<HelloWorldResourceA>,
    Write<HelloWorldResourceB>
) as DataRequirement>::Borrow
) {
    let (a, mut b) = resources;
    b.value += 1;
}

fn example_inline3<'a>(resources: (
    ReadBorrow<'a, HelloWorldResourceA>,
    WriteBorrow<'a, HelloWorldResourceB>
)
) {
    let (a, mut b) = resources;
    b.value += 1;
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

    let dispatcher = minimum::MinimumDispatcherBuilder::new()
        .insert(HelloWorldResourceA { value: 5 } )
        .insert(HelloWorldResourceB { value: 10 } )
        .build();

    use futures::future::Future;

    let world = dispatcher.enter_game_loop(move |ctx| {
        ExecuteSequential::new(vec![
            // Demo of three different styles
            ctx.run_fn(example_inline),
            ctx.run_fn2(example_inline2),
            //ctx.run_fn2(example_inline3),
            ctx.run_fn(example_typedef),
            ctx.run_fn(example_single_resource),

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
