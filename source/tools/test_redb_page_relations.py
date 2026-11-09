"""Offline source-fidelity, domain, and evidence-classification controls."""
import hashlib
import io
import json
from contextlib import redirect_stdout
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch
import check_redb_page_relations as relations
from infer_eval_support import ArtifactGuard
from test_infer_transfers import proof_report

class PageRelationsTests(unittest.TestCase):
    def test_changed_application_writes_no_fixtures(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d); source=root/relations.BASE
            source.parent.mkdir(parents=True);source.write_text('changed')
            with self.assertRaisesRegex(ValueError,'pinned application source changed'):
                relations.extract(root,root)
            self.assertFalse((root/'relations.rs').exists())

    def test_extract_checks_every_item_before_writing_and_preserves_bodies(self):
        parts={'PageNumber':'pub(crate) struct PageNumber {pub(crate) region:u32,pub(crate) page_index:u32,pub(crate) page_order:u8}',
               'page_size_bytes':'    pub(crate) fn page_size_bytes(&self,size:u32)->u64 {let pages=1u64<<self.page_order;pages*u64::from(size)}',
               'address_range':'    pub(crate) fn address_range(&self,offset:u64,size:u64,start:u64,page_size:u32)->Range<u64> {debug_assert!(start<size);let end=start+self.page_size_bytes(page_size);start..end}'}
        text=parts['PageNumber']+'\nimpl PageNumber {\n'+parts['page_size_bytes']+'\n'+parts['address_range']+'\n}\n'
        hashes={name:hashlib.sha256((value+'\n').encode()).hexdigest() for name,value in parts.items()}
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);source=root/relations.BASE;source.parent.mkdir(parents=True);source.write_text(text)
            with patch.object(relations,'SOURCE_HASH',relations.digest(source)),patch.object(relations,'ITEM_HASHES',dict(hashes,address_range='0'*64)):
                with self.assertRaisesRegex(ValueError,'pinned extraction changed'):
                    relations.extract(root,root)
                self.assertFalse((root/'PageNumber.original.rs').exists())
            with patch.object(relations,'SOURCE_HASH',relations.digest(source)),patch.object(relations,'ITEM_HASHES',hashes):
                provenance=relations.extract(root,root)
            self.assertEqual(source.read_text(),text)
            for name,value in parts.items():
                self.assertEqual(provenance[name]['sha256'],hashes[name])
                self.assertIn(value,(root/'relations.rs').read_text())
                self.assertIn(value.replace('pub(crate)','pub'),(root/'provider.rs').read_text())
            for name in ['relations','client','zero','witnesses']:
                fixture=(root/(name+'.rs')).read_text()
                for forbidden in ['proof {','by(bit_vector)','by(nonlinear_arith)','external_body','assume(','order<=20']:
                    self.assertNotIn(forbidden,fixture)
            self.assertIn('relay(&provider::PageNumber{',(root/'client.rs').read_text())

    def test_each_page_keeps_full_intermediate_domain(self):
        for index in ['first','second']:
            domain=relations.domain(index)
            self.assertEqual(len(domain),9)
            self.assertIn('order<64',domain)
            self.assertTrue(any(index+' as int *' in x and '<=u64::MAX' in x for x in domain))
            self.assertTrue(any('pages_start as int +' in x and '<region_size' in x for x in domain))
            self.assertTrue(any('offset as int +' in x and x.endswith('<=u64::MAX') for x in domain))
        self.assertEqual(len(relations.SAFETY),11)
        self.assertIn('size>0',relations.RELATIONS)
        self.assertNotIn('size>0',relations.ZERO)

    def test_zero_boundary_is_nonoverlap_and_mixed_orders_overlap(self):
        self.assertIn('result.0.end<=result.1.start',relations.ZERO)
        self.assertIn('result.0.start==result.1.start',relations.ZERO)
        self.assertIn('page_order:1}.address_range(0,8192,0,4096)',relations.WITNESSES)
        self.assertIn('page_index:1,page_order:0}.address_range(0,8192,0,4096)',relations.WITNESSES)
        self.assertLess(4096,8192)
        self.assertIn('mixed_witness()',relations.NEGATIVES['false_mixed_nonoverlap'])

    def test_classification_never_calls_auxiliary_frontend_or_resource_a_counterexample(self):
        report={'verification-results':{'success':False},'verus-infer':{'functions':[{'required_checks_passed':True}]}}
        cases=[('postcondition not satisfied','semantic_goal_failure'),('precondition not satisfied','caller_obligation_failure'),
               ('polynomial proposal not satisfied','auxiliary_theorem_failure'),('bitvector assertion not satisfied','auxiliary_theorem_failure'),
               ('Resource limit exceeded; postcondition not satisfied','resource_or_internal_failure'),('error[E0001]','frontend_rejection'),
               ('precondition not satisfied; postcondition not satisfied','proof_blocked')]
        for diagnostic,expected in cases:
            self.assertEqual(relations.classify(report,diagnostic,1),expected)
        bad=dict(report,**{'verus-infer':{'functions':[{'required_checks_passed':False}]}})
        self.assertEqual(relations.classify(bad,'postcondition not satisfied',1),'inference_failure')

    def test_runner_guards_provider_immediately_around_imports_and_at_completion(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            events = []
            positives = {"relations_cold", "relations_warm", "relations_poisoned_cache", "zero_ordinary",
                         "witnesses_checked", "provider_cold", "client_cold", "client_warm"}
            certificate = dict(schema=38, policy="synthetic", exactness=True, summaries="f" * 64,
                               trusted_models=[[{}, "f" * 64]])
            blob = json.dumps(certificate, separators=(",", ":")).encode() + b"VRINFER1"

            def artifact_guard(paths):
                guard = ArtifactGuard(paths)
                provider = any(Path(path).name == "provider_cold.vir" for path in paths)
                def check():
                    if provider:
                        events.append(("provider", "check"))
                    guard.check()
                def snapshot():
                    if provider:
                        events.append(("provider", "snapshot"))
                    return guard.snapshot()
                return Mock(initial=guard.initial, check=check, snapshot=snapshot)

            def logged_commands(output, cwd, commands, **options):
                def run(label, argv, environment):
                    events.append(("run", label))
                    if label.startswith("application_"):
                        stdout = relations.APPLICATION if label.endswith("_pin") else ""
                        stderr, status = "", 0
                    elif label == "engine_files":
                        stdout, stderr, status = "", "", 0
                    else:
                        positive = label in positives
                        report = proof_report(positive)
                        frontend = label.startswith("client_metadata_")
                        count = 0 if frontend else 1 if label.startswith("client_") else 2
                        report["verus-infer"]["functions"] *= count
                        if frontend:
                            report["verification-results"]["encountered-vir-error"] = True
                            stderr = "unsupported imported metadata"
                        elif label.startswith("false_polynomial_"):
                            stderr = "polynomial proposal not satisfied"
                        elif label.removesuffix("_negative") in relations.SAFETY:
                            stderr = "precondition not satisfied"
                        else:
                            stderr = "" if positive else "postcondition not satisfied"
                        stdout, status = json.dumps(report), 0 if positive else 1
                        artifact = Path(argv[argv.index("--export") + 1])
                        if positive:
                            artifact.write_bytes(blob)
                            Path(argv[argv.index("-o") + 1]).write_bytes(b"synthetic native artifact")
                        else:
                            artifact.unlink(missing_ok=True)
                    for suffix, text in (("stdout", stdout), ("stderr", stderr)):
                        (output / (label + "." + suffix)).write_text(text)
                    commands.append(dict(label=label, argv=argv, exit_code=status, cwd=str(cwd)))
                    return status
                return run

            argv = ["check_redb_page_relations.py", "--application", str(root / "application"),
                    "--verus", str(root / "verus"), "--solver", str(root / "z3"), "--output", str(root / "output")]
            output = io.StringIO()
            with patch.object(sys, "argv", argv), patch.object(relations, "ROOT", root), \
                    patch.object(relations, "extract", return_value={}), \
                    patch.object(relations, "tool_artifacts", return_value=[]), \
                    patch.object(relations, "snapshot_inputs", return_value=dict(source_hashes={})), \
                    patch.object(relations, "ArtifactGuard", side_effect=artifact_guard), \
                    patch.object(relations, "LoggedCommands", side_effect=logged_commands), redirect_stdout(output):
                relations.main()
            receipt = json.loads((Path(output.getvalue().strip()) / "receipt.json").read_text())
            self.assertIs(receipt["success"], True)
            clients = [(index, label) for index, (kind, label) in enumerate(events)
                       if kind == "run" and label.startswith("client_")]
            self.assertEqual(len(clients), 6)
            for index, label in clients:
                with self.subTest(label=label):
                    self.assertEqual(events[index - 1], ("provider", "check"))
                    self.assertEqual(events[index + 1], ("provider", "check"))
            self.assertEqual(events[-2:], [("provider", "check"), ("provider", "snapshot")])

if __name__=='__main__':unittest.main()
