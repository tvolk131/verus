"""Check specification migration and evidence classification for counting."""
import unittest

from eval_verus_infer_alephbft_counting import library_counting, require_assistance, spec_item

SOURCE = """
    spec fn count_prefix<T>(candidates: Seq<T>, round: u32, len: nat) -> nat
        recommends len <= candidates.len(), decreases len,
    { if len == 0 { 0 } else { count_prefix(candidates, round, (len-1) as nat) + 1 } }
    proof fn count_bound<T>(candidates: Seq<T>, round: u32, len: nat)
        ensures count_prefix(candidates, round, len) <= len, decreases len,
    { if len > 0 { count_bound(candidates, round, (len-1) as nat); } }
    proof fn count_zero<T>(candidates: Seq<T>, round: u32, len: nat)
        ensures count_prefix(candidates, round, len) == 0, decreases len,
    { if len > 0 { count_zero(candidates, round, (len-1) as nat); } }
    proof fn count_update<T>(candidates: Seq<T>, round: u32, index: int, value: T, len: nat)
        ensures count_prefix(candidates.update(index, value), round, len) as int
            == count_prefix(candidates, round, len) as int + if index < len { 1 } else { 0 },
        decreases len,
    { if len > 0 { count_update(candidates, round, index, value, (len-1) as nat); } }
    pub fn new() {
        #[cfg(verus_keep_ghost)]
        proof! {
            count_zero(candidates@, 1, candidates@.len());
        }
        keep_constructor();
    }
    pub fn next() {
        proof! {
            assert(candidates@ == self.candidates@);
            count_zero(candidates@, (self.for_round + 1) as u32, candidates@.len());
        }
    }
    pub fn add(&mut self) {
        if replace {
            #[cfg(verus_keep_ghost)]
            proof! { count_update(x); count_bound(x); }
            keep_update();
        }
    }
"""


class CountingExperimentTests(unittest.TestCase):
    def test_spec_body_search_does_not_mistake_an_ensures_branch_for_the_body(self):
        _, _, opening, definition = spec_item(SOURCE, "proof fn count_update<T>")
        self.assertTrue(SOURCE[opening:].startswith("{ if len > 0"))
        self.assertIn("else { 0 }", definition)
        self.assertTrue(definition.endswith("as nat); } }"))

    def test_migration_keeps_a_checked_reference_to_the_original_count(self):
        migrated = library_counting(SOURCE, False)
        self.assertIn("candidates.count_prefix(|slot: Option<(T, u16)>| direct(slot, round), len)", migrated)
        self.assertIn("reference_count_prefix(candidates, round, (len-1) as nat) + 1", migrated)
        self.assertIn("proof fn library_count_agrees<T>", migrated)
        self.assertIn("== reference_count_prefix(candidates, round, len)", migrated)
        self.assertIn("candidates.lemma_count_prefix_update(", migrated)
        self.assertIn("proof! { count_update(x); count_bound(x); }", migrated)

    def test_assisted_variant_removes_all_application_counting_helpers_and_calls(self):
        assisted = library_counting(SOURCE, True)
        for name in ("count_update", "count_bound", "count_zero"):
            self.assertNotIn("proof fn " + name + "<T>", assisted)
            self.assertNotIn(name + "(", assisted)
        self.assertIn("keep_constructor();", assisted)
        self.assertIn("keep_update();", assisted)
        self.assertIn("assert(candidates@ == self.candidates@);", assisted)

    def test_changed_hint_layout_is_rejected(self):
        with self.assertRaises(ValueError): library_counting(SOURCE.replace("count_bound(x);", "other(x);"), True)
        with self.assertRaises(ValueError): library_counting(SOURCE.replace("count_zero(candidates@, 1,", "other(candidates@, 1,"), True)

    def test_selection_is_required_for_assisted_cases_only(self):
        data = {"verus-infer": {"goal_assistance": []}}
        require_assistance(data, False)
        with self.assertRaises(ValueError): require_assistance(data, True)
        entry = dict(function="test::Collector::add", strategy="counting_broadcast",
                     lemmas=["vstd::seq_lib::impl::lemma_count_prefix_" + p for p in ("bound", "update", "zero", "full")])
        data["verus-infer"]["goal_assistance"] = [entry]
        require_assistance(data, True)
        with self.assertRaises(ValueError): require_assistance(data, False)
        entry["lemmas"].pop()
        with self.assertRaises(ValueError): require_assistance(data, True)


if __name__ == "__main__":
    unittest.main()
