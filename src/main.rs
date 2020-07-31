use hyper::service::Service;
use hyper::{Body, Request, Response, Server};

use {
  chaindata::db::{ClGraphAdapter, Db},
  serde::{Deserialize, Serialize},
  std::{
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
  },
};

type Counter = i32;

#[tokio::main(core_threads = 2, max_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let addr = ([208, 93, 231, 240], 3001).into();

  let db_path = "/home/alecm/clustering/lmdb.100000";
  let dp = PathBuf::from(db_path);
  let db = Db::new(&dp, 100_000_000_000, false).unwrap();
  let db = Arc::new(db);

  let server = Server::bind(&addr).serve(MakeSvc {
    counter: 81818,
    db: db,
  });
  println!("Listening on http://{}", addr);

  server.await?;
  Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct GraphReq {
  vx_vec: Vec<u32>,
  bl_min: u32,
  bl_max: u32,
  flux_threshold: u64,
}
struct Svc {
  counter: Counter,
  db: Arc<Db>,
}

impl Service<Request<Body>> for Svc {
  type Response = Response<Body>;
  type Error = hyper::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

  fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, req: Request<Body>) -> Self::Future {
    fn mk_response(s: String) -> Result<Response<Body>, hyper::Error> {
      Ok(Response::builder().body(Body::from(s)).unwrap())
    }

    let res = match req.uri().path() {
      "/graph" => {
        let db = self.db.clone();
        return Box::pin(async move {
          let bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
          let graph_req: GraphReq = serde_json::from_slice(&bytes).unwrap();
          println!("{:?}", graph_req);
          println!(
            "{:?}\n Strong count {}",
            std::thread::current(),
            Arc::strong_count(&db)
          );
          //let res = task::spawn_blocking(move || {
          let graph = db.create_graph_adapter().unwrap();
          let edges = graph
            .edges(
              graph_req.vx_vec,
              graph_req.bl_min,
              graph_req.bl_max,
              graph_req.flux_threshold,
              2,
            )
            .unwrap();
          mk_response(serde_json::to_string(&edges).unwrap())
        });
      }
      _ => {
        return Box::pin(async {
          mk_response(String::from_utf8_lossy(include_bytes!("../files/index.html")).to_string())
        })
      }
    };
  }
}

struct MakeSvc {
  counter: Counter,
  db: Arc<Db>,
}

impl<T> Service<T> for MakeSvc {
  type Response = Svc;
  type Error = hyper::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

  fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, _: T) -> Self::Future {
    //let counter = self.counter.clone();
    let counter = 0;
    dbg!("hi");
    dbg!(Arc::strong_count(&self.db));
    let db = self.db.clone();
    dbg!(Arc::strong_count(&self.db));
    let fut = async move { Ok(Svc { counter, db }) };
    Box::pin(fut)
  }
}
