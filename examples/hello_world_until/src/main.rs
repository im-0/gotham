//! A Hello World example application for working with Gotham.
//! Supports graceful shutdown on Ctrl+C.

extern crate gotham;
extern crate hyper;
extern crate mime;
extern crate tokio;
extern crate tokio_signal;

use hyper::{Response, StatusCode};

use gotham::helpers::http::response::create_response;
use gotham::state::State;

use tokio::prelude::Future;
use tokio::prelude::Stream;

/// Create a `Handler` which is invoked when responding to a `Request`.
///
/// How does a function become a `Handler`?.
/// We've simply implemented the `Handler` trait, for functions that match the signature used here,
/// within Gotham itself.
pub fn say_hello(state: State) -> (State, Response) {
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((String::from("Hello World!").into_bytes(), mime::TEXT_PLAIN)),
    );

    (state, res)
}

/// Start a server and call the `Handler` we've defined above for each `Request` we receive.
pub fn main() {
    // Future to wait for Ctrl+C.
    let signal = tokio_signal::ctrl_c()
        .flatten_stream()
        .map_err(|error| panic!("Error listening for signal: {}", error))
        .take(1)
        .for_each(|()| {
            println!("Ctrl+C pressed");
            Ok(())
        });

    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    println!("Press Ctrl+C to exit");

    let server = gotham::create_server(addr, || Ok(say_hello));
    // Wait either for server to finish (never happens) or for signal.
    // We are not interested on the other future, so we drop it using map() and map_err().
    let with_signal = server
        .select(signal)
        .map(|(ok, _)| ok)
        .map_err(|(error, _)| error);

    tokio::run(with_signal);

    println!("Shutting down gracefully");
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;

    #[test]
    fn receive_hello_world_response() {
        let test_server = TestServer::new(|| Ok(say_hello)).unwrap();
        let response = test_server
            .client()
            .get("http://localhost")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);

        let body = response.read_body().unwrap();
        assert_eq!(&body[..], b"Hello World!");
    }
}
