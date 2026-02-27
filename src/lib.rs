use std::fmt::Debug;

use crate::{
    data::{AxialMove, Move, Rotation, Z4},
    dp::DpArray,
};

pub mod data;
mod dp;

#[derive(Clone, Copy)]
pub enum MoveOrRot {
    Move(Move),
    Rot(Rotation),
}

impl Debug for MoveOrRot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Move(arg0) => Debug::fmt(arg0, f),
            Self::Rot(arg0) => Debug::fmt(arg0, f),
        }
    }
}

type Idx = (usize, usize, Rotation, AxialMove);
type Res = usize;
type Reconstructed = Vec<MoveOrRot>;
type DpChoice = (usize, Rotation, AxialMove);
type Val = Option<(Res, Option<DpChoice>)>;
type Arr = DpArray<Option<Val>, Idx>;

pub fn solve(alg: &[Move]) -> Option<Reconstructed> {
    let n = alg.len();
    let mut aux: Arr = DpArray::new((n + 1, n + 1, (), ()));
    let root = (0, n, Rotation::ID, AxialMove::ZERO);
    reconstruct(alg, &mut aux, root)
}

fn get(alg: &[Move], aux: &mut Arr, idx: Idx) -> Val {
    if let Some(val) = &aux[idx] {
        *val
    } else {
        let val = compute(alg, aux, idx);
        aux[idx] = Some(val);
        val
    }
}

fn rec(alg: &[Move], aux: &mut Arr, idx: Idx) -> Option<Res> {
    get(alg, aux, idx).map(|v| v.0)
}

enum BaseCase {
    Impossible,
    Just(Rotation),
}

fn base_case(alg: &[Move], (l, r, rot, ax): Idx) -> Option<BaseCase> {
    if l > r {
        return Some(BaseCase::Impossible);
    }

    if l == r {
        return Some(if ax.is_zero() {
            BaseCase::Just(rot)
        } else {
            BaseCase::Impossible
        });
    }

    if !ax.is_zero() && alg[l].axis() * rot != ax.axis() {
        return Some(BaseCase::Impossible);
    }

    None
}

fn compute(alg: &[Move], aux: &mut Arr, idx @ (l, r, _, _): Idx) -> Val {
    match base_case(alg, idx) {
        Some(BaseCase::Impossible) => return None,
        Some(BaseCase::Just(rot)) => return Some((if rot == Rotation::ID { 0 } else { 1 }, None)),
        None => {}
    }

    let mut min = None;
    for k in l + 1..=r {
        for r1 in Rotation::ALL {
            for t1_p in Z4::ALL {
                for t1_n in Z4::ALL {
                    let t1 = AxialMove::new((alg[l] * r1).axis(), t1_p, t1_n);
                    let choice = (k, r1, t1);
                    let (f1, sub1, sub2) = apply_choice(alg, idx, choice);
                    let sub1 = rec(alg, aux, sub1);
                    let sub2 = rec(alg, aux, sub2);
                    let new = post_computation((f1, sub1, sub2));
                    min_into(&mut min, new, choice);
                }
            }
        }
    }

    min
}

fn reconstruct(alg: &[Move], aux: &mut Arr, idx: Idx) -> Option<Reconstructed> {
    match base_case(alg, idx) {
        Some(BaseCase::Impossible) => return None,
        Some(BaseCase::Just(rot)) => {
            return Some(if rot == Rotation::ID {
                vec![]
            } else {
                vec![MoveOrRot::Rot(rot)]
            });
        }
        None => {}
    }

    let (_, choice) = get(alg, aux, idx)?;
    let choice = choice.unwrap();
    let (f1, sub1, sub2) = apply_choice(alg, idx, choice);
    let sub1 = reconstruct(alg, aux, sub1)?;
    let sub2 = reconstruct(alg, aux, sub2)?;
    Some(post_reconstruction((f1, sub1, sub2)))
}

fn apply_choice(alg: &[Move], (l, r, rot, ax): Idx, (k, r1, t1): DpChoice) -> (Move, Idx, Idx) {
    let r0 = rot;
    let t0 = ax;

    let f1 = alg[l];
    let t2 = -AxialMove::from(f1 * r1) + AxialMove::from(alg[l] * r1) + t1;

    let sub1 = (l + 1, k, r1, t2);
    let sub2 = (
        k,
        r,
        -r1 * r0,
        t0 - ((AxialMove::from(alg[l] * r1) + t1) * (-r1 * r0)),
    );

    return (f1, sub1, sub2);
}

fn post_computation((f1, sub1, sub2): (Move, Option<Res>, Option<Res>)) -> Option<Res> {
    let (sub1, sub2) = (sub1?, sub2?);

    let mut total = 0;
    if f1.by() != Z4::Zero {
        total += 1;
    }
    total += sub1;
    total += sub2;
    Some(total)
}

fn min_into(min: &mut Val, new: Option<Res>, choice: DpChoice) {
    match (&mut *min, new) {
        (None, Some(v)) => *min = Some((v, Some(choice))),
        (Some(min), Some(v)) if v < min.0 => *min = (v, Some(choice)),
        _ => {}
    }
}

fn post_reconstruction((f1, sub1, sub2): (Move, Reconstructed, Reconstructed)) -> Reconstructed {
    let mut total = vec![];
    if f1.by() != Z4::Zero {
        total.push(MoveOrRot::Move(f1));
    }
    total.extend(sub1);
    total.extend(sub2);

    total
}
