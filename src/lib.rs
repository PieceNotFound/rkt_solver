use core::{cell::UnsafeCell, mem::MaybeUninit};
use std::fmt::Debug;

use crate::{
    data::{AxialMove, Axis, Move, Rotation, Z4},
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

struct Slot<T> {
    inner: UnsafeCell<MaybeUninit<T>>,
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}

impl<T> Slot<T> {
    unsafe fn get(&self) -> &T {
        unsafe { (&*self.inner.get()).assume_init_ref() }
    }

    unsafe fn set(&self, val: T) {
        unsafe { &mut *self.inner.get() }.write(val);
    }
}

unsafe impl<T: Send> Send for Slot<T> {}
unsafe impl<T: Sync> Sync for Slot<T> {}

type Idx = (usize, usize, Rotation, AxialMove);
type Res = usize;
type Reconstructed = Vec<MoveOrRot>;
type DpChoice = (usize, Rotation, AxialMove);
type Val = Option<(Res, Option<DpChoice>)>;
type Arr = DpArray<Slot<Val>, Idx>;

struct Ctx<'a> {
    alg: &'a [Move],
    aux: Arr,
    #[cfg(debug_assertions)]
    up_to_sz: usize,
}

// TODO: some of these methods should be marked `unsafe` but aren't. eventually they should be made
//       safe by adding checks (but only under cfg(debug_assertions))
impl<'a> Ctx<'a> {
    fn new(alg: &'a [Move]) -> Self {
        let n = alg.len();
        let aux = DpArray::new((n + 1, n + 1, (), ()));
        Self {
            alg,
            aux,
            #[cfg(debug_assertions)]
            up_to_sz: 0,
        }
    }

    fn alg(&self) -> &'a [Move] {
        self.alg
    }

    fn get_full(&self, idx: Idx) -> Val {
        #[cfg(debug_assertions)]
        {
            let (l, r, _, _) = idx;
            let sz = r - l;
            if sz >= self.up_to_sz {
                panic!("Attempted to get value from DP array before it was initialised");
            }
        }

        *unsafe { self.aux[idx].get() }
    }

    fn get(&self, idx: Idx) -> Option<Res> {
        self.get_full(idx).map(|v| v.0)
    }

    fn set(&self, idx: Idx, val: Val) {
        #[cfg(debug_assertions)]
        {
            let (l, r, _, _) = idx;
            let sz = r - l;
            if sz != self.up_to_sz {
                panic!("Attempted to set value in DP array at wrong stage");
            }
        }

        unsafe { self.aux[idx].set(val) }
    }

    fn increment_sz(&mut self) {
        #[cfg(debug_assertions)]
        {
            self.up_to_sz += 1;
        }
    }
}

pub fn solve(alg: &[Move]) -> Option<Reconstructed> {
    let n = alg.len();
    let mut ctx = Ctx::new(alg);

    for sz in 0..=n {
        std::thread::scope(|scope| {
            for l in 0..=(n - sz) {
                let r = l + sz;
                for rotation in Rotation::ALL {
                    let ctx = &ctx;
                    scope.spawn(move || {
                        for axis in [Axis::X, Axis::Y, Axis::Z] {
                            for p in Z4::ALL {
                                for n in Z4::ALL {
                                    let ax = AxialMove::new(axis, p, n);
                                    let idx = (l, r, rotation, ax);
                                    ctx.set(idx, compute(ctx, idx));
                                }
                            }
                        }
                    });
                }
            }
        });
        ctx.increment_sz();
    }

    reconstruct(&ctx, (0, n, Rotation::ID, AxialMove::ZERO))
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

fn compute(ctx: &Ctx<'_>, idx @ (l, r, _, _): Idx) -> Val {
    match base_case(ctx.alg(), idx) {
        Some(BaseCase::Impossible) => return None,
        Some(BaseCase::Just(rot)) => return Some((if rot == Rotation::ID { 0 } else { 1 }, None)),

        None => {}
    }

    let mut min = None;
    for k in l + 1..=r {
        for r1 in Rotation::ALL {
            for t1_p in Z4::ALL {
                for t1_n in Z4::ALL {
                    let t1 = AxialMove::new((ctx.alg()[l] * r1).axis(), t1_p, t1_n);
                    let choice = (k, r1, t1);
                    let (f1, sub1, sub2) = apply_choice(ctx.alg(), idx, choice);
                    let sub1 = ctx.get(sub1);
                    let sub2 = ctx.get(sub2);
                    let new = post_computation((f1, sub1, sub2));
                    min_into(&mut min, new, choice);
                }
            }
        }
    }

    min
}

fn reconstruct(ctx: &Ctx<'_>, idx: Idx) -> Option<Reconstructed> {
    match base_case(ctx.alg(), idx) {
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

    let (_, choice) = ctx.get_full(idx)?;
    let choice = choice.unwrap();
    let (f1, sub1, sub2) = apply_choice(ctx.alg(), idx, choice);
    let sub1 = reconstruct(ctx, sub1)?;
    let sub2 = reconstruct(ctx, sub2)?;
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
