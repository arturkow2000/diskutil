use std::fmt;
use std::ops::{Add, Sub};

#[derive(Debug, Copy, Clone)]
pub struct Region<T>
where
    T: Add<Output = T> + Sub<Output = T> + Ord + From<u8> + Copy,
{
    start: T,
    end: T,
}

impl<T> Region<T>
where
    T: Add<Output = T> + Sub<Output = T> + Ord + From<u8> + Copy,
{
    pub fn new(start: T, end: T) -> Self {
        assert!(end >= start);
        Self { start, end }
    }
    pub fn new_with_size(start: T, size: T) -> Self {
        Self {
            start,
            end: start + size.sub(1u8.into()),
        }
    }
    pub fn overlaps(&self, other: &Self) -> bool {
        !(self.end < other.start || self.start > other.end)
    }
    pub fn belongs(&self, other: &Self) -> bool {
        self.start >= other.start && self.end <= other.end
    }

    #[inline]
    pub fn start(&self) -> T {
        self.start
    }

    #[inline]
    pub fn end(&self) -> T {
        self.end
    }

    #[inline]
    pub fn size(&self) -> T {
        (self.end - self.start).add(1u8.into())
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size() == 0u8.into()
    }
}

impl<T> fmt::Display for Region<T>
where
    T: Add<Output = T> + Sub<Output = T> + Ord + From<u8> + Copy + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{} - {}}}", self.start, self.end)
    }
}

#[cfg(test)]
#[test]
fn test_region_overlap() {
    macro_rules! test {
        (false, $r0:expr, $r1:expr) => {{
            assert!(!$r0.overlaps(&$r1));
            assert!(!$r1.overlaps(&$r0));
        }};
        (true, $r0:expr, $r1:expr) => {{
            assert!($r0.overlaps(&$r1));
            assert!($r1.overlaps(&$r0));
        }};
    }

    test!(false, Region::new(0, 4), Region::new(8, 20));
    test!(true, Region::new(0, 4), Region::new(4, 4));
    test!(true, Region::new(8, 524), Region::new(4, 22));
}

#[cfg(test)]
#[test]
fn test_region_belongs() {
    macro_rules! test {
        ($cond:ident, {$x0:expr, $x1:expr}, {$y0:expr, $y1:expr}) => {{
            let r0 = Region::new($x0, $x1);
            let r1 = Region::new($y0, $y1);
            assert!(r1.belongs(&r0) == $cond);
        }};
    }

    test!(true, {500, 10000}, {600, 5021});
    test!(true, {20, 20}, {20, 20});
    test!(true, {10, 40}, {10, 24});
    test!(true, {10, 40}, {20, 40});
    test!(true, {0, 4}, {4, 4});

    test!(false, {10, 40}, {9, 40});
    test!(false, {0, 4}, {4, 5});
}
