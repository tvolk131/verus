pub open spec fn valid(self) -> bool {
    self.len as int <= 64 * self.data@.len()
}

pub open spec fn live(self, index: int) -> bool {
    self.data@[index / 64] & (1u64 << ((index % 64) as u64)) != 0
}
