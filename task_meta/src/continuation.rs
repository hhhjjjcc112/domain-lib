#[repr(C)]
#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct Continuation {
    // all registers
    pub regs: [usize; 32],
    // function ptr
    pub func: usize,
}
