mod g3_scalar171;
use vstd::prelude::*;
use vstd::raw_ptr::{PointsTo, MemContents, ptr_mut_write};
verus! {
pub fn generated_native_store154<Frame>(
    mir_local_19: *mut u8, mir_local_18: u8, other_pointer: *mut u8,
    Tracked(perm): Tracked<&mut PointsTo<u8>>,
    Tracked(frame): Tracked<&mut PointsTo<Frame>>,
)
    requires
        old(perm).ptr() == mir_local_19,
        old(perm).ptr()@.addr as int + 1 <= old(frame).ptr()@.addr as int
          || old(frame).ptr()@.addr as int + vstd::layout::size_of::<Frame>() <= old(perm).ptr()@.addr as int,
        other_pointer@.addr != mir_local_19@.addr,
    ensures
        final(perm).ptr() == mir_local_19,
        final(perm).ptr()@.provenance == old(perm).ptr()@.provenance,
        final(perm).ptr()@.addr == old(perm).ptr()@.addr,
        final(perm).opt_value() == MemContents::Init(mir_local_18),
        final(frame)@ == old(frame)@,
        vstd::layout::size_of::<u8>() == 1,
    opens_invariants none
    no_unwind
{
    ptr_mut_write::<u8>(mir_local_19, Tracked(perm), mir_local_18);
    
}
}

