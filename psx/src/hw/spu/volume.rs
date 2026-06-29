//! SPU volume module

/// Volume sweep mode.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SweepMode {
    /// The volume sweeps linearly
    Linear = 0,
    /// The volume sweeps exponentially
    Exponential = 1,
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

/// Volume sweep phase
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SweepPhase {
    /// The volume sweeps normally (increase => +0x7FFF, decrease => 0x0000)
    Positive = 0,
    /// The volume sweeps from the negative sign (increase => -0x7FFF, decrease
    /// => 0x0000)
    Negative = 1,
}

/// The volume sweep structure.
pub struct VolumeSweep {
    mode: SweepMode,
    direction: SweepDirection,
    phase: SweepPhase,
    shift: u8,
    step: u8,
}

/// SPU volume setting.
pub enum Volume {
    /// Normal volume.
    Normal(i16),
    /// Sweeping volume.
    Sweep(VolumeSweep),
}

fn parse_sweep(value: u16) -> VolumeSweep {
    VolumeSweep {
        mode: match (value >> 14) & 1 {
            0 => SweepMode::Linear,
            1 => SweepMode::Exponential,
            _ => unreachable!(),
        },

        direction: match (value >> 13) & 1 {
            0 => SweepDirection::Increase,
            1 => SweepDirection::Decrease,
            _ => unreachable!(),
        },

        phase: match (value >> 12) & 1 {
            0 => SweepPhase::Positive,
            1 => SweepPhase::Negative,
            _ => unreachable!(),
        },

        shift: ((value >> 2) & 0x1F) as u8,
        step: (value & 3) as u8,
    }
}

fn encode_sweep(sweep: &VolumeSweep) -> u16 {
    0x8000 |
        ((sweep.mode as u16) << 14) |
        ((sweep.direction as u16) << 13) |
        ((sweep.phase as u16) << 12) |
        ((sweep.shift as u16) << 2) |
        (sweep.step as u16)
}

impl From<u16> for Volume {
    fn from(value: u16) -> Self {
        match value >> 15 {
            // Sign-extend the 15-bit signed volume
            0 => Volume::Normal(((value << 1) as i16) >> 1),
            1 => Volume::Sweep(parse_sweep(value)),
            _ => unreachable!(),
        }
    }
}

impl From<&Volume> for u16 {
    fn from(val: &Volume) -> Self {
        match val {
            Volume::Normal(vol) => (*vol as u16) & 0x7FFF,
            Volume::Sweep(sweep) => encode_sweep(sweep),
        }
    }
}
