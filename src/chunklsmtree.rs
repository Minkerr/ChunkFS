use std::io;
use crate::{ChunkHash, Database};
use crate::lsmtree::LsmTree;

impl<Hash: ChunkHash + Ord + , V: Clone> Database<Hash, V> for LsmTree<Hash, V> {
    fn insert(&mut self, key: Hash, value: V) -> io::Result<()> {
        let _ = self.insert(key, value);
        Ok(())
    }

    fn get(&self, key: &Hash) -> io::Result<V> {
        self.get(key)
    }

    fn remove(&mut self, _key: &Hash) {
        todo!()
    }

    fn contains(&self, key: &Hash) -> bool {
        self.get(key).is_ok()
    }
}