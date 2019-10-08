use serde::de::{
    Error,
    Unexpected,
};

use crate::{
    probe::{
        flash::{
            flasher::FlashAlgorithm,
            memory::{
                MemoryRegion,
            },
        },
        debug_probe::{
            MasterProbe,
            DebugProbeError,
            CpuInformation,
        },
    },
    collection::get_core,
};

pub trait CoreRegister: Clone + From<u32> + Into<u32> + Sized + std::fmt::Debug {
    const ADDRESS: u32;
    const NAME: &'static str;
}

#[derive(Debug, Copy, Clone)]
pub struct CoreRegisterAddress(pub u8);

impl From<CoreRegisterAddress> for u32 {
    fn from(value: CoreRegisterAddress) -> Self {
        u32::from(value.0)
    }
}

impl From<u8> for CoreRegisterAddress {
    fn from(value: u8) -> Self {
        CoreRegisterAddress(value)
    }
}

#[allow(non_snake_case)]
#[derive(Copy, Clone)]
pub struct BasicRegisterAddresses {
    pub R0: CoreRegisterAddress,
    pub R1: CoreRegisterAddress,
    pub R2: CoreRegisterAddress,
    pub R3: CoreRegisterAddress,
    pub R4: CoreRegisterAddress,
    pub R9: CoreRegisterAddress,
    pub PC: CoreRegisterAddress,
    pub LR: CoreRegisterAddress,
    pub SP: CoreRegisterAddress,
}

pub trait Core: std::fmt::Debug + objekt::Clone {
    fn wait_for_core_halted(&self, mi: &mut MasterProbe) -> Result<(), DebugProbeError>;
    
    fn halt(&self, mi: &mut MasterProbe) -> Result<CpuInformation, DebugProbeError>;

    fn run(&self, mi: &mut MasterProbe) -> Result<(), DebugProbeError>;

    fn reset(&self, mi: &mut MasterProbe) -> Result<(), DebugProbeError>;

    /// Steps one instruction and then enters halted state again.
    fn step(&self, mi: &mut MasterProbe) -> Result<CpuInformation, DebugProbeError>;

    fn read_core_reg(&self, mi: &mut MasterProbe, addr: CoreRegisterAddress) -> Result<u32, DebugProbeError>;

    fn write_core_reg(&self, mi: &mut MasterProbe, addr: CoreRegisterAddress, value: u32) -> Result<(), DebugProbeError>;

    fn get_available_breakpoint_units(&self, mi: &mut MasterProbe) -> Result<u32, DebugProbeError>;

    fn enable_breakpoints(&self, mi: &mut MasterProbe, state: bool) -> Result<(), DebugProbeError>;

    fn set_breakpoint(&self, mi: &mut MasterProbe, addr: u32) -> Result<(), DebugProbeError>;

    fn enable_breakpoint(&self, mi: &mut MasterProbe, addr: u32) -> Result<(), DebugProbeError>;

    fn disable_breakpoint(&self, mi: &mut MasterProbe, addr: u32) -> Result<(), DebugProbeError>;

    fn read_block8(&self, mi: &mut MasterProbe, address: u32, data: &mut [u8]) -> Result<(), DebugProbeError>;

    fn registers<'a>(&self) -> &'a BasicRegisterAddresses;
}


objekt::clone_trait_object!(Core);

#[derive(Debug, Clone, Deserialize)]
pub struct Target {
    pub names: Vec<String>,
    pub flash_algorithm: FlashAlgorithm,
    pub memory_map: Vec<MemoryRegion>,
    pub core: Box<dyn Core>,
}

impl Target {
    pub fn new(definition: &str) -> Option<Self> {
        match serde_yaml::from_str(definition) as serde_yaml::Result<Self> {
            Ok(target) => {
                let mut names = target.names.clone();
                for name in &mut names {
                    name.make_ascii_lowercase();
                }
                Some(target)
            },
            Err(e) => None,
        }
    }
}

struct CoreVisitor;

impl<'de> serde::de::Visitor<'de> for CoreVisitor {
    type Value = Box<dyn Core>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an existing core name")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(core) = get_core(s) {
            Ok(core)
        } else {
            Err(Error::invalid_value(Unexpected::Other(&format!("Core {} does not exist.", s)), &self))
        }
    }
}

impl<'de> serde::Deserialize<'de> for Box<dyn Core> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_identifier(CoreVisitor)
    }
}

pub enum TargetSelectionError {
    CouldNotAutodetect,
    TargetNotFound(String),
}

pub fn select_target(name: Option<String>) -> Result<Target, TargetSelectionError> {
    name
        .map_or_else(
            || identify_target()
                .map_or_else(|| Err(TargetSelectionError::CouldNotAutodetect), |target| Ok(target)),
            |name| crate::collection::get_target(name.clone())
                .map_or_else(|| Err(TargetSelectionError::TargetNotFound(name)), |target| Ok(target))
        )
}

pub fn select_target_graceful_exit(name: Option<String>) -> Target {
    use colored::*;
    match select_target(name) {
        Ok(target) => target,
        Err(TargetSelectionError::CouldNotAutodetect) => {
            println!("    {} Target could not automatically be identified. Please specify one.", "Error".red().bold());
            std::process::exit(0);
        },
        Err(TargetSelectionError::TargetNotFound(name)) => {
            println!("    {} Specified target ({}) was not found. Please select an existing one.", "Error".red().bold(), name);
            std::process::exit(0);
        },
    }
}

pub fn identify_target() -> Option<Target> {
    // TODO: Poll this from the connected target. For now return nRF51.
    Some(crate::collection::targets::nrf51822::nRF51822())
}