use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::{
    error::Error,
    fmt,
    net::{TcpListener, TcpStream},
};
use std::{fmt, fs};

pub struct SimpleServer {
    listener: TcpListener,
    port: u16,
    host: String,
    raw_http_response: String,
}

/// Request-Line = Method SP Request-URI SP HTTP-Version CRLF
struct Request<'a> {
    method: &'a str,
    uri: &'a str,
    http_version: &'a str,
}

impl<'a> fmt::Display for Request<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}\r\n", self.method, self.uri, self.http_version)
    }
}

/// This is a simple server that
impl SimpleServer {
    pub fn new(
        host: &str,
        port: Option<u16>,
        raw_http_response: &str,
    ) -> Result<Self, anyhow::Error> {
        let listener = TcpListener::bind(format!("{}:{}", host, port.ok_or(0).unwrap()))?;

        let port = listener.local_addr()?.port();

        Ok(Self {
            listener,
            port,
            host: host.to_string(),
            raw_http_response: raw_http_response.to_string(),
        })
    }

    pub fn uri(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    pub fn start(&self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    match self.handle_connection(stream, self.raw_http_response.clone()) {
                        Ok(_) => (),
                        Err(e) => println!("Error handling connection: {}", e),
                    }
                }
                Err(e) => println!("Connection failed: {}", e),
            }
        }
    }

    fn handle_connection(
        &self,
        mut stream: TcpStream,
        raw_http_response: String,
    ) -> Result<(), Box<dyn Error>> {
        // 512 bytes is enough for a toy HTTP server
        let mut buffer = [0; 512];

        // writes stream into buffer
        stream.read(&mut buffer).unwrap();

        let request = String::from_utf8_lossy(&buffer[..]);
        let request_line = request.lines().next().unwrap();

        match Self::parse_request_line(&request_line) {
            Ok(request) => {
                println!("Request: {}", &request);

                let response = format!("{}", raw_http_response.clone());

                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            Err(e) => print!("Bad request: {}", e),
        }

        Ok(())
    }

    fn parse_request_line(request: &str) -> Result<Request, Box<dyn Error>> {
        let mut parts = request.split_whitespace();

        let method = parts.next().ok_or("Method not specified")?;

        let uri = parts.next().ok_or("URI not specified")?;

        let http_version = parts.next().ok_or("HTTP version not specified")?;

        Ok(Request {
            method,
            uri,
            http_version,
        })
    }
}
