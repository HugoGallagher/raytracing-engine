use ash::vk;

use crate::renderer::device::Device;

#[derive(Copy, Clone)]
pub struct Fence {
    pub fence: vk::Fence,
}

impl Fence {
    pub unsafe fn new(d: &Device, signaled: bool) -> Fence {
        let mut fence_ci = vk::FenceCreateInfo::builder();

        if signaled {
            fence_ci = fence_ci.flags(vk::FenceCreateFlags::SIGNALED);
        }

        let fence = d.device.create_fence(&fence_ci, None).unwrap();

        Fence {
            fence
        }
    }
}