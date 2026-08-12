
pub enum UpdateAction {
    Upgrade { from: Version, to: Version },
    InstallNew(PackageId),
    Remove(PackageId),
}

pub fn diff(
    current: &SystemState,
    index: &RepositoryIndex,
) -> alloc::vec::Vec<UpdateAction> {
    todo!()
}