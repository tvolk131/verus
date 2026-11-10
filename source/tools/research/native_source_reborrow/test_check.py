"""Pure parser/gate controls only; no filesystem mutation or race experiments."""
import copy
import json
import unittest
from pathlib import Path
import check

class Gates(unittest.TestCase):
    def test_kernel_summary(self):
        good = '''CONTEXT SUMMARY
===============
* Theory: Set is predicative
* Theory: Rewrite rules are not allowed
* Axioms: <none>
* Constants/Inductives relying on type-in-type: <none>
* Constants/Inductives relying on unsafe (co)fixpoints: <none>
* Inductives whose positivity is assumed: <none>
'''
        self.assertEqual(check.kernel_summary_no_axioms(good), [])
        for bad in ['', good+good, good.replace('* Axioms: <none>', '* Axioms: X'),
                    good.replace('unsafe (co)fixpoints: <none>', 'unsafe (co)fixpoints: X')]:
            with self.assertRaises(ValueError): check.kernel_summary_no_axioms(bad)

    def test_source_shape(self):
        graph=json.loads(check.GRAPH.read_text())
        borrow=json.loads(Path('/tmp/native-semantic-bridge-e4a8dcab/source-v1/drop-borrowck.json').read_text())
        check.source_shape(graph,borrow)
        for field,value in [('assigned',{'local':1}),('loan',1),('activation','ActivatedAt(bb0[3])')]:
            bad=copy.deepcopy(borrow);bad['loans'][0][field]=value
            with self.assertRaises(ValueError):check.source_shape(graph,bad)
        bad=copy.deepcopy(graph);bad['operations'][3]['ir']['value']['place']['local']=4
        with self.assertRaises(ValueError):check.source_shape(bad,borrow)

    def test_inventory_generated(self):
        names=check.declarations(check.HERE/'candidates/Model.v')
        for n in ['permission','permission_rect','permission_ind','permission_rec','permission_sind',
                  'Reserved','result_rect','result_ind','result_rec','result_sind','Advanced',
                  'grant','Grant','grant_tag','node','Node','node_tag','history','History','prior_history',
                  'seek_parent','prepare','install']:
            self.assertIn(n,names)
        self.assertEqual(len(names),len(set(names)))
        with self.assertRaises(ValueError):
            check.printed_declarations('     Definition x : nat.\n     Definition x : nat.\n')

if __name__=='__main__':unittest.main()
