
#[macro_use]
extern crate log;

mod requires_resources_legion;
mod requires_resources_shred;
mod async_dispatcher;


use std::any::TypeId;



#[derive(Clone, Copy, Debug, PartialEq)]
struct Pos(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct Vel(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct ResA(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct ResB(f32, f32, f32);

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

    println!("shred reads: {:?}", reads);
    println!("shred writes: {:?}", writes);
}

fn main() {
    main_shred();
    main_legion();
}
