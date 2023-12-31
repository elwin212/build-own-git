use crate::files::hashobject;
use crate::object::{load_object, store_object, GitObject, GitObjectType};
use anyhow::{bail, Result};
use chrono::Utc;
use std::fs::{self, DirEntry};
use std::io::{self, Write};
use std::path::Path;
use std::slice::Iter;

#[derive(Debug)]
pub struct TreeNode {
    pub permissions: String,
    pub filename: String,
    pub hash: String,
}

#[derive(Debug)]
pub struct Tree {
    pub nodes: Vec<TreeNode>,
}

impl Tree {
    fn new(tree: &GitObject) -> Tree {
        return Tree {
            nodes: parse_tree(&tree),
        };
    }

    pub fn iter(&self) -> Iter<TreeNode> {
        return self.nodes.iter();
    }

    pub fn to_buf(&self, buf: &mut Vec<u8>) {
        for node in &self.nodes {
            let mut permission = node.permissions.as_bytes().to_vec();
            let mut filename = node.filename.as_bytes().to_vec();
            let hash = node.hash.as_bytes().to_vec();

            buf.append(&mut permission);
            buf.push(0x20);
            buf.append(&mut filename);
            buf.push(0);
            buf.append(&mut hex::decode(hash).unwrap());
        }
    }
}

fn parse_tree(tree: &GitObject) -> Vec<TreeNode> {
    if let GitObjectType::Tree = tree.type_ {
        let mut vec: Vec<TreeNode> = Vec::new();
        for line in tree
            .data
            .strip_suffix('\n')
            .unwrap_or(tree.data.as_str())
            .split('\n')
        {
            let mut iter = line.split('\t');
            vec.push(TreeNode {
                permissions: iter.next().unwrap().to_string(),
                filename: iter.next().unwrap().to_string(),
                hash: iter.next().unwrap().to_string(),
            })
        }
        return vec;
    }
    panic!("object not a tree")
}

pub fn lstree(treeid: &String) -> Result<Tree> {
    let obj = load_object(treeid)?;
    let tree = Tree::new(&obj);
    return Ok(tree);
}

pub fn writetree() -> Result<String> {
    let hash = hash_dir(&"./".to_string())?;
    return Ok(format!("{}", hash));
}

fn hash_dir(path: &String) -> Result<String> {
    let mut tree = Tree { nodes: Vec::new() };
    let mut files: Vec<DirEntry> = fs::read_dir(path)?.map(|f| f.unwrap()).collect();
    files.sort_by_key(|f| f.file_name());
    for node in files {
        let path = node.path();
        let path_string = path.as_os_str().to_str().unwrap().to_string();
        if path.starts_with("./.git") {
            continue;
        }
        if path.is_dir() {
            tree.nodes.push(TreeNode {
                permissions: "40000".to_string(),
                filename: node.file_name().to_str().unwrap().to_string(),
                hash: hash_dir(&path_string.to_string())?,
            });
        } else {
            tree.nodes.push(TreeNode {
                permissions: "100644".to_string(),
                filename: node.file_name().to_str().unwrap().to_string(),
                hash: hashobject(&path_string.to_string(), true)?,
            });
        }
    }
    let mut buf = Vec::new();
    tree.to_buf(&mut buf);
    return store_object(&"tree".to_string(), &buf);
}

#[allow(dead_code)]
// TODO: Use for actual `git commit`
fn get_commit_parent() -> String {
    let path = Path::new(".git/refs/heads/master");
    if path.exists() {
        return fs::read_to_string(path).unwrap();
    }
    return "".to_string();
}

fn current_time() -> String {
    let now = Utc::now();
    return now.format("%s %z").to_string();
}

fn store_commit(content: &Vec<u8>) -> Result<String> {
    return store_object(&"commit".to_string(), content);
}

fn update_master_ref(digest: &String) -> io::Result<()> {
    let path = Path::new(".git/refs/heads/master");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut file = fs::File::create(path)?;
    file.write(digest.as_bytes())?;
    return Ok(());
}

pub fn committree(
    author: &String,
    treeid: &String,
    parent_commitid: &String,
    message: &String,
) -> Result<String> {
    let timestamp = current_time();
    let mut content = Vec::new();

    content.write(format!("tree {}\n", treeid).as_bytes())?;
    if *parent_commitid != "".to_string() {
        content.write(format!("parent {}\n", parent_commitid).as_bytes())?;
    }
    content.write(format!("author {} {}\n", author, timestamp).as_bytes())?;
    content.write(format!("commiter {} {}\n", author, timestamp).as_bytes())?;
    content.write("\n".as_bytes())?;
    content.write(message.as_bytes())?;
    content.write("\n".as_bytes())?;

    let digest = store_commit(&content)?;
    update_master_ref(&digest)?;
    return Ok(digest);
}

/// Recursively creates files and directories in `base` directory
/// to match those of the given tree.
pub fn checkout_tree(sha1: &String, base: &String) -> Result<()> {
    let tree = lstree(sha1)?;
    for node in tree.iter() {
        let new_base = format!("{}/{}", base, node.filename);
        let new_base = new_base
            .strip_prefix("/")
            .unwrap_or(new_base.as_str())
            .to_string();
        if node.permissions == "40000" {
            fs::create_dir(&new_base)?;
            if let Err(e) = checkout_tree(&node.hash, &new_base) {
                bail!(e);
            };
        } else {
            let blob = load_object(&node.hash)?;
            if let GitObjectType::Blob = blob.type_ {
                let mut f = fs::File::create(new_base)?;
                f.write(blob.data.as_bytes())?;
            } else {
                bail!("treating {} as file", node.hash)
            }
        }
    }
    return Ok(());
}
