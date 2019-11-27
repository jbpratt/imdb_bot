extern crate argparse;
extern crate env_logger;
extern crate imdb_index;
extern crate serde_json;
extern crate ws;

use argparse::{ArgumentParser, StoreTrue};
use failure;
use imdb_index::{Index, IndexBuilder, MediaEntity, Query, SearchResults, Searcher};
use serde::Deserialize;
use std::path::Path;
use std::{fs, result};
use url;
use ws::{connect, Message, Handler, Sender, Result, Request, Response};

mod download;

type ImdbResult<T> = result::Result<T, failure::Error>;

#[derive(Debug, Deserialize, PartialEq)]
struct Msg {
    nick: String,
    data: String,
}

struct Client {
    out: Sender
}

impl Handler for Client {
    fn build_request(&mut self, url: &url::Url) -> Result<Request> {
        let req = Request::from_url(url).unwrap();
        //let mut headers = req.headers_mut();
        //headers.push();
        Ok(req)
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
            handle_rec(msg);
            Ok(())
    }
}

const IMDB_URL: &str = "https://www.imdb.com/title/";

fn main() {
    env_logger::init();

    let data_dir: &Path = Path::new("./data/");
    let index_dir: &Path = Path::new("./index/");
    let mut download = false;
    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Greet somebody.");
        ap.refer(&mut download)
            .add_option(&["--download"], StoreTrue, "download imdb index files");
        ap.parse_args_or_exit();
    }

    if download {
        download::download_all(&data_dir).unwrap();
    }
    if !path_exists("./index") {
        println!("Building indices... This will take a while.");
        create_index(data_dir, index_dir).unwrap();
    }
    if let Err(error) = connect("wss://chat2.strims.gg/ws", |out| {
        Client { out }
    }) {
        println!("Failed to create WebSocket due to: {:?}", error);
    }
}

fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

// return string result if found then send from handler
fn handle_rec(msg: Message) -> () {
    match msg {
        Message::Text(text) => {
            let x = split_once(&text);
            match x[0] {
                "MSG" => {
                    let _v = match parse(x) {
                        Ok(v) => {
                            println!("{:?}", v);
                            if v.data.starts_with("!imdb") {
                                let x = v.data.trim_start_matches("!imdb");
                                let y = search_imdb(x);
                                let first_result = y.as_slice().first().unwrap().value();
                                //let first_result_rating = first_result.rating().unwrap();
                                println!(
                                    "Found: {} {}",
                                    first_result.title().title,
                                    //first_result_rating.rating,
                                    format!("{}{}", IMDB_URL, first_result.title().id)
                                );
                            }
                        }
                        Err(e) => panic!(e),
                    };
                }
                "JOIN" | "QUIT" => println!("join or quit: {}", x[1]),
                _ => println!("memes: {:?}", x),
            }
        }
        Message::Binary(_) => println!("weow binary msg received"),
    }
}

fn split_once(in_string: &str) -> Vec<&str> {
    in_string.splitn(2, ' ').collect()
}

fn parse(in_msg: Vec<&str>) -> ImdbResult<Msg> {
    let m: Msg = serde_json::from_str(in_msg[1])?;
    Ok(m)
}

fn search_imdb(query: &str) -> SearchResults<MediaEntity> {
    println!("starting search with {:}", query);
    let z: Query = Query::new().name(query);
    let data_dir: &Path = Path::new("./data/");
    let index_dir: &Path = Path::new("./index/");
    let opened_index = Index::open(data_dir, index_dir).unwrap();
    let mut searcher = Searcher::new(opened_index);
    searcher.search(&z).unwrap()
}

fn create_index(data_dir: &Path, index_dir: &Path) -> ImdbResult<Index> {
    Ok(IndexBuilder::new().create(data_dir, index_dir)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_once() {
        let in_msg: &str = "MSG {\"nick\":\"jbpratt\",\"features\":[],\"timestamp\":1568160987374,\"data\":\"test\"}";
        let out = vec![
            "MSG",
            "{\"nick\":\"jbpratt\",\"features\":[],\"timestamp\":1568160987374,\"data\":\"test\"}",
        ];
        assert_eq!(split_once(in_msg), out)
    }

    #[test]
    fn test_parse() {
        let out = Msg {
            nick: String::from("jbpratt"),
            data: String::from("test"),
        };
        let in_msg = vec![
            "MSG",
            "{\"nick\":\"jbpratt\",\"features\":[],\"timestamp\":1568160987374,\"data\":\"test\"}",
        ];
        assert_eq!(parse(in_msg).unwrap(), out)
    }
}
