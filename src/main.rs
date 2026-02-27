use itertools::{Either, Itertools};
use rkt_solver::{MoveOrRot, data::Move, solve};

fn main() {
    let val = std::env::args().nth(1).unwrap();
    let alg = val
        .split_ascii_whitespace()
        .map(|v| v.parse::<Move>().unwrap())
        .collect::<Vec<_>>();
    let result = solve(&alg);
    if let Some(result) = result {
        println!(
            "{}",
            result
                .into_iter()
                .flat_map(|v| {
                    match v {
                        MoveOrRot::Move(mv) => Either::Left(core::iter::once(mv)),
                        MoveOrRot::Rot(rot) => Either::Right(rot.to_axials()),
                    }
                    .factor_into_iter()
                })
                .format(" ")
        );
    } else {
        println!("None");
    }
}
