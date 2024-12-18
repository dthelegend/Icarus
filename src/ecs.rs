use std::process::Output;
use frunk::{Generic, HCons, HList, HNil};
use frunk::prelude::*;
use frunk_core::coproduct::CoproductSubsetter;
use frunk_core::hlist::{Sculptor, HFoldLeftable, HMappable};
use frunk_core::poly_fn;
use frunk_core::traits::{Func, Poly, ToMut};
use nalgebra::{Quaternion, SVector};
use rayon::prelude::*;

trait Sealed {}
impl Sealed for HNil {}
impl <T, U> Sealed for HCons<T, U> {}


pub struct Entity(usize);
impl From<usize> for Entity {
    fn from(value: usize) -> Self {
        Entity(value)
    }
}

// Component lists use a modified HList pattern
pub type ComponentStorage<T> = Vec<T>;

pub trait ComponentList: HList + Sealed {
}

impl ComponentList for HNil {
}
impl <HeadT, TailT: ComponentList> ComponentList for HCons<ComponentStorage<HeadT>, TailT>
{
}

pub trait IntoParIter: HList + Sealed {
    type Item: HList + Send;

    fn get_parallel_mut(self) -> impl IndexedParallelIterator<Item = Self::Item>;
}

impl <HeadT> IntoParIter for HCons<HeadT, HNil>
where
    HeadT: IntoParallelIterator<Iter: IndexedParallelIterator>, <HeadT as IntoParallelIterator>::Item: Send
{
    type Item = HList![<HeadT as IntoParallelIterator>::Item];
    fn get_parallel_mut(self) -> impl IndexedParallelIterator<Item = Self::Item> {
        rayon::iter::repeat(HNil).zip(self.head.into_par_iter()).map(|(hnil, x)| hnil.prepend(x))
    }
}

impl <HeadT, TailT: IntoParIter> IntoParIter for HCons<HeadT, TailT>
where
    HeadT: IntoParallelIterator<Iter: IndexedParallelIterator>, <HeadT as IntoParallelIterator>::Item: Send
{
    type Item = HList![<HeadT as IntoParallelIterator>::Item, ...<TailT as IntoParIter>::Item];
    fn get_parallel_mut(self) -> impl IndexedParallelIterator<Item = Self::Item> {
        self.tail.get_parallel_mut().zip(self.head.into_par_iter()).map(|(tail, head)| tail.prepend(head))
    }
}

trait ToComponentList: HList {
    type Output: ComponentList;
}

impl ToComponentList for HNil {
    type Output = HNil;
}

impl <HeadT, TailT: ToComponentList> ToComponentList for HCons<HeadT, TailT>
where ComponentStorage<HeadT>: for<'a> IntoParallelRefMutIterator<'a, Iter: IndexedParallelIterator> {
    type Output = HCons<ComponentStorage<HeadT>, <TailT as ToComponentList>::Output>;
}

#[derive(Default)]
pub struct Archetype<ComponentListT: ToComponentList, EntityT : From<usize> = Entity> {
    entity_list: ComponentStorage<EntityT>,
    components: ComponentListT::Output
}

pub trait ArchetypeList: HList + Sealed {
    fn apply_system_list<SystemListT: SystemList>(&mut self);
}

impl ArchetypeList for HNil {
    fn apply_system_list<SystemListT: SystemList>(&mut self) {}
}

impl <T: ToComponentList + for<'a> ToMut<'a>, TailT: ArchetypeList> ArchetypeList for HCons<Archetype<T>, TailT>
{
    fn apply_system_list<SystemListT: SystemList>(&mut self) {
        
    }
}

#[macro_export]
macro_rules! Archetype {
    [$inner_type:ty] => {
        Archetype<<$inner_type as Generic>::Repr>
    };
}


// SYSTEM
trait System {
    type InstanceT: for<'a> ToMut<'a>;
    fn update_instance(instance: <Self::InstanceT as ToMut>::Output);
}

pub trait SystemList: HList + Sealed {
}

impl SystemList for HNil {
}

impl <T: System, TailT: SystemList> SystemList for HCons<T, TailT>{
}

struct World<SystemListT: SystemList, ArchetypeListT: ArchetypeList + Default> {
    systems: SystemListT,
    archetypes: ArchetypeListT,
}

struct ParallelArrayZip;

impl <AccT: IndexedParallelIterator<Item: HList>, InputT: IndexedParallelIterator> Func<(AccT, InputT)> for ParallelArrayZip {
    type Output = impl IndexedParallelIterator<Item: HList>;

    fn call((acc, input): (AccT, InputT)) -> Self::Output {
        acc.zip(input).map(|(acc, input)| acc.prepend(input))
    }
}

trait CanApplySystem<'a, SystemT: System, Indices> {
    fn apply_system(&'a mut self);
}

