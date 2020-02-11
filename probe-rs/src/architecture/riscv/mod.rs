//! RISCV Support

use crate::core::Architecture;
use crate::CoreInterface;
use communication_interface::{AccessRegisterCommand, RiscvCommunicationInterface};

use crate::core::CoreInformation;
use crate::CoreRegisterAddress;
use bitfield::bitfield;

pub mod communication_interface;
pub mod memory_interface;

#[derive(Clone)]
pub struct Riscv32 {
    interface: RiscvCommunicationInterface,
}

impl Riscv32 {
    pub fn new(interface: RiscvCommunicationInterface) -> Self {
        Self { interface }
    }

    // Read a core register using an abstract command
    fn abstract_cmd_register_read(&self, regno: u32) -> Result<u32, crate::Error> {
        // GPR

        // read from data0
        let mut command = AccessRegisterCommand(0);
        command.set_cmd_type(0);
        command.set_transfer(true);
        command.set_aarsize(2);

        command.set_regno(regno);

        self.execute_abstract_command(command.0)?;

        let register_value = self.interface.read_dm_register(0x04)?;

        Ok(register_value)
    }

    fn abstract_cmd_register_write(&self, regno: u32, value: u32) -> Result<(), crate::Error> {
        // read from data0
        let mut command = AccessRegisterCommand(0);
        command.set_cmd_type(0);
        command.set_transfer(true);
        command.set_write(true);
        command.set_aarsize(2);

        command.set_regno(regno);

        // write data0
        self.interface.write_dm_register(0x04, value)?;

        self.execute_abstract_command(command.0)
    }

    fn execute_abstract_command(&self, command: u32) -> Result<(), crate::Error> {
        // ensure that preconditions are fullfileld
        // haltreq      = 0
        // resumereq    = 0
        // ackhavereset = 0

        let mut dmcontrol = Dmcontrol(0);
        dmcontrol.set_haltreq(false);
        dmcontrol.set_resumereq(false);
        dmcontrol.set_ackhavereset(true);
        dmcontrol.set_dmactive(true);
        self.interface.write_dm_register(0x10, dmcontrol.0)?;

        // read abstractcs to see its state
        let abstractcs_prev = Abstractcs(self.interface.read_dm_register(0x16)?);

        log::debug!("abstractcs: {:?}", abstractcs_prev);

        if abstractcs_prev.cmderr() != 0 {
            //clear previous command error
            let mut abstractcs_clear = Abstractcs(0);
            abstractcs_clear.set_cmderr(0x7);

            self.interface.write_dm_register(0x16, abstractcs_clear.0)?;
        }

        self.interface.write_dm_register(0x17, command)?;

        // poll busy flag in abstractcs

        let repeat_count = 10;

        let mut abstractcs = Abstractcs(1);

        for _ in 0..repeat_count {
            abstractcs = Abstractcs(self.interface.read_dm_register(0x16)?);

            if !abstractcs.busy() {
                break;
            }
        }

        log::debug!("abstracts: {:?}", abstractcs);

        if abstractcs.busy() {
            todo!("Proper error, error executing abstract command");
        }

        // check cmderr
        if abstractcs.cmderr() != 0 {
            todo!(
                "Cmderr {} occured while executing command, add proper error",
                abstractcs.cmderr()
            );
        }

        Ok(())
    }
}

impl CoreInterface for Riscv32 {
    fn wait_for_core_halted(&self) -> Result<(), crate::Error> {
        // poll the
        let num_retries = 10;

        for _ in 0..num_retries {
            let dmstatus = Dmstatus(self.interface.read_dm_register(0x11)?);

            log::trace!("{:?}", dmstatus);

            if dmstatus.allhalted() {
                return Ok(());
            }
        }

        todo!("Proper error for core halt timeout")
    }
    fn core_halted(&self) -> Result<bool, crate::Error> {
        unimplemented!()
    }

    fn halt(&self) -> Result<CoreInformation, crate::Error> {
        // write 1 to the haltreq register, which is part
        // of the dmcontrol register

        // read the current dmcontrol register
        let current_dmcontrol = Dmcontrol(self.interface.read_dm_register(0x10)?);
        log::debug!("{:?}", current_dmcontrol);

        let mut dmcontrol = Dmcontrol(0);

        dmcontrol.set_haltreq(true);
        dmcontrol.set_dmactive(true);

        self.interface.write_dm_register(0x10, dmcontrol.0)?;

        self.wait_for_core_halted()?;

        // clear the halt request
        let mut dmcontrol = Dmcontrol(0);

        dmcontrol.set_dmactive(true);

        self.interface.write_dm_register(0x10, dmcontrol.0)?;

        let pc = self.read_core_reg(CoreRegisterAddress(0x7b1))?;

        Ok(CoreInformation { pc })
    }

