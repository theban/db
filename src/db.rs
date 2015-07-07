extern crate interval_tree;

use self::interval_tree::Range;
use self::interval_tree::IntervalTree;
use self::interval_tree::RangePairIter;
use std::collections::BTreeMap;

extern crate libc;
use std::mem::transmute;
use std::ffi::{CStr, CString};

pub struct DB {
    map: BTreeMap<String,IntervalTree<Vec<u8>>>
}

impl DB {
    pub fn new() -> DB{
        return DB{map: BTreeMap::new()}
    }

    pub fn insert(&mut self, table: String, r: Range, d: Vec<u8>){
        {
            let mut tab = self.map.get_mut(&table);
            if let Some(ref mut tree) = tab { tree.insert(r,d); return }
        }
        let mut tree = IntervalTree::new();
        tree.insert(r,d);
        self.map.insert(table,tree);
    }

    pub fn query(&mut self, table: &String, r:Range) -> interval_tree::RangePairIter<Vec<u8>>{
        match self.map.get_mut(table) {
            Some(tree) => tree.range(r.min,r.max),
            None => unreachable!()
        }
    }

    pub fn delete(&mut self, table: &String, r:Range){
        if let Some(mut tree) = self.map.get_mut(table) { tree.delete(r) }; 
    }

    pub fn delete_all(&mut self, table: &String, r:Range){
        let ranges = self.query(table, r).map(|(range,_)| range.clone()).collect::<Vec<Range>>();
        for range in ranges {
            self.delete(table, range)
        }
    }
}

#[test]
fn test_db() {
    let mut db = DB::new();
    let tbl = "foo".to_string();
    db.insert(tbl.clone(),Range::new(3,4),"foo".to_string().into_bytes());
    db.insert(tbl.clone(),Range::new(4,5),"foo".to_string().into_bytes());
    db.insert(tbl.clone(),Range::new(5,6),"foo".to_string().into_bytes());
    let mut is = db.query(&tbl,Range::new(4,4)).map(|(r,_)| r.clone()).collect::<Vec<Range>>();
    assert_eq!(is, vec!( Range::new(3,4), Range::new(4,5) ));
    db.delete_all(&tbl,Range::new(3,4));
    is = db.query(&tbl,Range::new(0,100)).map(|(r,_)| r.clone()).collect::<Vec<Range>>();
    assert_eq!(is, vec!( Range::new(5,6) ));
}


#[no_mangle]
pub extern fn hallo_rust() -> *const u8{
    "Hello World!".as_ptr()
}

#[no_mangle]
pub extern fn new_db() -> *mut DB {
    unsafe { transmute(Box::new(DB::new())) }
}

#[no_mangle]
pub extern fn delete_db(db: *mut DB) {
    let _drop_me: Box<DB> = unsafe{ transmute(db) };
}

#[no_mangle]
pub extern fn insert_db(db: *mut DB, table: *const libc::c_char, from: u64, to: u64, data_len: u64, val: *mut u8, ) {
    let tbl = ffi_string_to_ref_str(table);
    let data: Vec<u8>= unsafe{ Vec::from_raw_buf(val,data_len as usize) };
    unsafe{ (*db).insert(tbl.to_string(), Range::new(from, to), data); }
}

fn ffi_string_to_ref_str<'a>(r_string: *const libc::c_char) -> &'a str {
  unsafe { CStr::from_ptr(r_string) }.to_str().unwrap()
}
 
fn string_to_ffi_string(string: String) -> *const libc::c_char {
  CString::new(string).unwrap().into_ptr()
}
