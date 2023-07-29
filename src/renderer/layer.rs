use ash::vk;

pub enum LayerRef {
    Compute(usize),
    Graphics(usize),
}

#[derive(Copy, Clone)]
pub struct LayerDependencyInfo {
    pub stage: vk::PipelineStageFlags,
}

pub struct LayerSubmitInfo {
    pub wait_semaphores: Vec<vk::Semaphore>,
    pub wait_stages: Vec<vk::PipelineStageFlags>,
    pub signal_semaphores: Vec<vk::Semaphore>,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub queue: vk::Queue,
    pub fence: vk::Fence,
    pub submit_i: vk::SubmitInfo,
}