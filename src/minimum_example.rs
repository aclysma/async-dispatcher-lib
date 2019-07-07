
use std::sync::Arc;

use crate::minimum::{World, Read, Write, DataBorrow, DataRequirement};

use crate::async_dispatcher::{
    DispatcherBuilder,
    Dispatcher,
    ExecuteSequential,
    RequiresResources,
    minimum::acquire_resources,
};
use crate::async_dispatcher::minimum::AcquiredResources;

struct HelloWorldResourceA {
    value: i32,
}

struct HelloWorldResourceB {
    value: i32,
}

fn acquire_resources_and_use<F, RequirementT>(
    dispatcher: Arc<Dispatcher>,
    world: Arc<World>,
    f: F
) -> Box<futures::future::Future<Item=(), Error=()>>
    where
        RequirementT: RequiresResources + 'static + Send,
        F : Fn(AcquiredResources<RequirementT>) + 'static,
{
    use futures::future::Future;

    Box::new(
        acquire_resources::<RequirementT>(dispatcher.clone(), world.clone())
        .and_then(move |acquired_resources| {
            (f)(acquired_resources);
            Ok(())
        })
    )
}

fn example_fn2(resources: AcquiredResources<(Read<HelloWorldResourceA>, Write<HelloWorldResourceB>)>) {
    resources.visit(|r| {
        let (a, mut b) = r;
        b.value += 1;
    })
}

type ExampleFn3Args = (Read<HelloWorldResourceA>, Write<HelloWorldResourceB>);
fn example_fn3(resources: AcquiredResources<ExampleFn3Args>) {
    resources.visit(|r| {
        let (a, mut b) = r;
        b.value += 1;
    })
}


fn example_fn(resources: AcquiredResources<Write<HelloWorldResourceB>>) {
    resources.visit(|mut b| {
        println!("hi");
        b.value += 1;
    })
}

fn acquire<F, T>(
    dispatcher: Arc<Dispatcher>,
    world: Arc<World>,
    f: F
) -> Box<impl futures::future::Future<Item=(), Error=()>>
where
    F: Fn(AcquiredResources<T>) + 'static,
    T : RequiresResources + 'static + Send
{
    use futures::future::Future;

    Box::new(
        acquire_resources::<T>(dispatcher, world)
            //.map(|_| ())
            .map(move |x| {
                (f)(x);
            })
    )
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


        ExecuteSequential::new(vec![
            /*
            Box::new(acquire_resources::<(Read<HelloWorldResourceA>, Write<HelloWorldResourceB>)>(dispatcher.clone(), world.clone())
                .and_then(move |acquired_resources| {
                    acquired_resources.visit(|data| {
                        let (a, mut b) = data;
                        println!("value {}", a.value);
                        println!("value {}", b.value);
                        b.value += 5;
                    });
                    Ok(())
                })
            ),
            */

            acquire(dispatcher.clone(), world.clone(), example_fn),
            acquire(dispatcher.clone(), world.clone(), example_fn),
            acquire(dispatcher.clone(), world.clone(), example_fn),

            //acquire_resources_and_use(dispatcher.clone(), world.clone(), example_fn),

//            acquire_resources_and_use::<_, <(Read<HelloWorldResourceA>, Write<HelloWorldResourceB>)> >(
//                dispatcher.clone(),
//                world.clone(),
//                move |acquired_resources| {
//
//                }
//            ),

            /*
            Box::new(acquire_resources::<(Read<HelloWorldResourceB>)>(dispatcher.clone(), world.clone())
                .and_then(move |acquired_resources| {
                    acquired_resources.visit(|data| {
                        let (b) = data;

                        if b.value > 10000 {
                            dispatcher.end_game_loop();
                        }
                    });
                    Ok(())
                })
            )
            */
        ])

    })
}