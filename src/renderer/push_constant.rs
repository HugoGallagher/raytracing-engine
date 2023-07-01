use std::{ptr, mem};

pub struct PushConstant {
    pub data: Vec<u8>,
    pub size: usize,
}

impl PushConstant {
    pub fn new(size: usize) -> PushConstant {
        let mut data = Vec::<u8>::new();
        data.resize(size, 0);

        PushConstant {
            data,
            size
        }
    }

    pub unsafe fn set_data<T>(&mut self, data: &T) {
        assert!(mem::size_of::<T>() <= 128, "Error: Push constant data type is larger than 128 bytes");

        let data_ptr = self.data.as_mut_ptr();
        
        ptr::copy(data as *const T as *mut u8, data_ptr, self.size);
    }
}