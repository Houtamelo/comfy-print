#![doc = include_str!("../README.md")]
#![feature(doc_auto_cfg)]

#[cfg(feature = "async_impl" )] pub mod async_impl;
#[cfg(not(feature = "async_impl"))] pub mod sync_impl;

pub mod utils;

