extern crate rmp as msgpack;
extern crate rustc_serialize;
extern crate interval_tree;

use self::rustc_serialize::Encodable;
use self::msgpack::Encoder;

use db::DB;
use self::interval_tree::Range;


pub fn serialize(db: &DB) -> Vec<u8>{
    let mut buf = Vec::new();
    db.to_flat().encode(&mut Encoder::new(&mut buf)).unwrap();
    return buf;
}

#[test]
pub fn test_serialize() {

    let mut db = DB::new();
    let tbl = "foo".to_string();
    db.insert(tbl.clone(),Range::new(3,4),"foo".to_string().into_bytes());
    db.insert(tbl.clone(),Range::new(4,5),"foo".to_string().into_bytes());
    db.insert(tbl.clone(),Range::new(5,6),"foo".to_string().into_bytes());


    let encoded = vec![129, 163, 102, 111, 111, 131, 146, 3, 4, 147, 102, 111, 111, 146, 4, 5, 147, 102, 111, 111, 146, 5, 6, 147, 102, 111, 111];
    assert_eq!(serialize(&db), encoded);
}


