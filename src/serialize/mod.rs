extern crate rmp;
extern crate interval_tree;

use std::collections::HashMap;
use std::error::Error;

use std::io;
use std::io::prelude::*;
use std::io::Cursor;
use std::fs::File;


use db::DB;
use dberror::DBError;
use self::interval_tree::Range;
use std;

fn write_table<'a>(name: String, data: HashMap<(u64,u64), Vec<u8>>, mut w: &mut Write) -> Result<(), DBError<'a>>{
    try!(rmp::encode::write_str_len(&mut w, name.len() as u32));
    try!(w.write_all(&name.as_ref()));
    try!(rmp::encode::write_array_len(&mut w, data.len() as u32));
    for ((min,max),val) in data {
        try!(rmp::encode::write_array_len(&mut w, 3));
        try!(rmp::encode::write_uint(&mut w, min));
        try!(rmp::encode::write_uint(&mut w, max));
        try!(rmp::encode::write_str_len(&mut w, val.len() as u32));
        try!(w.write_all(&val));
    }
    return Ok(())
}

fn write_flat<'a>(flat: HashMap<String,HashMap<(u64,u64),Vec<u8>>>, mut w: &mut Write) -> Result<(), DBError<'a>> {
    try!(rmp::encode::write_map_len(&mut w, flat.len() as u32));
    for (tbl, data) in flat {
        try!(write_table(tbl,data, &mut w));
    }
    return Ok(())
}

fn parse_string<'a>(stream: &mut Read) -> Result<String, DBError<'a>>{
    let strref = try!(parse_bindata(stream));
    let strval = try!(String::from_utf8(strref));
    return Ok(strval)
}

fn parse_bindata<'a>(mut stream: &mut Read) -> Result<Vec<u8>, DBError<'a>>{
    let len = try!(rmp::decode::read_str_len(&mut stream));
    let ulen = len as usize;
    let mut buf: Vec<u8>= vec![0;ulen]; 
    let n = try!(rmp::decode::read_full(&mut stream,&mut buf[0..ulen]));
    if n != ulen {
        return Err(DBError::FileFormatError("unexpected EOF".into()))
    }
    return Ok(buf)
}

fn read_table<'a>(db: &mut DB, name: String, mut r: &mut Read) -> Result<(), DBError<'a>>{
    let len = try!(rmp::decode::read_array_size(&mut r));
    for _ in 0..len {
        let len = try!(rmp::decode::read_array_size(&mut r));
        if len != 3 {
            return Err(DBError::FileFormatError("Entry length should be 3".into()))
        }
        let min = try!(rmp::decode::read_u64_loosely(&mut r));
        let max = try!(rmp::decode::read_u64_loosely(&mut r));
        let data = try!(parse_bindata(&mut r));
        db.insert(&name, Range::new(min,max), data);
    }
    return Ok(());
}

#[must_use]
fn read_db<'a>(mut db: &mut DB, mut r: &mut Read) -> Result<(), DBError<'a>>{
    let len = try!(rmp::decode::read_map_size(&mut r));
    for _ in 0..len {
        let tbl_name = try!(parse_string(&mut r));
        read_table(&mut db, tbl_name, &mut r);
    }
    return Ok(())
}

#[must_use]
fn write_db<'a>(db: &DB, mut w: &mut Write) -> Result<(), DBError<'a>>{
    let flat = db.to_flat();
    try!(write_flat(flat, &mut w));
    return Ok(())
}

#[must_use]
pub fn serialize<'a>(db: &DB) -> Result<Vec<u8>, DBError<'a> >{
    let mut buf = Vec::new();
    try!(write_db(db,&mut buf));
    return Ok(buf)
}

#[must_use]
pub fn deserialize<'a>(mut buf: Vec<u8>) -> Result<DB, DBError<'a> >{
    let mut db = DB::new();
    try!(read_db(&mut db,&mut Cursor::new(buf)));
    return Ok(db)
}

#[must_use]
pub fn save_to_file<'a>(db: DB, filename: &String) -> Result<(), DBError<'a>>{
    let mut f = try!(File::create(filename));
    let flat = db.to_flat();
    try!(write_flat(flat, &mut f));
    return Ok(())
}

#[must_use]
pub fn new_from_file<'a>(filename: &String) -> Result<DB, DBError<'a>>{
    let mut f = try!(File::open(filename));
    let mut db = DB::new();
    try!(read_db(&mut db, &mut f));
    return Ok(db)
}

#[test]
pub fn test_serialize() {

    let mut db = DB::new();
    let tbl = "foo".to_string();
    db.insert(&tbl,Range::new(3,4),"foo".to_string().into_bytes());
    db.insert(&tbl,Range::new(4,5),"foo".to_string().into_bytes());
    db.insert(&tbl,Range::new(5,6),"foo".to_string().into_bytes());
    
    let bin = serialize(&db).unwrap();
    let db2 = deserialize(bin).unwrap();
    let db2_keys=db2.query(&tbl, Range::new(0,100)).unwrap().map(|(r,_)| r.clone()).collect::<Vec<Range>>();
    let db1_keys=db.query(&tbl, Range::new(0,100)).unwrap().map(|(r,_)| r.clone()).collect::<Vec<Range>>();
    assert_eq!(db1_keys, db2_keys);
}


