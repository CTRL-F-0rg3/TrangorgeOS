#![no_std]

pub mod alloc;
pub mod dma;
// pub mod vmm; // Do dodania później, jeśli sterownik potrzebuje własnych operacji na stronach

pub use dma::{MmioRegion, DmaBuffer, DmaFlags};