"""Source adaptation is explicit, bounded and fails if pinned bodies drift."""
import unittest
from eval_verus_infer_alephbft_units import method


class SourceAdaptationTests(unittest.TestCase):
    def test_hash_and_getter_bodies_are_preserved_exactly(self):
        for text,marker,typ in [
            ('fn hash(x: &[u8]) -> Self::Hash {let mut h=DefaultHasher::new();h.write(x);h.finish().to_ne_bytes()}', 'fn hash(', 'Self::Hash'),
            ('pub fn coord(&self) -> UnitCoord {self.pre_unit.coord}', 'pub fn coord(', 'UnitCoord'),
        ]:
            result=method(text,marker,typ,'true')
            self.assertEqual(text[text.index('{'):],result[result.index('{'):])

    def test_ghost_and_lock_adaptations_are_enumerated(self):
        text='fn hash(&self) -> H::Hash {let value=*self.hash.read();match value{Some(h)=>h,None=>{let h=self.using_encoded(H::hash);*self.hash.write()=Some(h);h}}}'
        changes=[('*self.hash.read()','self.hash.read_copy()'),('*self.hash.write()=Some(h);','self.hash.replace(Some(h));')]
        generated=method(text,'fn hash(','H::Hash','result==self.hash_value()',prefix='proof{use_type_invariant(self);}',replacements=changes)
        body=text[text.index('{')+1:]
        for old,new in changes:body=body.replace(old,new)
        self.assertEqual(generated[generated.index('{')+1:],'proof{use_type_invariant(self);}'+body)

    def test_missing_or_ambiguous_targets_are_rejected(self):
        with self.assertRaises(ValueError):method('fn hash()->u8{0}','fn hash(','u64','true')
        for body in ['0','read();read();0']:
            with self.assertRaises(ValueError):
                method('fn hash()->u8{'+body+'}','fn hash(','u8','true',replacements=[('read()','copy_read()')])


if __name__=='__main__':unittest.main()
