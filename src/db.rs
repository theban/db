extern crate interval_tree;

use self::interval_tree::Range;
use self::interval_tree::IntervalTree;
use self::interval_tree::RangePairIter;

use std::collections::BTreeMap;
use std::collections::HashMap;

#[derive(Clone)]
pub enum Content {
    Data(Vec<u8>),
    BitMap {
        datasize: u64,
        data: Vec<u8>,
    },
}

pub struct DB {
    map: BTreeMap<String, IntervalTree<Content>>,
}


fn extend_range(mut r: Range) -> Range {
    if r.min > 0 {
        r.min -= 1
    }
    if r.max < !0 {
        r.max += 1
    }
    return r;
}

impl DB {
    pub fn new() -> DB {
        return DB { map: BTreeMap::new() };
    }

    fn get_overlaping_bitmaps(&mut self,
                              table: &String,
                              r: Range,
                              datasize: u64)
                              -> Vec<(Range, Vec<u8>)> {
        let mut res = vec![];
        if let Some(iter) = self.query(table, r) {
            for (ref rng, ref cont) in iter {
                if let Content::BitMap{ datasize: ds, data: ref dat} = **cont {
                    if ds == datasize && (*rng).intersect(&r) {
                        res.push((**rng, dat.clone()));
                    }
                }
            }
        }
        return res;
    }

    fn merge_bitmaps(&mut self,
                     datasize: u64,
                     merge_partners: Vec<(Range, Vec<u8>)>)
                     -> (Range, Vec<u8>) {
        let mut min = !0;
        let mut max = 0;
        for &(rng, _) in &merge_partners {
            if min > rng.min {
                min = rng.min
            }
            if max < rng.max {
                max = rng.max
            }
        }
        let mut combined = Vec::with_capacity((max - min) as usize);
        for &(ref rng, ref cont) in &merge_partners {
            let offset = (rng.min - min) * datasize;
            if offset > (!(0 as usize) as u64) {
                panic!("That Range is way to big");
            }
            for (i, val) in cont.iter().enumerate() {
                combined[i + offset as usize] = *val;
            }
        }
        return (Range::new(min, max), combined);
    }

    fn delete_bitmaps_from_tree(&mut self, table: &String, bitmaps: &Vec<(Range, Vec<u8>)>) {
        for &(rng, _) in bitmaps {
            self.delete(table, rng);
        }
    }

    fn insert_bitmap(&mut self, table: &String, r: Range, d: Content) {
        if let Content::BitMap { datasize: ds, data } = d {
            let mut merge_partners = self.get_overlaping_bitmaps(table, extend_range(r), ds);
            self.delete_bitmaps_from_tree(table, &merge_partners);
            merge_partners.push((r, data));
            let (new_range, new_data) = self.merge_bitmaps(ds, merge_partners);
            self.insert(table,
                        new_range,
                        Content::BitMap {
                            datasize: ds,
                            data: new_data,
                        });
        } else {
            unreachable!();
        }

    }

    fn insert_subrange_bitmap(&mut self,
                              table: &String,
                              old_range: Range,
                              new_range: Range,
                              old_content: &Vec<u8>) {
        //TODO
    }

    fn add_trunkated_version_of_bitmap(&mut self,
                                       table: &String,
                                       old_range: Range,
                                       old_content: Vec<u8>,
                                       range_to_remove: Range) {
        let first_part = Range::new(old_range.min, range_to_remove.min);
        if first_part.min <= first_part.max {
            self.insert_subrange_bitmap(table, old_range, first_part, &old_content)
        }
        let last_part = Range::new(range_to_remove.max, old_range.max);
        if last_part.min <= last_part.max {
            self.insert_subrange_bitmap(table, old_range, last_part, &old_content)
        }
    }

    fn delete_bitmap(&mut self, table: &String, range_to_remove: Range, d: Content) {
        if let Content::BitMap { datasize: ds, data } = d {
            let mut bitmaps_to_delete = self.get_overlaping_bitmaps(table, range_to_remove, ds);
            self.delete_bitmaps_from_tree(table, &bitmaps_to_delete);
            for (rng, data) in bitmaps_to_delete {
                self.add_trunkated_version_of_bitmap(table, rng, data, range_to_remove)
            }
        } else {
            unreachable!();
        }
    }

    fn make_sure_table_exists(&mut self, table: &String) {
        if !self.map.contains_key(table) {
            let tree = IntervalTree::new();
            self.map.insert(table.clone(), tree);
        }
    }

    pub fn insert(&mut self, table: &String, r: Range, d: Content) {
        self.make_sure_table_exists(table);
        let mut tree = self.map.get_mut(table).unwrap();
        tree.insert(r, d);
    }

    pub fn add_table(&mut self, table: &String) {
        if !self.has_table(&table) {
            let tree = IntervalTree::new();
            self.map.insert(table.clone(), tree);
        }
    }

    pub fn has_table(&mut self, table: &String) -> bool {
        self.map.contains_key(table)
    }

    pub fn query(&self, table: &String, r: Range) -> Option<interval_tree::RangePairIter<Content>> {
        match self.map.get(table) {
            Some(tree) => Some(tree.range(r.min, r.max)),
            None => None,
        }
    }

    pub fn delete(&mut self, table: &String, r: Range) {
        if let Some(mut tree) = self.map.get_mut(table) {
            tree.delete(r)
        };
    }

    pub fn delete_all(&mut self, table: &String, r: Range) {
        let ranges = if let Some(iter) = self.query(table, r) {
            iter.map(|(range, _)| range.clone()).collect::<Vec<Range>>()
        } else {
            vec![]
        };
        for range in ranges {
            self.delete(table, range)
        }
    }

    pub fn to_flat(&self) -> HashMap<String, HashMap<(u64, u64), Content>> {
        let mut res = HashMap::new();
        for (tbl, tags) in self.map.iter() {
            let mut tagmap = HashMap::new();
            for (range, data) in tags.range(0, 0xff_ff_ff_ff__ff_ff_ff_ff) {
                let range_tup = (range.min, range.max);
                tagmap.insert(range_tup, data.clone());
            }
            res.insert(tbl.clone(), tagmap);
        }
        return res;
    }
}

#[test]
fn test_db() {
    let mut db = DB::new();
    let tbl = "foo".to_string();
    db.insert(&tbl,
              Range::new(3, 4),
              Content::Data("foo".to_string().into_bytes()));
    db.insert(&tbl,
              Range::new(4, 5),
              Content::Data("foo".to_string().into_bytes()));
    db.insert(&tbl,
              Range::new(5, 6),
              Content::Data("foo".to_string().into_bytes()));
    let mut is = db.query(&tbl, Range::new(4, 4))
                   .unwrap()
                   .map(|(r, _)| r.clone())
                   .collect::<Vec<Range>>();
    assert_eq!(is, vec![Range::new(3, 4), Range::new(4, 5)]);
    db.delete_all(&tbl, Range::new(3, 4));
    is = db.query(&tbl, Range::new(0, 100))
           .unwrap()
           .map(|(r, _)| r.clone())
           .collect::<Vec<Range>>();
    assert_eq!(is, vec![Range::new(5, 6)]);

    assert!(db.query(&"bar".to_string(), Range::new(0, 100)).is_none());
}
