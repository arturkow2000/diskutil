#![feature(repr128)]

extern crate byteorder;
extern crate uuid;
#[macro_use]
extern crate log;
extern crate crc;
extern crate fern;
extern crate uuid_macros;

pub mod disk;
mod error;
#[macro_use]
pub(crate) mod utils;
pub mod fs;

pub use error::*;

pub mod part;
pub mod region;

#[cfg(test)]
extern crate better_panic;

#[cfg(test)]
pub(crate) fn tests_init() {
    better_panic::install();
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use uuid::Uuid;
    use uuid_macros::{uuid, uuid_u128};

    #[test]
    fn test_uuid() {
        macro_rules! test {
            ($s:expr) => {{
                assert_eq!(uuid! {$s}, Uuid::from_str($s).unwrap());
                assert_eq!(uuid_u128! {$s}, Uuid::from_str($s).unwrap().as_u128());
            }};
        }

        test! {"EBD0A0A2-B9E5-4433-87C0-68B6B72699C7"};
    }
}
