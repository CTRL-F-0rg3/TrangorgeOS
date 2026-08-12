
pub struct PackageManifest {
    pub id: PackageId,
    pub kind: ComponentKind,
    pub description: alloc::string::String,
    pub dependencies: alloc::vec::Vec<Dependency>,
    pub files: alloc::vec::Vec<FileEntry>,
    pub checksum: [u8; 32],  // weryfikacja integralności
}

pub struct Dependency {
    pub name: alloc::string::String,
    pub min_version: Version,  // minimalna wymagana wersja
}

pub struct FileEntry {
    pub path: alloc::string::String,      // gdzie ma trafić
    pub source: alloc::string::String,    // skąd wziąć
    pub size: u64,
}