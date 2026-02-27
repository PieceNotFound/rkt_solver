use core::{
    fmt::{self, Debug, Display},
    str::FromStr,
};

use crate::data::{
    basic::{AxialMove, AxialRotation, Axis, Face, Move},
    rotation::Rotation,
    z4::Z4,
};

macro_rules! debug_as_display {
    ($ty:ty) => {
        impl Debug for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                Display::fmt(self, f)
            }
        }
    };
}

debug_as_display!(Move);
impl Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}{}",
            self.face(),
            match self.by() {
                Z4::Zero => "0",
                Z4::One => "",
                Z4::Two => "2",
                Z4::Three => "'",
            }
        )
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
                        "" => Z4::One,
                        "2" => Z4::Two,
                        "'" => Z4::Three,
                        _ => return Err("bad amount"),
                    },
                ));
            }
        }

        Err("bad face")
    }
}

debug_as_display!(AxialMove);
impl Display for AxialMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            write!(f, "(0)")
        } else {
            let (a, b) = self.moves();
            if a.by() == Z4::Zero {
                write!(f, "({b})")
            } else if b.by() == Z4::Zero {
                write!(f, "({a})")
            } else {
                write!(f, "({a} {b})")
            }
        }
    }
}

debug_as_display!(AxialRotation);
impl Display for AxialRotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            match self.axis() {
                Axis::X => "x",
                Axis::Y => "y",
                Axis::Z => "z",
            },
            match self.by() {
                Z4::Zero => "0",
                Z4::One => "",
                Z4::Two => "2",
                Z4::Three => "'",
            }
        )
    }
}

debug_as_display!(Rotation);
impl Display for Rotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
