use std::sync::Arc;
use std::thread;
use std::fs;
use std::collections::HashSet;
use chashmap::CHashMap;

#[macro_use]
use crate::utils::utilities::*;

use crate::utils::errors::*;

/// Path of a file
/// (Dir name, file name)
/// File will always be in the dir
#[derive(Debug)]
pub enum FilePath {
    Path(String, String)
}

use FilePath::*;
impl Clone for FilePath {
    fn clone(&self) -> Self {
        match self {
            Path(dir, file) => Path(dir.clone(), file.clone())
        }
    }
}

impl FilePath {
    /// Constructs a new file path from a given string, splitting it
    /// in the path to the directory then the actual file name
    #[inline]
    pub fn new(mut path: String) -> Self {
        let mut last_slash = 0;
        path.chars().enumerate().for_each(|(i, c)| {
            if c == '/' {
                last_slash = i;
            }
        });
        // Probably better to use the split function with regex
        // but cba to do it rn
        // TODO: use split()
        
        let last_slash = if last_slash != 0 { last_slash + 1 } else { 0 };
        let file = path.split_off(last_slash);

        // If the path is "./file.c" just remove the ./ for commodity
        if path == "./" {
            path = String::new();
        }
        Path(path, file)
    }

    fn file_to_header(&mut self) {
        match self {
            Path(_, ref mut file) => replace_with_h(file)
        };
    }

    fn file_to_c(&mut self) {
        match self {
            Path(_, ref mut file) => replace_with_c(file)
        };

    }

    /// Ref to the directory string
    #[inline]
    fn dir_ref(&self) -> &String {
        match self { 
            Path(dir, _) => dir
        }
    }

    /// Ref to the file string
    #[inline]
    pub fn file_ref(&self) -> &String {
        match self { 
            Path(_, file) => file
        }
    }

    /// Clone the file string
    #[inline]
    fn file_clone(&self) -> String {
        match self { 
            Path(_, file) => file.clone()
        }
    }

    /// Clone the directory string
    #[inline]
    fn dir_clone(&self) -> String {
        match self { 
            Path(dir, _) => dir.clone()
        }
    }

    /// Combine the directory and file into a whole valid path (String)
    #[inline]
    fn combined(&self) -> String {
        match self { 
            Path(dir, file) => { 
                let mut dir_c = dir.clone();
                dir_c.push_str(file.clone().as_str());
                dir_c
            }
        }
    }

    /// Combine the directory and file into a whole valid path (String)
    /// (consumes self)
    #[inline]
    fn into_combined(self) -> String {
        match self { 
            Path(mut dir, file) => { 
                dir.push_str(file.as_str());
                dir
            }
        }
    }

    /// Consumes self and splits the path
    /// into (dir, file)
    /// for internal use
    #[inline]
    fn into_items(self) -> (String, String) {
        match self { 
            Path(dir, file) => (dir, file)
        }
    }

    /// Composes the two paths, i.e. adds the other's directory to self's directory
    /// and overwrites the file to down_path's file
    fn compose_ref(&self, down_path: FilePath) -> FilePath {
        let (dir_new, file_new) = down_path.into_items();
        let mut dir_c = self.dir_clone();
        dir_c.push_str(dir_new.as_str());
        Path(dir_c, file_new)
    }

}

/// Makes a makefile adding dependencies from all the given files 
/// (vector of pairs of (path, contents)
pub fn run(entrypoints: Vec<(String, String)>) -> Makefile {
    let mut source_files = Vec::new();
    for (k, _) in &entrypoints {
        source_files.push(StrPath::new(k.clone()));
    }

    let dependencies: CHashMap<StrPath, HashSet<StrPath>> = CHashMap::new();
    let arc_dependencies = Arc::new(dependencies);
    let mut insert_threads = Vec::new();

    // Add all the files in a multi-threaded fashion to the map
    // This might not matter if the project is small or if you have few files
    // but if you're dealing with a huge project it will boost performance
    source_files.iter().for_each(|file| { 
        let deps = Arc::clone(&arc_dependencies); 
        let file_c = StrPath::clone(&file);
        insert_threads.push(thread::spawn(move || {
            deps.insert(file_c, HashSet::new());
        }));
    });
    insert_threads.into_iter().for_each(|t| { let _ = t.join(); });

    // Since we cloned to add to the hashmap then joined on all of the threads, there are no left
    // arcs besides this one, so it's safe to unwrap
    let deps = Arc::try_unwrap(arc_dependencies).expect("I was asked to unwrap an Arc with a strong count bigger than 1. This is a bug! Contact the maintainer");
    let makefile = Makefile::new("gcc", HashSet::new(), source_files, deps);
    let arc_file = Arc::new(makefile);

    let mut run_threads = Vec::new();
    entrypoints.into_iter().for_each(|(file, contents)| { 
            let arc_file_c = Arc::clone(&arc_file);
            run_threads.push(thread::spawn(move || {
                run_one_file(&file, contents, arc_file_c);
        }));
    });
    
    run_threads.into_iter().for_each(|t| { let _ = t.join(); });

    Arc::try_unwrap(arc_file).expect("Tried to unwrap an arc with a count bigger than 1. This is a bug, please contact maintainer")
}

