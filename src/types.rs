use std::{
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}
use Axis::*;

impl Axis {
    const fn next(self) -> Self {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Z4 {
    Zero,
    One,
    Two,
    Three,
}
use Z4::*;

impl Z4 {
    const fn next(self) -> Self {
        match self {
            Zero => One,
            One => Two,
            Two => Three,
            Three => Zero,
        }
    }

    const fn prev(self) -> Self {
        self.next().next().next()
    }

    pub const ALL: [Z4; 4] = [Zero, One, Two, Three];

    pub const fn val(self) -> u8 {
        self as u8
    }

    pub const fn neg(mut self) -> Self {
        let mut res = Zero;
        while !matches!(self, Zero) {
            self = self.prev();
            res = res.prev();
        }
        res
    }

    pub const fn add(mut self, mut rhs: Self) -> Self {
        while !matches!(rhs, Zero) {
            self = self.next();
            rhs = rhs.prev();
        }
        self
    }

    pub const fn sub(self, rhs: Self) -> Self {
        self.add(rhs.neg())
    }
}

impl Add for Z4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}

impl Neg for Z4 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.neg()
    }
}

impl Sub for Z4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.sub(rhs)
    }
}

impl AddAssign for Z4 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Z4 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
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

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}{}",
            self.face(),
            match self.by() {
                Zero => "0",
                One => "",
                Two => "2",
                Three => "'",
            }
        )
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl FromStr for Move {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (face_str, face) in [
            ("R", Face::R),
            ("U", Face::U),
            ("F", Face::F),
            ("L", Face::L),
            ("D", Face::D),
            ("B", Face::B),
        ] {
            if let Some(rest) = s.strip_prefix(face_str) {
                return Ok(Self::new(
                    face,
                    match rest {
                        "" => One,
                        "2" => Two,
                        "'" => Three,
                        _ => return Err("bad amount"),
                    },
                ));
            }
        }

        Err("bad face")
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
        if matches!((pos, neg), (Zero, Zero)) {
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

    pub const ZERO: Self = Self::new(X, Zero, Zero);

    pub const fn is_zero(self) -> bool {
        matches!((self.pos(), self.neg()), (Zero, Zero))
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
            Self::new(value.axis(), Zero, value.by())
        } else {
            Self::new(value.axis(), value.by(), Zero)
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

impl Debug for AxialMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_zero() {
            write!(f, "(0)")
        } else {
            let (a, b) = self.moves();
            if a.by() == Zero {
                write!(f, "({b})")
            } else if b.by() == Zero {
                write!(f, "({a})")
            } else {
                write!(f, "({a} {b})")
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Rotation(u8);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Diagonal {
    UFR,
    UFL,
    DFR,
    UBR,
}

impl Diagonal {
    const ALL: [Self; 4] = [Self::UFR, Self::UFL, Self::DFR, Self::UBR];

    const fn from_u8(val: u8) -> Self {
        Self::ALL[(val & 0b11) as usize]
    }
}

impl Rotation {
    const fn to_array(self) -> [Diagonal; 4] {
        let mut res = [Diagonal::UFR; 4];
        let mut i = 0;
        while i < res.len() {
            res[i] = Diagonal::from_u8(self.0 >> (2 * i));
            i += 1;
        }
        res
    }

    const fn from_array(arr: [Diagonal; 4]) -> Self {
        let mut res = 0;
        let mut i = 0;
        while i < arr.len() {
            res |= (arr[i] as u8) << (2 * i);
            i += 1;
        }
        Self(res)
    }

    const DATA: [[Diagonal; 4]; 6] = {
        use Diagonal::*;
        [
            [UFR, UBR, UFL, DFR],
            [UFR, UFL, DFR, UBR],
            [UFR, DFR, UBR, UFL],
            [UFR, DFR, UFL, UBR],
            [UFR, UBR, DFR, UFL],
            [UFR, UFL, UBR, DFR],
        ]
    };

    const fn get_(self, face: Face) -> Face {
        let this = self.to_array();
        let lookup = Self::DATA[face as usize];
        let mut pos = 0;
        let mut looked_up = [Diagonal::UFR; 4];
        let mut i = 0;
        while i < looked_up.len() {
            looked_up[i] = this[lookup[i] as usize];
            if matches!(looked_up[i], Diagonal::UFR) {
                pos = i;
            }
            i += 1;
        }
        looked_up.rotate_left(pos);
        let mut i = 0;
        loop {
            let el = Self::DATA[i];
            if el[1] as u8 == looked_up[1] as u8
                && el[2] as u8 == looked_up[2] as u8
                && el[3] as u8 == looked_up[3] as u8
            {
                break Face::ALL[i];
            }
            i += 1;
        }
    }

    pub const fn get(self, face: Face) -> Face {
        const LUT: [[Face; 6]; 256] = {
            let mut out = [[Face::R; 6]; 256];
            let mut i = 0;
            while i < Rotation::ALL.len() {
                let mut j = 0;
                while j < Face::ALL.len() {
                    let rot = Rotation::ALL[i];
                    out[rot.0 as usize][j] = rot.get_(Face::ALL[j]);
                    j += 1;
                }
                i += 1;
            }
            out
        };

        LUT[self.0 as usize][face as usize]
    }

    const fn apply_(self, face: Face) -> Face {
        self.inv().get(face)
    }

    pub const fn apply(self, face: Face) -> Face {
        const LUT: [[Face; 6]; 256] = {
            let mut out = [[Face::R; 6]; 256];
            let mut i = 0;
            while i < Rotation::ALL.len() {
                let mut j = 0;
                while j < Face::ALL.len() {
                    let rot = Rotation::ALL[i];
                    out[rot.0 as usize][j] = rot.apply_(Face::ALL[j]);
                    j += 1;
                }
                i += 1;
            }
            out
        };

        LUT[self.0 as usize][face as usize]
    }

    const fn inv_(self) -> Self {
        let mut out = [Diagonal::UFR; 4];
        let this = self.to_array();
        let mut i = 0;
        while i < this.len() {
            out[this[i] as usize] = Diagonal::from_u8(i as u8);
            i += 1;
        }
        Self::from_array(out)
    }

    pub const fn inv(self) -> Self {
        const LUT: [Rotation; 256] = {
            let mut out = [Rotation::ID; 256];
            let mut i = 0;
            while i < Rotation::ALL.len() {
                let rot = Rotation::ALL[i];
                out[rot.0 as usize] = rot.inv_();
                i += 1;
            }
            out
        };

        LUT[self.0 as usize]
    }

    pub const fn mul(self, rhs: Self) -> Self {
        let this = self.to_array();
        let rhs = rhs.to_array();
        let mut out = [Diagonal::UFR; 4];
        let mut i = 0;
        while i < out.len() {
            out[i] = this[rhs[i] as usize];
            i += 1;
        }
        Self::from_array(out)
    }

    pub const ID: Self = Self(0b_11_10_01_00);

    pub const ALL: [Rotation; 24] = {
        let mut result = [Rotation::ID; 24];
        let mut idx = 0;
        while idx < 24 {
            let j = idx / 6;
            let i = idx % 6;
            let mut arr = Self::DATA[i];
            arr.rotate_right(j);
            result[idx] = Self::from_array(arr);
            idx += 1;
        }
        result
    };

    pub const fn index(self) -> u8 {
        const MAP: [u8; 256] = {
            let mut out = [0; 256];
            let mut i = 0;
            while i < Rotation::ALL.len() {
                out[Rotation::ALL[i].0 as usize] = i as u8;
                i += 1;
            }
            out
        };

        MAP[self.0 as usize]
    }

    pub fn axial(axis: Axis, by: Z4) -> Self {
        let mut out = [Diagonal::UFR; 4];
        let data = Self::DATA[Face::new(axis, false) as usize];
        for i in Z4::ALL {
            out[data[i.val() as usize] as usize] = data[(i - by).val() as usize];
        }
        Self::from_array(out)
    }

    pub fn to_axials(self) -> impl Iterator<Item = AxialRotation> {
        let r_id = Face::R * self == Face::R;
        let u_id = Face::U * self == Face::U;
        let f_id = Face::F * self == Face::F;

        if r_id && u_id && f_id {
            [None, None]
        } else if r_id || u_id || f_id {
            let id = if r_id {
                Face::R
            } else if u_id {
                Face::U
            } else {
                Face::F
            };

            let axis = id.axis();
            let applied = self.apply(Face::new(id.axis().next(), false));
            let by = match (applied.axis() == id.axis().next(), applied.neg()) {
                (true, false) => Z4::Zero,
                (false, true) => Z4::One,
                (true, true) => Z4::Two,
                (false, false) => Z4::Three,
            };

            [Some(AxialRotation::new(axis, by)), None]
        } else {
            let (axis, by) = match self.apply(Face::R) {
                Face::R => unreachable!(),
                Face::U => (Axis::Z, Z4::Three),
                Face::F => (Axis::Y, Z4::One),
                Face::L => (Axis::Y, Z4::Two),
                Face::D => (Axis::Z, Z4::One),
                Face::B => (Axis::Y, Z4::Three),
            };

            let first = AxialRotation::new(axis, by);
            let rest = -Self::axial(axis, by) * self;
            let second = rest.to_axials().next().unwrap();
            [Some(first), Some(second)]
        }
        .into_iter()
        .flatten()
    }
}

impl Neg for Rotation {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.inv()
    }
}

impl Mul for Rotation {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.mul(rhs)
    }
}

impl Mul<Rotation> for Face {
    type Output = Self;

    fn mul(self, rhs: Rotation) -> Self::Output {
        rhs.apply(self)
    }
}

impl Mul<Rotation> for Move {
    type Output = Self;

    fn mul(self, rhs: Rotation) -> Self::Output {
        Self::new(self.face() * rhs, self.by())
    }
}

impl Mul<Rotation> for Axis {
    type Output = Self;

    fn mul(self, rhs: Rotation) -> Self::Output {
        (Face::new(self, false) * rhs).axis()
    }
}

impl Mul<Rotation> for AxialMove {
    type Output = Self;

    fn mul(self, rhs: Rotation) -> Self::Output {
        let new = Face::new(self.axis(), false) * rhs;
        if new.neg() {
            Self::new(new.axis(), self.neg(), self.pos())
        } else {
            Self::new(new.axis(), self.pos(), self.neg())
        }
    }
}

impl Display for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r_id = Face::R * *self == Face::R;
        let u_id = Face::U * *self == Face::U;
        let f_id = Face::F * *self == Face::F;

        if r_id && u_id && f_id {
            write!(f, "@[]")
        } else if r_id {
            write!(f, "@[U->{:?}]", Face::U * *self)
        } else if u_id {
            write!(f, "@[F->{:?}]", Face::F * *self)
        } else if f_id {
            write!(f, "@[R->{:?}]", Face::R * *self)
        } else {
            write!(f, "@[R->{:?} | U->{:?}]", Face::R * *self, Face::U * *self)
        }
    }
}

impl Debug for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
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

impl Display for AxialRotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            match self.axis {
                X => "x",
                Y => "y",
                Z => "z",
            },
            match self.by() {
                Zero => "0",
                One => "",
                Two => "2",
                Three => "'",
            }
        )
    }
}

impl Debug for AxialRotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
