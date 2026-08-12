

pub struct BootstrapResult {
    pub is_first_boot: bool,
    pub state: state::SystemState,
}

pub fn bootstrap() -> Result<BootstrapResult, CtrlInstallError> {
    todo!()
}