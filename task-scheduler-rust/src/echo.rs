use std::{
    io,
    io::{Error, ErrorKind},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Request {
    pub kind: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub message: String,
}

pub fn parse_request(b: &[u8]) -> io::Result<Request> {
    serde_json::from_slice(b).map_err(|e| {
        return Error::new(ErrorKind::InvalidInput, format!("invalid JSON: {}", e));
    })
}

#[test]
fn test_parse_request() {
    let ret = parse_request("{\"kind\":\"create\",\"message\":\"hello\"}".as_bytes());
    assert!(ret.is_ok());
    let t = ret.unwrap();
    assert_eq!(t.kind, "create");
}

#[derive(Debug)]
pub struct Manager {}

impl Manager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn apply(&mut self, req: &Request) -> io::Result<String> {
        println!("applying echo request");

        // check every 5-second
        // use std::{thread, time::Duration};
        // thread::sleep(Duration::from_secs(5));

        match req.kind.as_str() {
            "create" => Ok(format!("SUCCESS create {}", req.message)),
            "delete" => Ok(format!("SUCCESS delete {}", req.message)),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("unexpected none field value for kind {}", req.kind),
            )),
        }
    }
}
