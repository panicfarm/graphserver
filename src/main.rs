use {
    hyper::{
        http::{Request, Response, StatusCode},
        server::conn::AddrStream,
        service::Service,
        Body, Server,
    },
    std::boxed::Box,
    std::future::Future,
    std::net::SocketAddr,
    std::pin::Pin,
    std::task::{Context, Poll},
};

struct HelloWorld;

impl Service<Request<Vec<u8>>> for HelloWorld {
    type Response = Response<Vec<u8>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Vec<u8>>) -> Self::Future {
        // create the body
        let body: Vec<u8> = "hello, world!\n".as_bytes().to_owned();
        // Create the HTTP response
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .expect("Unable to create `http::Response`");

        // create a response in a future.
        let fut = async { Ok(resp) };

        // Return the response as an immediate future
        //Call let res = task::spawn_blocking(move || { graph.edges()
        Box::pin(fut)
    }
}

impl Service<AddrStream> for HelloWorld {
    type Response = Response<Vec<u8>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([208, 93, 231, 240], 3000));
    let srv = HelloWorld {};
    // Create a server bound on the provided address
    let serve_future = Server::bind(&addr).serve(srv);

    // Wait for the server to complete serving or exit with an error.
    // If an error occurred, print it to stderr.
    if let Err(e) = serve_future.await {
        eprintln!("server error: {}", e);
    }
}
