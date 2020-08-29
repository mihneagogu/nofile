use chashmap::{CHashMap, ReadGuard};
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use crate::maker::FilePath;
use std::hash::{Hash, Hasher};

#[doc = "makes a color format: usage:
``` color![(color::Red)] ```"]
macro_rules! color {
    ($col:tt) => {
        color::Fg($col)
    };
}

#[doc = "Prints to terminal with given color"]
macro_rules! color_print {
    ($color:expr, $($args:tt)*) => {
        let printed = format!($($args)*);
        println!("{}{}", $color, printed);
    }
}

macro_rules! print_yellow {
    ($($args:tt)*) => {
        color_print![color![(color::Yellow)], $($args)*];
    }
}

#[doc = "Prints to terminal with red"]
macro_rules! print_red {
    ($($args:tt)*) => {
        color_print![color![(color::Red)], $($args)*];
    }
}

#[doc = "Prints to terminal with white"]
macro_rules! print_white {
    ($($args:tt)*) => {
        color_print![color![(color::White)], $($args)*];
    }

}

static CC_IDENTIFIER: &str = "CC";
static CFLAGS_IDENTIFIER: &str = "CFLAGS";
// static SUFFIXES_IDENTIFIER: &str = ".SUFFIXES";
static CLEAN_PHONY: &str = ".PHONY: all clean";

// static SUFFIXES: &str = ".c .o";
// static CLEAN_RM: &str = "rm -f ";
// static CLEAN_COMM: &str = "clean: ";
static CLEAN_O: &str = "\trm -f $(wildcard *.o)";

/// The fields of a Makefile,
/// should probably change the Strings to &'a str to save allocating a lot,
/// but that's for the future
#[derive(Debug)]
pub struct Makefile {
    c_compiler: &'static str,
    //GCC by default
    c_flags: HashSet<String>,
    source_files: Vec<StrPath>,
    dependencies: CHashMap<StrPath, HashSet<StrPath>>,
}

/// Touple struct which contains the path which will be entered in the 
/// dependencies map. This type is necessary to compare identical files which
/// are accessed using different paths
#[derive(Debug)]
pub struct StrPath(String);

impl StrPath {
    pub fn new(path: String) -> Self {
        Self(path)
    }

    fn into(self) -> String {
        self.0
    }

    pub fn clone(&self) -> StrPath {
        StrPath::new(self.0.clone())
    }
}

impl Hash for StrPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Construct a hash for the StrPath ONLY according to the actual file name
        // not the whole string
        let path = FilePath::new(self.0.clone());
        path.file_ref().clone().hash(state);
    }
}

impl Eq for StrPath {}

impl PartialEq for StrPath {
    fn eq(&self, other: &Self) -> bool {
        let this_filepath = FilePath::new(self.0.clone());
        let other_filepath = FilePath::new(other.0.clone());
        let res = this_filepath.file_ref() == other_filepath.file_ref();
        res
    }
}


impl Deref for StrPath {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StrPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A struct which holds the Makefile data of a certain executable file Example: source: "emulate.c" exe_label: "emulate"
/// recipe: "emulate: emulate.o\n\t$(CC) $(CFLAGS) emulate.c $(EMULATE_SOURCE) -o $@"
/// clean_statement: "rm -f emulate"
struct ExecutableData {
    source_file: String,
    source: String,
    exe_label: String,
    recipe: String,
    clean_statement: String,
    clean_target: String,
}

impl ExecutableData {
    /// Strips the ".c" or ".h" from a file name
    /// PRE: File must have a ".c" or ".h" ending
    fn strip_ending(file: &mut String) {
        file.truncate(file.len() - 2);
    }

