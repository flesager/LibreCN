//! Table of live EOs, shared by every thread of a platform.
//!
//! Lookups take a per-slot read lock; platforms cache the result per worker
//! so the message path normally never touches the registry.

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use crate::core::EoCore;
use crate::header::system;
use crate::{EoConfig, EoId, EoInfo, Error, ExecutionObject, MsgHeader, Result};

struct Slot {
    next_generation: AtomicU32,
    entry: RwLock<Option<Arc<EoCore>>>,
}

pub(crate) struct Registry {
    slots: Box<[Slot]>,
    free: Mutex<Vec<u32>>,
    /// EOs added and neither stopped nor faulted.
    alive: AtomicUsize,
    created: AtomicU64,
}

impl Registry {
    pub(crate) fn new(capacity: u32) -> Self {
        Registry {
            slots: (0..capacity)
                .map(|_| Slot { next_generation: AtomicU32::new(0), entry: RwLock::new(None) })
                .collect(),
            free: Mutex::new((0..capacity).rev().collect()),
            alive: AtomicUsize::new(0),
            created: AtomicU64::new(0),
        }
    }

    pub(crate) fn capacity(&self) -> u32 {
        self.slots.len() as u32
    }

    /// Builds an EO in a free slot (allocating on the calling thread) and
    /// publishes it with `EO_START` queued. Returns it, and whether the caller
    /// must schedule it.
    pub(crate) fn create(
        &self,
        cfg: &EoConfig,
        pool: u16,
        eo: Box<dyn ExecutionObject>,
    ) -> Result<(Arc<EoCore>, bool)> {
        let index = self.free.lock().unwrap().pop().ok_or(Error::RegistryFull)?;
        let slot = &self.slots[index as usize];
        let id = EoId::new(index, slot.next_generation.load(Ordering::Relaxed));
        let created = self.created.fetch_add(1, Ordering::Relaxed);
        let core = EoCore::create(id, created, cfg, pool, eo).inspect_err(|_| self.free.lock().unwrap().push(index))?;
        let must_schedule = core.post_system(MsgHeader::system(system::EO_START, id, 0))?;
        *slot.entry.write().unwrap() = Some(core.clone());
        self.alive.fetch_add(1, Ordering::Relaxed);
        Ok((core, must_schedule))
    }

    /// The EO with exactly this id, stopped or faulted ones included until removed.
    pub(crate) fn get(&self, id: EoId) -> Option<Arc<EoCore>> {
        self.by_index(id.index()).filter(|core| core.id == id)
    }

    /// Whatever EO occupies slot `index`.
    pub(crate) fn by_index(&self, index: u32) -> Option<Arc<EoCore>> {
        self.slots.get(index as usize)?.entry.read().unwrap().clone()
    }

    /// The EO stopped running (stopped or faulted).
    pub(crate) fn retired(&self) {
        self.alive.fetch_sub(1, Ordering::Relaxed);
    }

    /// Frees the slot of a stopped EO; its id goes stale.
    pub(crate) fn remove(&self, id: EoId) {
        let slot = &self.slots[id.index() as usize];
        let mut entry = slot.entry.write().unwrap();
        if entry.as_ref().is_some_and(|core| core.id == id) {
            *entry = None;
            slot.next_generation.store(id.generation().wrapping_add(1), Ordering::Relaxed);
            drop(entry);
            self.free.lock().unwrap().push(id.index());
        }
    }

    pub(crate) fn alive(&self) -> usize {
        self.alive.load(Ordering::Relaxed)
    }

    /// Live and faulted EOs; `pool_name` resolves pool indices.
    pub(crate) fn infos<'a>(&self, pool_name: impl Fn(u16) -> &'a str) -> Vec<EoInfo> {
        (0..self.capacity())
            .filter_map(|i| self.by_index(i))
            .filter(|core| core.fault().is_some() || !core.is_closed())
            .map(|core| EoInfo {
                id: core.id,
                name: core.name.to_string(),
                pool: pool_name(core.pool).to_string(),
                alive: core.fault().is_none(),
            })
            .collect()
    }

    /// First live EO with this name.
    pub(crate) fn find(&self, name: &str) -> Option<EoId> {
        (0..self.capacity())
            .filter_map(|i| self.by_index(i))
            .find(|core| &*core.name == name && core.check_alive().is_ok())
            .map(|core| core.id)
    }

    /// Ids of the EOs not yet stopped or faulted.
    /// In creation order.
    pub(crate) fn alive_ids(&self) -> Vec<EoId> {
        let mut alive: Vec<_> =
            (0..self.capacity()).filter_map(|i| self.by_index(i)).filter(|core| !core.is_closed()).collect();
        alive.sort_by_key(|core| core.created);
        alive.into_iter().map(|core| core.id).collect()
    }
}
