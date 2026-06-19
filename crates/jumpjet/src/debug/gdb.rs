use std::net::TcpListener;

use gdbstub::conn::ConnectionExt;
use gdbstub::stub::GdbStub;
use gdbstub::stub::state_machine::GdbStubStateMachine;
use gdbstub::target::ext::base::singlethread::{
    SingleThreadBase, SingleThreadResume, SingleThreadResumeOps,
};
use gdbstub::target::ext::breakpoints::{Breakpoints, SwBreakpoint};
use gdbstub::target::{Target, TargetResult};

use cranelift_entity::EntityRef;
use wasmtime::{AsContextMut, StoreContextMut};

use crate::runtime::JumpjetRuntimeState;

/// Default port for the GDB server.
pub const DEFAULT_GDB_PORT: u16 = 9001;

/// Architecture definition for Wasm32 GDB.
pub struct Wasm32;

impl gdbstub::arch::Arch for Wasm32 {
    type Usize = u32;
    type Registers = Wasm32Regs;
    type RegId = Wasm32RegId;
    type BreakpointKind = usize;

    fn target_description_xml() -> Option<&'static str> {
        Some(r#"<?xml version="1.0"?><target><architecture>wasm32</architecture></target>"#)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Wasm32Regs {
    pub pc: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Wasm32RegId {
    Pc,
}

impl gdbstub::arch::RegId for Wasm32RegId {
    fn from_raw_id(id: usize) -> Option<(Self, Option<core::num::NonZero<usize>>)> {
        match id {
            0 => Some((Wasm32RegId::Pc, None)),
            _ => None,
        }
    }
}

impl gdbstub::arch::Registers for Wasm32Regs {
    type ProgramCounter = u32;

    fn pc(&self) -> Self::ProgramCounter {
        self.pc
    }

    fn gdb_serialize(&self, mut write_byte: impl FnMut(Option<u8>)) {
        for byte in self.pc.to_le_bytes() {
            write_byte(Some(byte));
        }
    }

    fn gdb_deserialize(&mut self, bytes: &[u8]) -> Result<(), ()> {
        if bytes.len() < 4 {
            return Err(());
        }
        self.pc = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        Ok(())
    }
}

/// GDB Target for Rune games.
pub struct RuneGdbTarget<'a> {
    pub store: StoreContextMut<'a, JumpjetRuntimeState>,
}

impl Target for RuneGdbTarget<'_> {
    type Arch = Wasm32;
    type Error = anyhow::Error;

    fn base_ops(&mut self) -> gdbstub::target::ext::base::BaseOps<'_, Wasm32, anyhow::Error> {
        gdbstub::target::ext::base::BaseOps::SingleThread(self)
    }

    fn support_breakpoints(
        &mut self,
    ) -> Option<gdbstub::target::ext::breakpoints::BreakpointsOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadBase for RuneGdbTarget<'_> {
    fn read_registers(&mut self, regs: &mut Wasm32Regs) -> TargetResult<(), Self> {
        unsafe {
            let store_ptr: *mut StoreContextMut<'_, JumpjetRuntimeState> =
                std::mem::transmute(&mut self.store);
            if let Some(frame) = (*store_ptr).debug_exit_frames().next() {
                if let Ok(Some((func_idx, pc_offset))) =
                    frame.wasm_function_index_and_pc(&mut *store_ptr)
                {
                    regs.pc = ((func_idx.index() as u32) << 16) | pc_offset.raw();
                }
            }
        }
        Ok(())
    }

    fn write_registers(&mut self, _regs: &Wasm32Regs) -> TargetResult<(), Self> {
        // Writing registers (like PC) is not supported currently
        Ok(())
    }

    fn read_addrs(&mut self, start_addr: u32, data: &mut [u8]) -> TargetResult<usize, Self> {
        let frame = self.store.debug_exit_frames().next();
        if let Some(frame) = frame {
            if let Ok(instance) = frame.instance(&mut self.store) {
                if let Some(memory) = instance.debug_memory(&mut self.store, 0) {
                    let memory_data = memory.data(&self.store);
                    let start = start_addr as usize;
                    if start < memory_data.len() {
                        let end = (start + data.len()).min(memory_data.len());
                        let len = end - start;
                        data[..len].copy_from_slice(&memory_data[start..end]);
                        return Ok(len);
                    }
                }
            }
        }
        Ok(0)
    }

    fn write_addrs(&mut self, _start_addr: u32, _data: &[u8]) -> TargetResult<(), Self> {
        // Writing memory is not supported currently
        Ok(())
    }

    fn support_resume(&mut self) -> Option<SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadResume for RuneGdbTarget<'_> {
    fn resume(&mut self, _signal: Option<gdbstub::common::Signal>) -> Result<(), anyhow::Error> {
        // This is called when GDB says "continue"
        // We return and let Wasmtime continue execution
        Ok(())
    }

    fn support_single_step(
        &mut self,
    ) -> Option<gdbstub::target::ext::base::singlethread::SingleThreadSingleStepOps<'_, Self>> {
        Some(self)
    }
}