impl <'a, SystemT: System, Indices, ArchetypeListT, EntityT> CanApplySystem<'a, SystemT, Indices> for Archetype<ArchetypeListT, EntityT>
where
    ArchetypeListT: ToComponentList,
    EntityT: From<usize>,
    ArchetypeListT: ToComponentList + ToMut<'a>,
    <ArchetypeListT as ToComponentList>::Output: ToMut<'a>,
    <<ArchetypeListT as ToComponentList>::Output as ToMut<'a>>::Output: Sculptor<<<<SystemT as System>::InstanceT as ToComponentList>::Output as ToMut<'a>>::Output, Indices>,
    SystemT: System,
    <SystemT as System>::InstanceT: ToMut<'a> + ToComponentList,
    <<SystemT as System>::InstanceT as ToComponentList>::Output: ToMut<'a>,
    <<<SystemT as System>::InstanceT as ToComponentList>::Output as ToMut<'a>>::Output: IntoParIter<Item = <<SystemT as System>::InstanceT as ToMut<'a>>::Output>,
{
    fn apply_system(&'a mut self) {
        let (resolved_components, _) : (<<<SystemT as System>::InstanceT as ToComponentList>::Output as ToMut<'a>>::Output, _) = self.components.to_mut().sculpt();
        resolved_components.get_parallel_mut().for_each(SystemT::update_instance);
    }
}

impl <ArchetypeListT, EntityT> Archetype<ArchetypeListT, EntityT>
where
    ArchetypeListT: ToComponentList,
    EntityT: From<usize>,
{
    // Mutable reference to system is for
    fn apply_system<'a, SystemT: System, Indices>(&'a mut self)
    where
        Self: CanApplySystem<'a, SystemT, Indices>
    {
        CanApplySystem::<'a, SystemT, Indices>::apply_system(self)
    }
}

trait CanApplySystemList<'a, SystemT: System, IndicesList> {
    fn apply_system(&'a mut self);
}

impl <'a, SystemT: System> CanApplySystemList<'a, SystemT, HNil> for HNil {
    fn apply_system(&'a mut self) {}
}

impl <'a, HeadIndicesT, TailIndicesT, SystemT: System, HeadT: CanApplySystem<'a, SystemT, HeadIndicesT>, TailT: CanApplySystemList<'a, SystemT, TailIndicesT>> CanApplySystemList<'a, SystemT, HCons<HeadIndicesT, TailIndicesT>> for HCons<HeadT, TailT> {
    fn apply_system(&'a mut self) {
        self.head.apply_system();
        self.tail.apply_system();
    }
}


#[cfg(test)]
mod test {
    use std::ops::{Add, AddAssign};
    use frunk::hlist;
    use nalgebra::vector;
    use super::*;

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
        type InstanceT = HList!(Transform, DeltaTransform);

        fn update_instance(instance: <Self::InstanceT as ToMut>::Output) {
            let (delta, instance) : (&mut DeltaTransform, _) = instance.pluck();
            let (transform, _) = instance.pluck();
            *transform += *delta;
        }
    }

    struct RenderSystem;
    impl System for RenderSystem {
        type InstanceT = HList!(Transform, Model);

        fn update_instance(_instance: <Self::InstanceT as ToMut>::Output) {
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
        
        let unit_arch: Archetype![Unit] = Archetype {
            entity_list: vec![0,1,2,3].into_iter().map(|x| (x as usize).into()).collect(),
            components: hlist![
                std::iter::repeat_n(TRANSFORM_BASE, COMPONENT_SIZE_UNIT).collect(),
                std::iter::repeat_n(DELTA_TRANSFORM_BASE, COMPONENT_SIZE_UNIT).collect(),
                std::iter::repeat_n(MODEL_BASE, COMPONENT_SIZE_UNIT).collect(),
            ]
        };

        let DELTA_TRANSFORM_BASE = DeltaTransform {
            position: vector![-10.0,4.0,2.0],
            rotation: Default::default(),
            scale: vector![1.0, 1.0, 1.0]
        };
        let TRANSFORM_BASE = Transform::default();
        let MODEL_BASE = Model{};
        const COMPONENT_SIZE_TILE: usize = 10;

        let tile_arch: Archetype![Tile] = Archetype {
            entity_list: vec![0,1,2,3].into_iter().map(|x| (x as usize).into()).collect(),
            components: hlist![
                std::iter::repeat_n(TRANSFORM_BASE, COMPONENT_SIZE_TILE).collect(),
                std::iter::repeat_n(MODEL_BASE, COMPONENT_SIZE_TILE).collect(),
            ]
        };
        
        let mut arches = hlist![unit_arch, tile_arch];
        
        println!("{:?}", arches.get::<Archetype![Unit],_>().components);

        CanApplySystemList::<MovementSystem, _>::apply_system(&mut arches);

        println!("{:?}", arches.get::<Archetype![Unit],_>().components);
    }
}
