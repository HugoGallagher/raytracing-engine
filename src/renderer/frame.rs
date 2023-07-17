use crate::renderer::device::Device;
use crate::renderer::semaphore::Semaphore;
use crate::renderer::fence::Fence;

pub struct Frame {
    pub compute_finished_semaphore: Semaphore,
    pub render_finished_semaphore: Semaphore,
    pub image_available_semaphore: Semaphore,
    pub in_flight_fence: Fence,
}

impl Frame {
    pub unsafe fn new(d: &Device) -> Frame {
        let compute_finished_semaphore = Semaphore::new(d);
        let render_finished_semaphore = Semaphore::new(d);
        let image_available_semaphore = Semaphore::new(d);

        let in_flight_fence = Fence::new(d, true);

        Frame {
            compute_finished_semaphore,
            render_finished_semaphore,
            image_available_semaphore,
            in_flight_fence,
        }
    }
}