fn run_one_file(start: &String, contents: String, makefile: Arc<Makefile>) {
    // Get the headers from the file
    let headers: Vec<&str> = contents
        .lines()
        .filter(|line| line.starts_with("#include") && !line.contains("std") && !line.contains("<")).collect();
    // headers.iter().for_each(|header| println!("{}", header));

    let no_include_headers: Vec<String> = headers.iter().map(|header| remove_include(header.to_string())).collect();
    let start_header = FilePath::new(start.clone());

    // let mut makefile = Makefile::new("gcc", HashSet::new(), start_header.combined(), HashSet::new());


    for header in no_include_headers {
        // traverse down the tree and add the extra implications for each header
        // then finally add each header to the dependency tree
        let header = FilePath::new(header);
        let header = FilePath::compose_ref(&start_header, header);

        let is_h_file = header.file_ref().ends_with(".h");

        let mut header_c = header.clone();
        header_c.file_to_c();
        
        // Traverse tree, trying the .c version first, so we can add it first
        traverse(start.clone(), &header_c, Arc::clone(&makefile));
        if is_h_file {
            traverse(start.clone(), &header, Arc::clone(&makefile));
        }
        else {
            let mut full_path = header.combined();
            replace_with_c(&mut full_path);
            makefile.add_dependency(&start, full_path);
        }
    } 

}

/// Replaces the ".h" at the end of the file name
/// with a ".c" for the linker
/// PRE: File must end with either ".h" or ".c"
#[inline]
fn replace_with_c(c_file: &mut String) {
    // Sanity check
    if !(c_file.ends_with(".c") || c_file.ends_with(".h")) {
        panic!("You gave me a file with no .c or .h extension");
    }
    c_file.pop();
    c_file.push('c');
}


/// Replaces the ".c" at the end of the file name
/// with a ".h" for the linker
/// PRE: File must end with either ".h" or ".c"
#[inline]
fn replace_with_h(c_file: &mut String) {
    // Sanity check
    if !(c_file.ends_with(".c") || c_file.ends_with(".h")) {
        panic!("You gave me a file with no .c or .h extension");
    }
    c_file.pop();
    c_file.push('h');
}

/// Traverses the file tree from given path to header
fn traverse(start: String, header: &FilePath, makefile: Arc<Makefile>) {
    // TODO: Change the header from &String to &Path
    let header_path = header.combined();
    let header_contents = fs::read_to_string(&header_path);
    match header_contents {
        Ok(contents) => {

            // If we succeeded opening the file and it is a .c file, add it to the dependencies
            if header_path.ends_with(".c") {
                makefile.add_dependency(&start, header_path);
            }
            // Get the next headers
            let new_headers: Vec<&str> = contents
                .lines()
                .filter(|line| line.starts_with("#include") && !line.contains("std") && !line.contains("<")).collect();
            let no_include_headers: Vec<String> = new_headers
                .iter()
                .map(|head| remove_include(head.to_string())).collect();


            let mut traversing_threads = Vec::new();

            no_include_headers.into_iter().for_each(|mut head| {
                if !(head.ends_with(".c") || head.ends_with(".h")) {
                    // Not a valid header, aborting
                    let mut dir = header.dir_clone();
                    let head_c = head.clone();
                    dir.push_str(head_c.as_str());
                    let nferr = NFError::InvalidFileExt(dir);
                    nferr.diagnostic();
                }

                // Use both ".c" and ".h" extensions to traverse the tree
                replace_with_c(&mut head);

                // Must add the parent node into the path as well
                let further_path = FilePath::new(head.clone());
                let further_path = FilePath::compose_ref(header, further_path);

                let makefile_c = Arc::clone(&makefile);

                // Traverse if header not currently in
                if !makefile.has_dependency(&start, &further_path.combined()) {
                    let start_c = start.clone();
                    traversing_threads.push(thread::spawn(move || {
                        traverse(start_c.clone(), &further_path, Arc::clone(&makefile_c));

                        let mut further_header = further_path.clone();
                        further_header.file_to_header();
                        traverse(start_c.clone(), &further_header, Arc::clone(&makefile_c));
                    }));
                }

            });

            traversing_threads.into_iter().for_each(|t| { let _ = t.join(); });

        }
        Err(e) => {
            // println!("This file does not exist, either it is the .c version of a header which exists,\
            // or it is an error with your #include setup: {}", header.combined());
        }

    };
}

/// Removes the '#include' and the quotes that wrap the header from a given header string
/// PRE: str must start with #include
fn remove_include(str: String) -> String {
    // Sanity check
    if !str.starts_with("#include") {
        panic!("You asked me to remove the '#include' from a String which doesn't contain it");
    }
    let valids = str.chars().enumerate().filter(|(i,_)| 
        // Exclude '#include "' and the last "
        i > &"#include ".len() && i != &(str.len() - 1)
    );
    return valids.map(|(_, c)| c).collect();
}
