use crate::object;

use anyhow::{bail, Result};
use flate2::bufread::ZlibDecoder;
use std::fs::{self, File};
use std::io::{BufReader, Read};

pub fn catfile(blobid: &String) -> Result<String> {
    let obj = object::load_object(blobid)?;
    return match obj.type_ {
        object::GitObjectType::Blob => Ok(obj.data),
        _ => bail!("object not a file"),
    };
}

pub fn hashobject(path: &String, write: bool) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    let type_ = &"blob".to_string();
    if write {
        return object::store_object(type_, &content);
    }
    return Ok(object::calculate_object_hash(type_, &content));
}

fn cat_file(options: &str, blob: &str) {
    if options == "-p" {
        let blob_header = &blob[..2];
        let blob_content = &blob[2..];

        let blob_path = format!(".git/objects/{}/{}", blob_header, blob_content);

        let file = File::open(blob_path).unwrap();
        let buffer_reader = BufReader::new(file);
        let mut decoded_reader = ZlibDecoder::new(buffer_reader);
        let mut buf_vec = Vec::new();
        decoded_reader.read_to_end(buf_vec.as_mut()).unwrap();

        let space_index = buf_vec.iter().position(|&r| r == b' ').unwrap();
        let null_index = buf_vec.iter().position(|&r| r == b'\0').unwrap();
        let header = &buf_vec[..space_index];
        let _header = String::from_utf8_lossy(header).into_owned();
        let size = &buf_vec[space_index + 1..null_index];
        let size = String::from_utf8_lossy(size).into_owned();
        let _size = size.parse::<u32>().unwrap();
        let content = &buf_vec[null_index + 1..];
        let content = String::from_utf8_lossy(content).into_owned();

        print!("{content}");
    }
}
