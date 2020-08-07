#[macro_export]
macro_rules! is_power_of_2 {
    ($x:expr) => {
        ($x) != 0 && ($x) & (($x) - 1) == 0
    };
}
#[macro_export]
macro_rules! round_up {
    ($x:expr, $y:expr) => {{
        debug_assert!(is_power_of_2!($y));
        (($x) + ($y) - 1) & (!($y) + 1)
    }};
}

#[cfg(test)]
#[test]
fn test_round_up() {
    crate::tests_init();

    assert_eq!(round_up!(54, 512), 512);
    assert_eq!(round_up!(513, 512), 1024);
    assert_eq!(round_up!(16384, 512), 16384);
}

#[cfg(test)]
#[test]
fn test_is_power_of_2() {
    crate::tests_init();

    assert!(!is_power_of_2!(0));
    assert!(!is_power_of_2!(7));
    assert!(is_power_of_2!(8));
    assert!(!is_power_of_2!(63));
    assert!(is_power_of_2!(64));
    assert!(!is_power_of_2!(65));
    assert!(is_power_of_2!(9223372036854775808u64));
}

pub fn allocate_u8_vector_uninitialized(capacity: usize) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::with_capacity(capacity);
    unsafe { v.set_len(capacity) };
    v
}

#[inline]
pub fn zero_u8_slice(s: &mut [u8]) {
    // https://github.com/rust-lang/rfcs/issues/2067
    use std::ptr;
    let len = s.len();

    unsafe { ptr::write_bytes(s.as_mut_ptr(), 0, len) }
}

#[macro_export]
macro_rules! u8_array_uninitialized {
    ($s:expr) => {
        unsafe { ::std::mem::MaybeUninit::<[u8; $s]>::uninit().assume_init() }
    };
}
