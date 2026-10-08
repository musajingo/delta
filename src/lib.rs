#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(feature = "utoipa")]
extern crate alloc;

mod patch_field;

#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "utoipa")]
mod utoipa;

pub use patch_field::PatchField;
