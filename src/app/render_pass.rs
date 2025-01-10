use std::sync::Arc;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::render_pass::RenderPass;
use vulkano::{Validated, VulkanError};
use vulkano::command_buffer::AutoCommandBufferBuilder;

struct AppRenderPass {
    render_pass: Arc<RenderPass>,
}

impl AppRenderPass {
    pub fn new(device: Arc<Device>) -> Result<Self, Validated<VulkanError>> {
        let render_pass = vulkano::single_pass_renderpass!(
            device,
            attachments: {
                color: {
                    format: Format::R8G8B8A8_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )?;

        Ok(Self { render_pass })
    }
    
    fn wow() {

        // let mut builder = AutoCommandBufferBuilder::primary(
        //     &command_buffer_allocator,
        //     queue.queue_family_index(),
        //     CommandBufferUsage::OneTimeSubmit,
        // )
        //     .unwrap();
        // 
        // builder
        //     .begin_render_pass(
        //         RenderPassBeginInfo {
        //             clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
        //             ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
        //         },
        //         SubpassBeginInfo {
        //             contents: SubpassContents::Inline,
        //             ..Default::default()
        //         },
        //     )
        //     .unwrap()
        //     .end_render_pass(SubpassEndInfo::default())
        //     .unwrap();

    }
}