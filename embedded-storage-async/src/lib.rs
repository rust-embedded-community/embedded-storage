//! # embedded-storage-async - An async Storage Abstraction Layer for Embedded Systems
//!
//! Storage traits to allow on and off board storage devices to read and write
//! data asynchronously.

#![no_std]
#![allow(async_fn_in_trait)]

pub mod nor_flash;
