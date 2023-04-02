use crate::{send_json, InfoMsg};
use std::net::TcpStream;
pub fn send(stream: TcpStream) {
    let info = InfoMsg {
        msg: "Hello from root".to_string(),
    };
    send_json(stream, info);
}
