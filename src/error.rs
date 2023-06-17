use ffi;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An unspecified error occurred.")]
    Failure,
    #[error("The system ran out of memory.")]
    OutOfMemory,
    #[error("An error occurred while initializing an external dependency.")]
    Initialization,
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn check<T>(status: ffi::IPLerror, value: T) -> Result<T> {
    match status {
        ffi::IPLerror_IPL_STATUS_SUCCESS => Ok(value),
        ffi::IPLerror_IPL_STATUS_FAILURE => Err(Error::Failure),
        ffi::IPLerror_IPL_STATUS_OUTOFMEMORY => Err(Error::OutOfMemory),
        ffi::IPLerror_IPL_STATUS_INITIALIZATION => Err(Error::Initialization),
        _ => unreachable!(),
    }
}
