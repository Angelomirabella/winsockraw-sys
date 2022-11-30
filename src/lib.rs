#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::windows::io::{AsRawHandle, HandleOrInvalid, OwnedHandle};

    #[test]
    /// Test we are able to correctly interact with `WinSockRaw` API.
    fn test_bindings() {
        unsafe {
            // Create socket.
            let raw_socket = OwnedHandle::try_from(HandleOrInvalid::from_raw_handle(SocketRawOpen()));

            assert_eq!(raw_socket.is_err(), false); 

            // Bind socket to `any` interface since we do not know in advance which interfaces
            // are available on a system. For the same reason, avoid testing `SocketRawRecv` (can't wait forever)
            // and `SocketRawSend`.
            assert_eq!(SocketRawBind(raw_socket.as_ref().unwrap().as_raw_handle(),
                                     WINSOCKRAW_INTERACE_ANY_INDEX),
                       TRUE as BOOL);

            // Close socket.
            SocketRawClose(raw_socket.unwrap().as_raw_handle());
        }
    }
}