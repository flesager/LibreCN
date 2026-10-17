//! UE IPv4 addresses, from a network prefix. The network address, the first
//! host (the UPF's N6 side) and the broadcast address are never given.

use std::net::Ipv4Addr;

/// An address pool: one bit per address, allocated at creation.
pub struct Pool {
    network: u32,
    used: Vec<u64>,
    size: u32,
    next: u32,
}

impl Pool {
    pub fn new(network: Ipv4Addr, prefix: u8) -> Self {
        let size = 1u32 << (32 - u32::from(prefix));
        let mut pool = Pool { network: u32::from(network), used: vec![0; size.div_ceil(64) as usize], size, next: 2 };
        for reserved in [0, 1, size - 1] {
            pool.set(reserved, true);
        }
        pool
    }

    fn set(&mut self, offset: u32, used: bool) {
        let (word, bit) = ((offset / 64) as usize, offset % 64);
        if used {
            self.used[word] |= 1 << bit;
        } else {
            self.used[word] &= !(1 << bit);
        }
    }

    fn is_used(&self, offset: u32) -> bool {
        self.used[(offset / 64) as usize] & (1 << (offset % 64)) != 0
    }

    /// The first free address after the last one given; `None` when full.
    pub fn allocate(&mut self) -> Option<Ipv4Addr> {
        let offset = (0..self.size).map(|i| (self.next + i) % self.size).find(|&o| !self.is_used(o))?;
        self.set(offset, true);
        self.next = (offset + 1) % self.size;
        Some(Ipv4Addr::from(self.network + offset))
    }

    /// Gives `ip` back (ignored if outside the pool or reserved).
    pub fn release(&mut self, ip: Ipv4Addr) {
        let offset = u32::from(ip).wrapping_sub(self.network);
        if (2..self.size - 1).contains(&offset) {
            self.set(offset, false);
        }
    }

    /// The UPF's N6 address: the first host.
    pub fn gateway(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.network + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses_are_unique_until_released() {
        let mut p = Pool::new(Ipv4Addr::new(10, 45, 0, 0), 29); // 8 addresses, 5 for UEs
        assert_eq!(p.gateway(), Ipv4Addr::new(10, 45, 0, 1));
        let got: Vec<_> = std::iter::from_fn(|| p.allocate()).collect();
        assert_eq!(got.first(), Some(&Ipv4Addr::new(10, 45, 0, 2)));
        assert_eq!(got.last(), Some(&Ipv4Addr::new(10, 45, 0, 6)), "not the broadcast address");
        assert_eq!(got.len(), 5);
        p.release(Ipv4Addr::new(10, 45, 0, 4));
        p.release(Ipv4Addr::new(10, 45, 0, 1));
        p.release(Ipv4Addr::new(192, 168, 0, 1));
        assert_eq!(p.allocate(), Some(Ipv4Addr::new(10, 45, 0, 4)));
        assert_eq!(p.allocate(), None);
    }
}
