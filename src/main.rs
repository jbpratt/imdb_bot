extern crate env_logger;
extern crate serde_json;
extern crate ws;

use serde::Deserialize;
use serde_json::Result;

use ws::{connect, Message};

#[derive(Debug, Deserialize, PartialEq)]
struct Msg {
    nick: String,
    data: String,
}

fn main() {
    env_logger::init();
    if let Err(error) = connect("wss://chat.strims.gg/ws", |_out| {
        move |msg| {
            handle_rec(msg);
            Ok(())
        }
    }) {
        println!("Failed to create WebSocket due to: {:?}", error);
    }
}

fn handle_rec(msg: Message) -> () {
    match msg {
        Message::Text(text) => {
            let x = split_once(&text);
            match x[0] {
                "MSG" => {
                    let _v = match parse(x) {
                        Ok(v) => println!("{:?}", v),
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

fn parse(in_msg: Vec<&str>) -> Result<Msg> {
    let m: Msg = serde_json::from_str(in_msg[1])?;
    Ok(m)
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
