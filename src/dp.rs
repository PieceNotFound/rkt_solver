use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::data::{AxialMove, Axis, Face, Rotation, Z4};

pub trait DpIndex {
    type Runtime;

    fn size(v: &Self::Runtime) -> usize;

    fn to_index(&self, v: &Self::Runtime) -> usize;
}

impl<T: DpIndex> DpIndex for &T {
    type Runtime = T::Runtime;

    fn size(v: &Self::Runtime) -> usize {
        T::size(v)
    }

    fn to_index(&self, v: &Self::Runtime) -> usize {
        (*self).to_index(v)
    }
}

macro_rules! dp_tuple {
    (@shrink) => {};
    (@shrink $T:ident $a:ident $b:ident $($rest:ident)*) => {
        dp_tuple!($($rest)*);
    };
    ($($T:ident $a:ident $b:ident)*) => {
        impl<$($T: DpIndex),*> DpIndex for ($($T,)*) {
            type Runtime = ($(<$T as DpIndex>::Runtime,)*);

            fn size(v: &Self::Runtime) -> usize {
                let ($($a,)*) = v;
                let mut size = 1;
                size *= 1;
                $(size *= <$T as DpIndex>::size($a);)*
                size
            }

            fn to_index(&self, v: &Self::Runtime) -> usize {
                let ($($a,)*) = self;
                let ($($b,)*) = v;
                let mut index = 0;
                index += 0;
                $(index *= <$T as DpIndex>::size($b);
                index += <$T as DpIndex>::to_index($a, $b);)*
                index
            }
        }

        dp_tuple!(@shrink $($T $a $b)*);
    };
}

dp_tuple!(A a1 a2 B b1 b2 C c1 c2 D d1 d2 E e1 e2 F f1 f2 G g1 g2 H h1 h2);

macro_rules! dp_as {
    (|$self:ident: $ty:ty| -> $as:ty $block:block) => {
        impl DpIndex for $ty {
            type Runtime = ();

            fn size(&(): &()) -> usize {
                <$as as DpIndex>::size(&Default::default())
            }

            fn to_index(&$self, &(): &()) -> usize {
                let val = $block;
                <$as as DpIndex>::to_index(&val, &Default::default())
            }
        }
    };
}

impl DpIndex for Axis {
    type Runtime = ();

    fn size(&(): &Self::Runtime) -> usize {
        3
    }

    fn to_index(&self, &(): &Self::Runtime) -> usize {
        *self as usize
    }
}

impl DpIndex for Z4 {
    type Runtime = ();

    fn size(&(): &Self::Runtime) -> usize {
        4
    }

    fn to_index(&self, &(): &Self::Runtime) -> usize {
        self.val() as usize
    }
}

impl DpIndex for bool {
    type Runtime = ();

    fn size(&(): &Self::Runtime) -> usize {
        2
    }

    fn to_index(&self, &(): &Self::Runtime) -> usize {
        usize::from(*self)
    }
}

impl DpIndex for usize {
    type Runtime = usize;

    fn size(v: &Self::Runtime) -> usize {
        *v
    }

    fn to_index(&self, v: &Self::Runtime) -> usize {
        debug_assert!(self < v);
        *self
    }
}

dp_as!(|self: Face| -> (Axis, bool) { (self.axis(), self.neg()) });
dp_as!(|self: AxialMove| -> (Axis, Z4, Z4) { (self.axis(), self.pos(), self.neg()) });

impl DpIndex for Rotation {
    type Runtime = ();

    fn size(&(): &Self::Runtime) -> usize {
        24
    }

    fn to_index(&self, &(): &Self::Runtime) -> usize {
        self.index() as usize
    }
}

pub struct DpArray<T, I: DpIndex> {
    inner: Vec<T>,
    v: I::Runtime,
    _phantom: PhantomData<fn(I) -> T>,
}

impl<T, I: DpIndex> DpArray<T, I> {
    pub fn new_with(v: I::Runtime, f: impl FnMut() -> T) -> Self
    where
        T: Default,
    {
        Self {
            inner: core::iter::repeat_with(f).take(I::size(&v)).collect(),
            v,
            _phantom: PhantomData,
        }
    }

    pub fn new(v: I::Runtime) -> Self
    where
        T: Default,
    {
        Self::new_with(v, Default::default)
    }

    pub fn get(&self, i: &I) -> &T {
        &self.inner[i.to_index(&self.v)]
    }

    pub fn get_mut(&mut self, i: &I) -> &mut T {
        &mut self.inner[i.to_index(&self.v)]
    }
}

impl<T, I: DpIndex> Index<I> for DpArray<T, I> {
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        &self[&index]
    }
}

impl<T, I: DpIndex> Index<&I> for DpArray<T, I> {
    type Output = T;

    fn index(&self, index: &I) -> &Self::Output {
        self.get(index)
    }
}

impl<T, I: DpIndex> IndexMut<I> for DpArray<T, I> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self[&index]
    }
}

impl<T, I: DpIndex> IndexMut<&I> for DpArray<T, I> {
    fn index_mut(&mut self, index: &I) -> &mut Self::Output {
        self.get_mut(index)
    }
}
