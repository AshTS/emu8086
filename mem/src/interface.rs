#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BusDeviceError {
    AddressOutOfBounds{address: usize, size: usize},
    AddressNotWritable{address: usize},
    AddressNotMapped{address: usize}
}

pub trait BusDevice {
    /// Reads the byte at the given `address`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the byte cannot be read.
    fn read(&self, address: usize) -> Result<u8, BusDeviceError>;

    
    /// Writes `data` to the byte at the given `address`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the byte cannot be written.
    fn write(&mut self, address: usize, data: u8) -> Result<(), BusDeviceError>;
}

pub trait RegionBusDevice : BusDevice {
    /// Reads a region of memory with the given starting `address`.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the required bytes cannot be read.
    fn read_region<const SIZE: usize>(&self, address: usize) -> Result<[u8; SIZE], BusDeviceError> {
        let mut result = [0; SIZE];

        for (i, slot) in result.iter_mut().enumerate() {
            *slot = self.read(address + i)?;
        }

        Ok(result)
    }

    /// Writes to a region of memory with the given starting `address`.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the required bytes cannot be written to.
    fn write_region(&mut self, address: usize, data: &[u8]) -> Result<(), BusDeviceError> {
        for (i, byte) in data.iter().enumerate() {
            self.write(address + i, *byte)?;
        }

        Ok(())
    }
}

impl<T: BusDevice> RegionBusDevice for T {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Memory<const SIZE: usize> ([u8; SIZE]);

impl<const SIZE: usize> Memory<SIZE> {
    #[must_use]
    /// Constructs a new, zeroed memory region.
    pub const fn empty() -> Self {
        Self ([0; SIZE])
    }

    #[must_use]
    /// Constructs a new memory region populated with the given data.
    pub const fn filled(data: [u8; SIZE]) -> Self {
        Self (data)
    }

    #[must_use]
    /// Constructs a new memory region populated with the given data, padded with zeros.
    ///
    /// # Panics
    /// This function will panic if the `data` provided has a length greater than the `SIZE` of the memory region.
    pub fn populated(data: &[u8]) -> Self {
        assert!(data.len() <= SIZE);
        let mut inner = [0; SIZE];
        inner[0..data.len()].copy_from_slice(data);

        Self(inner)
    }
}

impl<const SIZE: usize> BusDevice for Memory<SIZE> {

    fn read(&self, address: usize) -> Result<u8, BusDeviceError> {
        self.0.get(address).copied().ok_or(BusDeviceError::AddressOutOfBounds { address, size: self.0.len() })
    }

