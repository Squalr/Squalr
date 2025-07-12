use crate::dependency_injection::dependency_container::DependencyContainer;
use anyhow::Result;
use std::{any::type_name, collections::HashSet, sync::Arc};

pub trait DepTuple: Sized {
    fn missing_dependencies(container: &DependencyContainer) -> HashSet<String>;
    fn resolve_from(container: &DependencyContainer) -> Result<Self>;
}

fn key<T: 'static>() -> String {
    type_name::<T>().to_string()
}

// Resolve for 1-item list.
impl<A> DepTuple for Arc<A>
where
    A: Send + Sync + 'static,
{
    fn missing_dependencies(container: &DependencyContainer) -> HashSet<String> {
        let mut set = HashSet::new();
        if container.get_existing::<A>().is_err() {
            set.insert(key::<A>());
        }
        set
    }

    fn resolve_from(container: &DependencyContainer) -> Result<Self> {
        Ok(container.get_existing::<A>()?)
    }
}

// Resolve for 2-item list.
impl<A, B> DepTuple for (Arc<A>, Arc<B>)
where
    A: Send + Sync + 'static,
    B: Send + Sync + 'static,
{
    fn missing_dependencies(container: &DependencyContainer) -> HashSet<String> {
        let mut set = HashSet::new();
        if container.get_existing::<A>().is_err() {
            set.insert(key::<A>());
        }
        if container.get_existing::<B>().is_err() {
            set.insert(key::<B>());
        }
        set
    }

    fn resolve_from(container: &DependencyContainer) -> Result<Self> {
        Ok((container.get_existing::<A>()?, container.get_existing::<B>()?))
    }
}

// Resolve for 3-item list.
impl<A, B, C> DepTuple for (Arc<A>, Arc<B>, Arc<C>)
where
    A: Send + Sync + 'static,
    B: Send + Sync + 'static,
    C: Send + Sync + 'static,
{
    fn missing_dependencies(container: &DependencyContainer) -> HashSet<String> {
        let mut set = HashSet::new();
        if container.get_existing::<A>().is_err() {
            set.insert(key::<A>());
        }
        if container.get_existing::<B>().is_err() {
            set.insert(key::<B>());
        }
        if container.get_existing::<C>().is_err() {
            set.insert(key::<C>());
        }
        set
    }

    fn resolve_from(container: &DependencyContainer) -> Result<Self> {
        Ok((container.get_existing::<A>()?, container.get_existing::<B>()?, container.get_existing::<C>()?))
    }
}

// Resolve for 4-item list.
impl<A, B, C, D> DepTuple for (Arc<A>, Arc<B>, Arc<C>, Arc<D>)
where
    A: Send + Sync + 'static,
    B: Send + Sync + 'static,
    C: Send + Sync + 'static,
    D: Send + Sync + 'static,
{
    fn missing_dependencies(container: &DependencyContainer) -> HashSet<String> {
        let mut set = HashSet::new();
        if container.get_existing::<A>().is_err() {
            set.insert(key::<A>());
        }
        if container.get_existing::<B>().is_err() {
            set.insert(key::<B>());
        }
        if container.get_existing::<C>().is_err() {
            set.insert(key::<C>());
        }
        if container.get_existing::<D>().is_err() {
            set.insert(key::<D>());
        }
        set
    }

    fn resolve_from(container: &DependencyContainer) -> Result<Self> {
        Ok((
            container.get_existing::<A>()?,
            container.get_existing::<B>()?,
            container.get_existing::<C>()?,
            container.get_existing::<D>()?,
        ))
    }
}
