use core::ops::Range;

#[derive(Debug, Clone)]
pub struct SafePort {
    range: Range<u16>,
}

pub trait PortValue: Copy {
    fn size() -> usize;
    fn read(port: u16) -> Self;
    fn write(port: u16, value: Self);
}

impl PortValue for u8 {
    #[inline]
    fn size() -> usize {
        1
    }

    #[inline]
    fn read(port: u16) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            // x86_64 PIO 读取。
            unsafe { x86::io::inb(port) }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = port;
            0
        }
    }

    #[inline]
    fn write(port: u16, value: Self) {
        #[cfg(target_arch = "x86_64")]
        {
            // x86_64 PIO 写入。
            unsafe { x86::io::outb(port, value) }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = (port, value);
        }
    }
}

impl PortValue for u16 {
    #[inline]
    fn size() -> usize {
        2
    }

    #[inline]
    fn read(port: u16) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            unsafe { x86::io::inw(port) }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = port;
            0
        }
    }

    #[inline]
    fn write(port: u16, value: Self) {
        #[cfg(target_arch = "x86_64")]
        {
            unsafe { x86::io::outw(port, value) }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = (port, value);
        }
    }
}

impl PortValue for u32 {
    #[inline]
    fn size() -> usize {
        4
    }

    #[inline]
    fn read(port: u16) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            unsafe { x86::io::inl(port) }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = port;
            0
        }
    }

    #[inline]
    fn write(port: u16, value: Self) {
        #[cfg(target_arch = "x86_64")]
        {
            unsafe { x86::io::outl(port, value) }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = (port, value);
        }
    }
}

impl SafePort {
    pub fn new(range: Range<u16>) -> Result<Self, ()> {
        if range.start >= range.end {
            return Err(());
        }
        Ok(Self { range })
    }

    pub fn from_usize_range(range: Range<usize>) -> Result<Self, ()> {
        if range.start >= range.end || range.end > 0x1_0000 {
            return Err(());
        }
        let start = u16::try_from(range.start).map_err(|_| ())?;
        let end = u16::try_from(range.end).map_err(|_| ())?;
        Self::new(start..end)
    }

    pub fn read_at<T: PortValue>(&self, offset: usize) -> Result<T, ()> {
        if !self.check_bounds(offset, T::size()) {
            return Err(());
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            return Err(());
        }
        #[cfg(target_arch = "x86_64")]
        {
            let port = self
                .range
                .start
                .checked_add(u16::try_from(offset).map_err(|_| ())?)
                .ok_or(())?;
            Ok(T::read(port))
        }
    }

    pub fn write_at<T: PortValue>(&self, offset: usize, value: T) -> Result<(), ()> {
        if !self.check_bounds(offset, T::size()) {
            return Err(());
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = value;
            return Err(());
        }
        #[cfg(target_arch = "x86_64")]
        {
            let port = self
                .range
                .start
                .checked_add(u16::try_from(offset).map_err(|_| ())?)
                .ok_or(())?;
            T::write(port, value);
            Ok(())
        }
    }

    pub fn size(&self) -> usize {
        usize::from(self.range.end - self.range.start)
    }

    pub fn port_range(&self) -> Range<u16> {
        self.range.clone()
    }

    #[inline]
    fn check_bounds(&self, offset: usize, width: usize) -> bool {
        offset
            .checked_add(width)
            .is_some_and(|end| end <= self.size())
    }
}

#[cfg(test)]
mod tests {
    use super::SafePort;

    #[test]
    fn from_usize_range_valid() {
        let port = SafePort::from_usize_range(0x3f8..0x400).expect("valid port range");
        assert_eq!(port.size(), 8);
        assert_eq!(port.port_range().start, 0x3f8);
        assert_eq!(port.port_range().end, 0x400);
    }

    #[test]
    fn from_usize_range_reject_invalid_bounds() {
        assert!(SafePort::from_usize_range(0x3f8..0x3f8).is_err());
        assert!(SafePort::from_usize_range(0x400..0x3f8).is_err());
    }

    #[test]
    fn from_usize_range_reject_out_of_port_space() {
        assert!(SafePort::from_usize_range(0x1_0000..0x1_0001).is_err());
        assert!(SafePort::from_usize_range(0xfffe..0x1_0001).is_err());
    }

    #[test]
    fn read_write_reject_out_of_bounds_without_hardware_io() {
        let port = SafePort::from_usize_range(0x3f8..0x400).expect("valid port range");
        // 只验证边界拒绝路径，避免触发真实端口 I/O。
        assert!(port.read_at::<u8>(8).is_err());
        assert!(port.write_at(8, 0u8).is_err());
        assert!(port.read_at::<u16>(7).is_err());
        assert!(port.write_at(7, 0u16).is_err());
    }
}
