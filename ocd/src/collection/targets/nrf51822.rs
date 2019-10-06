use crate::target::*;
use crate::probe::flash::*;

#[allow(non_snake_case)]
pub fn nRF51822() -> Target {
    Target {
        flash_algorithm: FlashAlgorithm {
            load_address: 0x20000000,
            instructions: &[
                0xE00ABE00, 0x062D780D, 0x24084068, 0xD3000040, 0x1E644058, 0x1C49D1FA, 0x2A001E52, 0x4770D1F2,
                0x47702000, 0x47702000, 0x4c26b570, 0x60602002, 0x60e02001, 0x68284d24, 0xd00207c0, 0x60602000,
                0xf000bd70, 0xe7f6f82c, 0x4c1eb570, 0x60612102, 0x4288491e, 0x2001d302, 0xe0006160, 0x4d1a60a0,
                0xf81df000, 0x07c06828, 0x2000d0fa, 0xbd706060, 0x4605b5f8, 0x4813088e, 0x46142101, 0x4f126041,
                0xc501cc01, 0x07c06838, 0x1e76d006, 0x480dd1f8, 0x60412100, 0xbdf84608, 0xf801f000, 0x480ce7f2,
                0x06006840, 0xd00b0e00, 0x6849490a, 0xd0072900, 0x4a0a4909, 0xd00007c3, 0x1d09600a, 0xd1f90840,
                0x00004770, 0x4001e500, 0x4001e400, 0x10001000, 0x40010400, 0x40010500, 0x40010600, 0x6e524635,
                0x00000000,
            ],
            pc_init: Some(0x20000021),
            pc_uninit: None,
            pc_program_page: 0x20000071,
            pc_erase_sector: 0x20000049,
            pc_erase_all: Some(0x20000029),
            static_base: 0x20000170,
            begin_stack: 0x20001000,
            begin_data: 0x20002000,
            page_buffers: &[0x20002000, 0x20002400],
            min_program_length: Some(4),
            analyzer_supported: true,
            analyzer_address: 0x20003000,
        },
        memory_map: vec![
            MemoryRegion::Flash(FlashRegion {
                range: 0..0x40000,
                is_boot_memory: true,
                is_testable: true,
                blocksize: 0x400,
                sector_size: 0x400,
                page_size: 0x400,
                phrase_size: 0x400,
                erase_all_weight: ERASE_ALL_WEIGHT,
                erase_sector_weight: ERASE_SECTOR_WEIGHT,
                program_page_weight: PROGRAM_PAGE_WEIGHT,
                erased_byte_value: 0xFF,
                access: Access::RX,
                are_erased_sectors_readable: true,
            }),
            MemoryRegion::Flash(FlashRegion {
                range: 0x10001000..0x10001000 + 0x100,
                is_boot_memory: false,
                is_testable: false,
                blocksize: 0x100,
                sector_size: 0x100,
                page_size: 0x100,
                phrase_size: 0x100,
                erase_all_weight: ERASE_ALL_WEIGHT,
                erase_sector_weight: ERASE_SECTOR_WEIGHT,
                program_page_weight: PROGRAM_PAGE_WEIGHT,
                erased_byte_value: 0xFF,
                access: Access::RX,
                are_erased_sectors_readable: true,
            }),
            MemoryRegion::Ram(RamRegion {
                range: 0x20000000..0x20000000 + 0x4000,
                is_boot_memory: false,
                is_testable: true,
            }),
        ],
        core: Box::new(crate::collection::cores::m0::M0::default())
    }
}