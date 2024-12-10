use std::iter::repeat;
use frunk::{Generic, HCons, HList, HNil};
use frunk::prelude::*;
use frunk_core::coproduct::CoproductSubsetter;
use frunk_core::hlist::{Sculptor, HFoldLeftable, HMappable};
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

trait ToComponentList {
    type Output: HList;
}

impl ToComponentList for HNil {
    type Output = HNil;
}

impl <HeadT, TailT: ToComponentList> ToComponentList for HCons<HeadT, TailT> {
    type Output = HCons<HeadT, <TailT as ToComponentList>::Output>;
}

pub struct Archetype<ComponentListT: ComponentList, EntityT : Into<usize> = Entity> {
    entity_list: ComponentStorage<EntityT>,
    components: ComponentListT
}

trait IntoArchetype : Generic<Repr: ComponentList> {
    type AsArchetype = Archetype<<Self as Generic>::Repr>;
}

trait ArchetypeList: Sealed {
}

impl ArchetypeList for HNil {}

impl <T: ComponentList, TailT: ArchetypeList> ArchetypeList for HCons<Archetype<T>, TailT> {}

// trait ComponentListZippable : ComponentList {
//     type ZipT: HList;
//     fn zip<'a>(&'a mut self) -> impl ParallelIterator<Item = Self::ZipT>;
// }
//
// impl ComponentListZippable for HNil {
//     type ZipT = HNil;
//     fn zip<'a>(&'a mut self) -> impl ParallelIterator<Item = Self::ZipT> {
//         repeat(HNil).par_bridge()
//     }
// }
//
// impl <HeadT, TailT: ComponentListZippable> ComponentListZippable for HCons<ComponentStorage<HeadT>, TailT>
// where ComponentStorage<HeadT>: for<'a> IntoParallelRefMutIterator<'a, Iter: IndexedParallelIterator>,
//     // TailT::ZipT: IntoParallelIterator<Iter: IndexedParallelIterator>
// {
//     type ZipT = HCons<HeadT, TailT::ZipT>;
//
//     fn zip<'a>(&'a mut self) -> impl ParallelIterator<Item=Self::ZipT> {
//         self.head.par_iter_mut()
//             .zip(ComponentListZippable::zip(&mut self.tail))
//             .map(|(head, tail)| HCons::)
//     }
// }

// SYSTEM
trait System {
    type InstanceT: ComponentList;
    fn update_instance(instance: Self::InstanceT) -> Self::InstanceT;
}

pub trait SystemList: Sealed {
}

impl SystemList for HNil {
}

impl <T: System, TailT: SystemList> SystemList for HCons<T, TailT>{
}

struct World<ArchetypeListT: ArchetypeList, SystemListT: SystemList> {
    archetypes: ArchetypeListT,
    systems: SystemListT,
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

impl <ArchetypeListT, SystemListT> System<ArchetypeListT, SystemListT>
where
    SystemListT: SystemList,

    ArchetypeListT: ArchetypeList + for<'a> ToMut<'a>,
    for<'a> <ArchetypeListT as ToMut<'a>>::Output: HMappable<Poly<ParallelArrayMapping>>,
    for<'a> <<ArchetypeListT as ToMut<'a>>::Output as HMappable<Poly<ParallelArrayMapping>>>::Output: HFoldLeftable<Poly<ParallelArrayZip>, rayon::iter::Repeat<HNil>>,
    for<'a> <<<ArchetypeListT as ToMut<'a>>::Output as HMappable<Poly<ParallelArrayMapping>>>::Output as HFoldLeftable<Poly<ParallelArrayZip>, rayon::iter::Repeat<HNil>>>::Output: ParallelIterator<Item = >
{
    fn apply_all(&mut self) {
        let par_arrays = self.archetypes.to_mut().map(Poly(ParallelArrayMapping));
        let zipped_par_arrays = par_arrays.foldl(Poly(ParallelArrayZip), rayon::iter::repeat(HNil));
        
        zipped_par_arrays.
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
    // impl System for MovementSystem {
    //     type Components = HList!(Transform, DeltaTransform);
    // 
    //     fn update_instance(mut instance: Self::Components) -> Self::Components {
    //         let delta = *instance.get::<DeltaTransform, _>();
    //         *instance.get_mut::<Transform, _>() += delta;
    // 
    //         instance
    //     }
    // }

    struct RenderSystem;
    // impl System for RenderSystem {
    //     type Components = HList!(Transform, Model);
    // 
    //     fn update_instance(instance: Self::Components) -> Self::Components {
    //         todo!()
    //     }
    // }
}
