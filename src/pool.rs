#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::upper_case_acronyms)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern crate libc;

use crate::{ErrorKind, SolvInput};
use std::env;
use std::ffi::CStr;
use std::ffi::CString;
use std::io;

pub struct SPool {
    pool: *mut Pool,
}

pub struct SearchResult {
    pub Repo: String,
    pub Package: String,
    pub Path: String,
}

impl SPool {
    pub fn new(repos: &[SolvInput]) -> Result<SPool, ErrorKind<'_>> {
        let pool: *mut Pool = unsafe {
            let ptr = pool_create();
            if ptr.is_null() {
                return Err(ErrorKind::IsNULL("pool_create"));
            }
            ptr
        };

        for input in repos {
            let cname = CString::new(input.name.to_string()).map_err(
                |_e: std::ffi::NulError| -> ErrorKind { ErrorKind::IsNULL("input.name") },
            )?;
            let csolv = CString::new(input.path.display().to_string()).map_err(
                |_e: std::ffi::NulError| -> ErrorKind { ErrorKind::IsNULL("input.path") },
            )?;
            let repo: *mut Repo = unsafe { repo_create(pool, cname.into_raw()) };
            if repo.is_null() {
                return Err(ErrorKind::IsNULLNamed("pool_create", &input.name));
            }

            unsafe {
                const rdonly: std::os::raw::c_char = 114; // ASCII r
                let fp = fopen(csolv.into_raw(), &rdonly);
                if fp.is_null() {
                    return Err(ErrorKind::IOError(io::Error::last_os_error()));
                }
                let r = repo_add_solv(repo, fp, 0);
                fclose(fp);
                if r != 0 {
                    return Err(ErrorKind::RepoAddSolv(&input.path));
                }
            }
        }

        Ok(SPool { pool })
    }

    pub fn search(&self, term: &str) -> Result<Vec<SearchResult>, ErrorKind<'static>> {
        let cterm = CString::new(term).map_err(|_e: std::ffi::NulError| -> ErrorKind {
            ErrorKind::IsNULL("Ctring::New(term)")
        })?;
        let mut results: Vec<SearchResult> = Vec::new();
        let mut error: Option<ErrorKind> = None;
        let mut append = |result: Result<(String, String, String), ErrorKind<'static>>| match result
        {
            Err(err) => error = Some(err),
            Ok(result) => {
                let (repo, package, path) = result;
                if path != "/usr/bin" && path != "/usr/sbin" {
                    return;
                }
                results.push(SearchResult {
                    Repo: repo,
                    Package: package,
                    Path: path,
                });
            }
        };

        // https://stackoverflow.com/questions/38995701/how-do-i-pass-a-closure-through-raw-pointers-as-an-argument-to-a-c-function/38997480#38997480
        let mut trait_obj: &mut dyn FnMut(Result<(String, String, String), ErrorKind<'static>>) =
            &mut append;
        let trait_obj_ref = &mut trait_obj;

        unsafe {
            pool_search(
                self.pool,
                0,
                solv_knownid_SOLVABLE_FILELIST as i32,
                cterm.as_ptr(),
                SEARCH_STRING as i32,
                Some(callback),
                trait_obj_ref as *mut _ as *mut libc::c_void,
            );
        }

        match error {
            Some(err) => Err(err),
            None => Ok(results),
        }
    }
}

impl Drop for SPool {
    fn drop(&mut self) {
        unsafe { pool_free(self.pool) };
    }
}

unsafe extern "C" fn callback(
    cbdata: *mut libc::c_void,
    s: *mut s_Solvable,
    data: *mut s_Repodata,
    _key: *mut s_Repokey,
    kv: *mut s_KeyValue,
) -> i32 {
    // code does not assert callback data, those are a responsibility of a libsolv/caller

    let append: &mut &mut dyn FnMut(Result<(String, String, String), ErrorKind<'static>>) =
        &mut *(cbdata as *mut _);

    let result: Result<(String, String, String), ErrorKind<'static>> =
        CStr::from_ptr((*(*s).repo).name)
            .to_str()
            .map_err(|err: std::str::Utf8Error| -> ErrorKind { ErrorKind::String(err.to_string()) })
            .and_then(|repo: &str| {
                CStr::from_ptr(solvable_lookup_str(s, solv_knownid_SOLVABLE_NAME as i32))
                    .to_str()
                    .map_err(|err: std::str::Utf8Error| -> ErrorKind {
                        ErrorKind::String(err.to_string())
                    })
                    .map(|name| (repo, name))
            })
            .and_then(|(repo, name)| {
                CStr::from_ptr(repodata_dir2str(
                    data,
                    (*kv).id,
                    std::ptr::null::<std::os::raw::c_char>(),
                ))
                .to_str()
                .map_err(|err: std::str::Utf8Error| -> ErrorKind {
                    ErrorKind::String(err.to_string())
                })
                .map(|path| (repo, name, path))
            })
            .map(|(repo, name, path)| (String::from(repo), String::from(name), String::from(path)));

    let ret = match result {
        Err(_) => -1,
        _ => 0,
    };
    append(result);
    ret
}
