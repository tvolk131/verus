from matrix import *
src=base.PRE+'''verus! {
pub fn relation(a:&NonZero<u64>,b:&NonZero<u64>)->(r:Ordering)
 ensures (r==Ordering::Less) == (a@ < b@),
         (r==Ordering::Equal) == (a@ == b@),
         (r==Ordering::Greater) == (a@ > b@),
 {a.cmp(b)}
}'''
expect('review-three-relations',src,ok=True,verified=2,errors=0)
