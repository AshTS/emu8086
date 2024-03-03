use std::ops::RangeInclusive;

use crate::BusDeviceError;

use super::interface::BusDevice;

pub struct MemoryMap {
    // TODO: So this should be replaced with a different data structure that ensures two ranges can't overlap and can search for ranges using binary search.
    entries: Vec<(RangeInclusive<usize>, Box<dyn BusDevice>)>
}

impl MemoryMap {
    /// Construct a new, empty `MemoryMap`
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new()
        }
    }

    /// Builder pattern for adding a `range` mapped to a `bus_device` to the `MemoryMap`.
    ///
    /// # Panics
    ///
    /// Panics if `range` is already mapped.
    #[must_use]
    pub fn with_range(mut self, range: RangeInclusive<usize>, bus_device: Box<dyn BusDevice>) -> Self {
        self.add_range(range, bus_device);
        self
    }

    /// Adds a `range` mapped to a `bus_device` to the `MemoryMap`.
    ///
    /// # Panics
    ///
    /// Panics if `range` is already mapped.
    pub fn add_range(&mut self, range: RangeInclusive<usize>, bus_device: Box<dyn BusDevice>) {
        // Make sure that the range doesn't overlap another range
        for (r, _) in &self.entries {
            assert!(!(r.contains(range.start()) || r.contains(range.end())), "Memory Range {range:#x?} overlaps already mapped {r:#x?}");
        }

        // Add the mapping
        self.entries.push((range, bus_device));
    }

    /// Get a reference to the `dyn BusDevice` mapped to the given address
    #[must_use]
    pub fn mapping(&self, address: usize) -> Option<(&RangeInclusive<usize>, &dyn BusDevice)> {
        for (range, device) in &self.entries {
            if range.contains(&address) {
                return Some((range, device.as_ref()));
            }
        }

        None
    }

    /// Get a mutable reference to the `dyn BusDevice` mapped to the given address
    #[must_use]
    pub fn mut_mapping(&mut self, address: usize) -> Option<(&mut RangeInclusive<usize>, &mut dyn BusDevice)> {
        for (range, device) in &mut self.entries {
            if range.contains(&address) {
                return Some((range, device.as_mut()));
            }
        }

        None
    }
}

impl Default for MemoryMap {
    fn default() -> Self {
        Self::new()
    }
}

impl BusDevice for MemoryMap {
    fn read(&self, address: usize) -> Result<u8, crate::BusDeviceError> {
        self.mapping(address)
        .ok_or(BusDeviceError::AddressNotMapped { address })
        .map(|(range, mapped_device)| 
            mapped_device.read(address - range.start()))?
    }

    fn write(&mut self, address: usize, data: u8) -> Result<(), crate::BusDeviceError> {
        self.mut_mapping(address)
        .ok_or(BusDeviceError::AddressNotMapped { address })
        .map(|(range, mapped_device)| 
            mapped_device.write(address - range.start(), data))?
    }
}

#[cfg(test)]
mod tests {
    use crate::Memory;

    use super::*;

    const TEST_ADDRESSES: &[usize] = &[0, 1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 256, 257, 511, 512, 513, 1023, 1024, 1025, 2047, 2048, 2049, 4095, 4096, 4097];

    #[test]
    fn test_memory_map_creation() {
        let mut memory_map = MemoryMap::new();

        for addr in TEST_ADDRESSES {
            assert_eq!(memory_map.read(*addr), Err(BusDeviceError::AddressNotMapped { address: *addr }));
            assert_eq!(memory_map.write(*addr, 0), Err(BusDeviceError::AddressNotMapped { address: *addr }));
        }
    }

    #[test]
    fn test_memory_map_single_at_start() {
        let mut memory_map = MemoryMap::new().with_range((0..=7), Box::new(Memory::filled([0, 1, 2, 3, 4, 5, 6, 7])));

        for addr in TEST_ADDRESSES {
            if *addr < 8 {
                assert_eq!(memory_map.read(*addr), Ok((*addr % 256) as u8));
            }
            else {  
                assert_eq!(memory_map.read(*addr), Err(BusDeviceError::AddressNotMapped { address: *addr }));
            }
        }
    }

    #[test]
    fn test_memory_map_single_in_middle_start() {
        let mut memory_map = MemoryMap::new().with_range((4..=11), Box::new(Memory::filled([0, 1, 2, 3, 4, 5, 6, 7])));

        for addr in 0..16 {
            if addr >= 4 && addr < 12 {
                assert_eq!(memory_map.read(addr), Ok(((addr - 4) % 256) as u8));
            }
            else {  
                assert_eq!(memory_map.read(addr), Err(BusDeviceError::AddressNotMapped { address: addr }));
            }
        }
    }

    #[test]
    fn test_memory_map_multiple_continuous() {
        let mut memory_map = MemoryMap::new()
            .with_range((0..=3), Box::new(Memory::filled([0, 1, 2, 3])))
            .with_range((4..=7), Box::new(Memory::filled([4, 5, 6, 7])));

        for addr in 0..16 {
            if addr < 8 {
                assert_eq!(memory_map.read(addr), Ok((addr % 256) as u8));
            }
            else {  
                assert_eq!(memory_map.read(addr), Err(BusDeviceError::AddressNotMapped { address: addr }));
            }
        }
    }

    #[test]
    fn test_memory_map_multiple_discontinuous() {
        let mut memory_map = MemoryMap::new()
            .with_range((0..=3), Box::new(Memory::filled([0, 1, 2, 3])))
            .with_range((6..=7), Box::new(Memory::filled([6, 7])));

        for addr in 0..16 {
            if addr < 4 || (addr >= 6 && addr < 8) {
                assert_eq!(memory_map.read(addr), Ok((addr % 256) as u8));
            }
            else {  
                assert_eq!(memory_map.read(addr), Err(BusDeviceError::AddressNotMapped { address: addr }));
            }
        }
    }
}