#![no_std]
//#![deny(missing_docs)]

// To use its panic handler
// The `maybe_uninit_uninit_array` is because of the OBJ importer that the `psx` crate has. We
// aren't going to be use that here, so hopefully it won't be included in the library
#![feature(maybe_uninit_uninit_array)]
extern crate psx;

use bitfield_struct::bitfield;

// This crate is potentially unsafe in other platforms, So we have to stop the compilation if we
// detect that the compiler is not targetting the PS1
#[cfg(not(target_os = "psx"))]
compile_error!(
    "This crate is meant to be compiled for the PlayStation 1, and cannot be used anywhere else."
);

/// The SPU structure.
pub struct Spu {}

const SPU_CHANNELS: usize = 24;

const SPU_VXPITCH: *mut u16 = 0x1F801C04 as *mut u16;
const SPU_ADPCM: *mut u16 = 0x1F801C06 as *mut u16;

const SPU_KEYON: *mut u16 = 0x1F801D88 as *mut u16;
const SPU_KEYOFF: *mut u16 = 0x1F801D8C as *mut u16;
const SPU_NON: *mut u16 = 0x1F801D94 as *mut u16;
const SPU_CNT: *mut u16 = 0x1F801DAA as *mut u16;

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
    pub fn sample_start(channel: usize, mut sample: u16) {
        check_channel!(channel);

        // In the SPU, samples are indexed by 8-byte units.
        sample >>= 4;

        unsafe {
            core::ptr::write_volatile(SPU_ADPCM.wrapping_add(channel * 0x10), sample);
        }
    }

    pub fn pitch(channel: usize, pitch: u16) {
        check_channel!(channel);

        unsafe {
            core::ptr::write_volatile(SPU_VXPITCH.wrapping_add(channel * 0x10), pitch);
        }
    }

    pub fn key_on(channel: usize) {
        check_channel!(channel);

        unsafe {
            write_bit_32(SPU_KEYON, channel, true);
        }
    }

    pub fn key_off(channel: usize) {
        check_channel!(channel);

        unsafe {
            write_bit_32(SPU_KEYOFF, channel, true);
        }
    }

    /// Enable or disable noise mode on a specific channel. If enabled, the channel will stop
    /// outputting ADPCM samples and instead output noise samples from the SPU's Noise Generator.
    ///
    /// The Noise Generator can be configured, using the [`Self::noise_settings`] function.
    pub fn noise(channel: usize, enable: bool) {
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
    pub fn noise_settings(shift: usize, step: usize) {
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
