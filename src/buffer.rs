use std::collections::HashMap;
use std::fs::write;
use std::io::Write;
use std::marker::PhantomData;
use std::ops::Range;
use std::path::Path;
use std::slice::{ChunksExact, ChunksExactMut};

/// A generalized voxel.
///
/// Just needs to be able to convert to and from slice. Voxel data is stored in
/// the buffer densely as a byte stream.
pub trait Voxel {
    const SIZE: u8;

    fn as_slice(&self) -> &[u8];
    fn from_slice(slice: &[u8]) -> &Self;
    fn from_slice_mut(slice: &mut [u8]) -> &mut Self;
}

const CHANNEL_COUNT_RGBA: usize = 4;

/// A simple RGBA voxel.
///
/// This is the primary voxel type used for encoding to MagicaVoxel format.
/// MagicaVoxel does not support densely packed voxel arrays or RGBA voxels with
/// a transparency channel. To prevent a voxel from appearing in the output, set
/// the transparency channel to 0.
/// 
/// # Examples
/// 
/// ```
/// use voxgen::buffer::{Rgba, Voxel};
/// 
/// let red_raw = [255, 0, 0, 255];
/// let red_rgba = Rgba(red_raw);
/// 
/// /// Convert to and from slice.
/// assert_eq!(red_rgba.as_slice(), red_raw);
/// assert_eq!(*<Rgba>::from_slice(&red_raw), red_rgba);
/// 
/// /// Mutating underlying bytes changes the value.
/// let mut red_raw_copy = [0u8; 4];
/// red_raw_copy.copy_from_slice(&red_raw);
/// let voxel = <Rgba>::from_slice_mut(&mut red_raw_copy);
/// voxel.0[1] = 255;
/// assert_eq!(*voxel, Rgba([255, 255, 0, 255]));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Rgba(pub [u8; CHANNEL_COUNT_RGBA]);

impl Voxel for Rgba {
    const SIZE: u8 = CHANNEL_COUNT_RGBA as u8;

    #[inline(always)]
    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn from_slice(slice: &[u8]) -> &Rgba {
        assert_eq!(slice.len(), Self::SIZE as usize);
        unsafe { &*(slice.as_ptr() as *const Rgba) }
    }

    fn from_slice_mut(slice: &mut [u8]) -> &mut Rgba {
        assert_eq!(slice.len(), Self::SIZE as usize);
        unsafe { &mut *(slice.as_mut_ptr() as *mut Rgba) }
    }
}

