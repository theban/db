extern crate interval_tree;

use self::interval_tree::Range;
use self::interval_tree::IntervalTree;
use self::interval_tree::RangePairIter;

use std::collections::BTreeMap;

use content::Object;
use content::Bitmap;
use db_iterator::BitmapSliceIter;

pub struct DB {
    pub obj_map: BTreeMap<String, IntervalTree<Object>>,
    pub bit_map: BTreeMap<String, IntervalTree<Bitmap>>,
}


fn extend_range(mut r: Range) -> Range {
    if r.min > 0 {
        r.min -= 1
    }
    if r.max < !0 {
        r.max += 1
    }
    return r;
    //return Range::new(r.min.saturating_sub(1), r.max.saturating_add(1));
}


impl DB {
    pub fn new() -> DB {
        return DB { obj_map: BTreeMap::new(), bit_map: BTreeMap::new() };
    }

    pub fn new_from_data(obj_map: BTreeMap<String, IntervalTree<Object>>, bit_map: BTreeMap<String, IntervalTree<Bitmap>>) -> DB {
        return DB { obj_map: obj_map, bit_map: bit_map };
    }

    pub fn insert_object(&mut self, table: &String, r: Range, d: Object) {
        self.add_table(table);
        let mut tree = self.obj_map.get_mut(table).unwrap();
        tree.insert(r, d);
    }

    pub fn query_object(&self, table: &String, r: Range) -> Option<RangePairIter<Object>> {
        return self.obj_map.get(table).map(|tree| tree.range(r.min, r.max));
    }
    
    pub fn query_bitmap<'a>(&'a self, table: &String, r: Range) -> Option<BitmapSliceIter<'a>> {
        return self.bit_map.get(table).map(|tree| BitmapSliceIter::new(tree.range(r.min, r.max), r));
    }

    pub fn delete_object(&mut self, table: &String, r: Range) {
        if let Some(mut tree) = self.obj_map.get_mut(table) {
            tree.delete(r)
        };
    }

    pub fn delete_all_objects(&mut self, table: &String, r: Range) {
        let ranges = if let Some(iter) = self.query_object(table, r) {
            iter.map(|(range, _)| range.clone()).collect::<Vec<Range>>()
        } else {
            vec![]
        };
        for range in ranges {
            self.delete_object(table, range)
        }
    }

    fn get_overlaping_bitmaps(&mut self,
                              table: &String,
                              r: Range,
                              entry_size: u64)
                              -> Vec<(Range, Bitmap)> {
        let mut res = vec![];
        if let Some(iter) = self.bit_map.get(table).map(|tree| tree.range(r.min,r.max)) {
            for (ref rng, ref cont) in iter {
                if cont.entry_size == entry_size && (*rng).intersect(&r) {
                    res.push((**rng, (*cont).clone()));
                }
            }
        }
        return res;
    }

    fn delete_bitmaps_from_tree(&mut self, table: &String, bitmaps: &Vec<(Range, Bitmap)>) {
        for &(rng, _) in bitmaps {
            self.obj_map.get_mut(table).map(|mut tree| { println!("real delete rng: {:?}", rng); tree.delete(rng) });
        };
    }

    fn insert_bitmap(&mut self, table: &String, r: Range, d: Bitmap) {
            println!("insert: {:?} {:?}", &r, &d);
            assert_eq!(d.data.len() as u64, d.entry_size * r.len());
            self.add_table(table);

            let merge_partners = self.get_overlaping_bitmaps(table, extend_range(r), d.entry_size);

            println!("merge partners: {:?}", &merge_partners);
            self.delete_bitmaps_from_tree(table, &merge_partners);
            let (new_range, new_bitmap) = d.merge_bitmaps(r, merge_partners);

            println!("new data: {:?} {:?}", &new_range, &new_bitmap);
            let mut tree = self.bit_map.get_mut(table).unwrap();
            //println!("tree before: {:?}", &tree);
            tree.insert(new_range, new_bitmap);
            //println!("tree after: {:?}", &tree);
    }

    fn insert_subrange_bitmap(&mut self,
                              table: &String,
                              old_range: Range,
                              new_range: Range,
                              old_bitmap: &Bitmap,
                                ) {
        let data = old_bitmap.to_subbitmap(old_range, new_range);
        let mut tree = self.bit_map.get_mut(table).unwrap();
        tree.insert(new_range, data);
    }

    fn add_trunkated_version_of_bitmap(&mut self,
                                       table: &String,
                                       old_range: Range,
                                       old_bitmap: Bitmap,
                                       range_to_remove: Range) {
        let first_part = Range::new(old_range.min, range_to_remove.min);
        if first_part.min <= first_part.max {
            self.insert_subrange_bitmap(table, old_range, first_part, &old_bitmap)
        }
        let last_part = Range::new(range_to_remove.max, old_range.max);
        if last_part.min <= last_part.max {
            self.insert_subrange_bitmap(table, old_range, last_part, &old_bitmap)
        }
    }

    fn delete_bitmap(&mut self, table: &String, range_to_remove: Range, d: Bitmap) {
        let bitmaps_to_delete = self.get_overlaping_bitmaps(table, range_to_remove, d.entry_size);
        self.delete_bitmaps_from_tree(table, &bitmaps_to_delete);
        for (rng, data) in bitmaps_to_delete {
            self.add_trunkated_version_of_bitmap(table, rng, data, range_to_remove)
        }
    }

    fn add_table(&mut self, table: &String) {
        if !self.has_table(&table) {
            self.obj_map.insert(table.clone(), IntervalTree::new());
            self.bit_map.insert(table.clone(), IntervalTree::new());
        }
    }

    fn has_table(&mut self, table: &String) -> bool {
        assert!(self.obj_map.contains_key(table) == self.bit_map.contains_key(table));
        return self.obj_map.contains_key(table);
    }
}

