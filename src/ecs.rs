use frunk::{Generic, HCons, HList, HNil};
use frunk::prelude::*;
use frunk_core::coproduct::CoproductSubsetter;
use frunk_core::hlist::Sculptor;
use frunk_core::traits::ToMut;
use nalgebra::{Quaternion, SVector};
use rayon::prelude::*;

trait Sealed {}
impl Sealed for HNil {}
impl <T, U> Sealed for HCons<T, U> {}

pub struct Entity(usize);
impl From<Entity> for usize {
    fn from(value: Entity) -> Self {
        value.0
    }
}

// Component lists use a modified HList pattern
pub type ComponentStorage<T> = Vec<T>;

pub trait StorageList: HList + Sealed {}

impl StorageList for HNil {}
impl <HeadT, TailT: StorageList> StorageList for HCons<ComponentStorage<HeadT>, TailT> {}

pub trait ComponentList : HList + Sealed {
    type StorageList: StorageList;
}

impl ComponentList for HNil {
    type StorageList = HNil;
}

impl <T, TailT : ComponentList> ComponentList for HCons<T, TailT> {
    type StorageList = HCons<ComponentStorage<T>, TailT::StorageList>;
}

pub struct Archetype<ComponentListT: ComponentList, EntityT : Into<usize> = Entity> {
    entity_list: ComponentStorage<EntityT>,
    components: ComponentListT
}

impl <ComponentListT: ComponentList, EntityT : Into<usize>> Archetype<ComponentListT, EntityT> {
}

trait IntoArchetype : Generic<Repr: ComponentList> {
    type AsArchetype = Archetype<<Self as Generic>::Repr>;
}


trait ArchetypeList: Sealed {}

impl ArchetypeList for HNil {}

impl <T: ComponentList, TailT: ArchetypeList> ArchetypeList for HCons<Archetype<T>, TailT> {}

// SYSTEM
trait System {
    type InstanceT: ComponentList;

    fn instance_update(instance: Self::InstanceT) -> Self::InstanceT;
}

pub trait SystemList: Sealed {
}

impl SystemList for HNil {}

impl <T: System, TailT: SystemList> SystemList for HCons<T, TailT> {}

// WORLD
struct World<ArchetypeListT: ArchetypeList, SystemListT: SystemList> {
    archetype_list: ArchetypeListT,
    system_list: SystemListT
}

impl <ArchetypeListT: ArchetypeList> World<ArchetypeListT, HNil> {
}

impl <'a, SystemHeadT: System<InstanceT: ToMut<'a>>, SystemTailT: SystemList, ArchetypeHeadT: ComponentList + ToMut<'a>, ArchetypeTailT: ArchetypeList> World<HCons<Archetype<ArchetypeHeadT>, ArchetypeTailT>, HCons<SystemHeadT, SystemTailT>> {
    fn apply(&mut self) {
        todo!()
    }
}

mod example {
    use std::ops::{Add, AddAssign};
    use frunk::prelude::*;
    use super::*;

    #[derive(Copy, Clone)]
    pub struct Transform {
        position: SVector<f32, 3>,
        rotation: Quaternion<f32>,
        scale: SVector<f32, 3>
    }

    #[derive(Copy, Clone)]
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

        fn update_instance(mut instance: Self::Components) -> Self::Components {
            let delta = *instance.get::<DeltaTransform, _>();
            *instance.get_mut::<Transform, _>() += delta;

            instance
        }
    }

    struct RenderSystem;
    impl System for RenderSystem {
        type Components = HList!(Transform, Model);

        fn update_instance(instance: Self::Components) -> Self::Components {
            todo!()
        }
    }
}
