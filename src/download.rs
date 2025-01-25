use crate::result::Result;
use futures_util::TryStreamExt;
use reqwest;
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;

async fn download<T: AsyncRead + Unpin>(repo: &str, arch: &str, file_name: &str) -> Result<T> {
    let repo = reqwest::get(format!(
        "https://mirrors.tuna.tsinghua.edu.cn/archlinux/{repo}/os/{arch}/{file_name}"
    ))
    .await?;
    let mut stream = repo.bytes_stream();
    let stream = stream.map_err(|err| std::io::Error::other(err));
    let reader = StreamReader::new(stream);
    return Result::Ok(reader);
}