#[test]
fn test_ranges() {
    let mut db = DB::new();
    let tbl = "foo".to_string();
    db.insert_object(&tbl,
              Range::new(3, 4),
              Object{data: "foo".into() } );
    db.insert_object(&tbl,
              Range::new(4, 5),
              Object{data: "foo".into() } );
    db.insert_object(&tbl,
              Range::new(5, 6),
              Object{ data:"foo".into() } );
    let mut is = db.query_object(&tbl, Range::new(4, 4))
                   .unwrap()
                   .map(|(r, _)| r.clone())
                   .collect::<Vec<Range>>();
    assert_eq!(is, vec![Range::new(3, 4), Range::new(4, 5)]);
    db.delete_all_objects(&tbl, Range::new(3, 4));
    is = db.query_object(&tbl, Range::new(0, 100))
           .unwrap()
           .map(|(r, _)| r.clone())
           .collect::<Vec<Range>>();
    assert_eq!(is, vec![Range::new(5, 6)]);

    assert!(db.query_object(&"bar".to_string(), Range::new(0, 100)).is_none());
}


#[test]
fn test_bitmaps_insert() {
    let mut db = DB::new();
    let tbl = "tbl".to_string();

    db.insert_bitmap(&tbl,
              Range::new(2, 8),
              Bitmap{ entry_size: 1, data: "foofoo".into() }
              );

    db.insert_bitmap(&tbl,
              Range::new(5, 11),
              Bitmap{ entry_size: 1, data: "barbar".into() }
              );

    let is = db.query_bitmap(&tbl, Range::new(0, 50))
                   .unwrap()
                   .map(|(r, data)| (r.clone(), data.to_bitmap() ) )
                   .collect::<Vec<(Range,Bitmap)>>();

    assert_eq!(is, vec![(Range::new(2, 11), Bitmap{entry_size: 1, data: "foobarbar".into() } ) ]);

    db.insert_bitmap(&tbl,
              Range::new(7, 10),
              Bitmap{ entry_size: 1, data: "goo".into() }
              );

    let is = db.query_bitmap(&tbl, Range::new(0, 50))
                   .unwrap()
                   .map(|(r, data)| (r.clone(), data.to_bitmap() ))
                   .collect::<Vec<(Range,Bitmap)>>();

    assert_eq!(is, vec![(Range::new(2, 11), Bitmap{entry_size: 1, data: "foobagoor".into() } ) ]);

    db.insert_bitmap(&tbl,
              Range::new(7, 10),
              Bitmap{ entry_size: 2, data: "googoo".into() }
              );

    let is = db.query_bitmap(&tbl, Range::new(0, 50))
                   .unwrap()
                   .map(|(r, data)| (r.clone(), data.to_bitmap()) )
                   .collect::<Vec<(Range,Bitmap)>>();

    assert_eq!(is, vec![
               (Range::new(2, 11), Bitmap{entry_size: 1, data: "foobagoor".into() } ), 
               (Range::new(7, 10), Bitmap{entry_size: 2, data: "googoo".into() } )
               ]);


    let is = db.query_bitmap(&tbl, Range::new(0, 3))
                   .unwrap()
                   .map(|(r, data)| (r.clone(), data.to_bitmap()) )
                   .collect::<Vec<(Range,Bitmap)>>();

    assert_eq!(is, vec![
               (Range::new(2, 3), Bitmap{entry_size: 1, data: "fo".into() } ), 
               ]);
    //db.delete_all(&tbl, Range::new(3, 4));

    //is = db.query(&tbl, Range::new(0, 100))
    //       .unwrap()
    //       .map(|(r, _)| r.clone())
    //       .collect::<Vec<Range>>();
    //assert_eq!(is, vec![Range::new(5, 6)]);

    //assert!(db.query(&"bar".to_string(), Range::new(0, 100)).is_none());
}
