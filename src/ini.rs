use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::PathBuf;
use std::result::Result;

pub struct Repo {
    pub enabled: bool,
    pub name: String,
}

pub fn repo_enabled(path: &PathBuf) -> Result<Repo, String> {
    let lines = read_lines(&path).map_err(stringify)?;
    let mut name = String::from("N/A");

    for line in lines {
        if let Ok(ip) = line {
            if ip.starts_with("[") && ip.ends_with("]") {
                name = ip.replace(&['[', ']'][..], "");
            }
            if ip.starts_with("enabled") && ip.ends_with("1") {
                return Ok(Repo {
                    enabled: true,
                    name,
                });
            }
        }
    }
    Ok(Repo {
        enabled: false,
        name,
    })
}

//https://doc.rust-lang.org/stable/rust-by-example/std_misc/file/read_lines.html

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<std::path::Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn stringify<T>(e: T) -> String
where
    T: std::fmt::Display,
{
    return format!("{}", e);
}
