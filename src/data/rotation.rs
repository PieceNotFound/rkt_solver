use core::ops::{Mul, Neg};

use crate::data::{
    basic::{AxialMove, AxialRotation, Axis, Face, Move},
    z4::Z4,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Rotation(u8);

#[derive(Clone, Copy, PartialEq, Eq)]
#[expect(clippy::upper_case_acronyms)]
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
        while i as (usize) < this.len() {
            out[this[i as usize] as usize] = Diagonal::from_u8(i);
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

    pub const fn mul_(self, rhs: Self) -> Self {
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

    pub const fn mul(self, rhs: Self) -> Self {
        const LUT: [[Rotation; 24]; 24] = {
            let mut out = [[Rotation::ID; 24]; 24];
            let mut i = 0;
            while i < Rotation::ALL.len() {
                let mut j = 0;
                while j < Rotation::ALL.len() {
                    let lhs = Rotation::ALL[i];
                    let rhs = Rotation::ALL[j];
                    out[i][j] = lhs.mul_(rhs);
                    j += 1;
                }
                i += 1;
            }
            out
        };

        LUT[self.index() as usize][rhs.index() as usize]
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
            while i as (usize) < Rotation::ALL.len() {
                out[Rotation::ALL[i as usize].0 as usize] = i;
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

    #[expect(clippy::missing_panics_doc, reason = "the `unwrap` never fails")]
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
