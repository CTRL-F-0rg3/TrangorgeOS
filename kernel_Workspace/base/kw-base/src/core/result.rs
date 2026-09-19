use super::error::KernelError;

pub type KResult<T> = core::result::Result<T, KernelError>;

pub trait IntoKResult<T> {
    fn into_kresult(self) -> KResult<T>;
}

impl<T> IntoKResult<T> for Option<T> {
    fn into_kresult(self) -> KResult<T> {
        self.ok_or(KernelError::NotFound)
    }
}

impl<T> IntoKResult<T> for Result<T, kstd_base::Status> {
    fn into_kresult(self) -> KResult<T> {
        self.map_err(KernelError::from)
    }
}