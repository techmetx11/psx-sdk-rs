//! SPU volume module

use bitfield_struct::bitfield;

/// Volume sweep mode.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SweepMode {
    /// The volume sweeps linearly
    Linear = 0,
    /// The volume sweeps exponentially
    Exponential = 1,
}

impl SweepMode {
    const fn into_bits(self) -> u8 {
        self as _
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0 => Self::Linear,
            1 => Self::Exponential,
            _ => unreachable!(),
        }
    }
}

/// Volume direction sweep
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SweepDirection {
    /// The volume increases up to `+0x7FFF`
    Increase = 0,
    /// The volume decreases down to `0x0000`
    Decrease = 1,
}

impl SweepDirection {
    const fn into_bits(self) -> u8 {
        self as _
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0 => Self::Increase,
            1 => Self::Decrease,
            _ => unreachable!(),
        }
    }
}

/// Volume sweep phase
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SweepPhase {
    /// The volume sweeps normally (increase => +0x7FFF, decrease => 0x0000)
    Positive = 0,
    /// The volume sweeps from the negative sign (increase => -0x7FFF, decrease => 0x0000)
    Negative = 1,
}

impl SweepPhase {
    const fn into_bits(self) -> u8 {
        self as _
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0 => Self::Positive,
            1 => Self::Negative,
            _ => unreachable!(),
        }
    }
}

/// The volume sweep structure.
#[bitfield(u16, order = Msb)]
pub struct VolumeSweep {
    #[bits(1)]
    _type: usize,

    #[bits(1)]
    pub mode: SweepMode,

    #[bits(1)]
    pub direction: SweepDirection,

    #[bits(1)]
    pub phase: SweepPhase,

    #[bits(5)]
    _zero: usize,

    #[bits(5)]
    pub shift: usize,

    #[bits(2)]
    pub step: usize,
}

/// SPU volume setting.
#[derive(Clone, Copy)]
pub enum Volume {
    /// Normal volume.
    Normal(i16),
    /// Sweeping volume.
    Sweep(VolumeSweep),
}

impl From<u16> for Volume {
    fn from(value: u16) -> Self {
        match value >> 15 {
            // Sign-extend the 15-bit signed volume
            0 => Volume::Normal(((value << 1) as i16) >> 1),
            1 => Volume::Sweep(value.into()),
            _ => unreachable!(),
        }
    }
}

impl From<Volume> for u16 {
    fn from(val: Volume) -> Self {
        match val {
            Volume::Normal(vol) => (vol as u16) & 0x7FFF,
            Volume::Sweep(sweep) => sweep.into_bits() | 0x8000,
        }
    }
}