    fn write(&mut self, address: usize, data: u8) -> Result<(), BusDeviceError> {
        let size = self.0.len();
        *(self.0.get_mut(address).ok_or(BusDeviceError::AddressOutOfBounds { address, size })?) = data;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReadOnlyMemory<const SIZE: usize> ([u8; SIZE]);

impl<const SIZE: usize> ReadOnlyMemory<SIZE> {
    #[must_use]
    /// Constructs a new, zeroed read only memory region.
    pub const fn empty() -> Self {
        Self ([0; SIZE])
    }

    #[must_use]
    /// Constructs a new read only memory region populated with the given data.
    pub const fn filled(data: [u8; SIZE]) -> Self {
        Self (data)
    }

    #[must_use]
    /// Constructs a new read only memory region populated with the given data, padded with zeros.
    ///
    /// # Panics
    /// This function will panic if the `data` provided has a length greater than the `SIZE` of the memory region.
    pub fn populated(data: &[u8]) -> Self {
        assert!(data.len() <= SIZE);
        let mut inner = [0; SIZE];
        inner[0..data.len()].copy_from_slice(data);

        Self(inner)
    }
}

impl<const SIZE: usize> BusDevice for ReadOnlyMemory<SIZE> {

    fn read(&self, address: usize) -> Result<u8, BusDeviceError> {
        self.0.get(address).copied().ok_or(BusDeviceError::AddressOutOfBounds { address, size: self.0.len() })
    }

    fn write(&mut self, address: usize, _data: u8) -> Result<(), BusDeviceError> {
        Err(BusDeviceError::AddressNotWritable { address })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn test_memory_creation_size<const SIZE: usize>() {
        let empty = Memory::empty();
        assert_eq!(empty.0, [0; SIZE]);

        let filled = Memory::filled([42; SIZE]);
        assert_eq!(filled.0, [42; SIZE]);
    }

    fn test_read_only_memory_creation_size<const SIZE: usize>() {
        let empty = Memory::empty();
        assert_eq!(empty.0, [0; SIZE]);

        let filled = Memory::filled([42; SIZE]);
        assert_eq!(filled.0, [42; SIZE]);
    }

    #[test]
    fn test_memory_creation() {
        test_memory_creation_size::<0>();
        test_memory_creation_size::<1>();
        test_memory_creation_size::<2>();
        test_memory_creation_size::<4>();
        test_memory_creation_size::<16>();
        test_memory_creation_size::<256>();
        test_memory_creation_size::<1024>();
    }

    #[test]
    fn test_read_only_memory_creation() {
        test_read_only_memory_creation_size::<0>();
        test_read_only_memory_creation_size::<1>();
        test_read_only_memory_creation_size::<2>();
        test_read_only_memory_creation_size::<4>();
        test_read_only_memory_creation_size::<16>();
        test_read_only_memory_creation_size::<256>();
        test_read_only_memory_creation_size::<1024>();
    }

    #[test]
    fn test_memory_populate() {
        let mem: Memory<8> = Memory::populated(&[0, 1, 2, 3]);

        assert_eq!(mem.0, [0, 1, 2, 3, 0, 0, 0, 0]);

        let mem: Memory<4> = Memory::populated(&[0, 1, 2, 3]);

        assert_eq!(mem.0, [0, 1, 2, 3]);
    }

    #[test]
    fn test_read_only_memory_populate() {
        let mem: ReadOnlyMemory<8> = ReadOnlyMemory::populated(&[0, 1, 2, 3]);

        assert_eq!(mem.0, [0, 1, 2, 3, 0, 0, 0, 0]);

        let mem: ReadOnlyMemory<4> = ReadOnlyMemory::populated(&[0, 1, 2, 3]);

        assert_eq!(mem.0, [0, 1, 2, 3]);
    }

    #[test]
    #[should_panic]
    fn test_memory_populate_panic() {
        let _mem: Memory<2> = Memory::populated(&[0, 1, 2, 3]);
    }

    #[test]
    #[should_panic]
    fn test_read_only_memory_populate_panic() {
        let _mem: ReadOnlyMemory<2> = ReadOnlyMemory::populated(&[0, 1, 2, 3]);
    }

    #[test]
    fn test_memory_single_byte_read() {
        let empty = Memory::<0>::empty();
        
        for i in &[0, 16, 64, 1024, 1, 2, 3, 4, 5, 256, 257, 258] {
            assert_eq!(empty.read(*i), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 0 }));
        }

        let data: [u8; 512] = core::array::from_fn(|index| (index % 256) as u8);
        let mem = Memory::filled(data);

        for i in &[0, 16, 64, 512, 1024, 1, 2, 3, 4, 5, 256, 257, 258] {
            if *i < data.len() {
                assert_eq!(mem.read(*i), Ok((*i % 256) as u8));    
            }
            else {
                assert_eq!(mem.read(*i), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 512 }));
            }
        }
    }

