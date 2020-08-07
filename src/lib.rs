extern crate byteorder;
extern crate uuid;
#[macro_use]
extern crate log;
extern crate crc;
extern crate fern;

pub mod disk;
mod error;
#[macro_use]
pub(crate) mod utils;

pub use error::*;

pub mod part;

#[cfg(test)]
extern crate better_panic;

#[cfg(test)]
pub(crate) fn tests_init() {
    better_panic::install();
}
