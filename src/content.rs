use ::memrange::Range;
use std::borrow::Cow;

#[derive(Clone, PartialEq, Debug, RustcEncodable, RustcDecodable)]
pub struct Bitmap {
    pub entry_size: u64,
    pub data: Vec<u8>,
}

pub struct BitmapSlice<'a>{
    pub entry_size: u64,
    pub data: Cow<'a,[u8]>,
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct Object {
    pub data: Vec<u8>
}

impl<'a> BitmapSlice<'a> {
    pub fn to_bitmap(&self) -> Bitmap{
        return Bitmap{entry_size: self.entry_size, data: (*self.data).into()}
    }

    pub fn new_from_owned<'db>(b: Bitmap) -> BitmapSlice<'db>{
        return BitmapSlice{entry_size: b.entry_size, data: Cow::Owned(b.data.into())}
    }

    pub fn new_from_borrowed<'db>(b: &'db Bitmap) -> BitmapSlice<'db>{
        return BitmapSlice{entry_size: b.entry_size, data: Cow::Borrowed(&b.data)}
    }
}

impl Object {
    pub fn new(data: Vec<u8>) -> Object {
        return Object{data: data}
    }

}

impl Bitmap {

    pub fn new(es: u64, data: Vec<u8>) -> Bitmap {
        return Bitmap{entry_size: es, data: data}
    }

    pub fn to_subslice(&self, data_range: Range,  mut restriction_range: Range) -> BitmapSlice{

        restriction_range = restriction_range.get_intersection(&data_range);
        let num_entries = restriction_range.len();
        let startoffset = (restriction_range.min - data_range.min)*self.entry_size;
        let endoffset = startoffset+num_entries*self.entry_size;
        assert!( startoffset <= self.data.len() as u64 );
        assert!( endoffset <= self.data.len() as u64 );
        let slice = &self.data[startoffset as usize .. endoffset as usize ];
        return BitmapSlice{entry_size: self.entry_size, data: Cow::Borrowed(slice)}
    }

    pub fn to_subbitmap(&self, data_range: Range, restriction_range: Range) -> Bitmap{
        let BitmapSlice{data: slice, ..} = self.to_subslice(data_range, restriction_range);
        return Bitmap{entry_size: self.entry_size, data: slice.to_vec()}
    }

    fn copy_to_buffer(&self, offset: u64, buffer: &mut Vec<u8>) {
        for (i, val) in self.data.iter().enumerate() {
            buffer[i + offset as usize] = *val;
        }
    }

    pub fn merge_bitmaps(self, data_range: Range, merge_partners: Vec<(Range, Bitmap)>) -> (Range, Bitmap) {

        let mut new_range = data_range.clone();
        for &(rng, ref map) in &merge_partners {
            assert_eq!(map.entry_size, self.entry_size);
            new_range = new_range.get_union(&rng)
        }

        if new_range == data_range {
            return (data_range, self);
        }

        let combined_len = ( new_range.len()*self.entry_size ) as usize;
        let mut combined = Vec::with_capacity(combined_len);
        combined.resize(combined_len, 0);
        for &(ref rng, ref cont) in &merge_partners {
            let offset = (rng.min - new_range.min) * self.entry_size;
            cont.copy_to_buffer(offset, &mut combined);
        }
        self.copy_to_buffer( (data_range.min - new_range.min)*self.entry_size, &mut combined );

        return (new_range, Bitmap::new(self.entry_size, combined));
    }
}
