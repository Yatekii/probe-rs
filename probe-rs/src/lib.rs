#![allow(clippy::useless_let_if_seq)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::verbose_bit_mask)]

#[macro_use]
pub extern crate derivative;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate serde_derive;

pub mod config;
pub mod cores;
pub mod coresight;
pub mod debug;
pub mod flash;
pub mod probe;
pub mod session;
pub mod target;
