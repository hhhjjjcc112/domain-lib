use core::ops::Range;

use memory_addr::{PhysAddr, VirtAddr};

#[derive(Debug, Clone)]
pub struct SafeIORegion {
    range: Range<PhysAddr>,
}

impl From<Range<usize>> for SafeIORegion {
    fn from(value: Range<usize>) -> Self {
        let start = PhysAddr::from(value.start);
        let end = PhysAddr::from(value.end);
        Self { range: start..end }
    }
}

impl SafeIORegion {
    #[inline]
    fn is_port_io(&self) -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            let start = self.range.start.as_usize();
            let end = self.range.end.as_usize();
            return end <= 0x1_0000 && start < end;
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            false
        }
    }

    pub fn new(range: Range<PhysAddr>) -> Self {
        Self { range }
    }

    pub fn as_bytes(&self) -> &[u8] {
        let start = self.range.start.as_usize();
        unsafe { core::slice::from_raw_parts(start as *const u8, self.size()) }
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        let start = self.range.start.as_usize();
        unsafe { core::slice::from_raw_parts_mut(start as *mut u8, self.size()) }
    }

    pub fn read_at<T: Copy>(&self, offset: usize) -> Result<T, ()> {
        if offset + core::mem::size_of::<T>() > self.size() {
            return Err(());
        }
        if self.is_port_io() {
            #[cfg(target_arch = "x86_64")]
            {
                let port = (self.range.start.as_usize() + offset) as u16;
                let sz = core::mem::size_of::<T>();
                let mut out = core::mem::MaybeUninit::<T>::uninit();
                match sz {
                    1 => {
                        let val = unsafe { x86::io::inb(port) };
                        unsafe {
                            core::ptr::copy_nonoverlapping(
                                &val as *const u8,
                                out.as_mut_ptr() as *mut u8,
                                1,
                            );
                        }
                    }
                    2 => {
                        let val = unsafe { x86::io::inw(port) };
                        unsafe {
                            core::ptr::copy_nonoverlapping(
                                &val as *const u16 as *const u8,
                                out.as_mut_ptr() as *mut u8,
                                2,
                            );
                        }
                    }
                    4 => {
                        let val = unsafe { x86::io::inl(port) };
                        unsafe {
                            core::ptr::copy_nonoverlapping(
                                &val as *const u32 as *const u8,
                                out.as_mut_ptr() as *mut u8,
                                4,
                            );
                        }
                    }
                    _ => return Err(()),
                }
                return Ok(unsafe { out.assume_init() });
            }
        }
        let start = self.range.start.as_usize();
        let ptr = (start + offset) as *const T;
        unsafe { Ok(ptr.read_volatile()) }
    }

    pub fn write_at<T: Copy>(&self, offset: usize, value: T) -> Result<(), ()> {
        if offset + core::mem::size_of::<T>() > self.size() {
            return Err(());
        }
        if self.is_port_io() {
            #[cfg(target_arch = "x86_64")]
            {
                let port = (self.range.start.as_usize() + offset) as u16;
                let sz = core::mem::size_of::<T>();
                match sz {
                    1 => unsafe {
                        let mut raw = 0u8;
                        core::ptr::copy_nonoverlapping(
                            &value as *const T as *const u8,
                            &mut raw as *mut u8,
                            1,
                        );
                        x86::io::outb(port, raw);
                    },
                    2 => unsafe {
                        let mut raw = 0u16;
                        core::ptr::copy_nonoverlapping(
                            &value as *const T as *const u8,
                            &mut raw as *mut u16 as *mut u8,
                            2,
                        );
                        x86::io::outw(port, raw);
                    },
                    4 => unsafe {
                        let mut raw = 0u32;
                        core::ptr::copy_nonoverlapping(
                            &value as *const T as *const u8,
                            &mut raw as *mut u32 as *mut u8,
                            4,
                        );
                        x86::io::outl(port, raw);
                    },
                    _ => return Err(()),
                }
                return Ok(());
            }
        }
        let start = self.range.start.as_usize();
        let ptr = (start + offset) as *mut T;
        unsafe { ptr.write_volatile(value) }
        Ok(())
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.range.start
    }

    pub fn phys_addr_range(&self) -> Range<PhysAddr> {
        self.range.clone()
    }

    pub fn virt_addr(&self) -> VirtAddr {
        VirtAddr::from(self.range.start.as_usize())
    }

    pub fn size(&self) -> usize {
        self.range.end.as_usize() - self.range.start.as_usize()
    }
}
