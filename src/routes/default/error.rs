use std::{net::TcpStream, io::Write};
pub fn send(mut stream: TcpStream, error: &str, error_msg: &str) {
   let length = error_msg.len();
   let response = format!("{error}\r\nContent-Length: {length}\r\n\r\n{error_msg}");
   stream.write_all(response.as_bytes()).unwrap()
}
