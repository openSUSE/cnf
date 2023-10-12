extern crate glob;
#[macro_use]
extern crate tr;

use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::result::Result;
use std::vec::Vec;

mod ini;
mod pool;

const REPO_GLOB: &str = "/etc/zypp/repos.d/*.repo";

pub struct SolvInput {
    name: String,
    path: PathBuf,
}

fn main() {
    // use the tr_init macro to tell gettext where to look for translations
    tr_init!("/usr/share/locale");
    let args: Vec<String> = std::env::args().collect();

    let term = match args.get(1) {
        None => exit(127),
        Some(term) => term,
    };

    let bin_path = &Path::new("/usr/bin").join(Path::new(term));
    if Path::exists(bin_path) {
        println!("{}", tr!("Absolute path to '{}' is '{}'. Please check your $PATH variable to see whether it contains the mentioned path.", term, bin_path.display()));
        exit(0);
    }

    let sbin_path = &Path::new("/usr/sbin").join(Path::new(term));
    if Path::exists(sbin_path) {
        println!("{}", tr!("Absolute path to '{}' is '{}', so running it may require superuser privileges (eg. root).", term, sbin_path.display()));
        exit(0);
    }

    match search_solv(&term) {
        Err(err) => {
            print_error(&err);
            exit(127);
        }
        _ => {}
    }
}

fn search_solv(term: &str) -> Result<(), ErrorKind> {
    let repos = load_repos()?;

    let pool = pool::SPool::new(&repos)?;
    let results = pool.search(&term);

    if results.len() == 0 {
        return Err(ErrorKind::CommandNotFound(term));
    }

    let suggested_package = if results.len() == 1 {
        results[0].Package.clone()
    } else {
        String::from(tr!("<selected_package>"))
    };

    println!("");
    println!(
        "{}",
        tr!(
            "The program '{}' can be found in the following package:"
                | "The program '{}' can be found in following packages:" % results.len(),
            &term
        )
    );

    for r in results {
        println!(
            "{}",
            tr!(
                "  * {} [ path: {}/{}, repository: {} ]",
                r.Package,
                r.Path,
                &term,
                r.Repo
            )
        );
    }

    println!("");
    print!(
        "{}",
        tr!("Try installing with:
   ")
    );
    println!(" sudo zypper install {}\n", suggested_package);
    Ok(())
}

fn load_repos() -> Result<Vec<SolvInput>, ErrorKind<'static>> {
    let mut repos: Vec<SolvInput> = Vec::new();
    for repo in glob::glob(REPO_GLOB)? {
        let repo = repo?;

        let info = ini::repo_enabled(&repo)?;
        if info.enabled {
            let solv_glob = format!("/var/cache/zypp/solv/{}/solv", info.name.replace("/", "_"));
            for path in glob::glob(&solv_glob)? {
                let i = SolvInput {
                    name: info.name.clone(),
                    path: path?,
                };
                repos.push(i);
            }
        }
    }
    Ok(repos)
}

// ErrorKind encodes all errors which can happen in command not found handler
enum ErrorKind<'a> {
    CommandNotFound(&'a str),
    PatternError(glob::PatternError),
    GlobError(glob::GlobError),
    IOError(std::io::Error),
    String(String),
}

fn print_error<'a>(err: &'a ErrorKind) {
    match err {
        ErrorKind::CommandNotFound(term) => {
            println!(" {}: {}", term, tr!("command not found"));
        }
        ErrorKind::PatternError(err) => {
            println!("{}", err)
        }
        ErrorKind::GlobError(err) => {
            println!("{}", err)
        }
        ErrorKind::IOError(err) => {
            println!("{}", err)
        }
        ErrorKind::String(msg) => {
            println!("{}", msg);
        }
    }
}

impl From<glob::PatternError> for ErrorKind<'_> {
    fn from(value: glob::PatternError) -> Self {
        return ErrorKind::PatternError(value);
    }
}

impl From<glob::GlobError> for ErrorKind<'_> {
    fn from(value: glob::GlobError) -> Self {
        return ErrorKind::GlobError(value);
    }
}

impl From<std::io::Error> for ErrorKind<'_> {
    fn from(value: std::io::Error) -> Self {
        return ErrorKind::IOError(value);
    }
}

// TODO: drop this From implementation, the proper errors from src/pool.rs may be used instead
impl From<String> for ErrorKind<'_> {
    fn from(value: String) -> Self {
        return ErrorKind::String(value);
    }
}
