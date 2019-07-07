
use std::sync::Arc;

use crate::minimum::{
    World,
    Read,
    Write,
    DataBorrow,
    DataRequirement,
    ReadBorrow,
    WriteBorrow
};

use crate::async_dispatcher::{
    DispatcherBuilder,
    Dispatcher,
    ExecuteSequential,
    RequiresResources,
    minimum::acquire_resources,
    minimum::MinimumDispatcherBuilder,
    minimum::MinimumDispatcher
};

use crate::async_dispatcher::minimum::AcquiredResources;

struct HelloWorldResourceA {
    value: i32,
}

struct HelloWorldResourceB {
    value: i32,
}

fn example_inline(resources: AcquiredResources<(
    Read<HelloWorldResourceA>,
    Write<HelloWorldResourceB>
)>) {
    resources.visit(|r| {
        let (a, mut b) = r;
        b.value += 1;
    })
}

type ExampleFn3Args = (Read<HelloWorldResourceA>, Write<HelloWorldResourceB>);
fn example_typedef(resources: AcquiredResources<ExampleFn3Args>) {
    resources.visit(|r| {
        let (a, mut b) = r;
        b.value += 1;
    })
}

fn example_single_resource(resources: AcquiredResources<Write<HelloWorldResourceB>>) {
    resources.visit(|mut b| {
        println!("hi");
        b.value += 1;
    })
}

pub fn minimum_example() {

    let dispatcher = MinimumDispatcherBuilder::new()
        .insert(HelloWorldResourceA { value: 5 } )
        .insert(HelloWorldResourceB { value: 10 } )
        .build();

    use futures::future::Future;

    let world = dispatcher.enter_game_loop(move |ctx| {
        ExecuteSequential::new(vec![
            ctx.run(example_inline),
            ctx.run(example_typedef),
            ctx.run(example_single_resource),

            // Closure is still possible
            ctx.run(
                |resources: AcquiredResources<(
                    Read<HelloWorldResourceA>,
                    Write<HelloWorldResourceB>
                )>| {
                    resources.visit(|res| {
                        let (a, mut b) = res;
                        println!("a {}", a.value);
                    })
                }
            )
        ])
    });
}
