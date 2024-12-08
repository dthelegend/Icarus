use std::marker::PhantomData;
use nalgebra::{Quaternion, SVector};

struct Transform {
    position: SVector<f32, 3>,
    rotation: Quaternion<f32>,
    scale: SVector<f32, 3>,
    marker: PhantomData<AllComponentTS>,
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
trait ComponentList {}

struct ComponentListNil();
impl ComponentList for ComponentListNil {}

struct ComponentListCons<Head, Tail : ComponentList> {
    head: Vec<Head>,
    tail: Tail,
}

trait Subset<T: ComponentList> {}

impl <Head, Tail : ComponentList> ComponentList for ComponentListCons<Head, Tail> {}

#[allow(non_snake_case)]
macro_rules! ComponentList {
    [] => { ComponentListNil };
    [ $headT:ty ] => { ComponentListCons<$headT, ComponentList![]> };
    [$headT:ty $(, $tailTS:ty)+] => { ComponentListCons<$headT, ComponentList![$($tailTS),+]> };
}


struct Archetype<ComponentListT: ComponentList, EntityT : Into<usize> = Entity> {
    entity_list: Vec<EntityT>,
    components: ComponentListT
}

impl <ComponentListT: ComponentList, EntityT : Into<usize>> Archetype<ComponentListT, EntityT> {
}

type UnitArchetype = Archetype<ComponentList!(Transform, DeltaTransform, Model)>;
type TileArchetype = Archetype<ComponentList!(Transform, Model)>;

trait SystemList<AllComponentTS: ComponentList> {}
struct SystemListNil;
struct SystemListCons<HeadT : System<>, TailT : SystemList> {
    head: HeadT,
    tail: TailT,
}

// World also uses the HList pattern, but one is expected to construct this with a builder style pattern
trait World<AllComponentTS: ComponentList> {}

// Base aspects for all systems
struct WorldNil<AllComponentTS: ComponentList, SystemListT: SystemList<AllComponentTS>> {
    system_list: SystemListT,
    _all_components: PhantomData<AllComponentTS>,
}
struct WorldCons<AllComponentTS: ComponentList, HeadT: ComponentList, Tail : World<AllComponentTS>> {
    head: Archetype<HeadT>,
    tail: Tail,
    _all_components: PhantomData<AllComponentTS>,
}

impl <AllComponentTS: ComponentList, SystemListT: SystemList<AllComponentTS>> World<AllComponentTS> for WorldNil<AllComponentTS, SystemListT> {
}

impl <AllComponentTS: ComponentList, SubsetTS: Subset<AllComponentTS>, Tail: World<AllComponentTS>> World<AllComponentTS> for WorldCons<AllComponentTS, SubsetTS, Tail> {}

trait System {
    type Components: ComponentList;
    fn update(&mut self);
}

struct MovementSystem;

impl System for MovementSystem {
    type Components = ComponentList!(Transform, DeltaTransform);
    fn update(&mut self) {
        todo!()
    }
}

struct RenderSystem;

impl System for RenderSystem {
    type Components = ComponentList!(Transform, Model);

    fn update(&mut self) {
    }
}

struct ECS {

}
