#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(any(feature = "utoipa", feature = "sqlx"))]
extern crate alloc;

mod delta;

#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "utoipa")]
mod utoipa;

#[cfg(feature = "sqlx")]
mod sqlx;

pub use delta::Delta;
#[cfg(feature = "sqlx")]
pub use sqlx::UnchangedDeltaError;
