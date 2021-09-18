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

    pub fn substract(&self, other: &Self) -> (Option<Self>, Option<Self>) {
        if (self.start == other.start && self.end == other.end) || self.belongs(other) {
            (None, None)
        } else if other.belongs(self) && self.start != other.start && self.end != other.end {
            let first = Region::new(self.start, other.start.sub(1u8.into()));
            let second = Region::new(other.end.add(1u8.into()), self.end);

            (Some(first), Some(second))
        } else if self.overlaps(other) {
            if other.start <= self.start {
                (Some(Region::new(other.end.add(1u8.into()), self.end)), None)
            } else if other.start >= self.start {
                (
                    Some(Region::new(self.start, other.start.sub(1u8.into()))),
                    None,
                )
            } else {
                unreachable!()
            }
        } else {
            (Some(*self), None)
        }
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

#[cfg(test)]
#[test]
fn test_region_substract() {
    macro_rules! test {
        ({$x0:expr, $x1:expr}, {$y0:expr, $y1:expr}, $expect:pat) => {{
            let r0 = Region::new($x0, $x1);
            let r1 = Region::new($y0, $y1);

            let r = r0.substract(&r1);
            assert!(matches!(r, $expect))
        }};
    }

    test!({100, 200}, {100, 200}, (None, None));

    test!({100, 200}, {140, 160}, (Some(Region { start: 100, end: 139 }), Some(Region { start: 161, end: 200 })));

    test!({100, 200}, {50, 99}, (Some(Region { start: 100, end: 200 }), None));
    test!({100, 200}, {201, 208}, (Some(Region { start: 100, end: 200 }), None));

    test!({100, 200}, {50, 105}, (Some(Region { start: 106, end: 200 }), None));
    test!({100, 200}, {50, 100}, (Some(Region { start: 101, end: 200 }), None));
    test!({100, 200}, {150, 500}, (Some(Region { start: 100, end: 149 }), None));
    test!({100, 200}, {200, 500}, (Some(Region { start: 100, end: 199 }), None));

    test!({100, 200}, {100, 100}, (Some(Region { start: 101, end: 200 }), None));
    test!({100, 200}, {200, 200}, (Some(Region { start: 100, end: 199 }), None));

    test!({100, 200}, {50, 250}, (None, None));
    test!({100, 200}, {100, 200}, (None, None));
}
