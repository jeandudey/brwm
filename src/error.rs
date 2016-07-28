use super::xcb;

#[derive(Debug)]
pub enum WMError {
    GenericError(xcb::GenericError),
}

impl From<xcb::GenericError> for WMError {
    fn from(e: xcb::GenericError) -> WMError {
        WMError::GenericError(e)
    }
}
