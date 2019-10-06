use crate::probe::flash::{
    flasher::FlashAlgorithm,
    memory::{
        MemoryRegion,
    }
};
use crate::probe::debug_probe::{
    MasterProbe,
    DebugProbeError,
    CpuInformation,
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

pub trait Core: std::fmt::Debug {
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

#[derive(Debug)]
pub struct Target {
    pub flash_algorithm: FlashAlgorithm,
    pub memory_map: Vec<MemoryRegion>,
    pub core: Box<dyn Core>,
}

pub fn select_target(name: Option<String>) -> Target {
    name
        .map_or_else(
            || identify_target(),
            |name| crate::collection::get_target(name)
        )
        .unwrap_or_else(|| panic!("Target could not be identified. Please select one."))
}

pub fn identify_target() -> Option<Target> {
    Some(crate::collection::targets::nrf51822::nRF51822())
}