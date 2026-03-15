//! 架构抽象层。
//!
//! 仅支持 `riscv64` 与 `x86_64`。
//! 对外直接导出当前目标架构的实现与类型。

#![no_std]

#[cfg(not(any(target_arch = "riscv64", target_arch = "x86_64")))]
compile_error!("unsupported architecture: only riscv64 and x86_64 are supported");

#[cfg(target_arch = "riscv64")]
mod riscv;
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "riscv64")]
pub use riscv::*;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

/// 统一特权级别别名。
#[cfg(target_arch = "riscv64")]
pub type PrivilegedMode = SPP;

#[cfg(target_arch = "x86_64")]
pub type PrivilegedMode = PrivilegeLevel;

/// 统一处理器状态寄存器别名。
#[cfg(target_arch = "riscv64")]
pub type ProcessorStatus = ExtSstatus;

#[cfg(target_arch = "x86_64")]
pub type ProcessorStatus = Rflags;

/// 用户态特权级常量。
#[cfg(target_arch = "riscv64")]
pub const PRIVILEGE_USER: PrivilegedMode = SPP::User;

#[cfg(target_arch = "x86_64")]
pub const PRIVILEGE_USER: PrivilegedMode = PrivilegeLevel::User;

/// 内核态特权级常量。
#[cfg(target_arch = "riscv64")]
pub const PRIVILEGE_KERNEL: PrivilegedMode = SPP::Supervisor;

#[cfg(target_arch = "x86_64")]
pub const PRIVILEGE_KERNEL: PrivilegedMode = PrivilegeLevel::Kernel;
