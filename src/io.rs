use std::collections::HashMap;

use bytes::Buf;
use serde_json::Value;
use tokio_util::codec::{Decoder, Encoder};
use tracing::{trace, warn};

use crate::{event::Event, EslError};

#[derive(Debug, Clone)]
pub(crate) struct EslCodec {}

impl Encoder<&[u8]> for EslCodec {
    type Error = EslError;
    fn encode(&mut self, item: &[u8], dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(item);
        dst.extend_from_slice(b"\n\n");
        Ok(())
    }
}

fn get_header_end(src: &bytes::BytesMut) -> Option<usize> {
    trace!("get_header_end:=>{:?}", src);
    // get first new line character
    for (index, chat) in src[..].iter().enumerate() {
        if chat == &b'\n' && src.get(index + 1) == Some(&b'\n') {
            return Some(index + 1);
        }
    }
    None
}
fn parse_body(src: &[u8], length: usize) -> String {
    trace!("parse body src : {}", String::from_utf8_lossy(src));
    trace!("length src : {}", length);
    String::from_utf8_lossy(&src[..length]).to_string()
}
fn parse_header(src: &[u8]) -> Result<HashMap<String, Value>, std::io::Error> {
    trace!("parsing this header {:#?}", String::from_utf8_lossy(src));
    let data = String::from_utf8_lossy(src).to_string();
    let a = data.split('\n');
    let mut hash = HashMap::new();
    for line in a {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() == 2 {
            // SAFETY: Index access is safe beacue we have checked the length
            let key = parts[0].trim();
            let val = parts[1].trim();
            hash.insert(key.to_string(), serde_json::json!(val.to_string()));
        } else {
            warn!("Invalid formatting while parsing header");
        }
    }
    trace!("returning hashmap : {:?}", hash);
    Ok(hash)
}

impl Decoder for EslCodec {
    type Item = Event;
    type Error = EslError;
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        trace!("decode");
        let header_end = get_header_end(src);
        let header_end = match header_end {
            Some(he) => he,
            None => return Ok(None),
        };
        let headers = parse_header(&src[..(header_end - 1)])?;
        trace!("parsed headers are : {:?}", headers);
        let body_start = header_end + 1;
        let Some(length) = headers.get("Content-Length") else {
            src.advance(body_start);
            return Ok(Some(Event {
                headers,
                body: None,
            }));
        };

        let length = length.as_str().unwrap();
        let body_length = length.parse()?;
        if src.len() < (header_end + body_length + 1) {
            trace!("returned because size was not enough");
            return Ok(None);
        }
        let body = parse_body(&src[body_start..], body_length);
        src.advance(body_start + body_length);
        Ok(Some(Event {
            headers,
            body: Some(body),
        }))
    }
}
