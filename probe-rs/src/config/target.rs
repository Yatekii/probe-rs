use jep106::JEP106Code;
use crate::target::Core;
use super::registry::TargetIdentifier;
use super::flash_algorithm::FlashAlgorithm;
use super::memory::MemoryRegion;
use super::chip::Chip;

/// This describes a complete target with a fixed chip model and variant.
#[derive(Debug, Clone)]
pub struct Target {
    /// The complete identifier of the target.
    pub identifier: TargetIdentifier,
    /// The JEP106 code of the manufacturer.
    pub manufacturer: JEP106Code,
    /// The `PART` register of the chip.
    /// This value can be determined via the `cli info command`.
    pub part: u32,
    /// The name of the flash algorithm.
    pub flash_algorithm: FlashAlgorithm,
    /// The core type.
    pub core: Box<dyn Core>,
    /// The memory map of the target.
    pub memory_map: Vec<MemoryRegion>,
}

impl From<(&Chip, &FlashAlgorithm, Box<dyn Core>)> for Target {
    fn from(value: (&Chip, &FlashAlgorithm, Box<dyn Core>)) -> Target {
        let (chip, flash_algorithm, core) = value;
        Target {
            identifier: TargetIdentifier {
                chip_name: chip.name.clone(),
                flash_algorithm_name: Some(flash_algorithm.name.clone()),
            },
            manufacturer: chip.manufacturer,
            part: chip.part,
            flash_algorithm: flash_algorithm.clone(),
            core,
            memory_map: chip.memory_map.clone(),
        }
    }
}