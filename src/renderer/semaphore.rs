use ash::{vk, version::DeviceV1_0};

use crate::renderer::device::Device;

#[derive(Copy, Clone)]
pub struct Semaphore {
    pub semaphore: vk::Semaphore,
}

impl Semaphore {
    pub unsafe fn new(d: &Device) -> Semaphore {
        let semaphore_ci = vk::SemaphoreCreateInfo::builder();

        let semaphore = d.device.create_semaphore(&semaphore_ci, None).unwrap();

        Semaphore {
            semaphore
        }
    }
}