use crate::renderer::device::Device;
use crate::renderer::semaphore::Semaphore;
use crate::renderer::fence::Fence;

#[derive(Copy, Clone)]
pub struct Frame {
    pub image_available_semaphore: Semaphore,
    pub in_flight_fence: Fence,
}

impl Frame {
    pub unsafe fn new(d: &Device) -> Frame {
        let image_available_semaphore = Semaphore::new(d);

        let in_flight_fence = Fence::new(d, true);

        Frame {
            image_available_semaphore,
            in_flight_fence,
        }
    }
}