    #[test]
    fn test_read_only_memory_single_byte_read() {
        let empty = ReadOnlyMemory::<0>::empty();
        
        for i in &[0, 16, 64, 1024, 1, 2, 3, 4, 5, 256, 257, 258] {
            assert_eq!(empty.read(*i), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 0 }));
        }

        let data: [u8; 512] = core::array::from_fn(|index| (index % 256) as u8);
        let mem = ReadOnlyMemory::filled(data);

        for i in &[0, 16, 64, 512, 1024, 1, 2, 3, 4, 5, 256, 257, 258] {
            if *i < data.len() {
                assert_eq!(mem.read(*i), Ok((*i % 256) as u8));    
            }
            else {
                assert_eq!(mem.read(*i), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 512 }));
            }
        }
    }

    #[test]
    fn test_memory_single_byte_write() {
        let mut empty = Memory::<0>::empty();

        for i in &[0, 16, 64, 1024, 1, 2, 3, 4, 5, 256, 257, 258] {
            assert_eq!(empty.write(*i, 0), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 0 }));
        }

        let data: [u8; 512] = core::array::from_fn(|index| (index % 256) as u8);
        let mut mem = Memory::filled(data);

        let indexes = &[0, 16, 64, 512, 1024, 1, 2, 3, 4, 5, 256, 257, 258];

        for i in &[0, 16, 64, 512, 1024, 1, 2, 3, 4, 5, 256, 257, 258] {
            if *i < data.len() {
                assert_eq!(mem.write(*i, 255 - ((*i % 256) as u8)), Ok(()));    
            }
            else {
                assert_eq!(mem.write(*i, 255 - ((*i % 256) as u8)), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 512 }));
            }
        }

        for index in 0..512 {
            if indexes.contains(&index) {
                assert_eq!(mem.0[index], 255 - ((index % 256) as u8));
            }
            else {
                assert_eq!(mem.0[index], (index % 256) as u8);
            }
        }
    }

    #[test]
    fn test_read_only_memory_single_byte_write() {
        let mut empty = ReadOnlyMemory::<0>::empty();

        for i in &[0, 16, 64, 1024, 1, 2, 3, 4, 5, 256, 257, 258] {
            assert_eq!(empty.write(*i, 0), Err(BusDeviceError::AddressNotWritable { address: *i }));
        }

        let data: [u8; 512] = core::array::from_fn(|index| (index % 256) as u8);
        let mut mem = ReadOnlyMemory::filled(data);

        for i in &[0, 16, 64, 512, 1024, 1, 2, 3, 4, 5, 256, 257, 258] {
            assert_eq!(mem.write(*i, 255 - ((*i % 256) as u8)), Err(BusDeviceError::AddressNotWritable { address: *i }));    
        }
    }

    #[test]
    fn test_memory_region_read_interaction() {
        let empty = Memory::<0>::empty();

        let populated = Memory::<256>::populated(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

        for i in &[0, 16, 64, 1024, 1, 2, 3, 4, 5, 256, 257, 258] {
            assert_eq!(empty.read_region::<0>(*i), Ok([]));
            assert_eq!(empty.read_region::<1>(*i), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 0 }));
            assert_eq!(empty.read_region::<2>(*i), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 0 }));
            assert_eq!(empty.read_region::<4>(*i), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 0 }));
        
            assert_eq!(populated.read_region::<1>(*i).map(|v| v[0]), populated.read(*i));
        }

        assert_eq!(populated.read_region(0), Ok([0, 1, 2, 3]));
        assert_eq!(populated.read_region(0), Ok([0, 1, 2, 3, 4, 5, 6]));
        assert_eq!(populated.read_region(3), Ok([3, 4, 5, 6]));
        assert_eq!(populated.read_region::<4>(255), Err(BusDeviceError::AddressOutOfBounds { address: 256, size: 256 }));
        assert_eq!(populated.read_region::<4>(253), Err(BusDeviceError::AddressOutOfBounds { address: 256, size: 256 }));
        assert_eq!(populated.read_region::<4>(512), Err(BusDeviceError::AddressOutOfBounds { address: 512, size: 256 }));
    }

    #[test]
    fn test_read_only_memory_region_read_interaction() {
        let empty = ReadOnlyMemory::<0>::empty();

        let populated = ReadOnlyMemory::<256>::populated(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

        for i in &[0, 16, 64, 1024, 1, 2, 3, 4, 5, 256, 257, 258] {
            assert_eq!(empty.read_region::<0>(*i), Ok([]));
            assert_eq!(empty.read_region::<1>(*i), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 0 }));
            assert_eq!(empty.read_region::<2>(*i), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 0 }));
            assert_eq!(empty.read_region::<4>(*i), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 0 }));
        
            assert_eq!(populated.read_region::<1>(*i).map(|v| v[0]), populated.read(*i));
        }

        assert_eq!(populated.read_region(0), Ok([0, 1, 2, 3]));
        assert_eq!(populated.read_region(0), Ok([0, 1, 2, 3, 4, 5, 6]));
        assert_eq!(populated.read_region(3), Ok([3, 4, 5, 6]));
        assert_eq!(populated.read_region::<4>(255), Err(BusDeviceError::AddressOutOfBounds { address: 256, size: 256 }));
        assert_eq!(populated.read_region::<4>(253), Err(BusDeviceError::AddressOutOfBounds { address: 256, size: 256 }));
        assert_eq!(populated.read_region::<4>(512), Err(BusDeviceError::AddressOutOfBounds { address: 512, size: 256 }));
    }

    #[test]
    fn test_memory_region_write_interaction() {
        let mut empty = Memory::<0>::empty();

        let mut populated = Memory::<8>::empty();

        for i in &[0, 16, 64, 1024, 1, 2, 3, 4, 5, 256, 257, 258] {
            assert_eq!(empty.write_region(*i, &[]), Ok(()));
            assert_eq!(empty.write_region(*i, &[0]), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 0 }));
            assert_eq!(empty.write_region(*i, &[42, 43]), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 0 }));
            assert_eq!(empty.write_region(*i, &[52, 1, 0]), Err(BusDeviceError::AddressOutOfBounds { address: *i, size: 0 }));
        }

        assert_eq!(populated.write_region(0, &[0, 1, 2, 3, 4, 5, 6, 7]), Ok(()));
        assert_eq!(populated.0, [0, 1, 2, 3, 4, 5, 6, 7]);
        
        assert_eq!(populated.write_region(5, &[42, 43]), Ok(()));
        assert_eq!(populated.0, [0, 1, 2, 3, 4, 42, 43, 7]);
        
        assert_eq!(populated.write_region(5, &[42, 43, 45, 46]), Err(BusDeviceError::AddressOutOfBounds { address: 8, size: 8 }));
        assert_eq!(populated.0, [0, 1, 2, 3, 4, 42, 43, 45]);
        
        assert_eq!(populated.write_region(512, &[42, 43, 45, 46]), Err(BusDeviceError::AddressOutOfBounds { address: 512, size: 8 }));
        assert_eq!(populated.0, [0, 1, 2, 3, 4, 42, 43, 45]);
    }

    #[test]
    fn test_read_only_memory_region_write_interaction() {
        let mut empty = ReadOnlyMemory::<0>::empty();

        let mut populated = ReadOnlyMemory::<8>::empty();

        for i in &[0, 16, 64, 1024, 1, 2, 3, 4, 5, 256, 257, 258] {
            assert_eq!(empty.write_region(*i, &[]), Ok(()));
            assert_eq!(empty.write_region(*i, &[0]), Err(BusDeviceError::AddressNotWritable { address: *i }));
            assert_eq!(empty.write_region(*i, &[42, 43]), Err(BusDeviceError::AddressNotWritable { address: *i }));
            assert_eq!(empty.write_region(*i, &[52, 1, 0]), Err(BusDeviceError::AddressNotWritable { address: *i }));
        }

        assert_eq!(populated.write_region(0, &[0, 1, 2, 3, 4, 5, 6, 7]), Err(BusDeviceError::AddressNotWritable { address: 0 }));
        
        assert_eq!(populated.write_region(5, &[42, 43]), Err(BusDeviceError::AddressNotWritable { address: 5 }));
        
        assert_eq!(populated.write_region(5, &[42, 43, 45, 46]), Err(BusDeviceError::AddressNotWritable { address: 5 }));

        assert_eq!(populated.write_region(512, &[42, 43, 45, 46]), Err(BusDeviceError::AddressNotWritable { address: 512 }));
    }
}