//! This crate defines system configuration parameters for Hopter embedded
//! operating system. Clients of Hopter can change them as needed.
//!
//! For the correctness of system functioning, make sure the following
//! parameter is correctly set.
//! - [`SYSTICK_FREQUENCY_HZ`] : Hopter depends on it to generate 1 millisecond
//!   interval ticks.
//!
//! Names that are prefixed with single underscore are considered semi-private.
//! One should not change it unless being familiar with Hopter kernel's source
//! code. Names that are prefixed with double underscores are considered
//! private. One should not change it unless the corresponding kernel code and
//! the compiler's source code are also changed accordingly.

#![no_std]

#[cfg(armv7em)]
mod armv7em;
#[cfg(armv7em)]
pub use armv7em::*;

#[cfg(armv6m)]
mod armv6m;
#[cfg(armv6m)]
pub use armv6m::*;
