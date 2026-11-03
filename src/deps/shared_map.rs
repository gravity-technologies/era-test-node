//! A map whose copies share their allocation until one copy is changed.
//!
//! Every write detaches shared data first. Holding a cloned map therefore
//! captures a stable value even when another owner changes its copy.
use std::{
    collections::HashMap,
    fmt,
    hash::Hash,
    iter::FromIterator,
    ops::{Deref, DerefMut},
    sync::Arc,
};

pub struct SharedMap<K, V>(Arc<HashMap<K, V>>);

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for SharedMap<K, V> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<K, V> Clone for SharedMap<K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<K, V> Default for SharedMap<K, V> {
    fn default() -> Self {
        Self(Arc::new(HashMap::new()))
    }
}
impl<K, V> Deref for SharedMap<K, V> {
    type Target = HashMap<K, V>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<K: Clone + Eq + Hash, V: Clone> DerefMut for SharedMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}
impl<K, V> From<HashMap<K, V>> for SharedMap<K, V> {
    fn from(map: HashMap<K, V>) -> Self {
        Self(Arc::new(map))
    }
}
impl<K: Eq + Hash, V> FromIterator<(K, V)> for SharedMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        HashMap::from_iter(iter).into()
    }
}
impl<K: Eq + Hash, V: PartialEq> PartialEq for SharedMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cloned_map_detaches_on_all_mutable_access() {
        let mut original: SharedMap<u32, Vec<u8>> = vec![(1, vec![2])].into_iter().collect();
        let mut copy = original.clone();
        assert!(Arc::ptr_eq(&original.0, &copy.0));
        copy.get_mut(&1).unwrap().push(3);
        original.insert(2, vec![4]);
        assert_eq!(original.get(&1).unwrap(), &vec![2]);
        assert_eq!(copy.get(&1).unwrap(), &vec![2, 3]);
        assert_eq!(copy.get(&2), None);
    }
    #[test]
    fn mutable_hashmap_paths_preserve_all_retained_versions() {
        let original: SharedMap<u32, Vec<u8>> =
            vec![(1, vec![10]), (2, vec![20])].into_iter().collect();
        let mut branch = original.clone();
        branch.entry(1).and_modify(|value| value.push(11));
        let checkpoint = branch.clone();
        for (_, value) in branch.iter_mut() {
            value.push(99);
        }
        assert_eq!(original.get(&1), Some(&vec![10]));
        assert_eq!(checkpoint.get(&1), Some(&vec![10, 11]));
        assert_eq!(branch.get(&1), Some(&vec![10, 11, 99]));
        for value in branch.values_mut() {
            value.clear();
        }
        assert_eq!(checkpoint.get(&2), Some(&vec![20]));
        branch.retain(|key, _| *key == 1);
        assert_eq!(checkpoint.len(), 2);
        let drained: Vec<_> = branch.drain().collect();
        assert_eq!(drained.len(), 1);
        assert!(branch.is_empty());
        assert_eq!(original.len(), 2);
        assert_eq!(checkpoint.len(), 2);
        branch = checkpoint.clone();
        branch.clear();
        assert_eq!(checkpoint.len(), 2);
        branch = checkpoint.clone();
        branch.remove(&1);
        assert_eq!(checkpoint.get(&1), Some(&vec![10, 11]));
        branch.reserve(100);
        branch.shrink_to_fit();
        assert_eq!(checkpoint.len(), 2);
    }

    #[test]
    fn restored_version_can_branch_without_changing_older_values_or_bytecode() {
        let original: SharedMap<u32, Vec<u8>> = vec![(1, vec![10, 20])].into_iter().collect();
        let checkpoint = original.clone();
        let mut abandoned_branch = original.clone();
        abandoned_branch.get_mut(&1).unwrap()[0] = 99;
        abandoned_branch.insert(2, vec![30]);
        let mut restored = checkpoint.clone();
        restored.entry(1).and_modify(|value| value.push(40));
        assert_eq!(original.get(&1), Some(&vec![10, 20]));
        assert_eq!(checkpoint.get(&1), Some(&vec![10, 20]));
        assert_eq!(abandoned_branch.get(&1), Some(&vec![99, 20]));
        assert_eq!(restored.get(&1), Some(&vec![10, 20, 40]));
        assert_eq!(restored.get(&2), None);
    }

    #[test]
    fn debug_output_keeps_hashmap_format() {
        let map: SharedMap<u32, Vec<u8>> = vec![(1, vec![2])].into_iter().collect();
        assert_eq!(format!("{:?}", map), format!("{:?}", &*map));
        assert_eq!(format!("{:#?}", map), format!("{:#?}", &*map));
    }
}
