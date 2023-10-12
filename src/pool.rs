#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern crate libc;

use crate::SolvInput;
use std::env;
use std::ffi::CStr;
use std::ffi::CString;

pub struct SPool {
    pool: *mut Pool,
}

pub struct SearchResult {
    pub Repo: String,
    pub Package: String,
    pub Path: String,
}

impl SPool {
    pub fn new(repos: &Vec<SolvInput>) -> Result<SPool, String> {
        let pool: *mut Pool = unsafe {
            let ptr = pool_create();
            if ptr.is_null() {
                return Err(String::from("pool_create returned NULL"));
            }
            ptr
        };

        for input in repos {
            let cname = CString::new(input.name.to_string()).map_err(
                |_e: std::ffi::NulError| -> String { String::from("input.name is null") },
            )?;
            let csolv = CString::new(input.path.display().to_string()).map_err(
                |_e: std::ffi::NulError| -> String { String::from("input.path is null") },
            )?;
            let repo: *mut Repo = unsafe { repo_create(pool, cname.into_raw()) };
            if repo.is_null() {
                return Err(format!("pool_create({}) returned NULL", input.name));
            }

            unsafe {
                let fp = fopen(csolv.into_raw(), CString::new("r").unwrap().into_raw());
                if fp.is_null() {
                    return Err(format!("can't open {}", input.path.display()));
                }
                let r = repo_add_solv(repo, fp, 0);
                fclose(fp);
                if r != 0 {
                    return Err(format!("repo_add_solv failed on {}", input.path.display()));
                }
            }
        }

        Ok(SPool { pool })
    }

    pub fn search(&self, term: &str) -> Vec<SearchResult> {
        let cterm = CString::new(term).unwrap();
        // https://stackoverflow.com/questions/38995701/how-do-i-pass-a-closure-through-raw-pointers-as-an-argument-to-a-c-function/38997480#38997480
        let mut results: Vec<SearchResult> = Vec::new();
        let mut append = |repo: String, package: String, path: String| {
            if path != "/usr/bin" && path != "/usr/sbin" {
                return;
            }
            results.push(SearchResult {
                Repo: repo.clone(),
                Package: package,
                Path: path,
            });
        };
        let mut trait_obj: &mut dyn FnMut(String, String, String) = &mut append;
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
        results
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
    // TODO: handle NULL and error here gracefully
    let repo = CStr::from_ptr((*(*s).repo).name).to_str().unwrap();
    let name = CStr::from_ptr(solvable_lookup_str(s, solv_knownid_SOLVABLE_NAME as i32))
        .to_str()
        .unwrap();
    let path = CStr::from_ptr(repodata_dir2str(
        data,
        (*kv).id,
        0 as *const std::os::raw::c_char,
    ))
    .to_str()
    .unwrap();

    let append: &mut &mut dyn FnMut(String, String, String) = &mut *(cbdata as *mut _);
    append(String::from(repo), String::from(name), String::from(path));
    0
}
