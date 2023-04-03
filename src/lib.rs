use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread,
};
use urlencoding::decode;
mod routes;
use serde::Serialize;
const STATUS_LINE_OK: &str = "HTTP/1.1 200 OK";
const STATUS_LINE_NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND";
//const STATUS_LINE_BAD_REQUEST: &str = "HTTP/1.1 409 BAD REQUEST";
const NOT_FOUND: &str = "404 Not Found";
//const BAD_REQUEST: &str = "400 Bad Request";

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let msg = receiver.lock().unwrap().recv();
            match msg {
                Ok(job) => {
                    job()
                }
                Err(_) => {
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_response() {
        let test_info = InfoMsg {
            msg: "Test".to_string(),
        };
        let serialized = serde_json::to_string(&test_info).unwrap();
        let mut res = gen_json_response(STATUS_LINE_OK, &serialized);
        let mut res_good = format!(
            "{STATUS_LINE_OK}\r\nContent-Type: application/json\r\n\r\n{{\"msg\": \"Test\"}}"
        );
        res.retain(|char| !char.is_whitespace());
        res_good.retain(|char| !char.is_whitespace());
        assert_eq!(res, res_good);
    }
    #[test]
    fn text_response() {
        let text = "hello";
        let length = text.len();
        let res = gen_text_response(STATUS_LINE_OK, text);
        let res_good = format!("{STATUS_LINE_OK}\r\nContent-Length: {length}\r\n\r\n{text}");
        assert_eq!(res, res_good);
    }
    #[test]
    fn text_encoded_response() {
        let text = "hello world";
        let encoded_text = "hello%20world";
        let length = text.len();
        let res = gen_text_response(STATUS_LINE_OK, encoded_text);
        let res_good = format!("{STATUS_LINE_OK}\r\nContent-Length: {length}\r\n\r\n{text}");
        assert_eq!(res, res_good);
    }
}

#[derive(Serialize, Debug)]
struct InfoMsg {
    msg: String,
}

fn gen_json_response(status_line: &str, body: &str) -> String {
    let response = format!("{status_line}\r\nContent-Type: application/json\r\n\r\n{body}");
    response
}

fn gen_text_response(status_line: &str, body: &str) -> String {
    let body = decode(body).unwrap();
    let length = body.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{body}");
    response
}

fn send_json<T: serde::Serialize>(mut stream: TcpStream, body: T) {
    let serialized = serde_json::to_string(&body).unwrap();
    let response = gen_json_response(STATUS_LINE_OK, &serialized);
    stream.write_all(response.as_bytes()).unwrap()
}

fn send_text(mut stream: TcpStream, body: &str) {
    let response = gen_text_response(STATUS_LINE_OK, body);
    stream.write_all(response.as_bytes()).unwrap();
}

pub fn listen() {
    let listener = TcpListener::bind("127.0.0.1:5454").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_conn(stream);
        });
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
            x if (x.strip_prefix("/echo/").is_some())
                && !(x.strip_prefix("/echo/").unwrap().contains("/")) =>
            {
                let x = x.strip_prefix("/echo/").unwrap();
                routes::echo::send(stream, &x);
            }
            _ => {
                routes::default::error::send(stream, STATUS_LINE_NOT_FOUND, NOT_FOUND);
            }
        },
        &_ => {
            routes::default::error::send(stream, STATUS_LINE_NOT_FOUND, NOT_FOUND);
        }
    }
}
