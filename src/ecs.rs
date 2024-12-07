use std::path::Components;
use frunk::{hlist, HCons, HList};
use frunk::hlist::{HList, LiftInto};
use nalgebra::{Quaternion, SVector};

struct Transform {
    position: SVector<f32, 3>,
    rotation: Quaternion<f32>,
    scale: SVector<f32, 3>,
}
struct TransformComponent(Vec<Transform>);
struct DeltaTransformComponent(Vec<Transform>);

struct Model {
    // TODO
}
struct ModelComponent(Vec<Model>);

type Renderable<'a> = HList!(&'a mut TransformComponent, &'a ModelComponent);
type Transformable<'a> = HList!(&'a mut TransformComponent);
type Moveable<'a> = HList!(&'a mut TransformComponent, &'a mut DeltaTransformComponent);

type Cool<'a> = H Renderable<'a> Transformable<'a>;

type Entity = usize;

struct ManagedEntityList<Components: HList> {
    entity_list: Vec<Entity>,
    components: Components,
}

impl <Components: HList> ManagedEntityList<Components> {
    fn get_archetype<OtherComponent>(&mut self) -> OtherComponent {
        (&mut self.components).lift_into()
    }
}

type UnitEntityList = ManagedEntityList<HList!(TransformComponent, DeltaTransformComponent, Model)>;
type TileList = ManagedEntityList<HList!(TransformComponent, Model)>;

trait System {
    fn update(&mut self);
}

impl System for MovementSystem {
    fn update(&mut self) {
        todo!()
    }
}

struct Archetype<Components: HList> {
    inner_components: Components,
}
