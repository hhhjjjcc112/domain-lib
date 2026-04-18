//! 配置文件
#![no_std]

// Two-layer cfg guards: architecture + platform.
#[cfg(not(any(target_arch = "riscv64", target_arch = "x86_64")))]
compile_error!("Unsupported architecture. Expected target_arch = riscv64 or x86_64");

#[cfg(not(any(plat_qemu_riscv, plat_vf2, plat_qemu_x86_64)))]
compile_error!("No valid platform selected! Use --cfg plat_qemu_riscv, --cfg plat_vf2, or --cfg plat_qemu_x86_64");

#[cfg(any(
     all(plat_qemu_riscv, plat_vf2),
     all(plat_qemu_riscv, plat_qemu_x86_64),
     all(plat_vf2, plat_qemu_x86_64)
))]
compile_error!("Multiple platforms selected! Select exactly one platform cfg");

#[cfg(all(target_arch = "x86_64", not(plat_qemu_x86_64)))]
compile_error!("ARCH x86_64 requires PLATFORM=plat_qemu_x86_64");

#[cfg(all(target_arch = "riscv64", not(any(plat_qemu_riscv, plat_vf2))))]
compile_error!("ARCH riscv64 requires PLATFORM=plat_qemu_riscv or plat_vf2");

#[cfg(plat_qemu_riscv)]
mod qemu_riscv;
#[cfg(plat_vf2)]
mod vf2;
#[cfg(plat_qemu_x86_64)]
mod qemu_x86_64;

#[cfg(plat_qemu_riscv)]
pub use qemu_riscv::*;
#[cfg(plat_vf2)]
pub use vf2::*;
#[cfg(plat_qemu_x86_64)]
pub use qemu_x86_64::*;

/// Alien os的标志
pub const ALIEN_FLAG: &str = r"
     _      _   _
    / \    | | (_)   ___   _ __
   / _ \   | | | |  / _ \ | '_ \
  / ___ \  | | | | |  __/ | | | |
 /_/   \_\ |_| |_|  \___| |_| |_|
";

/// 物理页大小
pub const FRAME_SIZE: usize = 0x1000;
pub const PAGE_SIZE: usize = FRAME_SIZE;
/// 物理页大小的位数
pub const FRAME_BITS: usize = 12;
/// 内核启动栈大小
pub const STACK_SIZE: usize = 1024 * 64;
/// 内核启动栈大小的位数
pub const STACK_SIZE_BITS: usize = 16;

/// 可配置的启动cpu数量
pub const CPU_NUM: usize = 4;

#[cfg(target_arch = "riscv64")]
const HEAP_SIZE: usize = 0x400_0000;
// x86_64 下域镜像更大，提升内核堆避免早期 OOM。
#[cfg(target_arch = "x86_64")]
const HEAP_SIZE: usize = 0x1000_0000;
pub const KERNEL_HEAP_SIZE: usize = HEAP_SIZE;

pub const TRAMPOLINE: usize = usize::MAX - 2 * FRAME_SIZE + 1;

pub const TRAP_CONTEXT_BASE: usize = TRAMPOLINE - FRAME_SIZE;
#[cfg(target_arch = "x86_64")]
pub const PERCPU_MIRROR_BASE: usize = TRAMPOLINE - 0x1_0000_0000_00;
#[cfg(target_arch = "x86_64")]
pub const LOW_PHYS_MAP_BASE: usize = PERCPU_MIRROR_BASE + 0x20_0000;
#[cfg(target_arch = "x86_64")]
pub const LOW_PHYS_MAP_SIZE: usize = 0x10_0000;
pub const USER_KERNEL_STACK_SIZE: usize = FRAME_SIZE * 16;
pub const KTHREAD_STACK_SIZE: usize = FRAME_SIZE * 2;
/// 线程数量大小限制
pub const MAX_THREAD_NUM: usize = 65536;
/// 描述符数量大小限制
pub const MAX_FD_NUM: usize = 4096;

/// app用户栈大小
// pub const USER_STACK_SIZE: usize = 0x50_000;

pub const USER_STACK_SIZE: usize = 0x50_000;
pub const ELF_BASE_RELOCATE: usize = 0x400_0000;

pub const MAX_INPUT_EVENT_NUM: u32 = 1024;
pub const PROCESS_HEAP_MAX: usize = u32::MAX as usize + 1;

pub const PIPE_BUF: usize = 65536;
