//! Architecture abstraction layer
//!
//! This crate provides CPU-level architecture abstractions that allow code to work
//! with both RISC-V and x86-64 architectures.
//!
//! # Design Philosophy
//!
//! Each architecture uses its own native naming conventions:
//! - RISC-V: `SPP`, `ExtSstatus`, `hart_id`, `sstatus` semantics
//! - x86-64: `PrivilegeLevel`, `Rflags`, `cpu_id`, `rflags` semantics
//!
//! # Unified Interfaces
//!
//! The `traits` module provides unified trait interfaces via `Arch` type:
//! ```ignore
//! use arch::{Arch, CpuIf, InterruptIf, TimeIf, PagingIf};
//!
//! let cpu = Arch::cpu_id();
//! Arch::interrupt_enable();
//! let ticks = Arch::current_ticks();
//! Arch::flush_tlb_all();
//! ```
//!
//! # Architecture-Specific Types
//!
//! Use architecture-specific types directly when you need native semantics:
//! ```ignore
//! #[cfg(target_arch = "riscv64")]
//! use arch::{SPP, ExtSstatus};
//!
//! #[cfg(target_arch = "x86_64")]
//! use arch::{PrivilegeLevel, Rflags};
//! ```

#![no_std]

#[cfg(not(any(target_arch = "riscv64", target_arch = "x86_64")))]
compile_error!("unsupported architecture: only riscv64 and x86_64 are supported");

// ============================================================================
// Traits module (unified interface via Arch type)
// ============================================================================

pub mod traits;

// Re-export all traits and Arch type
pub use traits::{
    Arch,
    CpuIf, InterruptIf, TimeIf, PagingIf,
    MemoryAccessIf, HaltIf,
    ProcessorStatusIf, TrapContextIf,
};

// ============================================================================
// Architecture-specific module imports
// ============================================================================

#[cfg(target_arch = "riscv64")]
mod riscv;
#[cfg(target_arch = "x86_64")]
mod x86_64;

// Re-export all architecture-specific items
#[cfg(target_arch = "riscv64")]
pub use riscv::*;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

// ============================================================================
// Unified Type Aliases (for backward compatibility)
// ============================================================================

/// Unified privilege mode type alias
/// 
/// - RISC-V: `SPP` (Supervisor Previous Privilege)
/// - x86-64: `PrivilegeLevel` (CPL - Current Privilege Level)
#[cfg(target_arch = "riscv64")]
pub type PrivilegedMode = SPP;

#[cfg(target_arch = "x86_64")]
pub type PrivilegedMode = PrivilegeLevel;

/// Unified processor status register type alias
/// 
/// - RISC-V: `ExtSstatus` (sstatus register wrapper)
/// - x86-64: `Rflags` (RFLAGS register wrapper)
#[cfg(target_arch = "riscv64")]
pub type ProcessorStatus = ExtSstatus;

#[cfg(target_arch = "x86_64")]
pub type ProcessorStatus = Rflags;

// ============================================================================
// Unified Constants
// ============================================================================

/// User mode privilege constant
#[cfg(target_arch = "riscv64")]
pub const PRIVILEGE_USER: PrivilegedMode = SPP::User;

#[cfg(target_arch = "x86_64")]
pub const PRIVILEGE_USER: PrivilegedMode = PrivilegeLevel::User;

/// Kernel/Supervisor mode privilege constant
#[cfg(target_arch = "riscv64")]
pub const PRIVILEGE_KERNEL: PrivilegedMode = SPP::Supervisor;

#[cfg(target_arch = "x86_64")]
pub const PRIVILEGE_KERNEL: PrivilegedMode = PrivilegeLevel::Kernel;

// ============================================================================
// ProcessorStatusOps trait (backward compatibility alias)
// ============================================================================

/// Alias for ProcessorStatusIf (backward compatibility)
pub use traits::ProcessorStatusIf as ProcessorStatusOps;
