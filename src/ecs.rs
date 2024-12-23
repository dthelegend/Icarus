use frunk::hlist::{HFoldLeftable, HMappable, Sculptor};
use frunk::prelude::*;
use frunk::traits::{Func, ToMut};
use rayon::prelude::*;

#[cfg(test)]
mod test;
mod traits;

pub type ComponentStorage<T> = Vec<T>;

trait System {
    type Components: for<'a> ToMut<'a>;
    fn update_instance(instance: <Self::Components as ToMut>::Output);
}

pub struct Entity(usize);
impl From<usize> for Entity {
    fn from(value: usize) -> Self {
        Entity(value)
    }
}

#[derive(Default)]
pub struct Archetype<ComponentListT: traits::ToComponentList, EntityT: From<usize> = Entity> {
    entity_list: ComponentStorage<EntityT>,
    components: ComponentListT::Output,
}

impl<ArchetypeListT, EntityT> Archetype<ArchetypeListT, EntityT>
where
    ArchetypeListT: traits::ToComponentList,
    EntityT: From<usize>,
{
    fn apply_system<'a, SystemT: System, Indices>(&'a mut self)
    where
        Self: traits::CanApplySystem<'a, SystemT, Indices>,
    {
        traits::CanApplySystem::<'a, SystemT, Indices>::apply_system(self)
    }
}

#[allow(non_snake_case)]
#[macro_export]
macro_rules! Archetype {
    [$inner_type:ty] => {
        Archetype<<$inner_type as frunk::Generic>::Repr>
    };
}
