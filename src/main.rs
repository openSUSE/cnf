extern crate glob;
#[macro_use]
extern crate tr;

use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::result::Result;
use std::vec::Vec;

mod ini;
mod pool;

const ZYPPER_REPO_GLOB: &str = "/etc/zypp/repos.d/*.repo";
// Default value of the reposdir configuration directory
const DNF_REPOS_GLOBS: [&str; 3] = [
    "/etc/dnf/repos.d/*.repo",
    "/etc/yum.repos.d/*.repo",
    "/etc/distro.repos.d/*.repo"
];
// Default value of the reposdir configuration directory
// (Note that /etc/dnf/repos.d is patched in by the openSUSE dnf5 package)
const DNF5_REPOS_GLOBS: [&str; 4] = [
    "/etc/dnf/repos.d/*.repo",
    "/etc/yum.repos.d/*.repo",
    "/etc/distro.repos.d/*.repo",
    "/usr/share/dnf5/repos.d/*.repo",
];

#[derive(Clone, Copy)]
pub enum PackageManager {
    Zypper,
    Dnf,
    Dnf5,
}
pub struct SolvInput {
    name: String,
    path: PathBuf,
}

// ErrorKind encodes all errors whose can happen in command not found handler
#[derive(Debug)]
pub enum ErrorKind<'a> {
    CommandNotFound(&'a str),
    PatternError(glob::PatternError),
    GlobError(glob::GlobError),
    IOError(std::io::Error),
    IsNULL(&'static str),
    IsNULLNamed(&'static str, &'a str),
    RepoAddSolv(&'a PathBuf),
    String(String),
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

    let pm = if Path::exists(Path::new("/usr/bin/dnf5")) {
        // Use dnf5 if it's installed from the openSUSE repo (since zypper is the default, someone is only likely to have dnf5 installed if they want to use it)
        PackageManager::Dnf5
    } else if Path::exists(Path::new("/usr/bin/dnf")) {
        // Use an old dnf if it's installed, but dnf5 isn't
        // (in case a dnf5 user symlinks /usr/bin/dnf to /usr/bin/dnf5, this goes after the dnf5 check)
        PackageManager::Dnf
    } else if Path::exists(Path::new("/usr/bin/zypper")) {
        PackageManager::Zypper
    } else {
        println!("Neither /usr/bin/dnf5, /usr/bin/dnf, nor /usr/bin/zypper could be found.");
        exit(127);
    };

    let repos = match load_repos(pm) {
        Err(err) => {
            println!("{}", err);
            exit(127);
        }
        Ok(repos) => repos,
    };

    if let Err(err) = search_in_repos(pm, term, &repos) {
        println!("{}", err);
        exit(127);
    }
}

fn search_in_repos<'a>(
    pm: PackageManager,
    term: &'a str,
    repos: &'a [SolvInput],
) -> Result<(), ErrorKind<'a>> {
    let pool = pool::SPool::new(repos)?;
    let results = pool.search(term)?;

    if results.is_empty() {
        return Err(ErrorKind::CommandNotFound(term));
    }

    let suggested_package = if results.len() == 1 {
        results[0].package.clone()
    } else {
        tr!("<selected_package>")
    };

    println!();
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
                r.package,
                r.path,
                &term,
                r.repo
            )
        );
    }

    println!();
    print!(
        "{}",
        tr!("Try installing with:
   ")
    );
    match pm {
        PackageManager::Zypper => println!(" sudo zypper install {}\n", suggested_package),
        PackageManager::Dnf => println!(" sudo dnf install {}\n", suggested_package),
        PackageManager::Dnf5 => println!(" sudo dnf5 install {}\n", suggested_package),
    }
    Ok(())
}

fn load_repos<'a>(pm: PackageManager) -> Result<Vec<SolvInput>, ErrorKind<'a>> {
    let mut repos: Vec<SolvInput> = Vec::new();
    let globs = match pm {
        PackageManager::Zypper => &[ZYPPER_REPO_GLOB] as &[&str],
        PackageManager::Dnf => &DNF_REPOS_GLOBS as &[&str],
        PackageManager::Dnf5 => &DNF5_REPOS_GLOBS as &[&str],
    };
    for glob in globs {
        for repo in glob::glob(glob)? {
            let repo = repo?;
            let file = File::open(repo)?;
            let reader = BufReader::new(file);

            let info = ini::repo_enabled(reader)?;
            if info.enabled {
                let solv_glob = match pm {
                    PackageManager::Zypper => {
                        format!("/var/cache/zypp/solv/{}/solv", info.name.replace('/', "_"))
                    }
                    // This uses the default system_cachedir configuration option for dnf
                    // Non superusers however use cachedir option (which defaults to /var/tmp/dnf-<username>-<random suffix>)
                    // I'm choosing the system one as dnf is more likely to be run with sudo
                    PackageManager::Dnf => format!(
                        "/var/cache/dnf/{}.solv",
                        info.name.replace('/', "_")
                    ),

                    // As with the old dnf, this uses the default system_cachedir value
                    // (The default non-superuser cachedir is ~/.cache/libdnf5)
                    PackageManager::Dnf5 => format!(
                        "/var/cache/libdnf5/{}-*/solv/*.solv",
                        info.name.replace('/', "_")
                    ),
                };
                for path in glob::glob(&solv_glob)? {
                    let i = SolvInput {
                        name: info.name.clone(),
                        path: path?,
                    };
                    repos.push(i);
                }
            }
        }
    }
    Ok(repos)
}

impl fmt::Display for ErrorKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::CommandNotFound(term) => {
                write!(f, " {}: {}", term, tr!("command not found"))
            }
            ErrorKind::PatternError(err) => {
                write!(f, "{}", err)
            }
            ErrorKind::GlobError(err) => {
                write!(f, "{}", err)
            }
            ErrorKind::IOError(err) => {
                write!(f, "{}", err)
            }
            ErrorKind::IsNULL(label) => {
                write!(f, "{} is NULL", label)
            }
            ErrorKind::IsNULLNamed(label, name) => {
                write!(f, "{} {} is NULL", label, name)
            }
            ErrorKind::RepoAddSolv(file) => {
                write!(f, "repo_add_solv failed on {}", file.display())
            }
            ErrorKind::String(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

impl From<glob::PatternError> for ErrorKind<'_> {
    fn from(value: glob::PatternError) -> Self {
        ErrorKind::PatternError(value)
    }
}

impl From<glob::GlobError> for ErrorKind<'_> {
    fn from(value: glob::GlobError) -> Self {
        ErrorKind::GlobError(value)
    }
}

impl From<std::io::Error> for ErrorKind<'_> {
    fn from(value: std::io::Error) -> Self {
        ErrorKind::IOError(value)
    }
}
