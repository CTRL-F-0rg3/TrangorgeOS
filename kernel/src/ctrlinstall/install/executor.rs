// install/executor.rs

pub struct InstallExecutor<'a> {
    state: &'a mut SystemState,
}

impl<'a> InstallExecutor<'a> {
    pub fn execute(&mut self, tx: &Transaction) -> Result<(), CtrlInstallError> {
        for step in &tx.steps {
            match step {
                TransactionStep::InstallPackage(id) => { /* copy files */ }
                TransactionStep::RegisterDriver { .. } => { /* register driver */ }
                TransactionStep::LoadModule { .. } => { /* load module */ }
                TransactionStep::ApplyConfig { .. } => { /* apply config */ }
            }
        }
        todo!()
    }
}