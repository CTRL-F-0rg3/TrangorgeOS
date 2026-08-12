// install/executor.rs

pub struct InstallExecutor<'a> {
    state: &'a mut SystemState,
}

impl<'a> InstallExecutor<'a> {
    pub fn execute(&mut self, tx: &Transaction) -> Result<(), CtrlInstallError> {
        for step in &tx.steps {
            match step {
                TransactionStep::InstallPackage(id) => { /* kopiuj pliki */ }
                TransactionStep::RegisterDriver { .. } => { /* zarejestruj sterownik */ }
                TransactionStep::LoadModule { .. } => { /* załaduj moduł */ }
                TransactionStep::ApplyConfig { .. } => { /* zastosuj config */ }
            }
        }
        todo!()
    }
}