#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};

fn main() {
    //Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("init") => init(),
        Some("cat-file") if args.len() > 3 => cat_file(&args[2], &args[3]),
        Some("hash-object") if args.len() > 3 => hash_object(&args[2], &args[3]),
        Some(command) => println!("unknown command: {}", command),
        None => println!("No command provided"),
    }
}

fn init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
    println!("Initialized git directory")
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

fn hash_object(options: &str, file_path: &str) {
    if options == "-w" {
        let mut file = fs::File::open(&file_path).unwrap();
        let file_size = file.metadata().unwrap().len();

        // Initialize a new SHA1 hasher
        let mut hasher = Sha1::new();

        // Read the content of the file into a string
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        // Prepare the content in a "blob" format for hashing
        // It consists of the type ("blob"), the file size, a null byte, and the actual content
        let blob_content = [format!("blob {file_size}\0"), content].concat();

        // Hash the blob content
        hasher.update(&blob_content);
        let hash = hasher.finalize();
        let hash_str = format!("{:x}", hash);
        let dir = hash_str[0..2].to_string();
        let file_name = hash_str[2..].to_string();
        fs::create_dir(format!(".git/objects/{dir}")).unwrap();
        let blob = fs::File::create(format!(".git/objects/{dir}/{file_name}")).unwrap();
        let mut writer = ZlibEncoder::new(blob, Default::default());
        writer.write_all(&blob_content.as_bytes()).unwrap();
        print!("{}", hash_str);
    }
}
