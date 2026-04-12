use core::ptr::{addr_of, addr_of_mut};

#[repr(C)]
pub(crate) struct VolatileU16 {
    value: u16,
}

impl VolatileU16 {
    #[inline]
    pub fn get(&self) -> u16 {
        unsafe { core::ptr::read_volatile(addr_of!(self.value)) }
    }

    #[inline]
    pub fn set(&mut self, value: u16) {
        unsafe {
            core::ptr::write_volatile(addr_of_mut!(self.value), value);
        }
    }

    #[inline]
    pub fn set_bit(&mut self, index: u16, value: bool) {
        let bit = if value { 1 } else { 0 } << index;

        self.set((self.get() & !(1 << index)) | bit);
    }

    #[inline]
    pub fn get_bit(&self, index: u16) -> bool {
        (self.get() & (1 << index)) != 0
    }
}

#[repr(C)]
pub(crate) struct VolatileU32 {
    low: u16,
    high: u16,
}

impl VolatileU32 {
    #[inline]
    pub fn get(&self) -> u32 {
        unsafe {
            (core::ptr::read_volatile(addr_of!(self.low)) as u32)
                | ((core::ptr::read_volatile(addr_of!(self.high)) as u32) << 16)
        }
    }

    #[inline]
    pub fn set(&mut self, value: u32) {
        unsafe {
            core::ptr::write_volatile(addr_of_mut!(self.low), value as u16);
            core::ptr::write_volatile(addr_of_mut!(self.high), value.unbounded_shr(16) as u16);
        }
    }

    #[inline]
    pub fn set_bit(&mut self, index: u16, value: bool) {
        let bit = if value { 1 } else { 0 } << index;

        self.set((self.get() & !(1 << index)) | bit);
    }

    #[inline]
    pub fn get_bit(&self, index: u16) -> bool {
        (self.get() & (1 << index)) != 0
    }
}
