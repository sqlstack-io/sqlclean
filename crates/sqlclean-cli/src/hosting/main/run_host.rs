//! Send one request to a fresh Python host and read its response.

use crate::hosting::models::{HostRequest, HostResponse};

/// Run the Python host for `request`; a host failure or reported error is an `Err`.
pub fn run_host(request: &HostRequest) -> Result<HostResponse, String> {
    crate::hosting::_helpers::process::run_host(request)
}
