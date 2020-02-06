use super::chip::Chip;
use super::flash_algorithm::RawFlashAlgorithm;
use super::memory::MemoryRegion;
use super::registry::TargetIdentifier;
use crate::core::CoreType;

/// This describes a complete target with a fixed chip model and variant.
#[derive(Clone)]
pub struct Target {
    /// The complete identifier of the target.
    pub identifier: TargetIdentifier,
    /// The name of the flash algorithm.
    pub flash_algorithms: Vec<RawFlashAlgorithm>,
    /// The core type.
    pub core_type: CoreType,
    /// The memory map of the target.
    pub memory_map: Vec<MemoryRegion>,
}

impl std::fmt::Debug for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Target {{
            identifier: {:?},
            flash_algorithms: {:?},
            memory_map: {:?},
        }}",
            self.identifier, self.flash_algorithms, self.memory_map
        )
    }
}

pub type TargetParseError = serde_yaml::Error;

impl Target {
    pub fn new(
        chip: &Chip,
        flash_algorithms: Vec<RawFlashAlgorithm>,
        core_type: CoreType,
    ) -> Target {
        Target {
            identifier: TargetIdentifier {
                chip_name: chip.name.clone(),
            },
            flash_algorithms,
            core_type,
            memory_map: chip.memory_map.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TargetSelector {
    Unspecified(String),
    Specified(Target),
    Auto,
}

impl<'a> From<&'a str> for TargetSelector {
    fn from(value: &'a str) -> Self {
        TargetSelector::Unspecified(value.into())
    }
}

impl<'a> From<&'a String> for TargetSelector {
    fn from(value: &'a String) -> Self {
        TargetSelector::Unspecified(value.into())
    }
}

impl From<String> for TargetSelector {
    fn from(value: String) -> Self {
        TargetSelector::Unspecified(value)
    }
}

impl From<()> for TargetSelector {
    fn from(_value: ()) -> Self {
        TargetSelector::Auto
    }
}

impl From<Target> for TargetSelector {
    fn from(target: Target) -> Self {
        TargetSelector::Specified(target)
    }
}
