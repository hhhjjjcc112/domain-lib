//! QEMU x86-64 platform configuration
//!
//! Platform-specific constants for x86-64 QEMU virtual machine.

// Timer Configuration

/// TSC frequency in Hz (will be calibrated at runtime)
/// This is a fallback value for 4GHz CPU
pub const CLOCK_FREQ: usize = 4_000_000_000;

/// Nanoseconds per second
pub const NANOS_PER_SEC: usize = 1_000_000_000;

/// Microseconds per second
pub const MICROS_PER_SEC: usize = 1_000_000;

/// Milliseconds per second
pub const MILLIS_PER_SEC: usize = 1_000;

// Memory Configuration

/// Physical-to-virtual offset (high half kernel)
pub const PHYS_VIRT_OFFSET: usize = 0xFFFF_8000_0000_0000;

/// Kernel load address (physical)
pub const KERNEL_LOAD_PADDR: usize = 0x20_0000;

/// Kernel stack size
pub const BOOT_STACK_SIZE: usize = 0x8000; // 32KB

// APIC Configuration

/// Local APIC base physical address (default, can be remapped)
pub const LOCAL_APIC_BASE: usize = 0xFEE0_0000;

/// IO APIC base physical address
pub const IO_APIC_BASE: usize = 0xFEC0_0000;

/// APIC timer vector
pub const APIC_TIMER_VECTOR: u8 = 0xF0;

/// APIC spurious interrupt vector
pub const APIC_SPURIOUS_VECTOR: u8 = 0xF1;

/// APIC error interrupt vector
pub const APIC_ERROR_VECTOR: u8 = 0xF2;

// Interrupt Vectors

/// First IRQ vector (after CPU exceptions)
pub const IRQ_BASE_VECTOR: u8 = 32;

/// System call interrupt vector
pub const SYSCALL_VECTOR: u8 = 0x80;

// Serial/Console Configuration

/// COM1 port address
pub const COM1_PORT: u16 = 0x3F8;

/// COM2 port address
pub const COM2_PORT: u16 = 0x2F8;

// MMIO Ranges

/// MMIO ranges for x86-64 QEMU (APIC, IOAPIC, PCI config, etc.)
pub const MMIO_RANGES: [(usize, usize); 3] = [
    (0xFEC0_0000, 0x1000),   // IO APIC
    (0xFEE0_0000, 0x1000),   // Local APIC
    (0xE000_0000, 0x1000_0000), // PCI MMIO (typical QEMU mapping)
];

