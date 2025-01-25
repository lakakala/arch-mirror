use std::task::ready;

use axum::BoxError;
use axum::{
    body::Body,
    debug_handler,
    extract::{Request, Json, Path, Extension, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use bytes::Bytes;
use futures_core::{Stream, TryStream};
use pin_project_lite::pin_project;
use tokio::fs::{self, File};
use tokio::io;
use tokio::io::AsyncRead;
use tokio::io::ReadBuf;
pub struct Server {}

impl Server {
    pub fn new() -> Server {
        return Server {};
    }

    pub async fn start(&self) {
        // build our application with a route
        let app = axum::Router::new()
            // `GET /` goes to `root`
            .route("/mirror/{repo}/os/{arch}/{file_name}", get(mirror));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }


}

#[debug_handler]
async fn mirror(
    Path((repo, arch, file_name)): Path<(String, String,  String)>,
    // Path((repo, arch, file_name)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let file = fs::File::open("/home/dev/core.db").await.unwrap();

    // let buf_stream = BufStream::new(file);
    return FileResponse::new(StreamBufReader::new(file), "test");
}

struct FileResponse<S: TryStream> {
    stream: S,
    file_name: String,
}

impl<S: TryStream> FileResponse<S> {
    fn new(stream: S, file_name: &str) -> FileResponse<S> {
        FileResponse {
            stream,
            file_name: String::from(file_name),
        }
    }
}

impl<S> IntoResponse for FileResponse<S>
where
    S: TryStream + Send + 'static,
    S::Ok: Into<Bytes>,
    S::Error: Into<BoxError>,
{
    fn into_response(self) -> Response<Body> {
        let mut resp = Response::new(Body::from_stream(self.stream));
        resp.headers_mut().append(
            header::CONTENT_DISPOSITION,
            header::HeaderValue::from_str(&format!("attachment; filename={}", self.file_name))
                .unwrap(),
        );

        return resp;
    }
}

pin_project! {
    struct StreamBufReader< T> {
        #[pin]
        reader: T,
    }
}

impl<T> StreamBufReader<T> {
    fn new(reader: T) -> StreamBufReader<T> {
        return StreamBufReader { reader };
    }
}

impl<T> Stream for StreamBufReader<T>
where
    T: AsyncRead,
{
    type Item = std::result::Result<Bytes, io::Error>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let me = self.project();

        let mut buf: [u8; 1024] = [0; 1024];

        let mut read_buf = ReadBuf::new(&mut buf);

        let result = ready!(AsyncRead::poll_read(me.reader, cx, &mut read_buf));

        if let Err(err) = result {
            return std::task::Poll::Ready(Option::Some(std::result::Result::Err(err)));
        }

        let filled = read_buf.filled();
        if filled.len() == 0 {
            return std::task::Poll::Ready(Option::None);
        }

        return std::task::Poll::Ready(Option::Some(std::result::Result::Ok(
            Bytes::copy_from_slice(filled),
        )));
    }
}
