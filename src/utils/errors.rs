use std::process;

#[macro_use]
use crate::utils::utilities::*;

use termion::*;
pub enum NFError {
    NotEnoughArgs,
    IOError(std::io::Error, String),
    InvalidFileExt(String),
}

// macro_rules! color_print {
//     (
// }

macro_rules! red {
    () => {
        color::Fg(color::Red)
    };
}

use NFError::*;
impl NFError {
    pub fn diagnostic(&self) {
        match self {
            NotEnoughArgs => {
                color_print![
                    color![(color::Red)],
                    "You have not given me enough argmuents, please check the spec"
                ];
                print_red!["------------ EXITING -----------"];
                process::exit(1);
            }
            IOError(e, path) => {
                print_red!["{} <- for file {}", e, path];
                print_red!["Aborting, please rerun"];
                print_red!["------------ EXITING -----------"];
                process::exit(1);
            }
            InvalidFileExt(file) => {
                color_print![
                    color![(color::Red)],
                    "You have given me a path to a file that does not contain a .c or .h extension: which is {}",
                    file
                ];
                println!("{}------------ EXITING -----------", red![]);
                process::exit(1);
            }
        }
    }
}
