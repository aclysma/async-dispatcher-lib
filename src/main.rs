
#[macro_use]
extern crate log;

mod async_dispatcher;
mod shred_example;
mod legion_example;
mod minimum_example;

fn main() {
    //shred_example::shred_example();
    //legion_example::legion_example();
    minimum_example::minimum_example();
}
