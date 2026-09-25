#![no_std]

pub mod rights;
pub mod objects;

pub use rights::Rights;
pub use objects::{ObjectId, ObjectType, KernelObject};