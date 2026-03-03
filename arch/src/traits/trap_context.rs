//! Trap context operations trait

/// Trap context operations trait
///
/// Provides methods for creating and manipulating trap contexts (register state).
pub trait TrapContextIf: Sized {
    /// Create a new trap context for a user task
    fn new_user(entry: usize, sp: usize, tls: usize) -> Self;
    
    /// Create a new trap context for a kernel task
    fn new_kernel(entry: usize, sp: usize, arg: usize) -> Self;
    
    /// Get program counter (return address)
    fn get_pc(&self) -> usize;
    
    /// Set program counter
    fn set_pc(&mut self, pc: usize);
    
    /// Get stack pointer
    fn get_sp(&self) -> usize;
    
    /// Set stack pointer
    fn set_sp(&mut self, sp: usize);
    
    /// Get return value (a0/rax)
    fn get_ret(&self) -> usize;
    
    /// Set return value
    fn set_ret(&mut self, val: usize);
    
    /// Get argument 0
    fn get_arg0(&self) -> usize;
    
    /// Get argument 1
    fn get_arg1(&self) -> usize;
    
    /// Get argument 2
    fn get_arg2(&self) -> usize;
    
    /// Set argument 0
    fn set_arg0(&mut self, val: usize);
    
    /// Set TLS pointer
    fn set_tls(&mut self, tls: usize);
}
