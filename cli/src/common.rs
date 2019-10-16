use crate::SharedOptions;
use std::path::Path;
use std::fs::File;

use ron;

use ocd::{
    probe::{
        debug_probe::{
            MasterProbe,
            DebugProbe,
            FakeProbe,
            DebugProbeError,
            DebugProbeType,
        },
        daplink,
        stlink,
        protocol::WireProtocol,
        flash::{
            flasher::{
                AlgorithmSelectionError,
            },
        },
    },
    coresight::{
        access_ports::{
            AccessPortError,
        },
    },
    collection::{
        cores::{
            m0::FakeM0,
        },
    },
    target::{
        TargetSelectionError,
    },
    session::Session
};

use ocd_targets::{
    SelectionStrategy,
};

use std::error::Error; 
use std::fmt;

#[derive(Debug)]
pub enum CliError {
    DebugProbe(DebugProbeError),
    AccessPort(AccessPortError),
    TargetSelectionError(TargetSelectionError),
    StdIO(std::io::Error),
    MissingArgument,
    FlashAlgorithm(AlgorithmSelectionError),
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use CliError::*;

        match self {
            DebugProbe(ref e) => Some(e),
            AccessPort(ref e) => Some(e),
            TargetSelectionError(ref e) => Some(e),
            StdIO(ref e) => Some(e),
            MissingArgument => None,
            FlashAlgorithm(ref e) => Some(e),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CliError::*;

        match self {
            DebugProbe(ref e) => e.fmt(f),
            AccessPort(ref e) => e.fmt(f),
            TargetSelectionError(ref e) => e.fmt(f),
            StdIO(ref e) => e.fmt(f),
            MissingArgument => {
                write!(f, "Command expected more arguments")
            }
            FlashAlgorithm(ref e) => e.fmt(f),
        }
    }
}

impl From<AccessPortError> for CliError {
    fn from(error: AccessPortError) -> Self {
        CliError::AccessPort(error)
    }
}

impl From<DebugProbeError> for CliError {
    fn from(error: DebugProbeError) -> Self {
        CliError::DebugProbe(error)
    }
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        CliError::StdIO(error)
    }
}

impl From<TargetSelectionError> for CliError {
    fn from(error: TargetSelectionError) -> Self {
        CliError::TargetSelectionError(error)
    }
}

impl From<AlgorithmSelectionError> for CliError {
    fn from(error: AlgorithmSelectionError) -> Self {
        CliError::FlashAlgorithm(error)
    }
}

/// Takes a closure that is handed an `DAPLink` instance and then executed.
/// After the closure is done, the USB device is always closed,
/// even in an error case inside the closure!
pub(crate) fn with_device<F>(shared_options: &SharedOptions, f: F) -> Result<(), CliError>
where
    for<'a> F: FnOnce(Session) -> Result<(), CliError>
{
    let device = {
        let mut list = daplink::tools::list_daplink_devices();
        list.extend(stlink::tools::list_stlink_devices());

        list.remove(shared_options.n)
    };

    let probe = match device.probe_type {
        DebugProbeType::DAPLink => {
            let mut link = daplink::DAPLink::new_from_probe_info(&device)?;

            link.attach(Some(WireProtocol::Swd))?;
            
            MasterProbe::from_specific_probe(link)
        },
        DebugProbeType::STLink => {
            let mut link = stlink::STLink::new_from_probe_info(&device)?;

            link.attach(Some(WireProtocol::Swd))?;
            
            MasterProbe::from_specific_probe(link)
        },
    };

    let target = ocd_targets::select_target(&SelectionStrategy::Name(shared_options.target.clone().unwrap()))?;

    let flash_algorithm = match target.flash_algorithm.clone() {
        Some(name) => ocd_targets::select_algorithm(name),
        None => Err(AlgorithmSelectionError::NoAlgorithmSuggested),
    };

    let flash_algorithm = match flash_algorithm {
        Ok(flash_algorithm) => Some(flash_algorithm),
        Err(error) => { println!("{:?}", error); None },
    };
    
    let session = Session::new(target, probe, flash_algorithm);

    f(session)
}

pub(crate) fn with_dump<F>(shared_options: &SharedOptions, p: &Path, f: F) -> Result<(), CliError>
where
    for<'a> F: FnOnce(Session) -> Result<(), CliError>
{
    let mut dump_file = File::open(p)?;

    let dump = ron::de::from_reader(&mut dump_file).unwrap();

    let core = FakeM0::new(dump);
    let fake_probe = FakeProbe::new();

    let probe = MasterProbe::from_specific_probe(Box::new(fake_probe));

    let mut target = ocd_targets::select_target(
        &SelectionStrategy::Name(shared_options.target.clone().unwrap())
    )?;

    target.core = Box::new(core);

    let session = Session::new(target, probe, None);

    f(session)
}
