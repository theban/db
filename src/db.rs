extern crate interval_tree;

use self::interval_tree::Range;
use self::interval_tree::IntervalTree;
use self::interval_tree::RangePairIter;
use std::collections::BTreeMap;


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

    pub fn add_table(&mut self, table: String){
        if !self.has_table(&table) {
            let tree = IntervalTree::new();
            self.map.insert(table,tree);
        }
    }

    pub fn has_table(&mut self, table: &String) -> bool{
        self.map.contains_key(table)
    }

    pub fn query(&mut self, table: &String, r:Range) -> Option<interval_tree::RangePairIter<Vec<u8>>>{
        match self.map.get_mut(table) {
            Some(tree) => Some(tree.range(r.min,r.max)),
            None => None
        }
    }

    pub fn delete(&mut self, table: &String, r:Range){
        if let Some(mut tree) = self.map.get_mut(table) { tree.delete(r) }; 
    }

    pub fn delete_all(&mut self, table: &String, r:Range){
        let ranges =
        if let Some(iter) = self.query(table,r) {
            iter.map(|(range,_)| range.clone()).collect::<Vec<Range>>()
        } else {
            vec![]
        };
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
    let mut is = db.query(&tbl,Range::new(4,4)).unwrap().map(|(r,_)| r.clone()).collect::<Vec<Range>>();
    assert_eq!(is, vec!( Range::new(3,4), Range::new(4,5) ));
    db.delete_all(&tbl,Range::new(3,4));
    is = db.query(&tbl,Range::new(0,100)).unwrap().map(|(r,_)| r.clone()).collect::<Vec<Range>>();
    assert_eq!(is, vec!( Range::new(5,6) ));

    assert!(db.query(&"bar".to_string(),Range::new(0,100)).is_none());
}

