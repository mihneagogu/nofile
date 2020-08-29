use std::io::prelude::*;
use std::{
    env,
    fs::{self, File},
};
use termion::*;

mod maker;
use maker::*;

mod utils;
use utils::errors::*;
use utils::utilities::*;

/// Runs the Makefile maker (pun intended)
/// The binary of nofile must be put in the same FOLDER!
/// as the starting point of the program
/// Usage: ./nofile <start1.c> <start2.c> ...
fn main() {
    let args: Vec<String> = env::args().collect();
    match parse_args(args) {
        Ok(entrypoints) => {
            println!("Valid files. Proceeding\n");
            let makefile = run(entrypoints);
            let formatted = makefile.format();
            println!(
                "{}Makefile construction succeeded. Outputing\n",
                color::Fg(color::Green)
            );
            println!("{}\n", formatted);
            let file = File::create("_Makefile");
            match file {
                Ok(mut file) => {
                    let written = file.write_all(formatted.as_bytes());
                    match written {
                        Ok(_) => println!("{}--- SUCCESS ---", color::Fg(color::Green)),
                        Err(e) => {
                            println!(
                                "{}File writing failed with error:\n {}",
                                color::Fg(color::Red),
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "{}File creation failed, aborting. Error: \n {}",
                        color::Fg(color::Red),
                        e
                    );
                }
            }
        }
        Err(nf_err) => {
            nf_err.diagnostic();
        }
    }
}

/// Parses the arguments and returns a vector of the filenames and contents
/// or a NFError if failed for some reason
fn parse_args(args: Vec<String>) -> Result<Vec<(String, String)>, NFError> {
    // println!("{:?}", env::current_dir());
    if args.len() < 2 {
        return Err(NFError::NotEnoughArgs);
    }
    let mut arg_iter = args.into_iter();
    let mut entrypoints = Vec::new();

    // Skip first argument, which is just the executable name
    arg_iter.next();

    for arg in arg_iter {
        let path_to_start = arg;
        if !path_to_start.ends_with(".c") {
            // not a C file
            return Err(NFError::InvalidFileExt(path_to_start));
        }
        let startpoint = fs::read_to_string(&path_to_start);
        let contents;
        match startpoint {
            Ok(content) => {
                contents = content;
                entrypoints.push((path_to_start, contents));
            }
            Err(e) => {
                return Err(NFError::IOError(e, path_to_start));
            }
        };
    }
    Ok(entrypoints)
}