/// Generic volumetric image buffer.
pub struct VolumeBuffer<T> {
    width: u32,
    depth: u32,
    height: u32,
    data: Vec<u8>,
    _phantom: PhantomData<T>,
}
impl<T> VolumeBuffer<T>
where
    T: Voxel + Copy,
{
    pub fn new(width: u32, depth: u32, height: u32) -> VolumeBuffer<T> {
        match Self::len(width, depth, height) {
            None => panic!("VolumeBuffer len overflows usize"),
            Some(len) => VolumeBuffer {
                width: width,
                depth: depth,
                height: height,
                data: vec![0; len],
                _phantom: PhantomData,
            },
        }
    }

    pub fn len(width: u32, depth: u32, height: u32) -> Option<usize> {
        Some(<T>::SIZE as usize)
            .and_then(|size| size.checked_mul(width as usize))
            .and_then(|size| size.checked_mul(depth as usize))
            .and_then(|size| size.checked_mul(height as usize))
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn get(&self, x: u32, y: u32, z: u32) -> &T {
        match self.indices(x, y, z) {
            None => panic!(
                "VolumeBuffer index {:?} out of bounds {:?}",
                (x, y, z),
                (self.width, self.depth, self.height)
            ),
            Some(indices) => <T>::from_slice(&self.data[indices]),
        }
    }

    pub fn get_mut(&mut self, x: u32, y: u32, z: u32) -> &mut T {
        match self.indices(x, y, z) {
            None => panic!(
                "VolumeBuffer index {:?} out of bounds {:?}",
                (x, y, z),
                (self.width, self.depth, self.height)
            ),
            Some(indices) => <T>::from_slice_mut(&mut self.data[indices]),
        }
    }

    pub fn get_id(&self, id: usize) -> &T {
        match self.indices_id(id) {
            None => panic!(
                "VolumeBuffer id {:?} out of bounds {:?}",
                id,
                self.width * self.depth * self.height
            ),
            Some(indices) => <T>::from_slice(&self.data[indices]),
        }
    }

    pub fn get_id_mut(&mut self, id: usize) -> &mut T {
        match self.indices_id(id) {
            None => panic!(
                "VolumeBuffer id {:?} out of bounds {:?}",
                id,
                self.width * self.depth * self.height
            ),
            Some(indices) => <T>::from_slice_mut(&mut self.data[indices]),
        }
    }

    #[inline(always)]
    pub fn id(&self, x: u32, y: u32, z: u32) -> Option<usize> {
        if x >= self.width || y >= self.depth || z >= self.height {
            return None;
        }
        Some(self.id_unchecked(x, y, z))
    }

    #[inline(always)]
    fn id_unchecked(&self, x: u32, y: u32, z: u32) -> usize {
        x as usize
            + (y as usize * self.width as usize)
            + (z as usize * self.width as usize * self.depth as usize)
    }

    #[inline(always)]
    pub fn coordinate(&self, id: usize) -> Option<(u32, u32, u32)> {
        if id >= self.width as usize * self.depth as usize * self.height as usize {
            return None;
        }
        Some(self.coordinate_unchecked(id))
    }

    #[inline(always)]
    fn coordinate_unchecked(&self, id: usize) -> (u32, u32, u32) {
        let z = id / (self.width as usize * self.depth as usize);
        let plane = id - (z * self.width as usize * self.depth as usize);
        let y = plane / self.width as usize;
        let x = plane % self.width as usize;
        (x as u32, y as u32, z as u32)
    }

    #[inline(always)]
    fn indices(&self, x: u32, y: u32, z: u32) -> Option<Range<usize>> {
        let id = self.id(x, y, z);
        match id {
            None => None,
            Some(id) => Some(self.indices_unchecked(id)),
        }
    }

    #[inline(always)]
    fn indices_id(&self, id: usize) -> Option<Range<usize>> {
        if id >= (self.width * self.depth * self.height) as usize {
            return None;
        }
        Some(self.indices_unchecked(id))
    }

    #[inline(always)]
    fn indices_unchecked(&self, unsized_index: usize) -> Range<usize> {
        let min_index = unsized_index * <T>::SIZE as usize;
        min_index..min_index + <T>::SIZE as usize
    }

    pub fn enumerate_voxels(&self) -> EnumerateVoxels<T> {
        EnumerateVoxels {
            chunks: self.data.chunks_exact(<T>::SIZE as usize),
            x: 0,
            y: 0,
            z: 0,
            width: self.width,
            depth: self.depth,
            _phantom: PhantomData,
        }
    }

    pub fn enumerate_voxels_mut(&mut self) -> EnumerateVoxelsMut<T> {
        EnumerateVoxelsMut {
            chunks: self.data.chunks_exact_mut(<T>::SIZE as usize),
            x: 0,
            y: 0,
            z: 0,
            width: self.width,
            depth: self.depth,
            _phantom: PhantomData,
        }
    }
}

/// Rgba volumetric image buffer.
///
/// Used to save a MagicaVoxel .vox file.
///
/// # Examples
///
/// Generate a simple 2D red cross.
/// ```
/// use voxgen::buffer::{VolumeBuffer, Rgba};
///
/// let mut vol = VolumeBuffer::new(32, 32, 32);
///
/// for x in 15..=17 {
///     for y in 8..24 {
///         *vol.get_mut(x, y, 0) = Rgba([255, 0, 0, 255]);
///         *vol.get_mut(y, x, 0) = Rgba([255, 0, 0, 255]);
///     }
/// }
///
/// vol.save("test/volumes/red_cross.vox")?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
impl VolumeBuffer<Rgba> {
    pub fn save<P>(&self, path: P) -> std::io::Result<()>
    where
        P: AsRef<Path>,
    {
        // Calculate vox data
        let mut color_indices = HashMap::new();
        let mut index = 1;
        let mut xyzis = Vec::new();
        for (x, y, z, rgba) in self.enumerate_voxels() {
            let mut xyzi = [0; 4];
            xyzi[0] = x as u8;
            xyzi[1] = y as u8;
            xyzi[2] = z as u8;
            match color_indices.get(rgba) {
                None => {
                    color_indices.insert(rgba, index);
                    xyzi[3] = index;
                    index += 1;
                }
                Some(i) => {
                    xyzi[3] = *i as u8;
                }
            }
            if rgba.0[3] > 0 {
                xyzis.push(xyzi);
            }
        }
        // Vox spec: https://github.com/ephtracy/voxel-model/blob/master/MagicaVoxel-file-format-vox.txt
        let mut bytes = Vec::new();
        bytes.write(b"VOX ")?;
        bytes.write(&u32::to_le_bytes(150))?;

        const INT_SIZE: u32 = 4;
        const ZERO: [u8; 4] = [0; 4];
        let size_chunk_size = INT_SIZE * 3;
        // TODO: Handle cases where voxel count exeeds u32 bounds
        let voxel_count = xyzis.len() as u32;
        let xyzi_chunk_size = INT_SIZE + (voxel_count * INT_SIZE);
        const PALETTE_COUNT: u32 = 256;
        let rgba_chunk_size = PALETTE_COUNT * INT_SIZE;
        let chunk_header_size = INT_SIZE * 3;
        let chunk_count = 3;
        let main_child_chunks_size =
            (chunk_header_size * chunk_count) + size_chunk_size + xyzi_chunk_size + rgba_chunk_size;
        bytes.write(b"MAIN")?;
        bytes.write(&ZERO)?; // MAIN has no content
        bytes.write(&u32::to_le_bytes(main_child_chunks_size))?;

        bytes.write(b"SIZE")?;
        bytes.write(&u32::to_le_bytes(size_chunk_size))?;
        bytes.write(&ZERO)?; // SIZE has no children
        bytes.write(&u32::to_le_bytes(self.width()))?;
        bytes.write(&u32::to_le_bytes(self.depth()))?;
        bytes.write(&u32::to_le_bytes(self.height()))?;

        bytes.write(b"XYZI")?;
        bytes.write(&u32::to_le_bytes(xyzi_chunk_size))?;
        bytes.write(&ZERO)?; // XYZI has no children
        bytes.write(&u32::to_le_bytes(voxel_count))?;
        // TODO: Handle cases where xyzi exceeds u8 bounds
        for xyzi in &xyzis {
            bytes.write(xyzi)?;
        }

        bytes.write(b"RGBA")?;
        bytes.write(&u32::to_le_bytes(rgba_chunk_size))?;
        bytes.write(&ZERO)?; // RGBA has no children
        let mut palette = [[0; 4]; PALETTE_COUNT as usize];
        for (rgba, i) in color_indices {
            palette[i as usize - 1] = rgba.0;
        }
        bytes.write(&palette.concat())?;
        write(path, &bytes)?;
        Ok(())
    }
}

pub struct EnumerateVoxels<'a, T> {
    chunks: ChunksExact<'a, u8>,
    x: u32,
    y: u32,
    z: u32,
    width: u32,
    depth: u32,
    _phantom: PhantomData<T>,
}

impl<'a, T> Iterator for EnumerateVoxels<'a, T>
where
    T: Voxel + 'a,
{
    type Item = (u32, u32, u32, &'a T);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
        }
        if self.y >= self.depth {
            self.y = 0;
            self.z += 1;
        }
        let (x, y, z) = (self.x, self.y, self.z);
        self.x += 1;
        self.chunks.next().map(|t| (x, y, z, <T>::from_slice(t)))
    }
}

pub struct EnumerateVoxelsMut<'a, T> {
    chunks: ChunksExactMut<'a, u8>,
    x: u32,
    y: u32,
    z: u32,
    width: u32,
    depth: u32,
    _phantom: PhantomData<T>,
}

impl<'a, T> Iterator for EnumerateVoxelsMut<'a, T>
where
    T: Voxel + 'a,
{
    type Item = (u32, u32, u32, &'a mut T);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
        }
        if self.y >= self.depth {
            self.y = 0;
            self.z += 1;
        }
        let (x, y, z) = (self.x, self.y, self.z);
        self.x += 1;
        self.chunks
            .next()
            .map(|t| (x, y, z, <T>::from_slice_mut(t)))
    }
}