use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::{Duration, Instant},
};

use url::Url;

use super::oauth::{loopback_callback_address, loopback_callback_path};
use super::{LinearOAuthConfiguration, LinearOAuthError};

const CALLBACK_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const MAX_REQUEST_BYTES: usize = 8 * 1024;

pub struct LoopbackCallback {
    pub code: String,
    pub state: String,
}

pub fn bind_loopback_callback(
    configuration: &LinearOAuthConfiguration,
) -> Result<TcpListener, LinearOAuthError> {
    TcpListener::bind(loopback_callback_address(configuration)?)
        .and_then(|listener| {
            listener.set_nonblocking(true)?;
            Ok(listener)
        })
        .map_err(|error| LinearOAuthError::Callback(format!("loopback callback failed: {error}")))
}

pub fn await_loopback_callback(
    listener: TcpListener,
    configuration: &LinearOAuthConfiguration,
) -> Result<LoopbackCallback, LinearOAuthError> {
    let expected_path = loopback_callback_path(configuration)?;
    let deadline = Instant::now() + CALLBACK_TIMEOUT;
    while Instant::now() < deadline {
        match listener.accept() {
            Ok((mut stream, _)) => match callback_from_stream(&mut stream, &expected_path) {
                Ok(callback) => {
                    respond(
                        &mut stream,
                        "200 OK",
                        "Authorization received. You can return to Kanban AI Orchestrator.",
                    );
                    return Ok(callback);
                }
                Err(_) => {
                    respond(
                        &mut stream,
                        "400 Bad Request",
                        "This callback is not valid for the pending authorization.",
                    );
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(error) => {
                return Err(LinearOAuthError::Callback(format!(
                    "loopback callback failed: {error}"
                )));
            }
        }
    }
    Err(LinearOAuthError::Callback(
        "Linear authorization timed out".to_owned(),
    ))
}

fn callback_from_stream(
    stream: &mut TcpStream,
    expected_path: &str,
) -> Result<LoopbackCallback, LinearOAuthError> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| LinearOAuthError::Callback(error.to_string()))?;
    let request = read_request(stream)?;
    callback_from_request(&request, expected_path)
}

fn read_request(reader: &mut impl Read) -> Result<String, LinearOAuthError> {
    let mut bytes = Vec::new();
    let mut buffer = [0; 1024];
    while bytes.len() < MAX_REQUEST_BYTES {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| LinearOAuthError::Callback(error.to_string()))?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..read]);
        if bytes.windows(4).any(|window| window == b"\r\n\r\n") {
            return String::from_utf8(bytes)
                .map_err(|_| LinearOAuthError::Callback("callback was not UTF-8".to_owned()));
        }
    }
    Err(LinearOAuthError::Callback(
        "callback request was incomplete or too large".to_owned(),
    ))
}

fn callback_from_request(
    request: &str,
    expected_path: &str,
) -> Result<LoopbackCallback, LinearOAuthError> {
    let request_line = request
        .lines()
        .next()
        .ok_or_else(|| LinearOAuthError::Callback("callback request was empty".to_owned()))?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next();
    let target = parts.next();
    let version = parts.next();
    if method != Some("GET") || version != Some("HTTP/1.1") || parts.next().is_some() {
        return Err(LinearOAuthError::Callback(
            "callback request used an invalid method or target".to_owned(),
        ));
    }
    let target = target.ok_or_else(|| {
        LinearOAuthError::Callback("callback request was missing a target".to_owned())
    })?;
    let url = Url::parse(&format!("http://loopback{target}"))
        .map_err(|_| LinearOAuthError::Callback("callback target was invalid".to_owned()))?;
    if url.path() != expected_path {
        return Err(LinearOAuthError::Callback(
            "callback path did not match the pending authorization".to_owned(),
        ));
    }
    Ok(LoopbackCallback {
        code: required_query_parameter(&url, "code")?,
        state: required_query_parameter(&url, "state")?,
    })
}

fn required_query_parameter(url: &Url, name: &str) -> Result<String, LinearOAuthError> {
    let values = url
        .query_pairs()
        .filter(|(parameter, _)| parameter == name)
        .map(|(_, value)| value.into_owned())
        .collect::<Vec<_>>();
    match values.as_slice() {
        [value] if !value.trim().is_empty() => Ok(value.clone()),
        _ => Err(LinearOAuthError::Callback(format!(
            "callback must include exactly one non-empty {name} parameter"
        ))),
    }
}

fn respond(stream: &mut TcpStream, status: &str, message: &str) {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{message}",
        message.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

#[cfg(test)]
mod tests {
    use std::io::{self, Cursor, Read};

    use super::{MAX_REQUEST_BYTES, callback_from_request, read_request};

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    #[test]
    fn parses_only_a_valid_get_callback_for_the_expected_path() {
        let callback = callback_from_request(
            "GET /linear/oauth/callback?code=one&state=two HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
            "/linear/oauth/callback",
        )
        .expect("callback should parse");

        assert_eq!(callback.code, "one");
        assert_eq!(callback.state, "two");
    }

    #[test]
    fn rejects_unexpected_methods_paths_and_duplicate_parameters() {
        for request in [
            "POST /linear/oauth/callback?code=one&state=two HTTP/1.1\r\n\r\n",
            "GET /wrong?code=one&state=two HTTP/1.1\r\n\r\n",
            "GET /linear/oauth/callback?code=one&code=two&state=three HTTP/1.1\r\n\r\n",
            "GET /linear/oauth/callback?code=&state=two HTTP/1.1\r\n\r\n",
            "GET /linear/oauth/callback?code=one&state= HTTP/1.1\r\n\r\n",
            "GET /linear/oauth/callback?code=one&state=two HTTP/2\r\n\r\n",
            "GET /linear/oauth/callback?code=one&state=two HTTP/1.1 extra\r\n\r\n",
            "",
        ] {
            assert!(callback_from_request(request, "/linear/oauth/callback").is_err());
        }
    }

    #[test]
    fn reads_a_complete_http_request_from_a_plain_reader() {
        let mut reader =
            Cursor::new(b"GET /linear/oauth/callback?code=one&state=two HTTP/1.1\r\n\r\n".to_vec());

        let request = read_request(&mut reader).expect("complete request should be read");

        assert!(request.ends_with("\r\n\r\n"));
    }

    #[test]
    fn rejects_incomplete_or_oversized_callback_requests() {
        for request in [
            b"GET /callback HTTP/1.1\r\n".to_vec(),
            vec![b'x'; MAX_REQUEST_BYTES],
        ] {
            assert!(read_request(&mut Cursor::new(request)).is_err());
        }
    }

    #[test]
    fn rejects_non_utf8_or_unreadable_callback_requests() {
        let mut invalid_utf8 = Cursor::new(vec![0xff, b'\r', b'\n', b'\r', b'\n']);
        assert!(read_request(&mut invalid_utf8).is_err());
        assert!(read_request(&mut FailingReader).is_err());
    }
}
