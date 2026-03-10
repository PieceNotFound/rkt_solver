use itertools::{Either, Itertools};
use rkt_solver::{
    MoveOrRot,
    data::{AxialMove, Move, Rotation},
    solve,
};

fn main() {
    let mut was_err = false;

    let args = std::env::args().skip(1).collect_vec();

    if args.is_empty() || ["--help", "-h", "help"].contains(&&*args[0]) {
        println!(
            "An RKT solver for 3⁴ last layer.\n\nSupply a 3³ algorithm as arguments and the solver will print a table showing the optimal insertions of rotations into the algorithm to make the algorithm equivalent to the `Effect` column in the table.\n\nEffects where the optimal insertions can be found by appending a rotation to the start or end of a different solution are hidden."
        );
        return;
    }

    let alg = args
        .iter()
        .flat_map(|val| val.split_ascii_whitespace())
        .filter_map(|v| match v.parse::<Move>() {
            Ok(m) => Some(m),
            Err(_) => {
                eprintln!("`{v}` is not a valid 3³ move");
                was_err = true;
                None
            }
        })
        .collect::<Vec<_>>();

    if was_err {
        return;
    }

    let result = solve(&alg);

    let mut axial_moves = AxialMove::ALL.map(|turn| {
        let (a, b) = turn.moves();
        (
            turn,
            match (a.is_zero(), b.is_zero()) {
                (true, true) => String::new(),
                (true, false) => format!("{a}"),
                (false, true) => format!("{b}"),
                (false, false) => format!("{a} {b}"),
            },
        )
    });

    axial_moves.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));

    let min_set = axial_moves
        .map(|(turn, axial_name)| {
            let mut rotations = Rotation::ALL.map(|r| (r, r.to_axials().format(" ").to_string()));
            rotations.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));

            rotations
                .iter()
                .filter_map(|(r, rot_name)| {
                    result(*r, turn * *r)
                        .map(|(soln, cost)| (soln, cost, format!("{axial_name:5} {rot_name:5}")))
                })
                .min_set_by_key(|v| v.1)
        })
        .into_iter()
        .flatten()
        .collect_vec();

    println!("Effect      | Cost | Alg");
    println!("------------------------");

    if min_set.is_empty() {
        println!("None");
    } else {
        for (soln, cost, name) in min_set {
            println!(
                "{name:6} | {cost:<4} | {:3}",
                soln.into_iter()
                    .flat_map(|v| {
                        match v {
                            MoveOrRot::Move(mv) => Either::Left(core::iter::once(mv)),
                            MoveOrRot::Rot(rot) => Either::Right(rot.to_axials()),
                        }
                        .factor_into_iter()
                    })
                    .format(" ")
            );
        }
    }
}
