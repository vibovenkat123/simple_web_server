use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
mod routes;
use serde::Serialize;
const STATUS_LINE_OK: &str = "HTTP/1.1 200 OK";
const STATUS_LINE_NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND";

#[derive(Serialize, Debug)]
struct InfoMsg {
    msg: String,
}

fn gen_json_response(status_line: &str, body: &str) -> String {
    let response = format!("{status_line}\r\nContent-Type: application/json\r\n\r\n{body}");
    response
}

fn send_json<T: serde::Serialize>(mut stream: TcpStream, body: T) {
    let serialized = serde_json::to_string(&body).unwrap();
    let response = gen_json_response(STATUS_LINE_OK, &serialized);
    stream.write_all(response.as_bytes()).unwrap()
}

pub fn listen() {
    let listener = TcpListener::bind("127.0.0.1:5454").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_conn(stream)
    }
}

fn handle_conn(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let request_type = &request_line[0..request_line.find("/").unwrap()].trim();

    let route_path = request_line
        .strip_prefix(request_type)
        .unwrap()
        .strip_suffix("HTTP/1.1")
        .unwrap()
        .trim();
    handle_paths(stream, &request_type, &route_path)
}

fn handle_paths(stream: TcpStream, request_type: &str, route_path: &str) {
    match request_type {
        "GET" => match route_path {
            "/" => routes::root::send(stream),
            _ => {
                routes::default::notfound::send(stream, STATUS_LINE_NOT_FOUND);
            }
        },
        &_ => {
            routes::default::notfound::send(stream, STATUS_LINE_NOT_FOUND);
        }
    }
}
