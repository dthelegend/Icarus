pub mod archetypes {
    use crate::ecs::core::components::{ModelData, Transform};
    use crate::ecs::Archetype;

    #[derive(Archetype)]
    pub struct Drawable {
        transform: Transform,
        model_data: ModelData,
    }
}

pub mod components {
    use modelz::{Model3D, ModelError};
    use std::path::Path;
    use std::sync::Arc;
    use vulkano::buffer::{AllocateBufferError, Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
    use vulkano::device::DeviceOwned;
    use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter};
    use vulkano::Validated;

    pub struct Transform {
        position: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
    }

    #[derive(Clone)]
    pub struct ModelData {
        model_data: Subbuffer<[[f32; 3]]>,
    }

    #[derive(thiserror::Error, Debug)]
    pub enum ModelDataError {
        #[error("failed to load model!")]
        ModelError(ModelError),
        #[error("vulkan error! {0}")]
        AllocationError(#[from] Validated<AllocateBufferError>),
    }

    impl ModelData {
        pub fn teapot(memory_allocator: Arc<dyn MemoryAllocator>) -> Result<ModelData, ModelDataError> {
            let model_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("models").join("teapot.ply");
            Self::from_path(memory_allocator, model_path)
        }

        pub fn from_path<P: AsRef<Path>>(memory_allocator: Arc<dyn MemoryAllocator>, path: P) -> Result<ModelData, ModelDataError> {
            let model = Model3D::load(path).map_err(ModelDataError::ModelError)?;

            let vertices = model.meshes
                .first()
                .unwrap()
                .vertices
                .iter()
                .map(|x| x.position);

            Buffer::from_iter(
                memory_allocator,
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..BufferCreateInfo::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                vertices,
            ).map(|model_data| ModelData { model_data }).map_err(From::from)
        }
    }
}
