use ::interval_tree::RangePairIter;
use ::interval_tree::Range;

use content::Bitmap;
use content::BitmapSlice;

pub struct BitmapSliceIter<'a> {
    orig: RangePairIter<'a, Bitmap>,
    orig_rng: Range,
}

impl<'a> BitmapSliceIter<'a> {
    pub fn new(orig: RangePairIter<Bitmap>, rng: Range) -> BitmapSliceIter {
        return BitmapSliceIter{orig: orig, orig_rng: rng};
    }
}

impl<'a> Iterator for BitmapSliceIter<'a> {

    type Item = (&'a Range,BitmapSlice<'a>);

    fn next(&mut self) -> Option<(&'a Range,BitmapSlice<'a>)> {
        if let Some(n) =  self.orig.get_next_node() {
            return Some(( &n.key, n.data.to_subslice(n.key, self.orig_rng)) )
        }
        return None
    }
}
