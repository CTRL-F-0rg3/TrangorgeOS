pub struct RepositoryIndex {
    pub packages: alloc::vec::Vec<PackageManifest>,
}

impl RepositoryIndex {
    pub fn find(&self, name: &str) -> Option<&PackageManifest> { todo!() }
    pub fn find_by_kind(&self, kind: ComponentKind) -> alloc::vec::Vec<&PackageManifest> { todo!() }
    pub fn search(&self, query: &str) -> alloc::vec::Vec<&PackageManifest> { todo!() }
}
