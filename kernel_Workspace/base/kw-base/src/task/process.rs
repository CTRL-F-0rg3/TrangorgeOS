use kstd_base::{Pid, WorldId, Status};
use crate::core::{KResult, KernelError};
use gluecore::bridge::mm::pmm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Created,
    Running,
    Sleeping,
    Zombie,
    Dead,
}

pub struct Process {
    pub pid: Pid,
    pub world_id: WorldId,
    pub state: ProcessState,
    pub cr3_phys: u64,
}

impl Process {
    pub fn new(pid: Pid, world_id: WorldId) -> KResult<Self> {
        let cr3 = pmm::alloc_zero_frame().map_err(KernelError::from)?;
        Ok(Self {
            pid,
            world_id,
            state: ProcessState::Created,
            cr3_phys: cr3.0,
        })
    }

    pub fn transition(&mut self, new_state: ProcessState) -> KResult<()> {
        match (self.state, new_state) {
            (ProcessState::Created, ProcessState::Running) |
            (ProcessState::Running, ProcessState::Sleeping) |
            (ProcessState::Sleeping, ProcessState::Running) |
            (ProcessState::Running, ProcessState::Zombie) |
            (ProcessState::Zombie, ProcessState::Dead) => {
                self.state = new_state;
                Ok(())
            }
            _ => Err(KernelError::InvalidArg),
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if self.cr3_phys != 0 {
            let _ = pmm::free_frame(kstd_base::PhysAddr(self.cr3_phys));
        }
    }
}