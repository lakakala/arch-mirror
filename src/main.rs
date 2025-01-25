mod result;
mod server;
mod server_v2;
mod download;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    // download_db(String::from("/home/dev/arch-mirror"))
    //     .await
    //     .unwrap();

    // let mut server = server::Server::new().await.unwrap();
    // server.start().await.unwrap();


    server_v2::Server::new().start().await;
}

use bytes::Buf;
use std::collections::HashMap;
use std::fs::{self, File};
use std::hash::Hash;
use std::io::{BufRead, BufReader, Read};
use tar::Archive;

async fn download_db(data_dir: String) -> result::Result<()> {
    let resp = reqwest::Client::new()
        .get("https://mirrors.tuna.tsinghua.edu.cn/archlinux/core/os/x86_64/core.db")
        .send()
        .await?;
    let data = resp.bytes().await?;

    let temp_dir = format!("{data_dir}/temp");

    if fs::exists(&temp_dir)? {
        fs::remove_dir_all(&temp_dir)?;
    }

    fs::create_dir(&temp_dir)?;
    Archive::new(flate2::read::GzDecoder::new(data.reader())).unpack(&temp_dir)?;

    parse_db(&temp_dir).await?;
    Ok(())
}

async fn parse_db(data_dir: &str) -> result::Result<()> {
    let root_dir = fs::read_dir(&data_dir)?;

    for sub_dir in root_dir {
        let sub_dir = sub_dir?;
        let mut desc_path = sub_dir.path();
        desc_path.push("desc");

        let mut buf = BufReader::new(fs::File::open(desc_path)?);

        let mut line = String::new();
        println!("{}/{:?}/desc", data_dir, sub_dir.file_name());
        loop {
            buf.read_line(&mut line)?;

            println!("{line}");
        }
    }
    Ok(())
}

struct Desc {
    file_name: String,
    name: String,
    base: String,
    version: String,
    desc: String,
    csize: i64,
    isize: i64,
    sha256_sum: String,
    pgp_sig: String,
    url: String,
    licenses: Vec<String>,
    arch: String,
    build_date: std::time::Instant,
    packagers: Vec<String>,
    conflicts: Vec<String>,
    depends: Vec<String>,
    make_depends: Vec<String>,
}

enum ParseState {
    Key,
    Value,
}

const DESC_KEY_FILE_NAME: &'static str = "FILENAME";
const DESC_KEY_NAME: &'static str = "NAME";
const DESC_KEY_BASE: &'static str = "BASE";
const DESC_KEY_VERSION: &'static str = "VERSION";
const DESC_KEY_DESC: &'static str = "DESC";
const DESC_KEY_CSIZE: &'static str = "CSIZE";
const DESC_KEY_ISIZE: &'static str = "ISIZE";
const DESC_KEY_SHA256SUM: &'static str = "SHA256SUM";
const DESC_KEY_PGPSIG: &'static str = "PGPSIG";
const DESC_KEY_URL: &'static str = "URL";
const DESC_KEY_LICENSE: &'static str = "LICENSE";
const DESC_KEY_ARCH: &'static str = "ARCH";
const DESC_KEY_BUILDDATE: &'static str = "BUILDDATE";
const DESC_KEY_PACKAGER: &'static str = "PACKAGER";
const DESC_KEY_CONFLICTS: &'static str = "CONFLICTS";
const DESC_KEY_DEPENDS: &'static str = "DEPENDS";
const DESC_KEY_MAKEDEPENDS: &'static str = "MAKEDEPENDS";

async fn parse_desc<T>(desc_reader: T) -> result::Result<Desc>
where
    T: std::io::Read,
{
    let mut buf = BufReader::new(desc_reader);

    let mut key_values: HashMap<String, Vec<String>> = HashMap::new();

    let mut key = String::new();
    let mut state = ParseState::Key;

    let mut line = String::new();
    loop {
        line.clear();
        let num = buf.read_line(&mut line)?;
        if num == 0 {
            break;
        }

        match state {
            ParseState::Key => {
                key = line.clone();
                key_values.insert(key.clone(), Vec::new());
                state = ParseState::Value;
            }
            ParseState::Value => {
                if !line.is_empty() {
                    let values = key_values.get_mut(&key).unwrap();

                    values.push(line.clone());
                } else {
                    state = ParseState::Key;
                }
            }
        }
    }

    todo!();
}
