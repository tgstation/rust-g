use std::net::UdpSocket;
use std::cell::RefCell;

use crate::{error::Result};

fn init_socket() -> UdpSocket {
    let sock = UdpSocket::bind("0.0.0.0:0").expect("Socket init failed.");
    sock.set_nonblocking(true).expect("Setting socket to non-blocking failed.");
    sock
}

thread_local! {
    static UDP_LOCAL: RefCell<UdpSocket> = RefCell::new(init_socket());
}

byond_fn! { udp_shipper_send(addr, data) {
    UDP_LOCAL.with(|cell| -> Result<()> {
        let sock = cell.borrow_mut();
        sock.send_to(data.as_bytes(), addr.to_string())?;

        Ok(())
    }).err()
} }