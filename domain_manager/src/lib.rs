#![feature(allocator_api)]
#![no_std]
extern crate alloc;

pub mod resource;
pub mod sheap;
pub mod storage_heap;

pub const FRAME_SIZE: usize = 4096;

pub const FRAME_BITS: usize = 12;
