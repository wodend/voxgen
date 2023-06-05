use std::collections::HashMap;
use std::fs::write;
use std::io::Write;
use std::marker::PhantomData;
use std::ops::Range;
use std::path::Path;

/// A generic voxel buffer.
pub trait VoxelBuffer {
    type Voxel;

    /// Get the voxel buffer dimensions.
    ///
    /// Returns a tuple `(size_x, size_y, size_z)`.
    fn dimensions(&self) -> (u32, u32, u32);

    /// Get a reference to the voxel at location (`x`, `y`, `z`).
    ///
    /// # Panics
    ///
    /// Panics if (`x`, `y`, `z`) are outside the range of the volumetric image
    /// dimensions (`size_x`, `size_y`, `size_z`).
    fn voxel(&self, x: u32, y: u32, z: u32) -> &Self::Voxel;

    /// Get a mutable reference to the voxel at location (`x`, `y`, `z`).
    ///
    /// # Panics
    ///
    /// Panics if (`x`, `y`, `z`) are outside the range of the volumetric image
    /// dimensions (`size_x`, `size_y`, `size_z`).
    fn voxel_mut(&mut self, x: u32, y: u32, z: u32) -> &mut Self::Voxel;
}

/// A generic view of a voxel byte array.
pub trait Voxel {
    const SIZE: u8;

    /// Get a reference to the byte array of `self`.
    fn as_slice(&self) -> &[u8];

    /// Get a reference to a voxel view of `slice`.
    fn from_slice(slice: &[u8]) -> &Self;

    /// Get a mutable reference to a voxel view of `slice`.
    fn from_slice_mut(slice: &mut [u8]) -> &mut Self;
}

/// An RGBA voxel channel count.
pub const CHANNEL_COUNT_RGBA: usize = 4;

/// An RGBA voxel.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Rgba(pub [u8; CHANNEL_COUNT_RGBA]);

impl Voxel for Rgba {
    const SIZE: u8 = 4;

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

/// A generic array-based voxel buffer.
///
/// Array-based voxel buffers are dense. Every voxel in the image has data
/// stored for it, whether it is empty or not. This type of storage should be
/// used when fast updates are more important than memory usage. Coordinates use
/// MagicaVoxel conventions, where voxel position `(0, 0, 0)` is in the bottom
/// left corner closest to the camera. Increasing `x` moves to the right,
/// increasing `y` moves away from the camera, and increasing `z` moves up.
pub struct ArrayVoxelBuffer<T> {
    size_x: u32,
    size_y: u32,
    size_z: u32,
    data: Vec<u8>,
    _phantom: PhantomData<T>,
}
impl<T> ArrayVoxelBuffer<T>
where
    T: Voxel + Copy,
{
    /// Create a new empty generic array-based voxel buffer.
    ///
    /// The dimensions of the resulting voxel buffer are (`size_x`, `size_y`,
    /// `size_z`). The storage array will use `size_x * size_y * size_z *
    /// Voxel::SIZE` bytes of memory, so it cannot be used for large amounts of
    /// voxels.
    ///
    ///
    /// # Panics
    ///
    /// Panics when the storage array is larger than the maximum size of a
    /// vector.
    pub fn new(size_x: u32, size_y: u32, size_z: u32) -> ArrayVoxelBuffer<T> {
        match Self::len(size_x, size_y, size_z) {
            None => panic!("ArrayVoxelBuffer len overflows usize"),
            Some(len) => Self {
                size_x: size_x,
                size_y: size_y,
                size_z: size_z,
                data: vec![0; len],
                _phantom: PhantomData,
            },
        }
    }

    fn len(size_x: u32, size_y: u32, size_z: u32) -> Option<usize> {
        Some(<T>::SIZE as usize)
            .and_then(|size| size.checked_mul(size_x as usize))
            .and_then(|size| size.checked_mul(size_y as usize))
            .and_then(|size| size.checked_mul(size_z as usize))
    }

    #[inline(always)]
    fn voxel_indices(&self, x: u32, y: u32, z: u32) -> Option<Range<usize>> {
        if x >= self.size_x || y >= self.size_y || z >= self.size_z {
            None
        } else {
            Some(self.voxel_indices_unchecked(x, y, z))
        }
    }

    #[inline(always)]
    fn voxel_indices_unchecked(&self, x: u32, y: u32, z: u32) -> Range<usize> {
        let min_index_unsized = x as usize
            + (y as usize * self.size_x as usize)
            + (z as usize * self.size_x as usize * self.size_y as usize);
        let min_index = min_index_unsized * <T>::SIZE as usize;
        min_index..min_index + <T>::SIZE as usize
    }
}

impl<V> VoxelBuffer for ArrayVoxelBuffer<V>
where
    V: Voxel + Copy,
{
    type Voxel = V;

    fn dimensions(&self) -> (u32, u32, u32) {
        (self.size_x, self.size_y, self.size_z)
    }

    fn voxel(&self, x: u32, y: u32, z: u32) -> &V {
        match self.voxel_indices(x, y, z) {
            None => panic!(
                "ArrayVoxelBuffer index {:?} out of bounds {:?}",
                (x, y, z),
                (self.size_x, self.size_y, self.size_z)
            ),
            Some(indices) => <V>::from_slice(&self.data[indices]),
        }
    }

    fn voxel_mut(&mut self, x: u32, y: u32, z: u32) -> &mut V {
        match self.voxel_indices(x, y, z) {
            None => panic!(
                "ArrayVoxelBuffer index {:?} out of bounds {:?}",
                (x, y, z),
                (self.size_x, self.size_y, self.size_z)
            ),
            Some(indices) => <V>::from_slice_mut(&mut self.data[indices]),
        }
    }
}

/// An `ArrayVoxelBuffer` with RGBA voxels.
impl ArrayVoxelBuffer<Rgba> {
    /// Save the contents of `self` as a MagicaVoxel .vox file to `path`.
    ///
    /// MagicaVoxel does not support rendering the transparency channel of RGBA
    /// values. Set the transparency channel to 0 to remove it from the
    /// resulting MagicaVoxel .vox entirely.
    pub fn save<P>(&self, path: P) -> std::io::Result<()>
    where
        P: AsRef<Path>,
    {
        // Calculate vox data
        let mut color_indices = HashMap::new();
        let mut index = 1;
        let mut xyzis = Vec::new();
        let (size_x, size_y, size_z) = self.dimensions();
        for z in 0..size_z {
            for y in 0..size_y {
                for x in 0..size_x {
                    let mut xyzi = [0; 4];
                    xyzi[0] = x as u8;
                    xyzi[1] = y as u8;
                    xyzi[2] = z as u8;
                    let rgba = self.voxel(x, y, z);
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
        bytes.write(&u32::to_le_bytes(size_x))?;
        bytes.write(&u32::to_le_bytes(size_y))?;
        bytes.write(&u32::to_le_bytes(size_z))?;

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
