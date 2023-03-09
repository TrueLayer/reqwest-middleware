use futures::future::BoxFuture;
use std::error::Error;
use std::fmt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

type CustomMessageHandler = Box<
    dyn Fn(TcpStream) -> BoxFuture<'static, Result<(), Box<dyn std::error::Error>>> + Send + Sync,
>;

/// This is a simple server that returns the responses given at creation time: [`self.raw_http_responses`] following a round-robin mechanism.
pub struct SimpleServer {
    listener: TcpListener,
    port: u16,
    host: String,
    raw_http_responses: Vec<String>,
    calls_counter: usize,
    custom_handler: Option<CustomMessageHandler>,
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

impl SimpleServer {
    /// Creates an instance of a [`SimpleServer`]
    /// If [`port`] is None os Some(0), it gets randomly chosen between the available ones.
    pub async fn new(
        host: &str,
        port: Option<u16>,
        raw_http_responses: Vec<String>,
    ) -> Result<Self, anyhow::Error> {
        let port = port.unwrap_or(0);
        let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;

        let port = listener.local_addr()?.port();

        Ok(Self {
            listener,
            port,
            host: host.to_string(),
            raw_http_responses,
            calls_counter: 0,
            custom_handler: None,
        })
    }

    pub fn set_custom_handler(
        &mut self,
        custom_handler: impl Fn(TcpStream) -> BoxFuture<'static, Result<(), Box<dyn std::error::Error>>>
            + Send
            + Sync
            + 'static,
    ) -> &mut Self {
        self.custom_handler.replace(Box::new(custom_handler));
        self
    }

    /// Returns the uri in which the server is listening to.
    pub fn uri(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Starts the TcpListener and handles the requests.
    pub async fn start(mut self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, _)) => {
                    match self.handle_connection(stream).await {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error handling connection: {}", e);
                        }
                    }
                    self.calls_counter += 1;
                }
                Err(e) => {
                    println!("Connection failed: {}", e);
                }
            }
        }
    }

    /// Asyncrounously reads from the buffer and handle the request.
    /// It first checks that the format is correct, then returns the response.
    ///
    /// Returns a 400 if the request if formatted badly.
    async fn handle_connection(&self, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
        if let Some(ref custom_handler) = self.custom_handler {
            return custom_handler(stream).await;
        }

        let mut buffer = vec![0; 1024];

        let n = stream.read(&mut buffer).await.unwrap();

        let request = String::from_utf8_lossy(&buffer[..n]);
        let request_line = request.lines().next().unwrap();

        let response = match Self::parse_request_line(request_line) {
            Ok(request) => {
                println!("== Request == \n{}\n=============", request);
                self.get_response().clone()
            }
            Err(e) => {
                println!("++ Bad request: {} ++++++", e);
                self.get_bad_request_response()
            }
        };

        println!("-- Response --\n{}\n--------------", response.clone());
        stream.write_all(response.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();

        Ok(())
    }

    /// Parses the request line and checks that it contains the method, uri and http_version parts.
    /// It does not check if the content of the checked parts is correct. It just checks the format (it contains enough parts) of the request.
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

    /// Returns the response to use based on the calls counter.
    /// It uses a round-robin mechanism.
    fn get_response(&self) -> String {
        let index = if self.calls_counter >= self.raw_http_responses.len() {
            self.raw_http_responses.len() % self.calls_counter
        } else {
            self.calls_counter
        };
        self.raw_http_responses[index].clone()
    }

    /// Returns the raw HTTP response in case of a 400 Bad Request.
    fn get_bad_request_response(&self) -> String {
        "HTTP/1.1 400 Bad Request\r\n\r\n".to_string()
    }
}
