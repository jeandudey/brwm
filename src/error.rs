use super::xcb;

#[derive(Debug)]
pub enum WMError {
    GenericError(xcb::GenericError)
}
