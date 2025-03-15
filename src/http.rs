use std::{collections::HashMap, fmt::{self, Display, Formatter}, io::{BufRead, BufReader}, net::TcpStream};

const CRLF: &str = "\r\n";

#[derive(Copy, Clone)]
pub enum HttpStatusCode {
  Continue = 100,
  SwitchingProtocols = 101,
  OK = 200,
  Created = 201,
  Accepted = 202,
  NonAuthoritativeInformation = 203,
  NoContent = 204,
  ResetContent = 205,
  PartialContent = 206,
  MultipleChoices = 300,
  MovedPermanently = 301,
  Found = 302,
  SeeOther = 303,
  NotModified = 304,
  UseProxy = 305,
  TemporaryRedirect = 307,
  BadRequest = 400,
  Unauthorized = 401,
  PaymentRequired = 402,
  Forbidden = 403,
  NotFound = 404,
  MethodNotAllowed = 405,
  NotAcceptable = 406,
  ProxyAuthenticationRequired = 407,
  RequestTimeOut = 408,
  Conflict = 409,
  Gone = 410,
  LengthRequired = 411,
  PreconditionFailed = 412,
  RequestEntityTooLarge = 413,
  RequestURITooLarge = 414,
  UnsupportedMediaType = 415,
  Requestedrangenotsatisfiable = 416,
  ExpectationFailed = 417,
  InternalServerError = 500,
  NotImplemented = 501,
  BadGateway = 502,
  ServiceUnavailable = 503,
  GatewayTimeOut = 504,
  HTTPVersionnotsupported = 505,
}

pub enum HttpMethod {
  UNKNOWN,
  OPTIONS,
  GET,
  HEAD,
  POST,
  DELETE,
  TRACE,
  CONNECT
}

pub struct HttpRequest {
  pub method: HttpMethod,
  pub uri: String,
  pub version: String,
  pub headers: Option<HashMap<String, String>>,
  pub body: Option<String>,
}

impl HttpRequest {
  pub fn new(method: &str, uri: &str, version: &str) -> HttpRequest {
    let http_method = parse_method(method);

    HttpRequest {
      method: http_method,
      uri: uri.to_string(),
      version: version.to_string(),
      headers: None,
      body: None,
    }
  }
}

fn parse_method(method: &str) -> HttpMethod {
  match method.to_uppercase().as_str() {
    "OPTIONS" => HttpMethod::OPTIONS,
    "GET" => HttpMethod::GET,
    "HEAD" => HttpMethod::HEAD,
    "POST" => HttpMethod::POST,
    "DELETE" => HttpMethod::DELETE,
    "TRACE" => HttpMethod::TRACE,
    "CONNECT" => HttpMethod::CONNECT,
    _ => HttpMethod::UNKNOWN
  }
}

pub enum HttpErrorKind {
  NoData,
  MalformedRequest,
}

pub struct HttpError {
  pub kind: HttpErrorKind,
  pub message: &'static str,
}

impl HttpError {
  fn new(kind: HttpErrorKind, message: &'static str) -> HttpError {
    HttpError { kind, message }
  }
}

impl HttpRequest {
  pub fn parse(stream: &TcpStream) -> Result<HttpRequest, HttpError> {
    let buf_reader = BufReader::new(stream);
    let tcp_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap_or(String::new()))
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {tcp_request:#?}");

    let request_line = match tcp_request.first() {
        Some(x) => x,
        None => return Err(HttpError::new(HttpErrorKind::NoData, "No data on TCP stream")),
    };

    let request_line_tokens: Vec<&str> = request_line.split(' ').collect();

    if request_line_tokens.len() != 3 || !request_line_tokens[2].starts_with("HTTP/") {
        return Err(HttpError::new(HttpErrorKind::MalformedRequest, "Malformed request"));
    }

    let method = request_line_tokens[0];
    let uri = request_line_tokens[1];
    let version = request_line_tokens[2];

    Ok(HttpRequest::new(method, uri, version))
  }
}

pub struct HttpResponse {
  pub version: String,
  pub status_code: HttpStatusCode,
  pub headers: Option<HashMap<String, String>>,
  pub body: Option<String>,
}

