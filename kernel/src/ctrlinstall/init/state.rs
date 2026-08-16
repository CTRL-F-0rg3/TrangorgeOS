
pub struct SystemState {
    pub packages: alloc::vec::Vec<InstalledPackage>,
    pub last_update: u64,  // timestamp
}

pub struct InstalledPackage {
    pub id: PackageId,
    pub kind: ComponentKind,
    pub status: PackageStatus,
    pub installed_at: u64,
    pub files: alloc::vec::Vec<alloc::string::String>,  // paths to files
    pub dependencies: alloc::vec::Vec<alloc::string::String>,
}

impl SystemState {
    pub fn is_installed(&self, name: &str) -> bool { todo!() }
    pub fn get_package(&self, name: &str) -> Option<&InstalledPackage> { todo!() }
    pub fn register(&mut self, pkg: InstalledPackage) { todo!() }
    pub fn unregister(&mut self, name: &str) -> Result<(), CtrlInstallError> { todo!() }
}