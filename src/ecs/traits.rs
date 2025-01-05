use crate::ecs::ComponentStorage;
use frunk::hlist::{HCons, HList, HNil};
use frunk::HList;
use frunk_core::hlist;
use rayon::prelude::*;

// Seal these traits
trait Sealed {}

impl Sealed for HNil {}
impl<T, U> Sealed for HCons<T, U> {}

/// A heterogeneous list of components storages
pub trait ComponentStorageList: HList + Sealed {
}

impl <HeadT, TailT: ComponentStorageList> ComponentStorageList for HCons<ComponentStorage<HeadT>, TailT> {
}

impl <HeadT> ComponentStorageList for HCons<ComponentStorage<HeadT>, HNil> {
}

/// A trait to convert a heterogeneous list into a `ComponentList`
pub trait ComponentList: HList + Sealed {
    type Storage: ComponentStorageList;

    fn new_storage() -> Self::Storage;
    
    fn push_to_storage(this: &mut Self::Storage, instance: Self);

    fn swap_remove_from_storage(this: &mut Self::Storage, index: usize) -> Self;
}

impl <HeadT, TailT: ComponentList> ComponentList for HCons<HeadT, TailT> {
    type Storage = HCons<ComponentStorage<HeadT>, <TailT as ComponentList>::Storage>;
    
    fn new_storage() -> Self::Storage {
        hlist![ComponentStorage::new(), ...TailT::new_storage()]
    }
    
    fn push_to_storage(storage: &mut Self::Storage, instance: Self) {
        storage.head.push(instance.head);
        TailT::push_to_storage(&mut storage.tail, instance.tail);
    }

    fn swap_remove_from_storage(this: &mut Self::Storage, index: usize) -> Self {
        hlist![this.head.swap_remove(index), ...TailT::swap_remove_from_storage(&mut this.tail, index)]
    }
}

impl <HeadT> ComponentList for HCons<HeadT, HNil> {
    type Storage = HCons<ComponentStorage<HeadT>, HNil>;

    fn new_storage() -> Self::Storage {
        hlist![ComponentStorage::new()]
    }

    fn push_to_storage(this: &mut Self::Storage, instance: Self) {
        this.head.push(instance.head);
    }

    fn swap_remove_from_storage(this: &mut Self::Storage, index: usize) -> Self {
        hlist![this.head.swap_remove(index)]
    }
}

/// A trait to provide a way to get a parallel iterator over a component storage
pub trait ToParIter: HList + Sealed {
    type Item: HList + Send;

    fn to_par_iter(self) -> impl IndexedParallelIterator<Item = Self::Item>;
}

impl<HeadT> ToParIter for HCons<HeadT, HNil>
where
    HeadT: IntoParallelIterator<Iter: IndexedParallelIterator>,
    <HeadT as IntoParallelIterator>::Item: Send,
{
    type Item = HList![<HeadT as IntoParallelIterator>::Item];
    fn to_par_iter(self) -> impl IndexedParallelIterator<Item = Self::Item> {
        rayon::iter::repeat(HNil)
            .zip(self.head.into_par_iter())
            .map(|(hnil, x)| hnil.prepend(x))
    }
}

impl<HeadT, TailT: ToParIter> ToParIter for HCons<HeadT, TailT>
where
    HeadT: IntoParallelIterator<Iter: IndexedParallelIterator>,
    <HeadT as IntoParallelIterator>::Item: Send,
{
    type Item = HList![<HeadT as IntoParallelIterator>::Item, ...<TailT as ToParIter>::Item];
    fn to_par_iter(self) -> impl IndexedParallelIterator<Item = Self::Item> {
        self.tail
            .to_par_iter()
            .zip(self.head.into_par_iter())
            .map(|(tail, head)| tail.prepend(head))
    }
}

/// A trait to provide a way to get a parallel iterator over a component storage
pub trait ToIter: HList + Sealed {
    type Item: HList + Send;

    fn to_iter(self) -> impl Iterator<Item = Self::Item>;
}

impl<HeadT> ToIter for HCons<HeadT, HNil>
where
    HeadT: IntoIterator,
    <HeadT as IntoIterator>::Item: Send,
{
    type Item = HList![<HeadT as IntoIterator>::Item];
    fn to_iter(self) -> impl Iterator<Item = Self::Item> {
        core::iter::repeat(HNil)
            .zip(self.head.into_iter())
            .map(|(hnil, x)| hnil.prepend(x))
    }
}

impl<HeadT, TailT: ToIter> ToIter for HCons<HeadT, TailT>
where
    HeadT: IntoIterator,
    <HeadT as IntoIterator>::Item: Send,
{
    type Item = HList![<HeadT as IntoIterator>::Item, ...<TailT as ToIter>::Item];
    fn to_iter(self) -> impl Iterator<Item = Self::Item> {
        self.tail
            .to_iter()
            .zip(self.head.into_iter())
            .map(|(tail, head)| tail.prepend(head))
    }
}