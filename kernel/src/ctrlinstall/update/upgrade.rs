
pub struct UpgradePlan {
    pub actions: alloc::vec::Vec<UpdateAction>,
    pub transaction: Transaction,
}

pub fn plan_upgrade(
    current: &SystemState,
    index: &RepositoryIndex,
) -> Result<UpgradePlan, CtrlInstallError> {
    todo!()
}

pub fn execute_upgrade(plan: &UpgradePlan, executor: &mut InstallExecutor) -> Result<(), CtrlInstallError> {
    executor.execute(&plan.transaction)
}