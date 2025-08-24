use std::io::BufRead;
use std::result::Result;

#[derive(Debug, Clone)]
pub struct Repo {
    pub enabled: bool,
    pub name: String,
}

pub fn repo_enabled<R: BufRead>(reader: R) -> Result<Repo, std::io::Error> {
    let mut name = String::from("N/A");

    for line in reader.lines().map_while(Result::ok) {
        if line.starts_with('[') && line.ends_with(']') {
            name = line
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_string();
        }
        if line.starts_with("enabled") && line.ends_with('1') {
            return Ok(Repo {
                enabled: true,
                name,
            });
        }
    }
    Ok(Repo {
        enabled: false,
        name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_repo_enabled_true() {
        let repo_content = "[main]\nenabled=1\n";
        let reader = Cursor::new(repo_content);
        let repo = repo_enabled(reader).unwrap();
        assert!(repo.enabled);
        assert_eq!(repo.name, "main");
    }

    #[test]
    fn test_repo_enabled_false() {
        let repo_content = "[other]\nenabled=0\n";
        let reader = Cursor::new(repo_content);
        let repo = repo_enabled(reader).unwrap();
        assert!(!repo.enabled);
        assert_eq!(repo.name, "other");
    }

    #[test]
    fn test_repo_name_parsing() {
        let repo_content = "[GNOME_Next]\nenabled=1\n";
        let reader = Cursor::new(repo_content);
        let repo = repo_enabled(reader).unwrap();
        assert_eq!(repo.name, "GNOME_Next");
    }
}
