use crate::{
    SharedOptions,
    common::{
        CliError,
    },
};

use ocd::{
    coresight::{
        access_ports::{
            generic_ap::{APClass, IDR},
            memory_ap::{BaseaddrFormat, MemoryAP, BASE, BASE2},
        },
        ap_access::{valid_access_ports, APAccess},
    },
    memory::romtable::CSComponent,
    probe::{
        daplink,
        stlink,
        debug_probe::{
            DebugProbeType,
            MasterProbe,
            DebugProbe,
        },
        protocol::WireProtocol,
    },
};

pub(crate) fn show_info_of_device(shared_options: &SharedOptions) -> Result<(), CliError> {
    let device = {
        let mut list = daplink::tools::list_daplink_devices();
        list.extend(stlink::tools::list_stlink_devices());

        list.remove(shared_options.n)
    };

    let mut probe = match device.probe_type {
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

    /*
        The following code only works with debug port v2,
        which might not necessarily be present.

        Once the typed interface for the debug port is done, it 
        can be enabled again.

    println!("Device information:");


    let target_info = link
        .read_register(Port::DebugPort, 0x4)?;

    let target_info = parse_target_id(target_info);
    println!("\nTarget Identification Register (TARGETID):");
    println!(
        "\tRevision = {}, Part Number = {}, Designer = {}",
        target_info.0, target_info.3, target_info.2
    );

    */


    // Note: Temporary read to ensure the DP information is read at
    //       least once before reading the ROM table 
    //       (necessary according to STM manual).
    //
    // TODO: Move to proper place somewhere in init code
    //

    let target_info = probe.read_register_dp(0x0)?;
    println!("DP info: {:#08x}", target_info);

    println!("\nAvailable Access Ports:");

    for access_port in valid_access_ports(&mut probe) {
        let idr = probe.read_register_ap(access_port, IDR::default())?;
        println!("{:#x?}", idr);

        if idr.CLASS == APClass::MEMAP {
            let access_port: MemoryAP = access_port.into();

            let base_register = probe.read_register_ap(access_port, BASE::default())?;

            let mut baseaddr = if BaseaddrFormat::ADIv5 == base_register.Format {
                let base2 = probe.read_register_ap(access_port, BASE2::default())?;
                (u64::from(base2.BASEADDR) << 32)
            } else {
                0
            };
            baseaddr |= u64::from(base_register.BASEADDR << 12);

            let link_ref = &mut probe;

            let component_table = CSComponent::try_parse(&link_ref.into(), baseaddr as u64);

            component_table
                .iter()
                .for_each(|entry| println!("{:#08x?}", entry));

            // let mut reader = crate::memory::romtable::RomTableReader::new(&link_ref, baseaddr as u64);

            // for e in reader.entries() {
            //     if let Ok(e) = e {
            //         println!("ROM Table Entry: Component @ 0x{:08x}", e.component_addr());
            //     }
            // }
        }
    }

    Ok(())
}
