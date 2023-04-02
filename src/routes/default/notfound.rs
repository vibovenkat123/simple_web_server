use std::{net::TcpStream, io::Write};
pub fn send(mut stream: TcpStream, not_found: &str) {
   let not_found_msg = "Not Found";
   let length = not_found_msg.len();
   let response = format!("{not_found}\r\nContent-Length: {length}\r\n\r\n{not_found_msg}");
   stream.write_all(response.as_bytes()).unwrap()
}
