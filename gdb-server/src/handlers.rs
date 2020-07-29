use probe_rs::{config::MemoryRegion, Core, MemoryInterface, Session};
use std::time::Duration;

pub(crate) fn q_supported() -> Option<String> {
    Some("PacketSize=2048;swbreak-;hwbreak+;vContSupported+;qXfer:memory-map:read+".into())
}

pub(crate) fn reply_empty() -> Option<String> {
    Some("".into())
}

pub(crate) fn q_attached() -> Option<String> {
    Some("1".into())
}

pub(crate) fn halt_reason() -> Option<String> {
    Some("S05".into())
}

pub(crate) fn read_general_registers() -> Option<String> {
    Some("xxxxxxxx".into())
}

pub(crate) fn read_register(register: u32, mut core: Core) -> Option<String> {
    let _ = core.halt(Duration::from_millis(500));
    core.wait_for_core_halted(Duration::from_millis(100))
        .unwrap();

    let value = core.read_core_reg(register as u16).unwrap();

    format!(
        "{}{}{}{}",
        value as u8,
        (value >> 8) as u8,
        (value >> 16) as u8,
        (value >> 24) as u8
    );

    Some(format!(
        "{:02x}{:02x}{:02x}{:02x}",
        value as u8,
        (value >> 8) as u8,
        (value >> 16) as u8,
        (value >> 24) as u8
    ))
}

pub(crate) fn read_memory(address: u32, length: u32, mut core: Core) -> Option<String> {
    let mut readback_data = vec![0u8; length as usize];
    match core.read_8(address, &mut readback_data) {
        Ok(_) => Some(
            readback_data
                .iter()
                .map(|s| format!("{:02x?}", s))
                .collect::<Vec<String>>()
                .join(""),
        ),
        // We have no clue if this is the right error code since GDB doesn't feel like docs.
        // We just assume Linux ERRNOs and pick a fitting one: https://gist.github.com/greggyNapalm/2413028#file-gistfile1-txt-L138
        // This seems to work in practice and seems to be the way to do stuff around GDB.
        Err(_e) => Some("E79".to_string()),
    }
}

pub(crate) fn vcont_supported() -> Option<String> {
    Some("vCont;c;t;s".into())
}

pub(crate) fn host_info() -> Option<String> {
    // cputype    12 = arm
    // cpusubtype 14 = v6m
    // See https://llvm.org/doxygen/Support_2MachO_8h_source.html
    Some("cputype:12;cpusubtype:14;triple:armv6m--none-eabi;endian:litte;ptrsize:4".to_string())
}

pub(crate) fn run(mut core: Core, awaits_halt: &mut bool) -> Option<String> {
    core.run().unwrap();
    *awaits_halt = true;
    None
}

pub(crate) fn stop(mut core: Core, awaits_halt: &mut bool) -> Option<String> {
    core.halt(Duration::from_millis(100)).unwrap();
    *awaits_halt = false;
    Some("OK".into())
}

pub(crate) fn step(mut core: Core, awaits_halt: &mut bool) -> Option<String> {
    core.step().unwrap();
    *awaits_halt = false;
    Some("S05".into())
}

pub(crate) fn insert_hardware_break(address: u32, _kind: u32, mut core: Core) -> Option<String> {
    core.reset_and_halt(Duration::from_millis(100)).unwrap();
    core.set_hw_breakpoint(address).unwrap();
    core.run().unwrap();
    Some("OK".into())
}

pub(crate) fn remove_hardware_break(address: u32, _kind: u32, mut core: Core) -> Option<String> {
    core.reset_and_halt(Duration::from_millis(100)).unwrap();
    core.clear_hw_breakpoint(address).unwrap();
    core.run().unwrap();
    Some("OK".into())
}

pub(crate) fn write_memory(address: u32, data: &[u8], mut core: Core) -> Option<String> {
    core.write_8(address, data).unwrap();

    Some("OK".into())
}

pub(crate) fn get_memory_map(session: &Session) -> Option<String> {
    let mut xml_map = r#"<?xml version="1.0"?>
<!DOCTYPE memory-map PUBLIC "+//IDN gnu.org//DTD GDB Memory Map V1.0//EN" "http://sourceware.org/gdb/gdb-memory-map.dtd">
<memory-map>
"#.to_owned();

    for region in session.memory_map() {
        let region_entry = match region {
            MemoryRegion::Ram(ram) => format!(
                r#"<memory type="ram" start="{:#x}" length="{:#x}"/>\n"#,
                ram.range.start,
                ram.range.end - ram.range.start
            ),
            MemoryRegion::Generic(region) => format!(
                r#"<memory type="rom" start="{:#x}" length="{:#x}"/>\n"#,
                region.range.start,
                region.range.end - region.range.start
            ),
            MemoryRegion::Flash(region) => {
                // TODO: Use flash with block size
                format!(
                    r#"<memory type="rom" start="{:#x}" length="{:#x}"/>\n"#,
                    region.range.start,
                    region.range.end - region.range.start
                )
            }
        };

        xml_map.push_str(&region_entry);
    }

    xml_map.push_str(r#"</memory-map>"#);
    Some(String::from_utf8(gdb_sanitize_file(xml_map.as_bytes(), 0, 1000)).unwrap())
}

pub(crate) fn user_halt(mut core: Core, awaits_halt: &mut bool) -> Option<String> {
    let _ = core.halt(Duration::from_millis(100));
    *awaits_halt = false;
    Some("T02".into())
}

pub(crate) fn detach(break_due: &mut bool) -> Option<String> {
    *break_due = true;
    Some("OK".into())
}

pub(crate) fn reset_halt(mut core: Core) -> Option<String> {
    let _cpu_info = core.reset_and_halt(Duration::from_millis(400));
    Some("OK".into())
}

fn gdb_sanitize_file(data: &[u8], offset: u32, len: u32) -> Vec<u8> {
    let offset = offset as usize;
    let len = len as usize;
    let mut end = offset + len;
    if offset > data.len() {
        b"l".to_vec()
    } else {
        if end > data.len() {
            end = data.len();
        }
        let mut trimmed_data = Vec::from(&data[offset..end]);
        if trimmed_data.len() >= len {
            // XXX should this be <= or < ?
            trimmed_data.insert(0, b'm');
        } else {
            trimmed_data.insert(0, b'l');
        }
        trimmed_data
    }
}
