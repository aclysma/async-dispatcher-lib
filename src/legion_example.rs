
use legion::prelude::*;

use crate::async_dispatcher::{
    RequiresResources
};


#[derive(Clone, Copy, Debug, PartialEq)]
struct Pos(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct Vel(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct ResA(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct ResB(f32, f32, f32);


fn main_legion() {
    let reads = <(Write<Pos>, Read<Vel>, Write<ResA>, Write<ResB>)>::reads();
    let writes = <(Write<Pos>, Read<Vel>, Write<ResA>, Write<ResB>)>::writes();

    println!("legion reads: {:?}", reads);
    println!("legion writes: {:?}", writes);
}

pub fn legion_example() {

}