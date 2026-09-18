#![no_std]

pub mod dispatcher;
pub mod client;

pub use dispatcher::MessageDispatcher;
pub use client::ManagerClient;