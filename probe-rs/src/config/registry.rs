use crate::config::{
    chip::Chip,
    chip_family::ChipFamily,
    flash_algorithm::RawFlashAlgorithm,
    memory::{FlashRegion, MemoryRegion, RamRegion},
};
use crate::target::info::ChipInfo;
use crate::error::*;
use std::fs::File;
use std::path::Path;

use super::target::Target;
use crate::cores::get_core;

pub enum SelectionStrategy {
    TargetIdentifier(TargetIdentifier),
    ChipInfo(ChipInfo),
}

pub struct Registry {
    /// All the available chips.
    families: Vec<ChipFamily>,
}

impl Registry {
    #[allow(clippy::all)]
    pub fn from_builtin_families() -> Self {
        Self {
            families: include!(concat!(env!("OUT_DIR"), "/targets.rs")),
        }
    }

    pub fn families(&self) -> &Vec<ChipFamily> {
        &self.families
    }

    pub fn get_target(&self, strategy: SelectionStrategy) -> Result<Target> {
        let (family, chip, flash_algorithm) = match strategy {
            SelectionStrategy::TargetIdentifier(identifier) => {
                // Try get the corresponding chip.
                let mut selected_family_and_chip = None;
                for family in &self.families {
                    for variant in &family.variants {
                        if variant
                            .name
                            .to_ascii_lowercase()
                            .starts_with(&identifier.chip_name.to_ascii_lowercase())
                        {
                            if variant.name.to_ascii_lowercase()
                                != identifier.chip_name.to_ascii_lowercase()
                            {
                                log::warn!(
                                    "Found chip {} which matches given partial name {}. Consider specifying it's full name.",
                                    variant.name,
                                    identifier.chip_name,
                                )
                            }
                            selected_family_and_chip = Some((family, variant));
                        }
                    }
                }
                let (family, chip) = selected_family_and_chip.ok_or(err!(NotFound(NotFoundKind::Chip)))?;

                // Try get the correspnding flash algorithm.
                let flash_algorithm = family
                    .flash_algorithms
                    .iter()
                    .find(|flash_algorithm| {
                        if let Some(flash_algorithm_name) = identifier.flash_algorithm_name.clone()
                        {
                            flash_algorithm.name == flash_algorithm_name
                        } else {
                            flash_algorithm.default
                        }
                    })
                    .or_else(|| family.flash_algorithms.first())
                    .ok_or(err!(NotFound(NotFoundKind::Algorithm)))?;

                (family, chip, flash_algorithm)
            }
            SelectionStrategy::ChipInfo(chip_info) => {
                // Try get the corresponding chip.
                let mut selected_family_and_chip = None;
                for family in &self.families {
                    for variant in &family.variants {
                        if family
                            .manufacturer
                            .map(|m| m == chip_info.manufacturer)
                            .unwrap_or(false)
                            && family.part.map(|p| p == chip_info.part).unwrap_or(false)
                        {
                            selected_family_and_chip = Some((family, variant));
                        }
                    }
                }
                let (family, chip) = selected_family_and_chip.ok_or(err!(NotFound(NotFoundKind::Chip)))?;

                // Try get the correspnding flash algorithm.
                let flash_algorithm = family
                    .flash_algorithms
                    .first()
                    .ok_or(err!(NotFound(NotFoundKind::Algorithm)))?;

                (family, chip, flash_algorithm)
            }
        };

        // Try get the corresponding chip.
        let core = if let Some(core) = get_core(&family.core) {
            core
        } else {
            return Err(err!(NotFound(NotFoundKind::Core)));
        };

        let mut ram = None;
        let mut flash = None;
        for region in &chip.memory_map {
            match region {
                MemoryRegion::Ram(r) => ram = Some(r),
                MemoryRegion::Flash(r) => flash = Some(r),
                _ => (),
            };
        }

        Ok(Target::new(
            family,
            chip,
            ram.ok_or(err!(Missing(MissingKind::RamRegion)))?,
            flash.ok_or(err!(Missing(MissingKind::FlashRegion)))?,
            flash_algorithm,
            core,
        ))
    }

    pub fn add_target_from_yaml(&mut self, path_to_yaml: &Path) -> Result<()> {
        let file = File::open(path_to_yaml)?;
        let chip = ChipFamily::from_yaml_reader(file)?;

        let index = self
            .families
            .iter()
            .position(|old_chip| old_chip.name == chip.name);
        if let Some(index) = index {
            self.families.remove(index);
        }
        self.families.push(chip);

        Ok(())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetIdentifier {
    pub chip_name: String,
    pub flash_algorithm_name: Option<String>,
}

impl<S: AsRef<str>> From<S> for TargetIdentifier {
    fn from(value: S) -> TargetIdentifier {
        let split: Vec<_> = value.as_ref().split("::").collect();
        TargetIdentifier {
            // There will always be a 0th element, so this is safe!
            chip_name: split[0].to_owned(),
            flash_algorithm_name: split.get(1).map(|s| s.to_owned().to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_fetch1() {
        let registry = Registry::from_builtin_families();
        assert!(registry
            .get_target(SelectionStrategy::TargetIdentifier("nrf51".into()))
            .is_ok());
    }

    #[test]
    fn try_fetch2() {
        let registry = Registry::from_builtin_families();
        assert!(registry
            .get_target(SelectionStrategy::TargetIdentifier("nrf5182".into()))
            .is_ok());
    }

    #[test]
    fn try_fetch3() {
        let registry = Registry::from_builtin_families();
        assert!(registry
            .get_target(SelectionStrategy::TargetIdentifier("nrF51822_x".into()))
            .is_ok());
    }

    #[test]
    fn try_fetch4() {
        let registry = Registry::from_builtin_families();
        assert!(registry
            .get_target(SelectionStrategy::TargetIdentifier("nrf51822_Xxaa".into()))
            .is_ok());
    }
}
