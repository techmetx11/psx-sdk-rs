/// SPU reverb module.
use crate::{Spu, memory::VolatileU16};

const SPU_REVERB: *mut ReverbRegisters = 0x1F80_1D84 as *mut ReverbRegisters;

#[repr(C)]
struct ReverbRegisters {
    vlout: VolatileU16,
    vrout: VolatileU16,
    mbase: VolatileU16,
    dapf1: VolatileU16,
    dapf2: VolatileU16,
    viir: VolatileU16,
    vcomb1: VolatileU16,
    vcomb2: VolatileU16,
    vcomb3: VolatileU16,
    vcomb4: VolatileU16,
    vwall: VolatileU16,
    vapf1: VolatileU16,
    vapf2: VolatileU16,
    mlsame: VolatileU16,
    mrsame: VolatileU16,
    mlcomb1: VolatileU16,
    mrcomb1: VolatileU16,
    mlcomb2: VolatileU16,
    mrcomb2: VolatileU16,
    dlsame: VolatileU16,
    drsame: VolatileU16,
    mldiff: VolatileU16,
    mrdiff: VolatileU16,
    mlcomb3: VolatileU16,
    mrcomb3: VolatileU16,
    mlcomb4: VolatileU16,
    mrcomb4: VolatileU16,
    dldiff: VolatileU16,
    drdiff: VolatileU16,
    mlapf1: VolatileU16,
    mrapf1: VolatileU16,
    mlapf2: VolatileU16,
    mrapf2: VolatileU16,
    vlin: VolatileU16,
    vrin: VolatileU16,
}

/// The SPU's reverb settings structure.
pub struct SpuReverbSettings;

impl SpuReverbSettings {
    /// Sets the reverb output volume (on left and right) to the specified values.
    pub fn output_volume(&self, volume: (u16, u16)) -> &SpuReverbSettings {
        unsafe {
            (*SPU_REVERB).vlout.set(volume.0);
            (*SPU_REVERB).vrout.set(volume.1);
        }

        self
    }

    /// Resets all the reverb settings to normal, except for the echo buffer address.
    pub fn clear(&self) -> &SpuReverbSettings {
        macro_rules! wipe {
            (val $name:ident) => {
                (*SPU_REVERB).$name.set(0);
            };

            (addr $name:ident) => {
                (*SPU_REVERB).$name.set(1);
            };
        }

        unsafe {
            wipe!(val vlout);
            wipe!(val vrout);
            wipe!(val dapf1);
            wipe!(val dapf2);
            wipe!(val viir);
            wipe!(val vcomb1);
            wipe!(val vcomb2);
            wipe!(val vcomb3);
            wipe!(val vcomb4);
            wipe!(val vwall);
            wipe!(val vapf1);
            wipe!(val vapf2);
            wipe!(addr mlsame);
            wipe!(addr mrsame);
            wipe!(addr mlcomb1);
            wipe!(addr mrcomb1);
            wipe!(addr mlcomb2);

            wipe!(addr dlsame);
            wipe!(addr drsame);
            wipe!(addr mldiff);
            wipe!(addr mrdiff);

            wipe!(addr mrcomb2);
            wipe!(addr mlcomb3);
            wipe!(addr mrcomb3);
            wipe!(addr mlcomb4);
            wipe!(addr mrcomb4);

            wipe!(addr dldiff);
            wipe!(addr drdiff);
            wipe!(addr mlapf1);
            wipe!(addr mrapf1);
            wipe!(addr mlapf2);
            wipe!(addr mrapf2);

            wipe!(val vlin);
            wipe!(val vrin);
        }

        self
    }
}
