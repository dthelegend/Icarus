use frunk::{Generic, HCons, HList, HNil};
use frunk::prelude::*;
use frunk_core::coproduct::CoproductSubsetter;
use frunk_core::hlist::{Sculptor, HFoldLeftable, HMappable};
use frunk_core::poly_fn;
use frunk_core::traits::{Func, Poly, ToMut};
use nalgebra::{Quaternion, SVector};
use rayon::prelude::*;
use crate::ecs::example::Transform;

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

pub trait ComponentList: HList + Sealed {}

impl ComponentList for HNil {}
impl <HeadT, TailT: ComponentList> ComponentList for HCons<ComponentStorage<HeadT>, TailT> {}

trait ToComponentList: HList {
    type Output: HList;
}

impl ToComponentList for HNil {
    type Output = HNil;
}

impl <HeadT, TailT: ToComponentList> ToComponentList for HCons<HeadT, TailT> {
    type Output = HCons<HeadT, <TailT as ToComponentList>::Output>;
}

pub struct Archetype<ComponentListT: ToComponentList, EntityT : Into<usize> = Entity> {
    entity_list: ComponentStorage<EntityT>,
    components: ComponentListT::Output
}

pub trait ArchetypeList: Sealed {
}

impl ArchetypeList for HNil {}

impl <T: ToComponentList, TailT: ArchetypeList> ArchetypeList for HCons<Archetype<T>, TailT> {}

// SYSTEM
trait System {
    type InstanceT: ToComponentList;
    fn update_instance(instance: Self::InstanceT) -> Self::InstanceT;
}

pub trait SystemList: Sealed {
}

impl SystemList for HNil {
}

impl <T: System, TailT: SystemList> SystemList for HCons<T, TailT>{
}

struct World<ArchetypeListT: ToComponentList, SystemListT: SystemList> {
    archetypes: ArchetypeListT::Output,
    systems: SystemListT,
}



impl<ArchetypeListT: ToComponentList, SystemListT: SystemList> World<ArchetypeListT, SystemListT> {
    pub fn apply_over_all(self) {
        todo!()
    }
}

struct ParallelArrayMapping;

impl <'a, T> Func<&'a mut ComponentStorage<T>> for ParallelArrayMapping
where ComponentStorage<T>: IntoParallelRefMutIterator<'a, Iter: IndexedParallelIterator>
{
    type Output = impl IndexedParallelIterator;

    fn call(i: &'a mut ComponentStorage<T>) -> Self::Output {
        i.par_iter_mut()
    }
}

struct ParallelArrayZip;

impl <AccT: IndexedParallelIterator<Item: HList>, InputT: IndexedParallelIterator> Func<(AccT, InputT)> for ParallelArrayZip {
    type Output = impl IndexedParallelIterator<Item: HList>;

    fn call((acc, input): (AccT, InputT)) -> Self::Output {
        acc.zip(input).map(|(acc, input)| acc.prepend(input))
    }
}

impl <'a, ArchetypeListT, EntityT> Archetype<ArchetypeListT, EntityT>
where
    EntityT: Into<usize>,
    
    ArchetypeListT: ToComponentList + ToMut<'a>,
    <ArchetypeListT as ToComponentList>::Output: ToMut<'a>,
    <<ArchetypeListT as ToComponentList>::Output as ToMut<'a>>::Output: 'a
{
    // Mutable reference to system is for
    fn apply_system<SystemT: System, Indices>(&'a mut self, _system: &'a mut SystemT)
    where
        <<SystemT as System>::InstanceT as ToComponentList>::Output: ToMut<'a>,
        <<ArchetypeListT as ToComponentList>::Output as ToMut<'a>>::Output: Sculptor<<<<SystemT as System>::InstanceT as ToComponentList>::Output as ToMut<'a>>::Output, Indices>,
        <<<SystemT as System>::InstanceT as ToComponentList>::Output as ToMut<'a>>::Output: HMappable<Poly<ParallelArrayMapping>>,
        <<<<SystemT as System>::InstanceT as ToComponentList>::Output as ToMut<'a>>::Output as HMappable<Poly<ParallelArrayMapping>>>::Output: HFoldLeftable<Poly<ParallelArrayZip>, rayon::iter::Repeat<HNil>>,
        <<<<<SystemT as System>::InstanceT as ToComponentList>::Output as ToMut<'a>>::Output as HMappable<Poly<ParallelArrayMapping>>>::Output as HFoldLeftable<Poly<ParallelArrayZip>, rayon::iter::Repeat<HNil>>>::Output: ParallelIterator<Item = <SystemT as System>::InstanceT>
    {
        let exzy = self.components.to_mut();
        let (relevant_components, _): (<<<SystemT as System>::InstanceT as ToComponentList>::Output as ToMut<'a>>::Output, _) = exzy.sculpt();
        let par_arrays = relevant_components.map(Poly(ParallelArrayMapping));
        let zipped_par_arrays = par_arrays.foldl(Poly(ParallelArrayZip), rayon::iter::repeat(HNil));
        
        zipped_par_arrays.for_each(|instance| {
            SystemT::update_instance(instance);
        });
    }
}

mod example {
    use std::ops::{Add, AddAssign};
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
        type InstanceT = HList!(Transform, DeltaTransform);
    
        fn update_instance(mut instance: Self::InstanceT) -> Self::InstanceT {
            let delta = *instance.get::<DeltaTransform, _>();
            *instance.get_mut::<Transform, _>() += delta;
    
            instance
        }
    }

    struct RenderSystem;
    impl System for RenderSystem {
        type InstanceT = HList!(Transform, Model);
    
        fn update_instance(_instance: Self::InstanceT) -> Self::InstanceT {
            todo!()
        }
    }
}
