use bytes::{Buf, Bytes};
use futures_core::Stream;
use pin_project_lite::pin_project;
use reqwest::{self};
use std::io;
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use tokio::io::{AsyncBufRead, AsyncRead};
use tokio::sync::mpsc;
use tokio_util::io::{ReaderStream, StreamReader};

async fn download_v2(
    download_url: String,
) -> std::io::Result<(
    u64,
    impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>>,
)> {
    let mut resp = match reqwest::Client::new()
        .get(download_url)
        .header(http::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => return Err(std::io::Error::other(err)),
    };

    let content_length = resp.content_length();
    return Ok((content_length.unwrap(), resp.bytes_stream()));
}

async fn download(download_url: String, mut sender: mpsc::Sender<Bytes>) -> std::io::Result<()> {
    let mut resp = match reqwest::Client::new()
        .get(download_url)
        .header(http::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => return Err(std::io::Error::other(err)),
    };

    loop {
        let chunk = match resp.chunk().await {
            Ok(Some(buf)) => buf,
            Ok(None) => {
                sender.closed().await;
                return Ok(());
            }
            Err(err) => return Err(std::io::Error::other(err)),
        };

        if let Err(err) = sender.send(chunk).await {
            break;
        };
    }

    Ok(())
}

struct PackageInfo {}

pub struct PackageDownloadManager {
    // package_info_map: HashMap<String, PackageInfo>
}

impl PackageDownloadManager {
    pub fn new() -> PackageDownloadManager {
        PackageDownloadManager {}
    }

    pub fn download(&self, repo: &str, arch: &str, file_name: &str) -> PackageDownloader {
        let (sender, receiver) = mpsc::channel::<Bytes>(10);
        let download_url =
            format!("https://mirrors.tuna.tsinghua.edu.cn/archlinux/{repo}/os/{arch}/{file_name}");
        tokio::spawn(async move { download(download_url, sender).await });

        return PackageDownloader::new(receiver);
    }

    pub async fn download_v2(&self, resp: &str, arch: &str, file_name: &str) -> PackageDownloader {
        let (sender, receiver) = mpsc::channel::<Bytes>(10);
        let download_url =
            format!("https://mirrors.tuna.tsinghua.edu.cn/archlinux/{repo}/os/{arch}/{file_name}");

        let (content_length, body_stream) = download_v2(download_url).await.unwrap();

        tokio::spawn(async move { download(download_url, sender).await });

        return PackageDownloader::new(receiver);
    }
}

pin_project! {
    pub struct PackageDownloader {
        receiver: mpsc::Receiver<Bytes>,
        buf: Option<bytes::Bytes>,
    }
}

impl PackageDownloader {
    fn new(receiver: mpsc::Receiver<Bytes>) -> PackageDownloader {
        PackageDownloader {
            receiver,
            buf: Option::None,
        }
    }

    fn remaining(&self) -> usize {
        match self.buf {
            Some(ref buf) => buf.remaining(),
            None => 0,
        }
    }
}

impl AsyncRead for PackageDownloader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let inner_buf = match ready!(self.as_mut().poll_fill_buf(cx)) {
            Ok(buf) => buf,
            Err(err) => return Poll::Ready(Err(err)),
        };

        let len = std::cmp::min(inner_buf.len(), buf.remaining());
        buf.put_slice(&inner_buf[..len]);
        self.consume(len);

        return Poll::Ready(Ok(()));
    }
}

impl AsyncBufRead for PackageDownloader {
    fn poll_fill_buf(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<&[u8]>> {
        loop {
            if self.remaining() > 0 {
                return Poll::Ready(Ok(self.project().buf.as_ref().unwrap().chunk()));
            } else {
                match ready!(self.as_mut().receiver.poll_recv(cx)) {
                    Some(buf) => {
                        self.buf = Some(buf);
                    }
                    None => {
                        return Poll::Ready(Ok(&[]));
                    }
                }
            }
        }
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        if amt > 0 {
            self.project().buf.as_mut().unwrap().advance(amt);
        }
    }
}
