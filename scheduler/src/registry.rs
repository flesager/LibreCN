//! Table of live EOs, shared by every thread of a platform.
//!
//! Lookups take a per-slot read lock; platforms cache the result per worker
//! so the message path normally never touches the registry.

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use crate::core::EoCore;
use crate::header::system;
use crate::{EoConfig, EoId, Error, ExecutionObject, MsgHeader, Result};

struct Slot {
    next_generation: AtomicU32,
    entry: RwLock<Option<Arc<EoCore>>>,
}

pub(crate) struct Registry {
    slots: Box<[Slot]>,
    free: Mutex<Vec<u32>>,
    /// EOs added and neither stopped nor faulted.
    alive: AtomicUsize,
}

impl Registry {
    pub(crate) fn new(capacity: u32) -> Self {
        Registry {
            slots: (0..capacity)
                .map(|_| Slot { next_generation: AtomicU32::new(0), entry: RwLock::new(None) })
                .collect(),
            free: Mutex::new((0..capacity).rev().collect()),
            alive: AtomicUsize::new(0),
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
        let core = EoCore::create(id, cfg, pool, eo).inspect_err(|_| self.free.lock().unwrap().push(index))?;
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

    /// Ids of the EOs not yet stopped or faulted.
    pub(crate) fn alive_ids(&self) -> Vec<EoId> {
        (0..self.capacity())
            .filter_map(|i| self.by_index(i))
            .filter(|core| !core.is_closed())
            .map(|core| core.id)
            .collect()
    }
}
