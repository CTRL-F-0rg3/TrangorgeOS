pub mod common;
pub mod init;
pub mod repo;
pub mod install;
pub mod update;

use common::*;

pub struct CtrlInstall {
    state: init::state::SystemState,
    index: repo::index::RepositoryIndex,
}

impl CtrlInstall {
    pub fn new() -> Result<Self, CtrlInstallError> {
        let boot = init::bootstrap::bootstrap()?;
        let index = repo::index::RepositoryIndex::load()?;
        Ok(Self { state: boot.state, index })
    }

    pub fn install(&mut self, name: &str) -> Result<(), CtrlInstallError> {
        let manifest = self.index.find(name)
            .ok_or(CtrlInstallError::PackageNotFound(name.into()))?;

        let plan = install::resolver::resolve(&manifest.id, &self.index, &self.state)?;
        let tx = install::transaction::build(&plan, &self.index)?;
        let mut executor = install::executor::InstallExecutor::new(&mut self.state);
        executor.execute(&tx)?;

        Ok(())
    }

    pub fn update(&mut self) -> Result<(), CtrlInstallError> {
        let plan = update::upgrade::plan_upgrade(&self.state, &self.index)?;
        let mut executor = install::executor::InstallExecutor::new(&mut self.state);
        update::upgrade::execute_upgrade(&plan, &mut executor)
    }

    pub fn list_installed(&self) -> &[init::state::InstalledPackage] {
        &self.state.packages
    }
}
