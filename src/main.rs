#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern crate configparser;
extern crate glob;
extern crate libc;

use std::env;
use std::ffi::CString;
use std::ffi::CStr;
use std::path::PathBuf;
use std::path::Path;
use std::result::Result;
use std::vec::Vec;
use std::process::exit;

use configparser::ini::Ini;

const REPO_GLOB: &str = "/etc/zypp/repos.d/*.repo";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let term = match args.get(1) {
        None => exit(127),
        Some(term) => term,
    };

    let bin_path = &Path::new("/usr/bin").join(Path::new(term));
    if Path::exists(bin_path) {
        println!("Absolute path to '{}' is '{}'. Please check your $PATH variable to see whether it contains the mentioned path.", term, bin_path.display());
        exit(0);
    }
    
    let sbin_path = &Path::new("/usr/sbin").join(Path::new(term));
    if Path::exists(sbin_path) {
        println!("Absolute path to '{}' is '{}', so running it may require superuser privileges (eg. root).", term, sbin_path.display());
        exit(0);
    }

    match search_solv(&term) {
        Err(msg) => {println!("{}", msg); exit(127);},
        _ => {},
    }
}

fn search_solv(term: &str) -> Result<(), String> {
    let repos = load_repos()?;

    let pool = SPool::new(&repos)?;
    let results = pool.search(&term);

    if results.len() == 0 {
        return Err(format!(" {}: command not found", term));
    }

    let suggested_package = if results.len() == 1 {
        results[0].Package.clone()
    } else {
        String::from("<selected_package>")
    };

    println!("
The program '{}' can be found in following packages:", &term);

    for r in results {
        println!("  * {} [ path: {}/{}, repository: zypp ({}) ]", r.Package, r.Path, &term, r.Repo);
    }

    println!("
Try installing with:
    sudo zypper install {}
", suggested_package);
    Ok(())
}

struct SolvInput {
    name: String,
    path: PathBuf,
}

fn load_repos() -> Result<Vec<SolvInput>, String> {
    let mut repos: Vec<SolvInput> = Vec::new();
    for repo in glob::glob(REPO_GLOB).map_err(stringify)? {
        let repo = repo.map_err(stringify)?;
        let mut parser = Ini::new();

        let _ = parser.load(&repo)?;

        for section in parser.sections() {
            let enabled = parser.get(&section, "enabled").unwrap_or(String::new());
            if enabled == "1" {
                let solv_glob = format!("/var/cache/zypp/solv/{}/solv", section.replace("/", "_"));
                for path in glob::glob(&solv_glob).map_err(stringify)? {
                    let i = SolvInput {
                        name: section.clone(),
                        path: path.map_err(stringify)?,
                    };
                    repos.push(i);
                }
            }
        }
    }
    Ok(repos)
}

struct SPool {
    pool: *mut Pool,
}

impl SPool {
    fn new(repos: &Vec<SolvInput>) -> Result<SPool, String> {
        let pool: *mut Pool = unsafe {
            let ptr = pool_create();
            if ptr.is_null() {
                return Err(String::from("pool_create returned NULL"));
            }
            ptr
        };

        for input in repos {
            let cname = CString::new(input.name.to_string()).map_err(|_e: std::ffi::NulError|-> String {String::from("input.name is null")})?;
            let csolv = CString::new(input.path.display().to_string()).map_err(|_e: std::ffi::NulError|-> String {String::from("input.path is null")})?;
            let repo: *mut Repo = unsafe { repo_create(pool, cname.into_raw()) };
            if repo.is_null() {
                return Err(format!("pool_create({}) returned NULL", input.name));
            }

            unsafe {
                let fp = fopen(csolv.into_raw(), CString::new("r").unwrap().into_raw());
                if fp.is_null() {
                    return Err(format!("can't open {}", input.path.display()))
                }
                let r = repo_add_solv(repo, fp, 0);
                fclose(fp);
                if r != 0 {
                    return Err(format!("repo_add_solv failed on {}", input.path.display()))
                }
            }
        }

        Ok(SPool { pool })
    }

fn search(&self, term: &str) -> Vec<SearchResult> {
        let cterm = CString::new(term).unwrap();
        // https://stackoverflow.com/questions/38995701/how-do-i-pass-a-closure-through-raw-pointers-as-an-argument-to-a-c-function/38997480#38997480
        let mut results: Vec<SearchResult> = Vec::new();
        let mut append = |repo: String, package: String, path: String| {
            results.push(SearchResult{Repo: repo.clone(), Package: package, Path: path});
        };
        let mut trait_obj: &mut dyn FnMut(String, String, String) = &mut append;
        let trait_obj_ref = &mut trait_obj;

        unsafe {
            pool_search(self.pool, 0, solv_knownid_SOLVABLE_FILELIST as i32, cterm.as_ptr(), SEARCH_STRING as i32, Some(callback), trait_obj_ref as *mut _ as *mut libc::c_void);
        }
        results
    }

}

struct SearchResult {
    Repo: String,
    Package: String,
    Path: String,
}

impl Drop for SPool {
    fn drop(&mut self) {
        unsafe { pool_free(self.pool) };
    }
}

unsafe extern "C" fn callback(cbdata: *mut libc::c_void, s: *mut s_Solvable, data: *mut s_Repodata, _key: *mut s_Repokey, kv: *mut s_KeyValue) -> i32 {
    // TODO: handle NULL and error here gracefully
    let repo = CStr::from_ptr((*(*s).repo).name).to_str().unwrap();
    let name = CStr::from_ptr(solvable_lookup_str(s, solv_knownid_SOLVABLE_NAME as i32)).to_str().unwrap();
    let path = CStr::from_ptr(repodata_dir2str(data, (*kv).id, 0 as *const i8)).to_str().unwrap();

    let append: &mut &mut dyn FnMut(String, String, String) = &mut *(cbdata as *mut _);
    append(
        String::from(repo),
        String::from(name),
        String::from(path),
    );
    0
}

fn stringify<T>(e: T) -> String 
where T: std::fmt::Display
{
    return format!("{}", e);
}
