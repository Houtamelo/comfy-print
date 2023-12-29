#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg_attr(docsrs, doc(cfg(feature = "async_impl")))]
#[cfg(feature = "async_impl" )] pub mod async_impl;

#[cfg(not(feature = "async_impl"))] pub mod sync_impl;

pub mod utils;

