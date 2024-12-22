use std::ops::{Add, AddAssign};
use frunk::{hlist, Generic};
use frunk_core::HList;
use frunk_core::traits::ToMut;
use nalgebra::{vector, Quaternion, SVector};
use crate::Archetype;
use crate::ecs::{Archetype, System};

#[derive(Copy, Clone, Default, Generic, Debug)]
pub struct Transform {
    position: SVector<f32, 3>,
    rotation: Quaternion<f32>,
    scale: SVector<f32, 3>
}

#[derive(Copy, Clone, Default, Generic, Debug)]
pub struct DeltaTransform {
    position: SVector<f32, 3>,
    rotation: Quaternion<f32>,
    scale: SVector<f32, 3>,
}

impl Add<DeltaTransform> for Transform {
    type Output = Transform;

    fn add(mut self, rhs: DeltaTransform) -> Self::Output {
        self.position += rhs.position;
        self.rotation += rhs.rotation;
        self.scale += rhs.scale;

        self
    }
}

impl AddAssign<DeltaTransform> for Transform {
    fn add_assign(&mut self, rhs: DeltaTransform) {
        *self = *self + rhs;
    }
}

#[derive(Copy, Clone, Default, Generic, Debug)]
pub struct Model {
    // TODO
}

#[derive(Generic)]
struct Unit {
    transform: Transform,
    delta_transform: DeltaTransform,
    model: Model
}

#[derive(Generic)]
struct Tile {
    transform: Transform,
    model: Model
}

struct MovementSystem;
impl System for MovementSystem {
    type Components = HList!(Transform, DeltaTransform);

    fn update_instance(instance: <Self::Components as ToMut>::Output) {
        let (delta, instance) : (&mut DeltaTransform, _) = instance.pluck();
        let (transform, _) = instance.pluck();
        *transform += *delta;
    }
}

struct RenderSystem;
impl System for RenderSystem {
    type Components = HList!(Transform, Model);

    fn update_instance(_instance: <Self::Components as ToMut>::Output) {
        println!("I feel totally modeled rn");
    }
}

#[test]
fn test_ecs() {
    const DELTA_TRANSFORM_BASE: DeltaTransform = DeltaTransform {
        position: vector![-10.0,4.0,2.0],
        rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
        scale: vector![1.0, 1.0, 1.0]
    };
    const TRANSFORM_BASE: Transform = Transform{
        position: vector![0.0,0.0,0.0],
        rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
        scale: vector![1.0, 1.0, 1.0]
    };
    const MODEL_BASE: Model = Model{};

    const COMPONENT_SIZE_UNIT: usize = 10;

    let mut unit_arch: Archetype![Unit] = Archetype {
        entity_list: vec![0,1,2,3].into_iter().map(|x| (x as usize).into()).collect(),
        components: hlist![
            std::iter::repeat_n(TRANSFORM_BASE, COMPONENT_SIZE_UNIT).collect(),
            std::iter::repeat_n(DELTA_TRANSFORM_BASE, COMPONENT_SIZE_UNIT).collect(),
            std::iter::repeat_n(MODEL_BASE, COMPONENT_SIZE_UNIT).collect(),
        ]
    };

    const COMPONENT_SIZE_TILE: usize = 10;

    let mut tile_arch: Archetype![Tile] = Archetype {
        entity_list: vec![0,1,2,3].into_iter().map(|x| (x as usize).into()).collect(),
        components: hlist![
            std::iter::repeat_n(TRANSFORM_BASE, COMPONENT_SIZE_TILE).collect(),
            std::iter::repeat_n(MODEL_BASE, COMPONENT_SIZE_TILE).collect(),
        ]
    };

    println!("{:?}", unit_arch.components);

    unit_arch.apply_system::<MovementSystem, _>();
    tile_arch.apply_system::<RenderSystem, _>();

    println!("{:?}", unit_arch.components);
}