impl gdbstub::target::ext::base::singlethread::SingleThreadSingleStep for RuneGdbTarget<'_> {
    fn step(&mut self, _signal: Option<gdbstub::common::Signal>) -> Result<(), anyhow::Error> {
        self.store
            .as_context_mut()
            .edit_breakpoints()
            .expect("debug mode enabled")
            .single_step(true)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(())
    }
}

impl Breakpoints for RuneGdbTarget<'_> {
    fn support_sw_breakpoint(
        &mut self,
    ) -> Option<gdbstub::target::ext::breakpoints::SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl SwBreakpoint for RuneGdbTarget<'_> {
    fn add_sw_breakpoint(&mut self, addr: u32, _kind: usize) -> TargetResult<bool, Self> {
        let pc_offset = (addr & 0xFFFF) as u32;

        let module = unsafe {
            let store_ptr: *mut StoreContextMut<'_, JumpjetRuntimeState> =
                std::mem::transmute(&mut self.store);
            (*store_ptr)
                .debug_exit_frames()
                .next()
                .and_then(|f| f.module(&mut *store_ptr).ok().flatten())
        };

        if let Some(module) = module {
            self.store
                .as_context_mut()
                .edit_breakpoints()
                .expect("debug mode enabled")
                .add_breakpoint(&module, wasmtime::ModulePC::new(pc_offset))
                .map_err(|e| gdbstub::target::TargetError::Fatal(anyhow::anyhow!("{:?}", e)))?;
            return Ok(true);
        }
        Ok(false)
    }

    fn remove_sw_breakpoint(&mut self, addr: u32, _kind: usize) -> TargetResult<bool, Self> {
        let pc_offset = (addr & 0xFFFF) as u32;

        let module = unsafe {
            let store_ptr: *mut StoreContextMut<'_, JumpjetRuntimeState> =
                std::mem::transmute(&mut self.store);
            (*store_ptr)
                .debug_exit_frames()
                .next()
                .and_then(|f| f.module(&mut *store_ptr).ok().flatten())
        };

        if let Some(module) = module {
            self.store
                .as_context_mut()
                .edit_breakpoints()
                .expect("debug mode enabled")
                .remove_breakpoint(&module, wasmtime::ModulePC::new(pc_offset))
                .map_err(|e| gdbstub::target::TargetError::Fatal(anyhow::anyhow!("{:?}", e)))?;
            return Ok(true);
        }
        Ok(false)
    }
}

pub fn start_gdb_server(port: u16) -> std::io::Result<TcpListener> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    listener.set_nonblocking(true)?;
    Ok(listener)
}

pub fn handle_gdb_event(
    mut store: StoreContextMut<'_, JumpjetRuntimeState>,
    connection: &mut std::net::TcpStream,
) -> anyhow::Result<()> {
    let mut target = RuneGdbTarget {
        store: store.as_context_mut(),
    };

    let mut gdb = GdbStub::new(connection.try_clone()?).run_state_machine(&mut target)?;

    loop {
        gdb = match gdb {
            GdbStubStateMachine::Idle(mut inner) => {
                let byte = inner.borrow_conn().read().map_err(anyhow::Error::from)?;
                inner.incoming_data(&mut target, byte)?
            }
            GdbStubStateMachine::Running(_) => break,
            GdbStubStateMachine::CtrlCInterrupt(inner) => inner
                .interrupt_handled(&mut target, None::<gdbstub::stub::BaseStopReason<(), u32>>)?,
            GdbStubStateMachine::Disconnected(_) => break,
        };
    }

    Ok(())
}
