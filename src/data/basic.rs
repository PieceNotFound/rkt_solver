use std::{
    fmt::Debug,
    ops::{Add, Neg, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}
use Axis::*;

use crate::data::z4::Z4;

impl Axis {
    pub(super) const fn next(self) -> Self {
        match self {
            X => Y,
            Y => Z,
            Z => X,
        }
    }

    pub const fn pos_face(self) -> Face {
        Face::new(self, false)
    }

    pub const fn neg_face(self) -> Face {
        Face::new(self, true)
    }

    pub const fn eq(self, rhs: Self) -> bool {
        matches!((self, rhs), (X, X) | (Y, Y) | (Z, Z))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    R,
    U,
    F,
    L,
    D,
    B,
}

impl Face {
    pub const fn new(axis: Axis, neg: bool) -> Self {
        use Face::*;
        match (axis, neg) {
            (X, false) => R,
            (Y, false) => U,
            (Z, false) => F,
            (X, true) => L,
            (Y, true) => D,
            (Z, true) => B,
        }
    }

    pub const fn axis(self) -> Axis {
        use Face::*;
        match self {
            R | L => X,
            U | D => Y,
            F | B => Z,
        }
    }

    pub const fn neg(self) -> bool {
        use Face::*;
        match self {
            R | U | F => false,
            L | D | B => true,
        }
    }

    pub const fn opposite(self) -> Face {
        Self::new(self.axis(), !self.neg())
    }

    pub const fn is_opposite(self, rhs: Face) -> bool {
        self.axis().eq(rhs.axis()) && self.neg() != rhs.neg()
    }

    pub const fn is_coaxial(self, rhs: Face) -> bool {
        self.axis().eq(rhs.axis())
    }

    pub const ALL: [Self; 6] = {
        use Face::*;
        [R, U, F, L, D, B]
    };
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Move {
    face: Face,
    by: Z4,
}

impl Move {
    pub const fn new(face: Face, by: Z4) -> Self {
        Self { face, by }
    }

    pub const fn face(self) -> Face {
        self.face
    }

    pub const fn by(self) -> Z4 {
        self.by
    }

    pub const fn axis(self) -> Axis {
        self.face().axis()
    }

    pub const fn commutes(self, rhs: Move) -> bool {
        self.face().is_coaxial(rhs.face())
    }

    pub const fn inv(self) -> Move {
        Self::new(self.face(), self.by().neg())
    }
}

impl Neg for Move {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.inv()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AxialMove {
    axis: Axis,
    pos: Z4,
    neg: Z4,
}

impl AxialMove {
    pub const fn new(mut axis: Axis, pos: Z4, neg: Z4) -> Self {
        if matches!((pos, neg), (Z4::Zero, Z4::Zero)) {
            axis = X;
        }
        Self { axis, pos, neg }
    }

    pub const fn axis(self) -> Axis {
        self.axis
    }

    pub const fn pos(self) -> Z4 {
        self.pos
    }

    pub const fn neg(self) -> Z4 {
        self.neg
    }

    pub const ZERO: Self = Self::new(X, Z4::Zero, Z4::Zero);

    pub const fn is_zero(self) -> bool {
        matches!((self.pos(), self.neg()), (Z4::Zero, Z4::Zero))
    }

    pub const fn moves(self) -> (Move, Move) {
        (
            Move::new(self.axis().pos_face(), self.pos()),
            Move::new(self.axis().neg_face(), self.neg()),
        )
    }

    pub const fn from_moves(a: Move, b: Move) -> Option<Self> {
        if !a.face().is_opposite(b.face()) {
            None
        } else {
            Some(if a.face().neg() {
                Self::new(a.axis(), b.by(), a.by())
            } else {
                Self::new(a.axis(), a.by(), b.by())
            })
        }
    }

    pub const fn inv(self) -> AxialMove {
        Self::new(self.axis(), self.pos().neg(), self.neg().neg())
    }

    pub fn add(self, rhs: AxialMove) -> Option<AxialMove> {
        if self.is_zero() {
            Some(rhs)
        } else if rhs.is_zero() {
            Some(self)
        } else if self.axis() != rhs.axis() {
            None
        } else {
            Some(Self::new(
                self.axis(),
                self.pos() + rhs.pos(),
                self.neg() + rhs.neg(),
            ))
        }
    }
}

impl From<Move> for AxialMove {
    fn from(value: Move) -> Self {
        if value.face().neg() {
            Self::new(value.axis(), Z4::Zero, value.by())
        } else {
            Self::new(value.axis(), value.by(), Z4::Zero)
        }
    }
}

impl Add for AxialMove {
    type Output = Self;

    #[track_caller]
    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs).unwrap()
    }
}

impl Neg for AxialMove {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.inv()
    }
}

impl Sub for AxialMove {
    type Output = Self;

    #[track_caller]
    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AxialRotation {
    axis: Axis,
    by: Z4,
}

impl AxialRotation {
    pub const fn new(axis: Axis, by: Z4) -> Self {
        Self { axis, by }
    }

    pub const fn axis(self) -> Axis {
        self.axis
    }

    pub const fn by(self) -> Z4 {
        self.by
    }
}
