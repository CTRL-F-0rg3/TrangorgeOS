pub struct ResolvedPlan {
    pub order: alloc::vec::Vec<PackageId>,
}

pub fn resolve(
    target: &PackageId,
    index: &RepositoryIndex,
    state: &SystemState,
) -> Result<ResolvedPlan, CtrlInstallError> {
    todo!()
}