    fn run(&self) -> Result<(), crate::Error> {
        unimplemented!()
    }
    fn reset(&self) -> Result<(), crate::Error> {
        unimplemented!()
    }
    fn reset_and_halt(&self) -> Result<crate::core::CoreInformation, crate::Error> {
        unimplemented!()
    }
    fn step(&self) -> Result<crate::core::CoreInformation, crate::Error> {
        unimplemented!()
    }

    fn read_core_reg(&self, address: crate::CoreRegisterAddress) -> Result<u32, crate::Error> {
        // We need to sue the "Access Register Command",
        // which has cmdtype 0

        // write needs to be clear
        // transfer has to be set

        log::debug!("Reading core register at address {:#x}", address.0);

        // if it is a gpr (general purpose register) read using an abstract command,
        // otherwise, use the program buffer
        if address.0 >= 0x1000 && address.0 <= 0x101f {
            self.abstract_cmd_register_read(address.0 as u32)
        } else {
            // todo: need to preserve s0?

            let s0 = self.abstract_cmd_register_read(0x1008)?;

            // csrrs,
            // with rd  = s0
            //      rs1 = x0
            //      csr = address

            let mut csrrs_cmd: u32 = 0b_00000_010_01000_1110011;
            csrrs_cmd |= ((address.0 as u32) & 0xfff) << 20;
            let ebreak_cmd = 0b000000000001_00000_000_00000_1110011;

            // write progbuf0: csrr xxxxxx s0, (address) // lookup correct command
            self.interface.write_dm_register(0x20, csrrs_cmd)?;

            // write progbuf1: ebreak
            self.interface.write_dm_register(0x21, ebreak_cmd)?;

            // command: postexec
            let mut postexec_cmd = AccessRegisterCommand(0);
            postexec_cmd.set_postexec(true);

            self.execute_abstract_command(postexec_cmd.0)?;

            // command: transfer, regno = 0x1008
            let reg_value = self.abstract_cmd_register_read(0x1008)?;

            // restore original value in s0
            self.abstract_cmd_register_write(0x1008, s0)?;

            Ok(reg_value)
        }
    }

    fn write_core_reg(
        &self,
        address: crate::CoreRegisterAddress,
        value: u32,
    ) -> Result<(), crate::Error> {
        unimplemented!()
    }
    fn get_available_breakpoint_units(&self) -> Result<u32, crate::Error> {
        unimplemented!()
    }
    fn enable_breakpoints(&mut self, state: bool) -> Result<(), crate::Error> {
        unimplemented!()
    }
    fn set_breakpoint(&self, bp_unit_index: usize, addr: u32) -> Result<(), crate::Error> {
        unimplemented!()
    }
    fn clear_breakpoint(&self, unit_index: usize) -> Result<(), crate::Error> {
        unimplemented!()
    }

    fn registers<'a>(&self) -> &'a crate::core::BasicRegisterAddresses {
        unimplemented!()
    }
    fn memory(&self) -> crate::Memory {
        unimplemented!()
    }

    fn hw_breakpoints_enabled(&self) -> bool {
        unimplemented!()
    }

    fn architecture(&self) -> Architecture {
        Architecture::RISCV
    }
}

bitfield! {
    // `dmcontrol` register, located at
    // address 0x10
    pub struct Dmcontrol(u32);
    impl Debug;

    _, set_haltreq: 31;
    _, set_resumereq: 30;
    hartreset, set_hartreset: 29;
    _, set_ackhavereset: 28;
    hasel, set_hasel: 26;
    hartsello, set_hartsello: 25, 16;
    hartselhi, set_hartselhi: 15, 6;
    _, set_resethaltreq: 3;
    _, set_clrresethaltreq: 2;
    ndmreset, set_ndmreset: 1;
    dmactive, set_dmactive: 0;
}

bitfield! {
    /// Readonly `dmstatus` register.
    ///
    /// Located at address 0x11
    pub struct Dmstatus(u32);
    impl Debug;

    impebreak, _: 22;
    allhavereset, _: 19;
    anyhavereset, _: 18;
    allresumeack, _: 17;
    anyresumeack, _: 16;
    allnonexistent, _: 15;
    anynonexistent, _: 14;
    allunavail, _: 13;
    anyunavail, _: 12;
    allrunning, _: 11;
    anyrunning, _: 10;
    allhalted, _: 9;
    anyhalted, _: 8;
    authenticated, _: 7;
    authbusy, _: 6;
    hasresethaltreq, _: 5;
    confstrptrvalid, _: 4;
    version, _: 3, 0;
}

bitfield! {
    pub struct Abstractcs(u32);
    impl Debug;

    progbufsize, _: 28, 24;
    busy, _: 12;
    cmderr, set_cmderr: 10, 8;
    datacount, _: 3, 0;
}
