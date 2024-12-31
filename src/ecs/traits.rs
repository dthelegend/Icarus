use frunk::hlist::{HCons, HList, HNil, Sculptor};
use frunk::ToMut;
use frunk::HList;
use rayon::prelude::*;

// Seal these traits
trait Sealed {}

impl Sealed for HNil {}
impl<T, U> Sealed for HCons<T, U> {}

/// A heterogeneous list of components storages
pub trait ComponentList: HList + Sealed {}

impl ComponentList for HNil {}
impl<HeadT, TailT: ComponentList> ComponentList for HCons<super::ComponentStorage<HeadT>, TailT> {}

/// A trait to convert a heterogeneous list into a `ComponentList`
pub trait ToComponentList: HList + Sealed {
    type Output: ComponentList;
}

impl ToComponentList for HNil {
    type Output = HNil;
}

impl<HeadT, TailT: ToComponentList> ToComponentList for HCons<HeadT, TailT> {
    type Output = HCons<super::ComponentStorage<HeadT>, <TailT as ToComponentList>::Output>;
}

/// A trait to provide a way to get a parallel iterator over a component storage
pub trait IntoParIter: HList + Sealed {
    type Item: HList + Send;

    fn get_parallel_mut(self) -> impl IndexedParallelIterator<Item = Self::Item>;
}

impl<HeadT> IntoParIter for HCons<HeadT, HNil>
where
    HeadT: IntoParallelIterator<Iter: IndexedParallelIterator>,
    <HeadT as IntoParallelIterator>::Item: Send,
{
    type Item = HList![<HeadT as IntoParallelIterator>::Item];
    fn get_parallel_mut(self) -> impl IndexedParallelIterator<Item = Self::Item> {
        rayon::iter::repeat(HNil)
            .zip(self.head.into_par_iter())
            .map(|(hnil, x)| hnil.prepend(x))
    }
}

impl<HeadT, TailT: IntoParIter> IntoParIter for HCons<HeadT, TailT>
where
    HeadT: IntoParallelIterator<Iter: IndexedParallelIterator>,
    <HeadT as IntoParallelIterator>::Item: Send,
{
    type Item = HList![<HeadT as IntoParallelIterator>::Item, ...<TailT as IntoParIter>::Item];
    fn get_parallel_mut(self) -> impl IndexedParallelIterator<Item = Self::Item> {
        self.tail
            .get_parallel_mut()
            .zip(self.head.into_par_iter())
            .map(|(tail, head)| tail.prepend(head))
    }
}

pub trait CanApplySystem<'a, SystemT: super::System, Indices> {
    fn apply_system(&'a mut self);
}

impl <'a, SystemT: super::System, IndicesHead, IndicesTail, ArchetypeListT, EntityT> CanApplySystem<'a, SystemT, HCons<IndicesHead, IndicesTail>> for super::Archetype<ArchetypeListT, EntityT>
where
    ArchetypeListT: ToComponentList,
    EntityT: From<usize>,
    ArchetypeListT: ToComponentList + ToMut<'a>,
    <ArchetypeListT as ToComponentList>::Output: ToMut<'a>,
    <<ArchetypeListT as ToComponentList>::Output as ToMut<'a>>::Output: Sculptor<<<<SystemT as super::System>::Components as ToComponentList>::Output as ToMut<'a>>::Output, HCons<IndicesHead, IndicesTail>>,
    SystemT: super::System,
    <SystemT as super::System>::Components: ToMut<'a> + ToComponentList,
    <<SystemT as super::System>::Components as ToComponentList>::Output: ToMut<'a>,
    <<<SystemT as super::System>::Components as ToComponentList>::Output as ToMut<'a>>::Output: IntoParIter<Item = <<SystemT as super::System>::Components as ToMut<'a>>::Output>,
{
    fn apply_system(&'a mut self) {
        let (resolved_components, _) : (<<<SystemT as super::System>::Components as ToComponentList>::Output as ToMut<'a>>::Output, _) = self.components.to_mut().sculpt();
        resolved_components.get_parallel_mut().for_each(SystemT::update_instance);
    }
}
