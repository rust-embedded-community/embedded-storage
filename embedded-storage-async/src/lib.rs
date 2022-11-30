//! # embedded-storage-async - An async Storage Abstraction Layer for Embedded Systems
//!
//! Storage traits to allow on and off board storage devices to read and write
//! data asynchronously.

#![no_std]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]

pub mod nor_flash;
