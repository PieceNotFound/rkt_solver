#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Z4 {
    Zero,
    One,
    Two,
    Three,
}
use core::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use Z4::*;

impl Z4 {
    pub const ALL: [Z4; 4] = [Zero, One, Two, Three];

    pub const fn val(self) -> u8 {
        self as u8
    }

    pub const fn from_val(val: u8) -> Self {
        match val & 0b11 {
            0 => Self::Zero,
            1 => Self::One,
            2 => Self::Two,
            3 => Self::Three,
            _ => unreachable!(),
        }
    }

    pub const fn neg(self) -> Self {
        Self::from_val(self.val().wrapping_neg())
    }

    pub const fn add(self, rhs: Self) -> Self {
        Self::from_val(self.val() + rhs.val())
    }

    pub const fn sub(self, rhs: Self) -> Self {
        Self::from_val(self.val().wrapping_sub(rhs.val()))
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
