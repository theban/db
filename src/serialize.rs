//extern crate rmp as msgpack;
//extern crate rustc_serialize;
//extern crate interval_tree;

//use self::rustc_serialize::Encodable;
//use self::rustc_serialize::Decodable;
//use self::msgpack::Encoder;
//use self::msgpack::Decoder;
//
//use std::collections::HashMap;
//
//use db::DB;
//use self::interval_tree::Range;
//
//
//pub fn serialize(db: &DB) -> Vec<u8>{
//    let mut buf = Vec::new();
//    db.to_flat().encode(&mut Encoder::new(&mut buf)).unwrap();
//    return buf;
//}
//
//pub fn deserialize(buf: Vec<u8>) -> DB{
//    let buf_slice = &mut buf.as_slice();
//    let mut decoder = Decoder::new(buf_slice);
//    let flat_db : HashMap<String,HashMap<(u64,u64),Vec<u8>>> = Decodable::decode(&mut decoder).unwrap();
//    return DB::new_from_flat(flat_db);
//}
//
//#[test]
//pub fn test_serialize() {
//
//    let mut db = DB::new();
//    let tbl = "foo".to_string();
//    db.insert(tbl.clone(),Range::new(3,4),"foo".to_string().into_bytes());
//    db.insert(tbl.clone(),Range::new(4,5),"foo".to_string().into_bytes());
//    db.insert(tbl.clone(),Range::new(5,6),"foo".to_string().into_bytes());
//
//
//    let db2 = deserialize(serialize(&db));
//    let db2_keys=db2.query(&tbl, Range::new(0,100)).unwrap().map(|(r,_)| r.clone()).collect::<Vec<Range>>();
//    let db1_keys=db.query(&tbl, Range::new(0,100)).unwrap().map(|(r,_)| r.clone()).collect::<Vec<Range>>();
//    assert_eq!(db1_keys, db2_keys);
//}


