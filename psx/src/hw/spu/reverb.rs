/// SPU reverb module.
use crate::hw::mmio::MemRegister;
use crate::hw::Register;

pub(crate) type ReverbOutVolumeLeft = MemRegister<i16, 0x1F80_1D84>;
pub(crate) type ReverbOutVolumeRight = MemRegister<i16, 0x1F80_1D86>;
pub(crate) type ReverbWorkArea = MemRegister<u16, 0x1F80_1DA2>;

/// The SPU's reverb structure.
pub struct SpuReverb {
    pub(crate) out_volume_left: ReverbOutVolumeLeft,
    pub(crate) out_volume_right: ReverbOutVolumeRight,
    pub(crate) work_ram: ReverbWorkArea,
}

impl SpuReverb {
    pub fn volume_left(&mut self, volume: i16) {
        self.out_volume_left.assign(volume).store();
    }

    pub fn volume_right(&mut self, volume: i16) {
        self.out_volume_right.assign(volume).store();
    }

    pub fn work_ram_addr(&mut self, address: u16) {
        self.work_ram.assign(address).store();
    }

    pub fn config(&mut self, config: &[u16; 0x1F]) {
        const SPU_REVERB_SETTINGS: *mut u16 = 0x1F80_1DC0 as _;

        // SAFETY: The size of the configuration data is as big as the area of reverb
        // registers, therefore there isn't a risk of some buffer overflow
        // happening.
        //
        // Unfortunately, since we can't use [`volatile_copy_nonoverlapping_memory`]
        // yet, we have to write our own volatile memcpy loop
        for (i, word) in config.iter().enumerate() {
            unsafe {
                SPU_REVERB_SETTINGS.add(i).write_volatile(*word);
            }
        }
    }
}
