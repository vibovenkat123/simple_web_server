use std::net::TcpStream;

use crate::send_text;

pub fn send(stream: TcpStream, text: &str) {
    send_text(stream, text)
}
