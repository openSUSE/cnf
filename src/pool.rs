mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(unnecessary_transmutes)]
    #![allow(clippy::upper_case_acronyms)]
    #![allow(clippy::ptr_offset_with_cast)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
use bindings::*;

extern crate libc;

use crate::{ErrorKind, SolvInput};
use libc::{uname, utsname};
use std::ffi::CStr;
use std::ffi::CString;
use std::io;
use std::os::raw::c_char;

pub struct SPool {
    pool: *mut Pool,
}

pub struct SearchResult {
    pub repo: String,
    pub package: String,
    pub path: String,
}

impl SPool {
    pub fn new(repos: &[SolvInput]) -> Result<SPool, ErrorKind<'_>> {
        let pool: *mut Pool = unsafe {
            let ptr = pool_create();
            if ptr.is_null() {
                return Err(ErrorKind::IsNULL("pool_create"));
            }

            let mut uts = std::mem::zeroed::<utsname>();
            if uname(&mut uts) == 0 {
                pool_setarch(ptr, uts.machine.as_ptr() as *const c_char);
            } else {
                let errno = *libc::__errno_location();
                return Err(ErrorKind::IOError(io::Error::from_raw_os_error(errno)));
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
                const RDONLY: std::os::raw::c_char = 114; // ASCII r
                let fp = fopen(csolv.into_raw(), &RDONLY);
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
        let mut append = |result: Result<SearchResult, ErrorKind<'static>>| match result {
            Err(err) => error = Some(err),
            Ok(result) => {
                if result.path != "/usr/bin" && result.path != "/usr/sbin" {
                    return;
                }
                results.push(result);
            }
        };

        // https://stackoverflow.com/questions/38995701/how-do-i-pass-a-closure-through-raw-pointers-as-an-argument-to-a-c-function/38997480#38997480
        let mut trait_obj: &mut dyn FnMut(Result<SearchResult, ErrorKind<'static>>) = &mut append;
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
    let append: &mut &mut dyn FnMut(Result<SearchResult, ErrorKind<'static>>) =
        &mut *(cbdata as *mut _);

    let search_result: Result<SearchResult, ErrorKind<'static>> = (|| {
        let repo = cstr_to_string((*(*s).repo).name)?;
        let package = cstr_to_string(solvable_lookup_str(s, solv_knownid_SOLVABLE_NAME as i32))?;
        let path = cstr_to_string(repodata_dir2str(
            data,
            (*kv).id,
            std::ptr::null::<std::os::raw::c_char>(),
        ))?;
        Ok(SearchResult {
            repo,
            package,
            path,
        })
    })();

    match search_result {
        Err(_) => -1,
        Ok(result) => {
            if is_installable(s) {
                append(Ok(result));
            }
            0
        }
    }
}

fn cstr_to_string(ptr: *const libc::c_char) -> Result<String, ErrorKind<'static>> {
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map(|s| s.to_owned())
        .map_err(|e| ErrorKind::String(e.to_string()))
}

fn is_installable(s: *mut s_Solvable) -> bool {
    let solvable: &s_Solvable = unsafe { &*s };
    let s_repo = solvable.repo;
    let mut installable = false;
    if !s_repo.is_null() {
        let pool = unsafe { *s_repo }.pool;
        installable = unsafe { cnf_pool_installable(pool, s) } == 1;
    }
    installable
}
