use frunk::{HCons, HList, HNil, Poly};
use frunk::prelude::*;
use nalgebra::{Quaternion, SVector};

struct Transform {
    position: SVector<f32, 3>,
    rotation: Quaternion<f32>,
    scale: SVector<f32, 3>
}

struct DeltaTransform {
    position: SVector<f32, 3>,
    rotation: Quaternion<f32>,
    scale: SVector<f32, 3>,
}

struct Model {
    // TODO
}

struct Entity(usize);
impl From<Entity> for usize {
    fn from(value: Entity) -> Self {
        value.0
    }
}


// Component lists use a modified HList pattern
trait Component {
}

type ComponentStorage<T> = Vec<T>;

trait ComponentList {
    type StorageList;
}

impl ComponentList for HNil {
    type StorageList = HNil;
}

impl <T: Component, TailT : ComponentList> ComponentList for HCons<T, TailT> {
    type StorageList = HCons<ComponentStorage<T>, TailT::StorageList>;
}

struct ComponentListCons<Head, Tail : ComponentList> {
    head: Vec<Head>,
    tail: Tail,
}

struct Archetype<ComponentListT: ComponentList, EntityT : Into<usize> = Entity> {
    entity_list: Vec<EntityT>,
    components: ComponentListT
}

impl <ComponentListT: ComponentList, EntityT : Into<usize>> Archetype<ComponentListT, EntityT> {
}

type UnitArchetype = Archetype<HList!(Transform, DeltaTransform, Model)>;
type TileArchetype = Archetype<HList!(Transform, Model)>;

// World also uses the HList pattern, but one is expected to construct this with a builder style pattern
trait ArchetypeList {
}

// Base aspects for all systems
struct ArchetypeListNil;

struct ArchetypeListCons<HeadT: ComponentList, Tail : ArchetypeList> {
    head: Archetype<HeadT>,
    tail: Tail
}

struct World<ArchetypeListT: ArchetypeList, SystemListT: SystemList> {
    archetype_list: ArchetypeListT,
    system_list: SystemListT
}

impl <ArchetypeListT: ArchetypeList, SystemListT: SystemList> World<ArchetypeListNil, SystemListNil> {
    fn apply_systems(&mut self) {
        self.system_list.update_all(self.archetype_list.components().sculpt())
    }
}

trait System {
    type Components: ComponentList;
    fn update(&mut self, components: &mut Self::Components);
}

struct MovementSystem;

impl System for MovementSystem {
    type Components = ComponentList!(Transform, DeltaTransform);

    fn update(&mut self, components: &mut Self::Components) {
        todo!()
    }
}

struct RenderSystem;

impl System for RenderSystem {
    type Components = ComponentList!(Transform, Model);

    fn update(&mut self, components: &mut Self::Components) {
        todo!()
    }
}
