//! `msf` is a *Multi-channel signed distance field generator* for fonts.
//!
//! This crate is #![no_std]-compatibne, when the "no-std" feature is activated,
//! but still requires the alloc crate.

#![cfg_attr(feature = "no-std", no_std)]

extern crate alloc;

#[cfg(feature = "ttf-parser")]
mod font;
mod math;
pub mod raster;
mod scanline;
pub mod shape;