// TODO: change to builder pattern
impl HttpResponse {
  pub fn new(
    status_code: HttpStatusCode,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
  ) -> HttpResponse {
    let headers = match (headers, &body) {
        (None, None) => None,
        (None, Some(b)) => {
          let mut new_headers = HashMap::new();
          new_headers.insert("Content-Type".to_string(), b.len().to_string());

          Some(new_headers)
        },
        (Some(h), None) => Some(h),
        (Some(h), Some(b)) => {
          let mut new_headers = h.clone();
          new_headers.insert("Content-Type".to_string(), b.len().to_string());

          Some(new_headers)
        },
    };

    let body = if body.is_some() {
      Some(body.unwrap().clone())
    } else {
      None
    };

    HttpResponse {
      version: "HTTP/1.1".to_string(),
      status_code,
      headers,
      body,
    }
  }
}

impl Display for HttpResponse {
  fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
    let reason = map_http_status_reason(&self.status_code);

    write!(fmt, "{} {} {reason} {CRLF}", self.version, self.status_code as u16);

    if (&self.headers).is_some() {
      let headers_strings: Vec<String> = self.headers
        .as_ref()
        .unwrap()
        .iter()
        .map(|(key, value)| format!("{key}: {value}"))
        .collect();
  
      if headers_strings.len() != 0 {
        write!(fmt, "{}", headers_strings.join("{CRLF}"));
      }
    }

    write!(fmt, "{CRLF}");

    if self.body.is_some() {
      write!(fmt, "{}", self.body.as_ref().unwrap());
    }

    return Ok(());
  }
}

fn map_http_status_reason (status_code: &HttpStatusCode) -> &str {
  match status_code {
    HttpStatusCode::Continue => "Continue",
    HttpStatusCode::SwitchingProtocols => "Switching Protocols",
    HttpStatusCode::OK => "OK",
    HttpStatusCode::Created => "Created",
    HttpStatusCode::Accepted => "Accepted",
    HttpStatusCode::NonAuthoritativeInformation => "Non-Authoritative Information",
    HttpStatusCode::NoContent => "No Content",
    HttpStatusCode::ResetContent => "Reset Content",
    HttpStatusCode::PartialContent => "Partial Content",
    HttpStatusCode::MultipleChoices => "Multiple Choices",
    HttpStatusCode::MovedPermanently => "Moved Permanently",
    HttpStatusCode::Found => "Found",
    HttpStatusCode::SeeOther => "See Other",
    HttpStatusCode::NotModified => "Not Modified",
    HttpStatusCode::UseProxy => "Use Proxy",
    HttpStatusCode::TemporaryRedirect => "Temporary Redirect",
    HttpStatusCode::BadRequest => "Bad Request",
    HttpStatusCode::Unauthorized => "Unauthorized",
    HttpStatusCode::PaymentRequired => "Payment Required",
    HttpStatusCode::Forbidden => "Forbidden",
    HttpStatusCode::NotFound => "Not Found",
    HttpStatusCode::MethodNotAllowed => "Method Not Allowed",
    HttpStatusCode::NotAcceptable => "Not Acceptable",
    HttpStatusCode::ProxyAuthenticationRequired => "Proxy Authentication Required",
    HttpStatusCode::RequestTimeOut => "Request Time-out",
    HttpStatusCode::Conflict => "Conflict",
    HttpStatusCode::Gone => "Gone",
    HttpStatusCode::LengthRequired => "Length Required",
    HttpStatusCode::PreconditionFailed => "Precondition Failed",
    HttpStatusCode::RequestEntityTooLarge => "Request Entity Too Large",
    HttpStatusCode::RequestURITooLarge => "Request-URI Too Large",
    HttpStatusCode::UnsupportedMediaType => "Unsupported Media Type",
    HttpStatusCode::Requestedrangenotsatisfiable => "Requested range not satisfiable",
    HttpStatusCode::ExpectationFailed => "Expectation Failed",
    HttpStatusCode::InternalServerError => "Internal Server Error",
    HttpStatusCode::NotImplemented => "Not Implemented",
    HttpStatusCode::BadGateway => "Bad Gateway",
    HttpStatusCode::ServiceUnavailable => "Service Unavailable",
    HttpStatusCode::GatewayTimeOut => "Gateway Time-out",
    HttpStatusCode::HTTPVersionnotsupported => "HTTP Version not supported",
  }
}