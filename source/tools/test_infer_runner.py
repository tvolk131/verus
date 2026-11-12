"""Suite orchestration must bind compiled binaries and exact executed test counts."""
import json
import unittest
from pathlib import Path

from run_infer_tests import artifacts, check_rust_summary, integration_targets, test_names


class RunnerTests(unittest.TestCase):
    def artifact(self, name='vir', executable='/tmp/vir-tests', test=True):
        return json.dumps(dict(reason='compiler-artifact', target=dict(name=name),
                               executable=executable, profile=dict(test=test)))

    def test_artifact_inventory_excludes_dependencies_and_rejects_omissions(self):
        text='progress\n'+self.artifact()+'\n'+self.artifact('dependency',test=False)
        self.assertEqual(artifacts(text, ['vir']), {'vir': Path('/tmp/vir-tests')})
        with self.assertRaises(ValueError): artifacts(text, ['vir', 'air'])
        with self.assertRaises(ValueError): artifacts(text+'\n'+self.artifact(executable='/tmp/other'), ['vir'])

    def test_names_are_exact_nonempty_unique_and_exclude_benchmarks(self):
        self.assertEqual(test_names('a: test\na_longer: test\nb: benchmark\n'), ['a', 'a_longer'])
        for text in ['', 'a: benchmark', 'a: test\na: test']:
            with self.subTest(text=text), self.assertRaises(ValueError): test_names(text)

    def test_summary_preserves_ignored_counts_and_refuses_empty_execution(self):
        self.assertEqual(check_rust_summary('test result: ok. 2 passed; 0 failed; 1 ignored;',3),
                         dict(passed=2, ignored=1, selected=3))
        for text in ['no result', 'test result: ok. 0 passed; 0 failed; 0 ignored;',
                     'test result: ok. 2 passed; 1 failed; 0 ignored;']:
            with self.subTest(text=text), self.assertRaises(ValueError): check_rust_summary(text,3)

    def test_application_target_runs_once_and_live_targets_exist(self):
        self.assertEqual(integration_targets('applications'), ['infer_application_regressions'])
        self.assertNotIn('infer_application_regressions', integration_targets('compiler'))
        self.assertIn('infer', integration_targets('compiler'))
        self.assertNotIn('local_checked_owned_cell', integration_targets('compiler'))


if __name__ == '__main__':
    unittest.main()
