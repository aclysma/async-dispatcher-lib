
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
};
use crate::async_dispatcher::minimum::AcquiredResources;

struct HelloWorldResourceA {
    value: i32,
}

struct HelloWorldResourceB {
    value: i32,
}

fn acquire_resources_and_use<RequirementT, F>(
    dispatcher: Arc<Dispatcher>,
    world: Arc<World>,
    f: F
) -> Box<impl futures::future::Future<Item=(), Error=()>>
    where
        RequirementT: RequiresResources + 'static + Send,
        F : Fn(AcquiredResources<RequirementT>) + 'static,
{
    use futures::future::Future;

    Box::new(
        acquire_resources::<RequirementT>(dispatcher.clone(), world.clone())

            //TODO: Both and_then and map work, not sure why or if one is better than the other

//        .and_then(move |acquired_resources| {
//            (f)(acquired_resources);
//            Ok(())
//        })

        .map(move |acquired_resources| {
            (f)(acquired_resources);
        })
    )
}


fn acquire_resources_and_visit<'a, RequirementT, F>(
    dispatcher: Arc<Dispatcher>,
    world: Arc<World>,
    f: F
) -> Box<impl futures::future::Future<Item=(), Error=()>>
    where
        RequirementT: RequiresResources + 'static + Send,
        RequirementT: DataRequirement<'a>,
        //F : Fn(AcquiredResources<RequirementT>) + 'static,
        F : FnOnce(<RequirementT as DataRequirement<'a>>::Borrow)
{
    use futures::future::Future;

    Box::new(
        acquire_resources::<RequirementT>(dispatcher.clone(), world.clone())

            //TODO: Both and_then and map work, not sure why or if one is better than the other

//        .and_then(move |acquired_resources| {
//            (f)(acquired_resources);
//            Ok(())
//        })

            .map(move |acquired_resources| {
                //(f)(acquired_resources);
                acquired_resources.visit(f);
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
//
//
//type ExampleFn3ArgsBorrow = ExampleFn3Args::Borrow;
//fn example_fn_2(borrow: ExampleFn3ArgsBorrow) {
//
//}

fn example_fn_borrow(resources: (ReadBorrow<HelloWorldResourceA>, WriteBorrow<HelloWorldResourceB>)) {
    let (a, mut b) = resources;
    b.value += 1;
}

//fn example_fn_borrow2(resources: <(ReadBorrow<HelloWorldResourceA>, WriteBorrow<HelloWorldResourceB>)>) {
//    let (a, mut b) = resources;
//    b.value += 1;
//}


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

            // Clean way
            acquire_resources_and_use(dispatcher.clone(), world.clone(), example_fn),
            acquire_resources_and_use(dispatcher.clone(), world.clone(), example_fn2),
            acquire_resources_and_use(dispatcher.clone(), world.clone(), example_fn3),

            // Closure is still possible
            acquire_resources_and_use(
                dispatcher.clone(),
                world.clone(),
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




            acquire_resources_and_visit(
                dispatcher.clone(),
                world.clone(),
                |resources: (
                    ReadBorrow<HelloWorldResourceA>,
                    WriteBorrow<HelloWorldResourceB>
                )| {
                    let (a, mut b) = resources;
                    println!("a {}", a.value);
                }
            ),

/*
            acquire_resources_and_visit(
                dispatcher.clone(),
                world.clone(),
                example_fn_borrow
            ),
            */
        ])

    })
}