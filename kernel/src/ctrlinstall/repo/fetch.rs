
pub trait PackageFetcher {
    fn fetch(&mut self, manifest: &PackageManifest) -> Result<alloc::vec::Vec<u8>, CtrlInstallError>;
}

pub struct LocalFetcher {
    pub base_path: alloc::string::String,
}

pub struct RemoteFetcher {
    pub url: alloc::string::String,
}