    /// Constructs the Makefile data for the given executable file
    /// from the string and the entries in the makefile
    fn from(source_file: &String, sources: HashSet<String>) -> Self {
        let source_file = source_file.clone();
        let mut exe_label = source_file.clone();
        ExecutableData::strip_ending(&mut exe_label);
        let dependencies_descriptor = format!("{}_SOURCE", exe_label).to_uppercase();

        let recipe = format!(
            "{label}: {label}.o\n\t$({compiler}) $({cflags}) {start} $({dependencies}) -o $@",
            label = exe_label,
            compiler = CC_IDENTIFIER,
            cflags = CFLAGS_IDENTIFIER,
            start = source_file,
            dependencies = dependencies_descriptor,
        );
        let clean_statement = format!("\trm -f {}", exe_label);
        let clean_target = format!("\trm -f {}.o", exe_label);

        let mut source = format!("{} = ", dependencies_descriptor);
        let dependencies = sources;
        dependencies
            .iter()
            .for_each(|dep| source.push_str(format!("{} ", dep).as_str()));

        Self {
            source_file,
            source,
            exe_label,
            recipe,
            clean_statement,
            clean_target,
        }
    }
}

impl Makefile {
    pub fn new(
        c_compiler: &'static str,
        c_flags: HashSet<String>,
        source_files: Vec<StrPath>,
        dependencies: CHashMap<StrPath, HashSet<StrPath>>,
    ) -> Self {
        Makefile {
            c_compiler,
            c_flags,
            source_files,
            dependencies,
        }
    }

    fn ref_dependencies_for(&self, source: &String) -> ReadGuard<StrPath, HashSet<StrPath>> {
        let source = StrPath::new(source.clone());
        let dependencies = self.dependencies.get(&source);
        match dependencies {
            Some(deps) => deps,
            None => {
                panic!("Something went wrong, asked for the dependencies for {source}\
                    , but {source}, wasn't inside the Map. This is an internal error. If for some reason this statement is printed, please\
                    contact the mantainer. This IS a BUG!", source = source.0);
            }
        }
    }

    /// Adds the given header dependency to the dependencies of "source", if it's not inside
    pub fn add_dependency(&self, source: &String, dependency: String) {
        let source = StrPath::new(source.clone());
        let deps = self.dependencies.get_mut(&source);
        match deps {
            Some(mut deps) => {
                deps.insert(StrPath::new(dependency));
            }
            None => {}
        };
    }

    /// Returns whether the dependencies of "source" has the given "dependency"
    pub fn has_dependency(&self, source: &String, dependency: &String) -> bool {
        let source = StrPath::new(source.clone());
        let ref deps = self.dependencies.get(&source);
        match deps {
            Some(deps) => {
                let res = deps.contains(&StrPath::new(dependency.clone()));
                res
            },
            None => false
        }
    }

    /// Formats the items of the Makefile struct into
    /// the actual Makefile
    /// PRE: self.source_files are guaranteed to have a .c at the end
    pub fn format(self) -> String {
        let c_compiler = format!("CC = {}", self.c_compiler);
        let c_flags = "CFLAGS = -Wall -g -pedantic -std=c99".to_string();
        let suffixes = ".SUFFIXES: .c .o";
        let phony_clean = format!("{}", CLEAN_PHONY);

        let map = self.dependencies.clear();
        let files_data: Vec<ExecutableData> = map
            .into_iter()
            .map(|(source_file, deps)| {
                let deps = deps.into_iter().map(|strpath| strpath.into()).collect();
                ExecutableData::from(&source_file, deps)
            })
            .collect();

        let mut sources = String::new();
        files_data
            .iter()
            .for_each(|data| sources.push_str(format!("{}\n", data.source).as_str()));
        sources.push('\n');

        // Collect all the tags
        let mut all = String::from("all: ");

        let mut recipes = String::new();

        let mut clean = String::from("clean:\n");
        clean.push_str(CLEAN_O);
        clean.push('\n');

        files_data.iter().for_each(|data| {
            all.push_str(format!("{} ", data.exe_label).as_str());
            recipes.push_str(format!("{}\n\n", data.recipe).as_str());
            clean.push_str(format!("{}\n{}\n", data.clean_statement, data.clean_target).as_str());
        });

        let makefile = format!("{cc}\n{cflags}\n\n{sources}{suffixes}\n\n{phony_clean}\n\n{all_exes}\n\n{recipes}{clean}",
                               cc = c_compiler,
                               cflags = c_flags,
                               sources = sources,
                               suffixes = suffixes,
                               phony_clean = phony_clean,
                               all_exes = all,
                               recipes = recipes,
                               clean = clean
        );
        makefile
    }
}
