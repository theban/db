extern crate rmp;
extern crate interval_tree;

use std::collections::BTreeMap;
use std::u64;
use std::u32;

use std::io::prelude::*;
use std::io::Cursor;
use std::fs::File;


use db::DB;
use content::Bitmap;
use content::Object;
use dberror::DBError;
use self::interval_tree::Range;
use self::interval_tree::IntervalTree;

trait Serialized where Self:Sized{
    fn write<'a>(&self, mut w: &mut Write) -> Result<(), DBError<'a>>;
    fn read<'a>(mut r: &mut Read) -> Result<Self, DBError<'a>>;
}

fn write_vec<'a>(vec: &Vec<u8>, mut w: &mut Write) -> Result<(), DBError<'a>> {
    try!(rmp::encode::write_str_len(&mut w, vec.len() as u32));
    try!(w.write_all(vec));
    return Ok(());
}

fn parse_string<'a>(stream: &mut Read) -> Result<String, DBError<'a>> {
    let strref = try!(parse_bindata(stream));
    let strval = try!(String::from_utf8(strref));
    return Ok(strval);
}

fn parse_bindata<'a>(mut stream: &mut Read) -> Result<Vec<u8>, DBError<'a>> {
    let len = try!(rmp::decode::read_str_len(&mut stream));
    let ulen = len as usize;
    let mut buf: Vec<u8> = vec![0;ulen];
    let n = try!(rmp::decode::read_full(&mut stream, &mut buf[0..ulen]));
    if n != ulen {
        return Err(DBError::FileFormatError("unexpected EOF".into()));
    }
    return Ok(buf);
}

fn parse_range<'a>(mut r: &mut Read) -> Result<Range, DBError<'a>> {
    let min = try!(rmp::decode::read_u64_loosely(&mut r));
    let max = try!(rmp::decode::read_u64_loosely(&mut r));
    return Ok(Range::new(min, max));
}

impl Serialized for Bitmap {
    fn write<'a>(&self, mut w: &mut Write) -> Result<(), DBError<'a>> {
        try!(rmp::encode::write_array_len(&mut w, 2));
        try!(rmp::encode::write_uint(&mut w, self.entry_size));
        try!(write_vec(&self.data, &mut w));
        return Ok(())
    }

    fn read<'a>(mut r: &mut Read) -> Result<Self, DBError<'a>> {
        let ds = try!(rmp::decode::read_u64_loosely(&mut r));
        let vec = try!(parse_bindata(&mut r));
        let data = Bitmap{entry_size: ds, data:vec};
        return Ok( data );
    }
}

impl Serialized for Object {
    fn write<'a>(&self, mut w: &mut Write) -> Result<(), DBError<'a>> {
        try!(write_vec(&self.data, &mut w));
        return Ok(())
    }

    fn read<'a>(mut r: &mut Read) -> Result<Self, DBError<'a>> {
        let vec = try!(parse_bindata(&mut r));
        let data = Object{data: vec};
        return Ok( data );
    }

}

impl<T: Serialized> Serialized for IntervalTree<T> {
    fn write<'a>(&self, mut w: &mut Write) -> Result<(), DBError<'a>> {
        let len = self.range(0, u64::MAX).count() as u64;
        assert!(3*len < u32::MAX as u64);
        try!(rmp::encode::write_array_len(&mut w, 3*len as u32));
        for (range, data) in self.range(0, u64::MAX) {
            try!(rmp::encode::write_uint(&mut w, range.min));
            try!(rmp::encode::write_uint(&mut w, range.max));
            try!(data.write(&mut w));
        }
        return Ok(())
    }

    fn read<'a>(mut r: &mut Read) -> Result<Self, DBError<'a>> {
        let len = try!(rmp::decode::read_array_size(&mut r));
        let mut tree = IntervalTree::new();
        for _ in 0..len {
            let rng = try!(parse_range(&mut r));
            let data = try!(T::read(&mut r));
            tree.insert(rng, data);
        }
        return Ok( tree );
    }
}

impl<T: Serialized> Serialized for BTreeMap<String, IntervalTree<T>> {
    fn write<'a>(&self, mut w: &mut Write) -> Result<(), DBError<'a>> {
        try!(rmp::encode::write_map_len(&mut w, self.len() as u32));
        for (name, tree) in self {
            try!(rmp::encode::write_str_len(&mut w, name.len() as u32));
            try!(w.write_all(&name.as_ref()));
            try!(tree.write(&mut w))
        }
    return Ok(())
    }

    fn read<'a>(mut r: &mut Read) -> Result<Self, DBError<'a>> {
        let len = try!(rmp::decode::read_map_size(&mut r));
        let mut res = BTreeMap::new();
        for _ in 0..len {
            let tbl_name = try!(parse_string(&mut r));
            let tree = try!(IntervalTree::<T>::read(&mut r));
            res.insert(tbl_name, tree);
        }
        return Ok( res );
    }
}

impl Serialized for DB {
    fn write<'a>(&self, mut w: &mut Write) -> Result<(), DBError<'a>> {
        try!(self.obj_map.write(&mut w));
        try!(self.bit_map.write(&mut w));
        return Ok(());
    }

    fn read<'a>(mut r: &mut Read) -> Result<Self, DBError<'a>> {
        let objects = try!(BTreeMap::<String, IntervalTree<Object>>::read(r));
        let bitmaps = try!(BTreeMap::<String, IntervalTree<Bitmap>>::read(r));
        return Ok( DB::new_from_data(objects,bitmaps) );
    }
}

impl DB {
    #[must_use]
    pub fn serialize<'a>(&self) -> Result<Vec<u8>, DBError<'a>> {
        let mut buf = Vec::new();
        try!(self.write(&mut buf));
        return Ok(buf);
    }

    #[must_use]
    pub fn deserialize<'a>(buf: Vec<u8>) -> Result<DB, DBError<'a>> {
        let db = try!(DB::read(&mut Cursor::new(buf)));
        return Ok(db);
    }

    #[must_use]
    pub fn save_to_file<'a>(&self, filename: &String) -> Result<(), DBError<'a>> {
        let mut f = try!(File::create(filename));
        try!(self.write(&mut f));
        return Ok(());
    }

    #[must_use]
    pub fn new_from_file<'a>(filename: &String) -> Result<DB, DBError<'a>> {
        let mut f = try!(File::open(filename));
        let db = try!(DB::read(&mut f));
        return Ok(db);
    }
}


#[test]
pub fn test_serialize() {

    let mut db = DB::new();
    let tbl = "foo".to_string();
    db.insert_object(&tbl,
              Range::new(3, 4),
              Object{data: "foo".into()}); 
    db.insert_object(&tbl,
              Range::new(4, 5),
              Object{data: "foo".into()});
    db.insert_object(&tbl,
              Range::new(5, 6),
              Object{data: "foo".into()});

    let bin = db.serialize().unwrap();
    let hex: Vec<String> = bin.iter().map(|b| format!("{:02X}", b)).collect();
    println!("{}", hex.join(" "));
    let db2 = DB::deserialize(bin).unwrap();
    let db2_keys = db2.query_object(&tbl, Range::new(0, 100))
                      .unwrap()
                      .map(|(r, _)| r.clone())
                      .collect::<Vec<Range>>();
    let db1_keys = db.query_object(&tbl, Range::new(0, 100))
                     .unwrap()
                     .map(|(r, _)| r.clone())
                     .collect::<Vec<Range>>();
    assert_eq!(db1_keys, db2_keys);
}
