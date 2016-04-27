extern crate interval_tree;
extern crate memrange;

use self::memrange::Range;
use self::interval_tree::IntervalTree;
use self::interval_tree::RangePairIter;

use std::collections::BTreeMap;
use std::u64;

use content::Object;
use content::Bitmap;
use db_iterator::BitmapSliceIter;

pub struct DB {
    pub obj_map: BTreeMap<String, IntervalTree<Object>>,
    pub bit_map: BTreeMap<String, IntervalTree<Bitmap>>,
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
            self.bit_map.get_mut(table).map(|mut tree| { tree.delete(rng) });
        };
    }

    pub fn insert_bitmap(&mut self, table: &String, r: Range, d: Bitmap) {
            assert_eq!(d.data.len() as u64, d.entry_size * r.len());
            self.add_table(table);

            let merge_partners = self.get_overlaping_bitmaps(table, r.get_extended(), d.entry_size);

            self.delete_bitmaps_from_tree(table, &merge_partners);
            let (new_range, new_bitmap) = d.merge_bitmaps(r, merge_partners);

            let mut tree = self.bit_map.get_mut(table).unwrap();
            tree.insert(new_range, new_bitmap);
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

    //fn add_trunkated_version_of_bitmap(&mut self,
    //                                   table: &String,
    //                                   old_range: Range,
    //                                   old_bitmap: Bitmap,
    //                                   range_to_remove: Range) {


    //    if (old_range.min <= range_to_remove.min) && (range_to_remove.min > 0){
    //        let first_part = Range::new(old_range.min, range_to_remove.min);
    //        self.insert_subrange_bitmap(table, old_range, first_part, &old_bitmap)
    //    }
    //    if (range_to_remove.max <= old_range.max) && (range_to_remove.max < u64::MAX){
    //        let last_part = Range::new(range_to_remove.max, old_range.max);
    //        self.insert_subrange_bitmap(table, old_range, last_part, &old_bitmap)
    //    }
    //}

    fn add_trunkated_version_of_bitmap(&mut self,
                                       table: &String,
                                       old_range: Range,
                                       old_bitmap: Bitmap,
                                       range_to_remove: Range) {

        let (first,last) = old_range.get_difference(&range_to_remove);
        if let Some(first_part) = first {
            self.insert_subrange_bitmap(table, old_range, first_part, &old_bitmap)
        }
        if let Some(last_part) = last {
            self.insert_subrange_bitmap(table, old_range, last_part, &old_bitmap)
        }
    }

    pub fn delete_bitmap(&mut self, table: &String,entry_size: u64, range_to_remove: Range) {
        let bitmaps_to_delete = self.get_overlaping_bitmaps(table, range_to_remove, entry_size);
        self.delete_bitmaps_from_tree(table, &bitmaps_to_delete);
        for (rng, data) in bitmaps_to_delete {
            self.add_trunkated_version_of_bitmap(table, rng, data, range_to_remove.get_extended());
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

#[cfg(test)]
fn query_bitmap_test(db: &mut DB, tbl: &String, rng: Range) -> Vec<(Range,Bitmap)>{
    return db.query_bitmap(&tbl, rng)
                   .unwrap()
                   .map(|(r, data)| (r.clone(), data.to_bitmap()) )
                   .collect::<Vec<(Range,Bitmap)>>();
}

#[test]
fn test_bitmaps_insert() {
    let mut db = DB::new();
    let tbl = "tbl".to_string();

    db.insert_bitmap(&tbl,
              Range::new(2, 7),
              Bitmap{ entry_size: 1, data: "foofoo".into() }
              );

    db.insert_bitmap(&tbl,
              Range::new(5, 10),
              Bitmap{ entry_size: 1, data: "barbar".into() }
              );

    let is = query_bitmap_test(&mut db, &tbl, Range::new(0, 50));
    assert_eq!(is, vec![(Range::new(2, 10), Bitmap{entry_size: 1, data: "foobarbar".into() } ) ]);

    db.insert_bitmap(&tbl,
              Range::new(7, 9),
              Bitmap{ entry_size: 1, data: "goo".into() }
              );

    let is = query_bitmap_test(&mut db, &tbl, Range::new(0, 50));
    assert_eq!(is, vec![(Range::new(2, 10), Bitmap{entry_size: 1, data: "foobagoor".into() } ) ]);

    db.insert_bitmap(&tbl,
              Range::new(7, 9),
              Bitmap{ entry_size: 2, data: "googoo".into() }
              );

    let is = query_bitmap_test(&mut db, &tbl, Range::new(0, 50));
    assert_eq!(is, vec![
               (Range::new(2, 10), Bitmap{entry_size: 1, data: "foobagoor".into() } ), 
               (Range::new(7, 9), Bitmap{entry_size: 2, data: "googoo".into() } )
               ]);

    let is = query_bitmap_test(&mut db, &tbl, Range::new(0, 3));
    assert_eq!(is, vec![
               (Range::new(2, 3), Bitmap{entry_size: 1, data: "fo".into() } ), 
               ]);

    db.delete_bitmap(&tbl,1, Range::new(0, 1000));
    db.delete_bitmap(&tbl,2, Range::new(0, 1000));
    db.delete_bitmap(&tbl,3, Range::new(0, 1000));

    let is = query_bitmap_test(&mut db, &tbl, Range::new(0, 1000));
    assert_eq!(is, vec![ ]);

    db.insert_bitmap(&tbl,
              Range::new(0, 10),
              Bitmap{ entry_size: 1, data: "googooazabu".into() }
              );
    db.delete_bitmap(&tbl,1, Range::new(2, 3));
    let is = query_bitmap_test(&mut db, &tbl, Range::new(0, 1000));

    assert_eq!(is, vec![
               (Range::new(0, 1), Bitmap{entry_size: 1, data: "go".into() } ), 
               (Range::new(4, 10), Bitmap{entry_size: 1, data: "ooazabu".into() } ), 
               ]);

    db.delete_bitmap(&tbl,1, Range::new(0, 0));
    let is = query_bitmap_test(&mut db, &tbl, Range::new(0, 1000));

    assert_eq!(is, vec![
               (Range::new(1, 1), Bitmap{entry_size: 1, data: "o".into() } ), 
               (Range::new(4, 10), Bitmap{entry_size: 1, data: "ooazabu".into() } ), 
               ]);

}
