#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageId {
    pub name: alloc::string::String,
    pub version: Version,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKind {
    Driver,
    FileSystem,
    KernelModule,
    Config,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageStatus {
    Available,
    Installed,
    Broken,
    Pending,
}

#[derive(Debug)]
pub enum CtrlInstallError {
    PackageNotFound(alloc::string::String),
    DependencyMissing(alloc::string::String),
    VersionConflict {
        package: alloc::string::String,
        required: Version,
        found: Version,
    },
    IoError,
    InvalidManifest,
}
