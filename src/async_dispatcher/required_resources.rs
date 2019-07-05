
use std::marker::PhantomData;
use super::ResourceId;

// This is a helper that determines the reads/writes required for a system. I would have preferred
// not to need this structure at all, but many of the shred types require lifetimes that just
// don't play nicely with tasks. This gets rid of those lifetimes.
#[derive(Debug)]
pub struct RequiredResources<T> {
    pub(super) reads: Vec<ResourceId>,
    pub(super) writes: Vec<ResourceId>,
    phantom_data: PhantomData<T>,
}

impl<T> RequiredResources<T> {
    pub fn new(reads: Vec<ResourceId>, writes: Vec<ResourceId>) -> Self {
        RequiredResources {
            reads,
            writes,
            phantom_data: PhantomData,
        }
    }
}

pub trait RequiresResources {
    fn reads() -> Vec<super::ResourceId>;
    fn writes() -> Vec<super::ResourceId>;
}

impl RequiresResources for () {
    fn reads() -> Vec<ResourceId> { vec![] }
    fn writes() -> Vec<ResourceId> { vec![] }
}

macro_rules! impl_data {
    ( $($ty:ident),* ) => {
        impl<$($ty),*> RequiresResources for ( $( $ty , )* )
            where $( $ty : RequiresResources ),*
            {
                fn reads() -> Vec<ResourceId> {
                    #![allow(unused_mut)]

                    let mut r = Vec::new();

                    $( {
                        let mut reads = <$ty as RequiresResources>::reads();
                        r.append(&mut reads);
                    } )*

                    r
                }

                fn writes() -> Vec<ResourceId> {
                    #![allow(unused_mut)]

                    let mut r = Vec::new();

                    $( {
                        let mut writes = <$ty as RequiresResources>::writes();
                        r.append(&mut writes);
                    } )*

                    r
                }
            }
    };
}

mod impl_data {
    #![cfg_attr(rustfmt, rustfmt_skip)]

    use super::*;

    impl_data!(A);
    impl_data!(A, B);
    impl_data!(A, B, C);
    impl_data!(A, B, C, D);
}
