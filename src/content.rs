use std::cmp;
use ::interval_tree::Range;

#[derive(Clone, PartialEq, Debug)]
pub struct Bitmap {
    pub entry_size: u64,
    pub data: Vec<u8>,
}

pub struct BitmapSlice<'a>{
    pub entry_size: u64,
    pub data: &'a [u8],
}

pub struct Object {
    pub data: Vec<u8>
}

impl<'a> BitmapSlice<'a> {
    pub fn to_bitmap(&self) -> Bitmap{
        return Bitmap{entry_size: self.entry_size, data: self.data.into()}
    }
}

impl Bitmap {

    pub fn new(es: u64, data: Vec<u8>) -> Bitmap {
        return Bitmap{entry_size: es, data: data}
    }

    pub fn to_subslice(&self, data_range: Range,  restriction_range: Range) -> BitmapSlice{
        assert!(restriction_range.max <= data_range.max );
        assert!(restriction_range.min >= data_range.min );
        let startoffset = (restriction_range.min - data_range.min)*self.entry_size;
        let endoffset = startoffset+(restriction_range.max - restriction_range.min)*self.entry_size;
        assert!(startoffset > 0 && startoffset < self.data.len() as u64 );
        assert!(endoffset > 0 && endoffset < self.data.len() as u64 );
        return BitmapSlice{entry_size: self.entry_size, data: &self.data[startoffset as usize ..endoffset as usize ]}
    }

    pub fn to_subbitmap(&self, data_range: Range, restriction_range: Range) -> Bitmap{
        let BitmapSlice{data: slice, ..} = self.to_subslice(data_range, restriction_range);
        return Bitmap{entry_size: self.entry_size, data: slice.to_vec()}
    }

    pub fn merge_bitmaps(&self, data_range: Range, merge_partners: Vec<(Range, Bitmap)>) -> (Range, Bitmap) {
        let mut min = data_range.max;
        let mut max = data_range.min;
        for &(rng, ref map) in &merge_partners {
            assert_eq!(map.entry_size, self.entry_size);
            min = cmp::min(min, rng.min);
            max = cmp::max(max, rng.max);
        }

        assert!(max - min > 0);

        let combined_len = ( (max - min)*self.entry_size ) as usize;
        let mut combined = Vec::with_capacity(combined_len);
        combined.resize(combined_len, 0);
        for &(ref rng, ref cont) in &merge_partners {
            let offset = (rng.min - min) * self.entry_size;
            for (i, val) in cont.data.iter().enumerate() {
                combined[i + offset as usize] = *val;
            }
        }
        return (Range::new(min, max), Bitmap::new(self.entry_size, combined));
    }
}
