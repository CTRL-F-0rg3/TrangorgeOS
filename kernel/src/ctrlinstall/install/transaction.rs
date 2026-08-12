

pub enum TransactionStep {
    InstallPackage(PackageId),
    RegisterDriver { package: PackageId, driver_name: alloc::string::String },
    LoadModule { package: PackageId, module_path: alloc::string::String },
    ApplyConfig { package: PackageId, config_path: alloc::string::String },
}

pub struct Transaction {
    pub steps: alloc::vec::Vec<TransactionStep>,
    pub rollback: alloc::vec::Vec<TransactionStep>,  
}