//! Architecture abstraction traits
//!
//! This module provides unified trait interfaces for architecture-specific operations.
//! Each trait groups related operations and has conditional compilation implementations.
//!
//! # Design
//!
//! All operations are organized into traits, each representing a category:
//! - `CpuIf` - CPU identification
//! - `InterruptIf` - Interrupt control
//! - `TimeIf` - Time and counter operations
//! - `PagingIf` - Paging and TLB operations
//! - `MemoryAccessIf` - User memory access control
//! - `HaltIf` - Halt and wait operations
//! - `ProcessorStatusIf` - Processor status register operations
//! - `TrapContextIf` - Trap context operations
//!
//! # Usage
//!
//! ```ignore
//! use arch::traits::*;
//!
//! // Use Arch type for unified access
//! let cpu = Arch::cpu_id();
//! Arch::interrupt_enable();
//! let ticks = Arch::current_ticks();
//! Arch::flush_tlb_all();
//! ```

mod cpu;
mod halt;
mod interrupt;
mod memory;
mod paging;
mod processor_status;
mod time;
mod trap_context;

pub use cpu::CpuIf;
pub use halt::HaltIf;
pub use interrupt::InterruptIf;
pub use memory::MemoryAccessIf;
pub use paging::PagingIf;
pub use processor_status::ProcessorStatusIf;
pub use time::TimeIf;
pub use trap_context::TrapContextIf;

// ============================================================================
// Architecture marker type with trait implementations
// ============================================================================

/// Unified architecture type
///
/// This type implements all architecture traits with conditional compilation.
pub struct Arch;
