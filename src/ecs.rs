use frunk::traits::ToMut;

#[cfg(test)]
mod test;
pub mod traits;
pub mod core;

use crate::ecs::traits::ComponentList;
pub use frunk::Generic as Archetype;
use frunk_core::generic::Generic;
use frunk_core::hlist::Sculptor;

pub type ComponentStorage<T> = Vec<T>;

pub trait System {
    type Components: for<'a> ToMut<'a>;
    fn update_instance(instance: <Self::Components as ToMut>::Output);
}

#[derive(Copy, Clone)]
pub struct Entity(usize);

impl Entity {
    fn new(num: usize) -> Self {
        Entity(num)
    }
}


pub struct ArchetypeStorage<ArchetypeT>
where
    ArchetypeT: Generic,
    <ArchetypeT as Generic>::Repr: ComponentList,
{
    entity_list: ComponentStorage<Entity>,
    components: <<ArchetypeT as Generic>::Repr as ComponentList>::Storage,
}

impl<ArchetypeT> ArchetypeStorage<ArchetypeT>
where
    ArchetypeT: Generic,
    <ArchetypeT as Generic>::Repr: ComponentList,
{
    pub fn new() -> Self {
        Self {
            entity_list: ComponentStorage::new(),
            components: <<ArchetypeT as Generic>::Repr as ComponentList>::new_storage(),
        }
    }
}

impl<ArchetypeT: ComponentList> ArchetypeStorage<ArchetypeT>
where
    ArchetypeT: Generic,
    <ArchetypeT as Generic>::Repr: ComponentList,
{
    pub fn push(&mut self, instance: ArchetypeT) -> Entity
    {
        let entity = Entity::new(self.entity_list.len());
        self.entity_list.push(entity);
        <ArchetypeT as Generic>::Repr::push_to_storage(&mut self.components, instance.into());

        entity
    }
}

impl<ArchetypeT> ArchetypeStorage<ArchetypeT>
where
    ArchetypeT: Generic,
    <ArchetypeT as Generic>::Repr: ComponentList,
{
    pub fn get_components<'a, SubArchetype, Indices>(&'a mut self) -> <<<SubArchetype as Generic>::Repr as ComponentList>::Storage as ToMut<'a>>::Output
    where
        SubArchetype: Generic,
        <SubArchetype as Generic>::Repr: ComponentList,
        <<ArchetypeT as Generic>::Repr as ComponentList>::Storage: ToMut<'a>,
        <<SubArchetype as Generic>::Repr as ComponentList>::Storage: ToMut<'a>,
        <<<ArchetypeT as Generic>::Repr as ComponentList>::Storage as ToMut<'a>>::Output: Sculptor<<<<SubArchetype as Generic>::Repr as ComponentList>::Storage as ToMut<'a>>::Output, Indices>,
    {
        let (x, _) = self.components.to_mut().sculpt();
        x
    }
}
