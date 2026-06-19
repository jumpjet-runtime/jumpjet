use crate::runtime::JumpjetRuntimeState;
use cranelift_entity::EntityRef;
use wasmtime::{AsContextMut, StoreContextMut};

pub struct DebugContext<'a> {
    pub store: StoreContextMut<'a, JumpjetRuntimeState>,
    pub cached_stack: Option<Vec<StackFrameInfo>>,
}

impl<'a> DebugContext<'a> {
    pub fn new(
        store: StoreContextMut<'a, JumpjetRuntimeState>,
        cached_stack: Option<Vec<StackFrameInfo>>,
    ) -> Self {
        Self {
            store,
            cached_stack,
        }
    }

    /// Reads the current PC as a 64-bit value: (func_idx << 32) | pc_offset.
    /// Returns None if no frame is active or PC cannot be determined.
    pub fn read_pc(&mut self) -> Option<u64> {
        let mut store = self.store.as_context_mut();

        // Wasmtime's debug_exit_frames() borrows the store immutably/mutably depending on usage.
        // frame.wasm_function_index_and_pc() requires &mut store.
        // This creates a borrow conflict: we hold the iterator (frame) which borrows store,
        // and try to pass &mut store to a method on frame.
        // We must use unsafe to bypass the borrow checker here, knowing that inspection shouldn't invalidate the store.
        unsafe {
            let store_ptr: *mut StoreContextMut<'_, JumpjetRuntimeState> =
                std::mem::transmute(&mut store);
            if let Some(frame) = (*store_ptr).debug_exit_frames().next() {
                if let Ok(Some((func_idx, pc_offset))) =
                    frame.wasm_function_index_and_pc(&mut *store_ptr)
                {
                    return Some(((func_idx.index() as u64) << 32) | (pc_offset.raw() as u64));
                }
            }
        }
        None
    }

    pub fn read_memory(&mut self, start_addr: u32, data: &mut [u8]) -> usize {
        let mut store = self.store.as_context_mut();

        unsafe {
            let store_ptr: *mut StoreContextMut<'_, JumpjetRuntimeState> =
                std::mem::transmute(&mut store);
            if let Some(frame) = (*store_ptr).debug_exit_frames().next() {
                if let Ok(instance) = frame.instance(&mut *store_ptr) {
                    if let Some(memory) = instance.debug_memory(&mut *store_ptr, 0) {
                        let memory_data = memory.data(&*store_ptr);
                        let start = start_addr as usize;
                        if start < memory_data.len() {
                            let end = (start + data.len()).min(memory_data.len());
                            let len = end - start;
                            data[..len].copy_from_slice(&memory_data[start..end]);
                            return len;
                        }
                    }
                }
            }
        }

        0
    }

    pub fn add_breakpoint(&mut self, addr: u32) -> anyhow::Result<bool> {
        let pc_offset = (addr & 0xFFFF) as u32;
        let mut store = self.store.as_context_mut();

        let module = unsafe {
            let store_ptr: *mut StoreContextMut<'_, JumpjetRuntimeState> =
                std::mem::transmute(&mut store);
            let frames = (*store_ptr).debug_exit_frames();
            // Check if frames is empty
            let mut frames_clone = (*store_ptr).debug_exit_frames();
            if frames_clone.next().is_none() {
                // eprintln!("add_breakpoint: debug_exit_frames is empty! Cannot find Module to set breakpoint.");
                return Ok(false);
            }

            (*store_ptr)
                .debug_exit_frames()
                .next()
                .and_then(|f| f.module(&mut *store_ptr).ok().flatten())
        };

        if let Some(module) = module {
            eprintln!(
                "add_breakpoint: Setting breakpoint at offset {:#x}",
                pc_offset
            );
            store
                .edit_breakpoints()
                .expect("debug mode enabled")
                .add_breakpoint(&module, wasmtime::ModulePC::new(pc_offset))?;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn add_breakpoint_with_module(
        &mut self,
        module: &wasmtime::Module,
        addr: u32,
    ) -> anyhow::Result<bool> {
        let pc_offset = (addr & 0xFFFF) as u32;
        let mut store = self.store.as_context_mut();

        eprintln!(
            "add_breakpoint_with_module: Setting breakpoint at offset {:#x} using provided module",
            pc_offset
        );
        store
            .edit_breakpoints()
            .expect("debug mode enabled")
            .add_breakpoint(module, wasmtime::ModulePC::new(pc_offset))?;
        Ok(true)
    }

    pub fn remove_breakpoint(&mut self, addr: u32) -> anyhow::Result<bool> {
        let pc_offset = (addr & 0xFFFF) as u32;
        let mut store = self.store.as_context_mut();

        let module = unsafe {
            let store_ptr: *mut StoreContextMut<'_, JumpjetRuntimeState> =
                std::mem::transmute(&mut store);
            (*store_ptr)
                .debug_exit_frames()
                .next()
                .and_then(|f| f.module(&mut *store_ptr).ok().flatten())
        };

        if let Some(module) = module {
            store
                .edit_breakpoints()
                .expect("debug mode enabled")
                .remove_breakpoint(&module, wasmtime::ModulePC::new(pc_offset))?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn single_step(&mut self) -> anyhow::Result<()> {
        self.store
            .as_context_mut()
            .edit_breakpoints()
            .expect("debug mode enabled")
            .single_step(true)?;
        Ok(())
    }

    pub fn get_stack_trace(&mut self) -> Vec<StackFrameInfo> {
        if let Some(cached) = &self.cached_stack {
            return cached.clone();
        }

        let mut store = self.store.as_context_mut();
        let mut frames = Vec::new();

        unsafe {
            let store_ptr: *mut StoreContextMut<'_, JumpjetRuntimeState> =
                std::mem::transmute(&mut store);
            let exit_frames = (*store_ptr).debug_exit_frames();
            let count = exit_frames.count(); // consumes iterator, so we need to collect or re-iterate? 
            // debug_exit_frames returns an iterator. We can't consume it to count and then iterate.
            // Let's just iterate and count.
            eprintln!("Getting stack trace...");

            for (index, frame) in (*store_ptr).debug_exit_frames().enumerate() {
                let func_idx = frame
                    .wasm_function_index_and_pc(&mut *store_ptr)
                    .ok()
                    .flatten()
                    .map(|(idx, _)| idx.index())
                    .unwrap_or(0); // Default to 0 if unknown

                let pc_offset = frame
                    .wasm_function_index_and_pc(&mut *store_ptr)
                    .ok()
                    .flatten()
                    .map(|(_, pc)| pc.raw())
                    .unwrap_or(0);

                let func_name = format!("func<{}>", func_idx);
                eprintln!("Frame {}: {} @ {:#x}", index, func_name, pc_offset);

                // TODO: Get source file and line number if available

                frames.push(StackFrameInfo {
                    id: index as i64,
                    name: func_name,
                    source: None,
                    line: 0,
                    column: 0,
                    instruction_pointer_reference: Some(format!("0x{:x}", pc_offset)),
                });
            }
        }

        eprintln!("Found {} frames", frames.len());
        frames
    }
}

#[derive(Debug, Clone)]
pub struct StackFrameInfo {
    pub id: i64,
    pub name: String,
    pub source: Option<String>,
    pub line: i64,
    pub column: i64,
    pub instruction_pointer_reference: Option<String>,
}
