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

pub trait ToParMut: HList + Sealed {
    type Output: IndexedParallelIterator<Item: HList>;

    fn get_parallel_mut(self) -> Self::Output;
}

impl ToParMut for HNil {
    type Output = impl IndexedParallelIterator<Item: HList>;

    fn get_parallel_mut(self) -> Self::Output {
        rayon::iter::repeatn(HNil, usize::MAX) // Realistically the largest value
    }
}
impl <HeadT, TailT: ToParMut> ToParMut for HCons<HeadT, TailT>
where
    HeadT: IntoParallelIterator<Iter: IndexedParallelIterator>
{
    type Output = impl IndexedParallelIterator<Item: HList>;
    fn get_parallel_mut(self) -> Self::Output {
        self.tail.get_parallel_mut().zip(self.head.into_par_iter()).map(|(x, a)| x.prepend(a))
    }
}

trait ToComponentList: HList {
    type Output: ComponentList;
}

impl ToComponentList for HNil {
    type Output = HNil;
}

impl <HeadT, TailT: ToComponentList> ToComponentList for HCons<HeadT, TailT>
where Vec<HeadT>: for<'a> IntoParallelRefMutIterator<'a, Iter: IndexedParallelIterator> {
    type Output = HCons<ComponentStorage<HeadT>, <TailT as ToComponentList>::Output>;
}

#[derive(Default)]
pub struct Archetype<ComponentListT: ToComponentList, EntityT : From<usize> = Entity> {
    entity_list: ComponentStorage<EntityT>,
    components: ComponentListT::Output
}

pub trait ArchetypeList: Sealed {
    // fn apply_system<SystemT: System>(&mut self);
}

impl ArchetypeList for HNil {
    // fn apply_system<SystemT: System>(&mut self) {}
}

impl <'a, T: ToComponentList + ToMut<'a>, TailT: ArchetypeList> ArchetypeList for HCons<Archetype<T>, TailT> {
    // fn apply_system<SystemT: System>(&mut self)
    // {
    //     self.head.apply_system::<SystemT, _>();
    //     self.tail.apply_system::<SystemT, _>();
    // }
}

// SYSTEM
trait System {
    type InstanceT: for<'a> ToMut<'a>;
    fn update_instance(instance: <Self::InstanceT as ToMut>::Output);
}

pub trait SystemList: Sealed {
}

impl SystemList for HNil {
}

impl <T: System, TailT: SystemList> SystemList for HCons<T, TailT>{
}

struct World<SystemListT: SystemList, ArchetypeListT: ArchetypeList + Default> {
    systems: SystemListT,
    archetypes: ArchetypeListT,
}

// impl<ArchetypeListT: ToComponentList, SystemListT: SystemList> World<ArchetypeListT, SystemListT> {
//     pub fn apply_over_all(&mut self) {
//     }
// }

struct ParallelArrayZip;

impl <AccT: IndexedParallelIterator<Item: HList>, InputT: IndexedParallelIterator> Func<(AccT, InputT)> for ParallelArrayZip {
    type Output = impl IndexedParallelIterator<Item: HList>;

    fn call((acc, input): (AccT, InputT)) -> Self::Output {
        acc.zip(input).map(|(acc, input)| acc.prepend(input))
    }
}

impl <ArchetypeListT, EntityT> Archetype<ArchetypeListT, EntityT>
where
    ArchetypeListT: ToComponentList,
    EntityT: From<usize>
{
    // Mutable reference to system is for
    fn apply_system<'a, SystemT, Indices>(&'a mut self)
    where
        ArchetypeListT: ToComponentList + ToMut<'a>,
        <ArchetypeListT as ToComponentList>::Output: ToMut<'a>,
        <<ArchetypeListT as ToComponentList>::Output as ToMut<'a>>::Output: Sculptor<<<<SystemT as System>::InstanceT as ToComponentList>::Output as ToMut<'a>>::Output, Indices>,
        SystemT: System,
        <SystemT as System>::InstanceT: ToMut<'a> + ToComponentList,
        <<SystemT as System>::InstanceT as ToComponentList>::Output: ToMut<'a>,
        <<<SystemT as System>::InstanceT as ToComponentList>::Output as ToMut<'a>>::Output: ToParMut<Output: ParallelIterator<Item = <<SystemT as System>::InstanceT as ToMut<'a>>::Output>>,
    {
        let (resolved_components, _) : (<<<SystemT as System>::InstanceT as ToComponentList>::Output as ToMut<'a>>::Output, _) = self.components.to_mut().sculpt();
        resolved_components.get_parallel_mut().for_each(SystemT::update_instance);
    }
}

#[cfg(test)]
mod test {
    use std::ops::{Add, AddAssign};
    use frunk_core::{hlist, generic::Generic};
    use super::*;

    #[derive(Copy, Clone, Default, Generic)]
    pub struct Transform {
        position: SVector<f32, 3>,
        rotation: Quaternion<f32>,
        scale: SVector<f32, 3>
    }

    #[derive(Copy, Clone, Default, Generic)]
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
    
    #[derive(Copy, Clone, Default, Generic)]

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
        type TestSystem = RenderSystem;
        type TestArchetype = Archetype<<Tile as frunk::Generic>::Repr>;
        
        let transform_base = Transform::default();
        let model_base = Model{};
        
        let mut test_arch: TestArchetype = Archetype {
            entity_list: vec![0,1,2,3].into_iter().map(|x| (x as usize).into()).collect(),
            components: hlist![
                std::iter::repeat_n(transform_base, 4).collect(),
                std::iter::repeat_n(model_base, 4).collect(),
            ]
        };
        
        let x = test_arch.components.sculpt::<<<Tile as frunk::Generic>::Repr as ToComponentList>::Output, _>();
        
        test_arch.apply_system::<TestSystem, frunk::indices::IdentityTransMog>();
        
        // test_world.archetypes.to_mut().foldl(poly_fn![
        //     [T] | x: &mut Archetype<T> | -> () { x.apply_system::<MovementSystem, _>() },
        // ], &mut test_world.systems.head);
        // test_world.archetypes.to_mut().map(Poly(ArchMap));

        assert!(false, "YAY!");
    }
    
    // #[test]
    // fn test_ecs() {
    //     // type TestSystems = HList![MovementSystem, RenderSystem];
    //     // type TestArchetypes = HList![Archetype<<Unit as frunk::Generic>::Repr>, Archetype<<Tile as frunk::Generic>::Repr>];
    //     // 
    //     // let mut test_world : World<TestSystems, TestArchetypes> = World {
    //     //     systems: hlist![MovementSystem, RenderSystem],
    //     //     archetypes: Default::default(),
    //     // };
    //     // 
    //     // // test_world.archetypes.to_mut().foldl(poly_fn![
    //     // //     [T] | x: &mut Archetype<T> | -> () { x.apply_system::<MovementSystem, _>() },
    //     // // ], &mut test_world.systems.head);
    //     // // test_world.archetypes.to_mut().map(Poly(ArchMap));
    //     // 
    //     // assert!(false, "YAY!");
    // }
}