verus! {
use vstd::raw_ptr::{PointsToRaw, Dealloc};
use vstd::layout::valid_layout;
use g3_scalar171::{
    OriginalLayoutValues171, original_u8_value_relations171,
    original_u8_normal_guards171, observed_header156_171,
};
broadcast use {vstd::layout::group_layout_axioms, vstd::layout::align_of_u8};
pub const PAYLOAD_OFFSET: usize = 16;
pub const ALLOCATION_ALIGN: usize = 8;
pub open spec fn allocation_size156(len: usize) -> int {
    PAYLOAD_OFFSET as int + ((len as int + ALLOCATION_ALIGN as int - 1) / ALLOCATION_ALIGN as int) * ALLOCATION_ALIGN as int
}
pub fn generated_allocate156(len: usize) -> (result: (*mut u8, Tracked<PointsToRaw>, Tracked<PointsToRaw>, Tracked<Dealloc>))
    requires
        len <= isize::MAX as int - PAYLOAD_OFFSET as int - (ALLOCATION_ALIGN as int - 1),
    ensures
        result.1@.is_range(result.0@.addr as int, PAYLOAD_OFFSET as int),
        result.2@.is_range(result.0@.addr as int + PAYLOAD_OFFSET as int, allocation_size156(len) - PAYLOAD_OFFSET as int),
        result.1@.provenance() == result.0@.provenance,
        result.2@.provenance() == result.0@.provenance,
        result.3@.addr() == result.0@.addr,
        result.3@.size() == allocation_size156(len),
        result.3@.align() == ALLOCATION_ALIGN,
        result.3@.provenance() == result.0@.provenance,
        result.0@.addr as int + allocation_size156(len) <= usize::MAX as int + 1,
        allocation_size156(len) >= PAYLOAD_OFFSET as int + len as int,
        len == 0 ==> result.2@.dom() == Set::<int>::empty(),
    opens_invariants none
{
    let size = PAYLOAD_OFFSET + ((len + ALLOCATION_ALIGN - 1) / ALLOCATION_ALIGN) * ALLOCATION_ALIGN;
    proof {
        reveal(vstd::arithmetic::power2::is_pow2);
        assert(vstd::arithmetic::power2::is_pow2(ALLOCATION_ALIGN as int)) by (compute);
        assert(valid_layout(size, ALLOCATION_ALIGN));
    }
    let (base, Tracked(raw), Tracked(dealloc)) = vstd::raw_ptr::allocate(size, 8usize);
    let ghost start = base@.addr as int;
    let tracked (header, payload) = raw.split(Set::range(start, start + PAYLOAD_OFFSET as int));
    proof {
        assert(header.is_range(start, PAYLOAD_OFFSET as int));
        assert(payload.dom() =~= Set::range(start + PAYLOAD_OFFSET as int, start + size as int));
        if len == 0 { assert(payload.dom() =~= Set::<int>::empty()); }
    }
    (base, Tracked(header), Tracked(payload), Tracked(dealloc))
}
pub fn generated_allocation_to_store156(len: usize, index: usize, byte: u8, Ghost(original_layout): Ghost<OriginalLayoutValues171>) -> (result: (*mut u8, *mut u8, Tracked<PointsTo<u8>>, Tracked<PointsToRaw>, Tracked<PointsToRaw>, Tracked<Dealloc>))
    requires
        original_layout.len == len,
        g3_scalar171::original_u8_value_relations171(original_layout),
        g3_scalar171::original_u8_normal_guards171(original_layout),
        g3_scalar171::observed_header156_171(original_layout),
        index < len,

        
    ensures
        result.1@.addr == result.0@.addr as int + PAYLOAD_OFFSET as int + index as int,
        result.1@.provenance == result.0@.provenance,
        result.2@.ptr() == result.1,
        result.2@.opt_value() == MemContents::Init(byte),
        result.3@.is_range(result.0@.addr as int, PAYLOAD_OFFSET as int),
        result.4@.dom() == Set::range(result.0@.addr as int + PAYLOAD_OFFSET as int, result.0@.addr as int + allocation_size156(len)).difference(Set::range(result.1@.addr as int, result.1@.addr as int + 1)),
        result.3@.provenance() == result.0@.provenance,
        result.4@.provenance() == result.0@.provenance,
        result.5@.addr() == result.0@.addr,
        result.5@.size() == allocation_size156(len),
        result.5@.align() == ALLOCATION_ALIGN,
        result.5@.provenance() == result.0@.provenance,
        result.3@.dom().disjoint(result.4@.dom()),
        result.3@.dom().disjoint(Set::range(result.1@.addr as int, result.1@.addr as int + 1)),
        result.4@.dom().disjoint(Set::range(result.1@.addr as int, result.1@.addr as int + 1)),
        result.3@.dom().union(result.4@.dom()).union(Set::range(result.1@.addr as int, result.1@.addr as int + 1))
            == Set::range(result.0@.addr as int, result.0@.addr as int + allocation_size156(len)),
    opens_invariants none
{
    proof {
        g3_scalar171::derive_original_layout171(len, original_layout);
        g3_scalar171::derive_original_cell171(len, index, original_layout);
        assert(original_layout.padded_size as int == allocation_size156(len));
    }
    let (base, Tracked(header), Tracked(payload), Tracked(dealloc)) = generated_allocate156(len);
    proof { g3_scalar171::derive_original_address171(base@.addr, len, index, original_layout); }
    let base_address = base.addr();
    let cell_address = base_address + 16usize + index;
    let size = PAYLOAD_OFFSET + ((len + ALLOCATION_ALIGN - 1) / ALLOCATION_ALIGN) * ALLOCATION_ALIGN;
    let actual_pointer = base.with_addr(cell_address);
    let ghost cell_start = cell_address as int;
    let tracked (cell_raw, complement) = payload.split(Set::range(cell_start, cell_start + 1));
    let tracked mut cell = cell_raw.into_typed::<u8>(cell_address);
    let tracked (header_byte_raw, header_tail) = header.split(Set::range(base_address as int, base_address as int + 1));
    let tracked mut header_byte = header_byte_raw.into_typed::<u8>(base_address);
    let ghost header_before = header_byte@;
    proof {
        assert(cell.ptr()@.addr == actual_pointer@.addr);
        assert(cell.ptr() == actual_pointer);
        assert(header_byte.is_uninit());
    }
    let mir_local_19 = actual_pointer;
    let mir_local_18 = byte;
    generated_native_store154::<u8>(mir_local_19, mir_local_18, base, Tracked(&mut cell), Tracked(&mut header_byte));
    proof { assert(header_byte@ == header_before); }
    let tracked header_raw = header_byte.into_raw().join(header_tail);
    proof { assert(header_raw.dom() =~= Set::range(base_address as int, base_address as int + PAYLOAD_OFFSET as int)); }
    (base, actual_pointer, Tracked(cell), Tracked(header_raw), Tracked(complement), Tracked(dealloc))
}
pub fn generated_reclaim156(len: usize, index: usize, byte: u8, Ghost(original_layout): Ghost<OriginalLayoutValues171>)
    requires
        original_layout.len == len,
        original_u8_value_relations171(original_layout),
        original_u8_normal_guards171(original_layout),
        observed_header156_171(original_layout),
        index < len,
        
    opens_invariants none
{
    let (base, cell_pointer, Tracked(mut cell), Tracked(header), Tracked(complement), Tracked(dealloc)) = generated_allocation_to_store156(len, index, byte, Ghost(original_layout));
    let size = PAYLOAD_OFFSET + ((len + ALLOCATION_ALIGN - 1) / ALLOCATION_ALIGN) * ALLOCATION_ALIGN;
    proof { cell.leak_contents(); }
    let tracked raw = cell.into_raw().join(complement).join(header);
    proof { assert(raw.dom() =~= Set::range(base@.addr as int, base@.addr as int + size as int)); }
    vstd::raw_ptr::deallocate(base, size, ALLOCATION_ALIGN, Tracked(raw), Tracked(dealloc));
}
pub fn generated_zero_length156()
    opens_invariants none
{
    let (base, Tracked(header), Tracked(payload), Tracked(dealloc)) = generated_allocate156(0);
    proof {
        assert(payload.dom() == Set::<int>::empty());
        assert(dealloc.size() == PAYLOAD_OFFSET);
        assert(PAYLOAD_OFFSET > 0);
    }
    let tracked raw = header.join(payload);
    vstd::raw_ptr::deallocate(base, PAYLOAD_OFFSET, ALLOCATION_ALIGN, Tracked(raw), Tracked(dealloc));
}
}
