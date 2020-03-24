extern crate env_logger;
extern crate imdb_index;
extern crate serde_json;
extern crate ws;

use std::env;
use std::path::Path;
use std::{fs, result};

use failure;
use imdb_index::{Index, IndexBuilder, MediaEntity, Query, Rating, SearchResults, Searcher};
use serde::Deserialize;
use url;
use ws::{connect, Handler, Handshake, Message, Request, Result, Sender};

mod download;

type ImdbResult<T> = result::Result<T, failure::Error>;

#[derive(Debug, Deserialize, PartialEq)]
struct Msg {
    nick: String,
    data: String,
}

struct Client {
    ws: Sender,
}

impl Handler for Client {
    fn build_request(&mut self, url: &url::Url) -> Result<Request> {
        let mut req = Request::from_url(url).unwrap();
        let key = "STRIMS_TOKEN";
        let val = env::var(key).unwrap();
        let cookie = format!("jwt={}", val);
        req.headers_mut().push(("Cookie".into(), cookie.into()));
        Ok(req)
    }

    fn on_open(&mut self, _: Handshake) -> Result<()> {
        // Now we don't need to call unwrap since `on_open` returns a `Result<()>`.
        // If this call fails, it will only result in this connection disconnecting.
        self.ws.send("Hello WebSocket")
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Text(text) => {
                let x = split_once(&text);
                match x[0] {
                    "MSG" => {
                        let _ = match parse(x) {
                            Ok(v) => {
                                if v.data.starts_with("!imdb") {
                                    let query = v.data.trim_start_matches("!imdb");
                                    // Search imdb index
                                    let mut results = search_imdb(&query).into_vec();
                                    let temp = Rating {
                                        id: String::from("X"),
                                        rating: 0.0,
                                        votes: 0,
                                    };
                                    // Sort by rating votes
                                    results.sort_by(|a, b| {
                                        a.value()
                                            .rating()
                                            .unwrap_or(&temp)
                                            .votes
                                            .cmp(&b.value().rating().unwrap_or(&temp).votes)
                                    });
                                    let last = results.last().unwrap().clone();
                                    let (rating, result) = last.into_pair();
                                    let title = result.title();
                                    let imdb_rating: f32 = match result.rating() {
                                        Some(v) => v.rating,
                                        None => 0.,
                                    };
                                    let start_year = title.start_year.unwrap_or(0);
                                    // attempt to send msg
                                    match self.ws.send(format!(
                                        "MSG {{\"data\": \"{} ({} - {}) https://www.imdb.com/title/{}/\"}}",
                                        title.title, start_year, imdb_rating, title.id
                                    )) {
                                        Ok(_) => println!("Sent"),
                                        Err(error) => panic!("Failed to send msg: {}", error),
                                    }
                                    println!(
                                        "{} {} {} https://www.imdb.com/title/{}/\n",
                                        rating, title.title, title.genres, title.id
                                    );
                                }
                            }
                            Err(e) => panic!(e),
                        };
                    }
                    _ => (),
                }
            }
            Message::Binary(_) => (),
        };
        Ok(())
    }
}

fn main() {
    let _ = env_logger::try_init();

    let data_dir: &Path = Path::new("./data/");
    let index_dir: &Path = Path::new("./index/");
    let mut download = false;

    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => (),
        2 => download = true,
        _ => (),
    }

    if download {
        download::download_all(&data_dir).unwrap();
    }

    if !path_exists("./index") {
        println!("Building indices... This will take a while.");
        create_index(data_dir, index_dir).unwrap();
    }

    println!("Connecting to chat...");

    if let Err(error) = connect("wss://chat.strims.gg/ws", |ws| Client { ws }) {
        println!("Failed to create WebSocket due to: {:?}", error);
    }
}

fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
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
