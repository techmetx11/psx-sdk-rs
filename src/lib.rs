#![no_std]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod memory;
pub mod volume;

use core::ops::Range;

use crate::{
    memory::{VolatileU16, VolatileU32},
    volume::Volume,
};
use bitfield_struct::bitfield;

// This crate is potentially unsafe in other platforms, So we have to stop the compilation if we
// detect that the compiler is not targetting the PS1
#[cfg(not(target_os = "psx"))]
compile_error!(
    "This crate is meant to be compiled for the PlayStation 1, and cannot be used anywhere else."
);

const SPU_CHANNELS: usize = 24;

const SPU_CHANNEL_REGS: *mut SpuChannelRegs = 0x1F80_1C00 as *mut SpuChannelRegs;

const SPU_KEYON: *mut VolatileU32 = 0x1F80_1D88 as *mut VolatileU32;
const SPU_KEYOFF: *mut VolatileU32 = 0x1F80_1D8C as *mut VolatileU32;
const SPU_NON: *mut VolatileU32 = 0x1F80_1D94 as *mut VolatileU32;
const SPU_PMON: *mut VolatileU32 = 0x1F80_1D90 as *mut VolatileU32;
const SPU_CNT: *mut VolatileU16 = 0x1F80_1DAA as *mut VolatileU16;

const SPU_MVOLL: *mut VolatileU16 = 0x1F80_1D80 as *mut VolatileU16;
const SPU_MVOLR: *mut VolatileU16 = 0x1F80_1D82 as *mut VolatileU16;

/// The SPU structure.
pub struct Spu;

#[derive(Debug)]
#[repr(u8)]
enum SpuRamTransfer {
    Stop,
    ManualWrite,
    DMAWrite,
    DMARead,
}

impl SpuRamTransfer {
    const fn into_bits(self) -> u8 {
        self as _
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0 => Self::Stop,
            1 => Self::ManualWrite,
            2 => Self::DMAWrite,
            3 => Self::DMARead,
            _ => unreachable!(),
        }
    }
}

#[bitfield(u16, order = Msb)]
struct SpuControlRegs {
    #[bits(1)]
    enable: bool,

    #[bits(1)]
    mute: bool,

    #[bits(4)]
    noise_freq_shift: usize,

    #[bits(2)]
    noise_freq_step: usize,

    #[bits(1)]
    reverb_master: bool,

    #[bits(1)]
    irq9: bool,

    #[bits(2)]
    ram_transfer_mode: SpuRamTransfer,

    #[bits(1)]
    ext_reverb: bool,

    #[bits(1)]
    cdda_reverb: bool,

    #[bits(1)]
    ext_enable: bool,

    #[bits(1)]
    cdda_enable: bool,
}

#[repr(C)]
struct SpuChannelRegs {
    volume_left: VolatileU16,
    volume_right: VolatileU16,
    frequency: VolatileU16,
    sample_start: VolatileU16,
    adsr: VolatileU32,
    adsr_volume: VolatileU16,
    sample_repeat: VolatileU16,
}

struct SpuChannel {
    regs: &mut SpuChannelRegs,
    num: usize,
}

impl SpuChannel {
    /// Resets a channel
    pub fn reset(&self) {
        self.frequency(0);
        self.volume(Volume::Normal(0));
        self.sample_start(0);
        self.key_off();
        self.pitch_mod(false);
        self.noise(false);
    }

    /// Sets the left volume of a channel.
    pub fn volume_left(&self, vol: Volume) {
        let vol_bits: u16 = vol.into();

        self.regs.volume_left.set(vol_bits);
    }

    /// Sets the right volume of a channel.
    pub fn volume_right(&self, vol: Volume) {
        let vol_bits: u16 = vol.into();

        self.regs.volume_right.set(vol_bits);
    }

    /// Sets the volume (both left/right) of a channel.
    pub fn volume(&self, vol: Volume) {
        self.volume_left(vol);
        self.volume_right(vol);
    }

    /// Sets the address of the sample the channel should be playing off of.
    pub fn sample_start(&self, mut sample: u32) {
        if sample > 1 << 19 {
            panic!("Sample address is bigger than the maximum addressable address in the SPU");
        }

        // In the SPU, samples are indexed by 8-byte units.
        sample >>= 4;

        self.regs.sample_start.set(sample as u16);
    }

    /// Sets the ADPCM sample rate of the channel to the specified frequency (0x1000 == 441000Hz).
    ///
    /// Note: This does not affect the frequency of the channel if noise mode is active on it. For
    /// that, you should check out [`Self::noise_settings`]
    pub fn frequency(&self, frequency: u16) {
        self.regs.frequency.set(frequency);
    }

