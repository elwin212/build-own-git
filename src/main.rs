use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

use std::fs;
use std::str;

mod clone;
mod files;
mod object;
mod pack;
mod tree;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// initialize git repository
    Init,

    /// print contents of blob objects
    CatFile {
        object: String,

        #[clap(short = 'p', help = "pretty print object")]
        pretty: bool,
    },

    /// calculate sha1 for file and optionally store the object
    HashObject {
        path: String,

        #[clap(short = 'w', help = "write object to object store")]
        write: bool,
    },

    /// print contents of tree objects
    LsTree {
        treeid: String,

        #[clap(long, help = "print only object names")]
        name_only: bool,
    },

    /// recursively store current working directory as repository objects
    WriteTree,

    /// create commit from a written tree
    CommitTree {
        treeid: String,

        #[clap(short = 'p', help = "parent commit")]
        parent: String,

        #[clap(short = 'm', help = "commit message")]
        message: String,
    },

    /// Clone remote repository
    Clone { url: String, path: String },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            init().unwrap();
        }
        Commands::CatFile { object, pretty: _ } => {
            print!("{}", files::catfile(object).unwrap());
        }
        Commands::HashObject { write, path } => {
            println!("{}", files::hashobject(path, *write).unwrap())
        }
        Commands::LsTree { treeid, name_only } => {
            let tree = tree::lstree(&treeid).unwrap();
            for node in tree.iter() {
                if *name_only {
                    println!("{}", node.filename);
                } else {
                    println!("{}\t{}\t{}", node.permissions, node.filename, node.hash);
                }
            }
        }
        Commands::WriteTree => {
            let digest = tree::writetree().unwrap();
            println!("{}", digest);
        }
        Commands::CommitTree {
            treeid,
            parent,
            message,
        } => {
            let newcommitid =
                tree::committree(&"manuel@manuel.com".to_string(), treeid, parent, message)
                    .unwrap();
            println!("{}", newcommitid);
        }
        Commands::Clone { url, path } => {
            clone::clone(url, path).unwrap();
        }
    }
}

fn init() -> Result<()> {
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/master\n")?;
    println!("Initialized git directory");
    Ok(())
}
