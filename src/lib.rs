#![no_std]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod volume;

// To use its panic handler
// The `maybe_uninit_uninit_array` is because of the OBJ importer that the `psx` crate has. We
// aren't going to be use that here, so hopefully it won't be included in the library
//#![feature(maybe_uninit_uninit_array)]
//extern crate psx;

use bitfield_struct::bitfield;

use crate::volume::Volume;

// This crate is potentially unsafe in other platforms, So we have to stop the compilation if we
// detect that the compiler is not targetting the PS1
#[cfg(not(target_os = "psx"))]
compile_error!(
    "This crate is meant to be compiled for the PlayStation 1, and cannot be used anywhere else."
);

/// The SPU structure.
pub struct Spu;

const SPU_CHANNELS: usize = 24;

const SPU_VOLL: *mut u16 = 0x1F80_1C00 as *mut u16;
const SPU_VOLR: *mut u16 = 0x1F80_1C02 as *mut u16;
const SPU_VXPITCH: *mut u16 = 0x1F80_1C04 as *mut u16;
const SPU_ADPCM: *mut u16 = 0x1F80_1C06 as *mut u16;

const SPU_KEYON: *mut u16 = 0x1F80_1D88 as *mut u16;
const SPU_KEYOFF: *mut u16 = 0x1F80_1D8C as *mut u16;
const SPU_NON: *mut u16 = 0x1F80_1D94 as *mut u16;
const SPU_CNT: *mut u16 = 0x1F80_1DAA as *mut u16;

const SPU_MVOLL: *mut u16 = 0x1F80_1D80 as *mut u16;
const SPU_MVOLR: *mut u16 = 0x1F80_1D82 as *mut u16;

/// Check if the specified SPU channel is in range.
/// A panic is used here because the sound driver should never use a channel above the amount of
/// channels in the SPU.
macro_rules! check_channel {
    ($channel:ident) => {
        if $channel > SPU_CHANNELS {
            panic!("Channel does not exist. {{$channel}} > {SPU_CHANNELS}");
        }
    };
}

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

/// This function reads a 32-bit value from a memory address using a pair of 16-bit reads. This is
/// to avoid issues with the SPU data bus when using 32-bit hardware accesses.
unsafe fn read_32(ptr: *mut u16) -> u32 {
    let lower: u16;
    let upper: u16;
    unsafe {
        lower = core::ptr::read_volatile(ptr);
        upper = core::ptr::read_volatile(ptr.wrapping_add(1));
    }

    lower as u32 | (upper as u32) << 16
}

/// This function writes a 32-bit value to a memory address using a pair of 16-bit writes. This is
/// to avoid issues with the SPU data bus when using 32-bit hardware accesses.
unsafe fn write_32(ptr: *mut u16, value: u32) {
    unsafe {
        core::ptr::write_volatile(ptr, value as u16);
        core::ptr::write_volatile(ptr.wrapping_add(1), value.unbounded_shr(16) as u16);
    }
}

unsafe fn write_bit_32(ptr: *mut u16, bit: usize, value: bool) {
    unsafe {
        let mut data = read_32(ptr);
        data |= (if value { 1 } else { 0 }) << bit;
        write_32(ptr, data);
    }
}

impl Spu {
    /// Sets the left volume of a channel.
    pub fn volume_left(&self, channel: usize, vol: Volume) {
        let vol_bits: u16 = vol.into();

        unsafe {
            core::ptr::write_volatile(SPU_VOLL.wrapping_add(channel * 0x10), vol_bits);
        }
    }

    /// Sets the right volume of a channel.
    pub fn volume_right(&self, channel: usize, vol: Volume) {
        let vol_bits: u16 = vol.into();

        unsafe {
            core::ptr::write_volatile(SPU_VOLR.wrapping_add(channel * 0x10), vol_bits);
        }
    }

    /// Sets the volume (both left/right) of a channel.
    pub fn volume(&self, channel: usize, vol: Volume) {
        self.volume_left(channel, vol);
        self.volume_right(channel, vol);
    }

    /// Sets the SPU's main left volume.
    pub fn main_volume_left(&self, vol: Volume) {
        let vol_bits: u16 = vol.into();

        unsafe {
            core::ptr::write_volatile(SPU_MVOLL, vol_bits);
        }
    }

    /// Sets the SPU's main right volume.
    pub fn main_volume_right(&self, vol: Volume) {
        let vol_bits: u16 = vol.into();

        unsafe {
            core::ptr::write_volatile(SPU_MVOLR, vol_bits);
        }
    }

    /// Sets the SPU's main volume.
    pub fn main_volume(&self, vol: Volume) {
        self.main_volume_left(vol);
        self.main_volume_right(vol);
    }

    /// Sets the address of the sample the channel should be playing off of.
    pub fn sample_start(&self, channel: usize, mut sample: u32) {
        check_channel!(channel);

        if sample > 1 << 20 {
            panic!("Sample address is bigger than the maximum addressable address in the SPU");
        }

        // In the SPU, samples are indexed by 8-byte units.
        sample >>= 4;

        unsafe {
            core::ptr::write_volatile(SPU_ADPCM.wrapping_add(channel * 0x10), sample as u16);
        }
    }

    /// Sets the ADPCM sample rate of the channel to the specified frequency (0x1000 == 441000Hz).
    ///
    /// Note: This does not affect the frequency of the channel if noise mode is active on it. For
    /// that, you should check out [`Self::noise_settings`]
    pub fn frequency(&self, channel: usize, frequency: u16) {
        check_channel!(channel);

        unsafe {
            core::ptr::write_volatile(SPU_VXPITCH.wrapping_add(channel * 0x10), frequency);
        }
    }

    /// Starts the ADSR envelope and automatically initializes the ADSR volume to zero
    pub fn key_on(&self, channel: usize) {
        check_channel!(channel);

        unsafe {
            write_bit_32(SPU_KEYON, channel, true);
        }
    }

    /// Releases the key in the channel, which starts the Release stage of the ADSR envelope, if
    /// set.
    pub fn key_off(&self, channel: usize) {
        check_channel!(channel);

        unsafe {
            write_bit_32(SPU_KEYOFF, channel, true);
        }
    }

    /// Enable or disable noise mode on a specific channel. If enabled, the channel will stop
    /// outputting ADPCM samples and instead output noise samples from the SPU's Noise Generator.
    ///
    /// The Noise Generator can be configured, using the [`Self::noise_settings`] function.
    pub fn noise(&self, channel: usize, enable: bool) {
        check_channel!(channel);

        unsafe {
            write_bit_32(SPU_NON, channel, enable);
        }
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
            config = core::ptr::read_volatile(SPU_CNT).into();
        }

        config.set_noise_freq_shift(shift);
        config.set_noise_freq_step(step);

        unsafe {
            core::ptr::write_volatile(SPU_CNT, config.into_bits());
        }
    }
}
