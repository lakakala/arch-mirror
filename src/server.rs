use std::{
    collections::HashMap,
    sync::{atomic, Arc},
};
use crate::result::Result;
use http_body_util::Full;
use hyper::{body::Bytes, server::conn::http1, service::service_fn, Request, Response};
use std::convert::Infallible;
use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};

type ClientIdType = u64;

pub struct Server {
    listener: TcpListener,
    cli_handlers: HashMap<ClientIdType, JoinHandle<()>>,
    client_manager: ClientManager,
}

impl Server {
    pub async fn new() -> Result<Server> {
        let listener = TcpListener::bind("localhost:8080").await?;

        let server = Server {
            listener,
            cli_handlers: HashMap::new(),
            client_manager: ClientManager::new(),
        };

        return Result::Ok(server);
    }

    pub async fn start(&mut self) -> Result<()> {
        loop {
            let (conn, _) = self.listener.accept().await?;

            let client_id = self.client_manager.gen_client_id();

            let cli = Arc::new(Client::new(client_id));

            let cli_handler = tokio::task::spawn(cli.clone().start(conn));

            self.cli_handlers.insert(client_id, cli_handler);
        }
    }

    pub async fn stop(&self) {}
}

struct Client {
    client_id: ClientIdType,
}

impl Client {
    fn new(client_id: ClientIdType) -> Client {
        Client { client_id }
    }

    async fn start(self: Arc<Self>, conn: TcpStream) {
        // Finally, we bind the incoming connection to our `hello` service
        if let Err(err) = http1::Builder::new()
            // `service_fn` converts our function in a `Service`
            .serve_connection(
                hyper_util::rt::TokioIo::new(conn),
                service_fn(|req: Request<hyper::body::Incoming>| async {
                    return self.handle_request(req).await;
                }),
            )
            .await
        {
            eprintln!("Error serving connection: {:?}", err);
        }
    }

    async fn handle_request(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        // https://mirrors.tuna.tsinghua.edu.cn/archlinux/core/os/x86_64/core.db

        req.uri();
        Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
    }
}

fn parse_uri(uri: &hyper::Uri) -> Result<(String, String, String)> {

    let uri = uri.path();

    // let splits:Vec<> = uri.split("/").collect();


    todo!()
}

struct ClientManager {
    id: atomic::AtomicU64,
}

impl ClientManager {
    pub fn new() -> ClientManager {
        ClientManager {
            id: atomic::AtomicU64::new(1),
        }
    }

    pub fn gen_client_id(&self) -> ClientIdType {
        let cli_id = self.id.fetch_add(1, atomic::Ordering::Acquire);

        return cli_id;
    }
}
