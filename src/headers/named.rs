use nom::character::*;
use nom::error::ErrorKind;

use crate::Uri;
use crate::parse::*;
use crate::uri::parse_uri;

use std::fmt;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct NamedHeader {
    pub display_name: Option<String>,
    pub uri: Uri,
    pub params: HashMap<String, String>
}

impl fmt::Display for NamedHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = &self.display_name {
            if name.contains(' ') {
                write!(f, "\"{}\" <{}>", name, self.uri)?;
            } else {
                write!(f, "{} <{}>", name, self.uri)?;
            }
        } else {
            write!(f, "<{}>", self.uri)?;
        }
        for (key, value) in (&self.params).iter() {
            write!(f, ";{}={}", key, value)?;
        }
        Ok(())
    }
}

named!(pub parse_named_field_param<(String, String)>, do_parse!(
    char!(';') >>
    key: map_res!(take_while!(is_alphabetic), slice_to_string) >>
    char!('=') >>
    value: map_res!(take_while!(is_alphanumeric), slice_to_string) >>
    (key, value)
));

named!(pub parse_name<String>, alt!(
        parse_quoted_string |
        map_res!(take_while!(is_alphabetic), slice_to_string)
));

named!(pub parse_named_field_value<(Option<String>, Uri)>, do_parse!(
    name: opt!(parse_name) >>
    opt!(take_while!(is_space)) >>
    char!('<') >>
    value: parse_uri>>
    char!('>') >>
    ((name, value))
));

pub fn parse_named_field_params(input: &[u8]) -> Result<(&[u8], HashMap<String, String>), nom::Err<(&[u8], ErrorKind)>> {
    let mut map = HashMap::new();
    let mut input = input;
    loop {
        match parse_named_field_param(input) {
            Ok((data, (key, value))) => {
                map.insert(key, value);
                input = data;
            },
            _ => break
        }
    }
    Ok((input, map))
}