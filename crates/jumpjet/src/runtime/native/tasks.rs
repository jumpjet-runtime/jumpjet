use wasmtime::Result;
use wasmtime::component::Resource;

use super::state::JumpjetRuntimeState;
use crate::jumpjet::runtime::tasks::*;
use crate::runtime::common::tasks::TaskState;

impl Host for JumpjetRuntimeState {}

impl HostBuffer for JumpjetRuntimeState {
    async fn size(&mut self, buffer: Resource<Buffer>) -> u32 {
        self.table.get(&buffer).unwrap().0.len() as u32
    }

    async fn read(&mut self, buffer: Resource<Buffer>, offset: u32, length: u32) -> Vec<u8> {
        let bytes = &self.table.get(&buffer).unwrap().0;
        let start = (offset as usize).min(bytes.len());
        let end = start.saturating_add(length as usize).min(bytes.len());
        bytes[start..end].to_vec()
    }

    async fn drop(&mut self, rep: Resource<Buffer>) -> Result<()> {
        self.table.delete(rep).ok();
        Ok(())
    }
}

impl HostTask for JumpjetRuntimeState {
    async fn result(&mut self, task: Resource<Task>) -> TaskResult {
        // Clone the shared handle so the immutable `table` borrow is released
        // before we push a buffer resource on completion.
        let state = self.table.get(&task).unwrap().state.clone();
        let snapshot = {
            let guard = state.lock().unwrap();
            match &*guard {
                TaskState::Pending => None,
                TaskState::Complete(bytes) => Some(Ok(bytes.clone())),
                TaskState::Failed(err) => Some(Err(err.clone())),
            }
        };
        match snapshot {
            None => TaskResult::Pending,
            Some(Err(err)) => TaskResult::Failed(err),
            Some(Ok(bytes)) => {
                let buffer = self.table.push(Buffer(bytes)).unwrap();
                TaskResult::Complete(Some(TaskData::Buffer(buffer)))
            }
        }
    }

    async fn drop(&mut self, rep: Resource<Task>) -> Result<()> {
        self.table.delete(rep).ok();
        Ok(())
    }
}
