
pub struct PackageManifest {
    pub id: PackageId,
    pub kind: ComponentKind,
    pub description: alloc::string::String,
    pub dependencies: alloc::vec::Vec<Dependency>,
    pub files: alloc::vec::Vec<FileEntry>,
    pub checksum: [u8; 32],  // integrity verification
}

pub struct Dependency {
    pub name: alloc::string::String,
    pub min_version: Version,  // minimum required version
}

pub struct FileEntry {
    pub path: alloc::string::String,      // gdzie ma trafić
    pub source: alloc::string::String,    // skąd wziąć
    pub size: u64,
}