use std::fmt::Debug;

use rkt_solver::{
    MoveOrRot, solve,
    types::{AxialRotation, Move},
};

enum Foo {
    Move(Move),
    Rot(AxialRotation),
}

impl Debug for Foo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Move(arg0) => Debug::fmt(arg0, f),
            Self::Rot(arg0) => Debug::fmt(arg0, f),
        }
    }
}

impl Foo {
    fn map_iter(iter: impl Iterator<Item = MoveOrRot>) -> impl Iterator<Item = Self> {
        iter.flat_map(|v| match v {
            MoveOrRot::Move(v) => vec![Self::Move(v)],
            MoveOrRot::Rot(rotation) => rotation.to_axials().map(Self::Rot).collect(),
        })
    }
}

fn main() {
    let val = std::env::args().nth(1).unwrap();
    let alg = val
        .split_ascii_whitespace()
        .map(|v| v.parse::<Move>().unwrap())
        .collect::<Vec<_>>();
    let result = solve(&alg);
    if let Some(result) = result {
        for (i, v) in Foo::map_iter(result.into_iter()).enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{v:?}");
        }
        println!();
    } else {
        println!("None");
    }
}
