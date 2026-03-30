use core::ops::Range;

#[derive(Clone, Debug)]
pub enum VirtioInitInfo {
    Mmio {
        range: Range<usize>,
        irq: Option<u32>,
    },
    Pci {
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        irq: Option<u32>,
        legacy_io: Option<Range<usize>>,
        modern_common: Option<Range<usize>>,
        modern_notify: Option<Range<usize>>,
        modern_notify_off_multiplier: Option<u32>,
        modern_isr: Option<Range<usize>>,
        modern_device: Option<Range<usize>>,
    },
}

impl VirtioInitInfo {
    pub fn mmio(range: Range<usize>, irq: Option<u32>) -> Self {
        Self::Mmio { range, irq }
    }

    pub fn pci(
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        irq: Option<u32>,
        legacy_io: Option<Range<usize>>,
    ) -> Self {
        Self::Pci {
            segment,
            bus,
            device,
            function,
            irq,
            legacy_io,
            modern_common: None,
            modern_notify: None,
            modern_notify_off_multiplier: None,
            modern_isr: None,
            modern_device: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_modern_pci(
        mut self,
        modern_common: Option<Range<usize>>,
        modern_notify: Option<Range<usize>>,
        modern_notify_off_multiplier: Option<u32>,
        modern_isr: Option<Range<usize>>,
        modern_device: Option<Range<usize>>,
    ) -> Self {
        if let Self::Pci {
            modern_common: common,
            modern_notify: notify,
            modern_notify_off_multiplier: notify_mul,
            modern_isr: isr,
            modern_device: device,
            ..
        } = &mut self
        {
            *common = modern_common;
            *notify = modern_notify;
            *notify_mul = modern_notify_off_multiplier;
            *isr = modern_isr;
            *device = modern_device;
        }
        self
    }

    pub fn mmio_range(&self) -> Option<&Range<usize>> {
        match self {
            Self::Mmio { range, .. } => Some(range),
            Self::Pci { legacy_io, .. } => legacy_io.as_ref(),
        }
    }
}
