//! Paging and TLB operations trait

use super::Arch;

/// Paging and TLB operations trait
///
/// Provides methods for page table management and TLB maintenance.
pub trait PagingIf {
    /// Activate page table with given root physical page number
    fn activate_paging_mode(root_ppn: usize);
    
    /// Flush all TLB entries
    fn flush_tlb_all();
    
    /// Flush TLB entry for a specific virtual address
    fn flush_tlb(vaddr: usize);
}

// ============================================================================
// RISC-V implementation
// ============================================================================

#[cfg(target_arch = "riscv64")]
impl PagingIf for Arch {
    #[inline]
    fn activate_paging_mode(root_ppn: usize) {
        crate::riscv::activate_paging_mode(root_ppn)
    }
    
    #[inline]
    fn flush_tlb_all() {
        crate::riscv::sfence_vma_all()
    }
    
    #[inline]
    fn flush_tlb(vaddr: usize) {
        crate::riscv::sfence_vma(vaddr)
    }
}

// ============================================================================
// x86-64 implementation
// ============================================================================

#[cfg(target_arch = "x86_64")]
impl PagingIf for Arch {
    #[inline]
    fn activate_paging_mode(root_ppn: usize) {
        crate::x86_64::activate_paging_mode(root_ppn)
    }
    
    #[inline]
    fn flush_tlb_all() {
        crate::x86_64::sfence_vma_all()
    }
    
    #[inline]
    fn flush_tlb(vaddr: usize) {
        crate::x86_64::sfence_vma(vaddr)
    }
}
