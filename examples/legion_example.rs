use legion::prelude::*;

use std::sync::Arc;

use async_dispatcher::support::legion::{
    create_query,
    LegionDispatcherBuilder
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Pos(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct Vel(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct ResA(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct ResB(f32, f32, f32);

fn main() {
    let universe = Universe::new(None);
    let mut world = universe.create_world();

    // create entities
    world.insert_from(
        (),
        vec![
            (Pos(1., 2., 3.), Vel(1., 2., 3.)),
            (Pos(1., 2., 3.), Vel(1., 2., 3.)),
            (Pos(1., 2., 3.), Vel(1., 2., 3.)),
            (Pos(1., 2., 3.), Vel(1., 2., 3.)),
        ],
    );

    let world = Arc::new(world);

    let dispatcher = LegionDispatcherBuilder::new()
        .with_resource_type::<Pos>()
        .with_resource_type::<Vel>()
        .with_resource_type::<ResA>()
        .with_resource_type::<ResB>()
        .build();

    use futures::future::Future;

    dispatcher.enter_game_loop(move |dispatcher| {
        let world = world.clone();

        create_query::<(Write<Pos>, Read<Vel>)>(dispatcher.clone()).and_then(move |mut x| {
            let world = world.clone();
            for (pos, vel) in x.query_mut().iter(&world) {
                pos.0 += vel.0;
                pos.1 += vel.1;
                pos.2 += vel.2;

                println!("{:?}", pos);

                if pos.0 > 1000. {
                    dispatcher.end_game_loop();
                }
            }

            Ok(())
        })
    });
}
