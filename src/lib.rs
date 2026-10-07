#![doc = include_str!("../README.md")]

#![no_std]

mod patch_field;

#[cfg(feature = "serde")]
mod serde;

pub use patch_field::PatchField;