    /// Starts the ADSR envelope and automatically initializes the ADSR volume to zero
    pub fn key_on(&self) {
        unsafe {
            (*SPU_KEYON).set_bit(self.num as u16, true);
        }
    }

    /// Releases the key in the channel, which starts the Release stage of the ADSR envelope, if
    /// set.
    pub fn key_off(&self) {
        unsafe {
            (*SPU_KEYOFF).set_bit(self.num as u16, true);
        }
    }

    /// Enable or disable noise mode on a specific channel. If enabled, the channel will stop
    /// outputting ADPCM samples and instead output noise samples from the SPU's Noise Generator.
    ///
    /// The Noise Generator can be configured, using the [`Self::noise_settings`] function.
    pub fn noise(&self, enable: bool) {
        unsafe {
            (*SPU_NON).set_bit(self.num as u16, enable);
        }
    }

    /// Enables or disables pitch modulation of the specified channel from the amplitude of the
    /// previous channel.
    ///
    /// Note: Setting pitch modulation on channel 0 will do nothing, as there is no previous
    /// channel.
    pub fn pitch_mod(&self, enable: bool) {
        unsafe {
            (*SPU_PMON).set_bit(self.num as u16, enable);
        }
    }
}

struct ChannelIterator {
    spu: &Spu,
    channels: Range<usize>,
}

impl Iterator for ChannelIterator {
    type Item = SpuChannel;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(num) = self.channels.next() {
            // SAFETY: The channel iterator is always initialized from [`Spu::channels()`], with
            // the range being set from 0 to [`SPU_CHANNELS`]
            unsafe { Some(self.spu.unchecked_channel(num)) }
        } else {
            None
        }
    }
}
impl ExactSizeIterator for ChannelIterator {
    fn len(&self) -> usize {
        self.channels.len()
    }
}

impl Spu {
    /// Initializes the SPU to default values and returns a [`Spu`] structure.
    pub fn new() -> Self {
        let spu = Spu;

        spu.noise_settings(0, 0);
        spu.main_volume(Volume::Normal(0x3FFF));

        spu.channels().for_each(|channel| {
            channel.reset();
        });

        spu
    }

    /// Gets a specific channel from the SPu without checking bounds.
    ///
    /// # Safety
    ///
    /// Calling this method with non-existent channel number is *undefined behavior*. You must make
    /// sure that the channel is within the range of the channels that the SPU has.
    pub unsafe fn unchecked_channel(&self, channel: usize) -> SpuChannel {
        unsafe {
            SpuChannel {
                regs: &mut (*SPU_CHANNEL_REGS.wrapping_add(channel as usize)),
                num: channel,
            }
        }
    }

    /// Gets a specific channel from the SPU. If the channel number is not in range of the amount
    /// of channels that the SPU has, it'll return [`None`].
    pub fn channel(&self, channel: usize) -> Option<SpuChannel> {
        if channel < SPU_CHANNELS {
            Some(unsafe { self.unchecked_channel(channel) })
        } else {
            None
        }
    }

    pub fn channels(&self) -> ChannelIterator {
        ChannelIterator {
            spu: self,
            channels: (0..SPU_CHANNELS),
        }
    }

    /// Sets the SPU's main left volume.
    pub fn main_volume_left(&self, vol: Volume) {
        let vol_bits: u16 = vol.into();

        unsafe {
            (*SPU_MVOLL).set(vol_bits);
        }
    }

    /// Sets the SPU's main right volume.
    pub fn main_volume_right(&self, vol: Volume) {
        let vol_bits: u16 = vol.into();

        unsafe {
            (*SPU_MVOLR).set(vol_bits);
        }
    }

    /// Sets the SPU's main volume.
    pub fn main_volume(&self, vol: Volume) {
        self.main_volume_left(vol);
        self.main_volume_right(vol);
    }

    /// Configure the Noise Generator for all channels that have noise mode enabled.
    ///
    /// `step` finetunes the frequency of the noise output (by skipping over steps in the timer),
    /// while `shift` coarsely tunes the frequency (by the shifting the initial value of the timer)
    ///
    /// See [The PlayStation Specifications](https://psx-spx.consoledev.net/soundprocessingunitspu/#spu-noise-generator_1) for more details.
    pub fn noise_settings(&self, shift: usize, step: usize) {
        if shift > 0x0F || step > 0x03 {
            panic!("Invalid noise settings.");
        }
        let mut config: SpuControlRegs;

        unsafe {
            config = (*SPU_CNT).get().into();
        }

        config.set_noise_freq_shift(shift);
        config.set_noise_freq_step(step);

        unsafe {
            (*SPU_CNT).set(config.into_bits());
        }
    }
}

impl Default for Spu {
    fn default() -> Self {
        Self::new()
    }
}
