"""Focused acceptance tests; all checks also run with Python optimization enabled."""

import ast
import copy
import inspect
import json
from pathlib import Path
import tempfile
import textwrap
import unittest
from unittest.mock import patch

from infer_transfer_identity import CACHE_VERSION, INTERFACE_POLICY, INTERFACE_SCHEMA
import check_fedimint_paymentfee76 as fee


def fixture():
    functions = [dict(mode="exact", required_checks_passed=True, exported=True,
        contract=dict(requires="true"), metrics=dict(cache="miss", cache_policy="candidate_cache", solver_micros=1),
        proofs={name: dict(status="proved", required=True)
            for name in ("soundness", "maximality", "exactness")}) for _index in range(5)]
    return dict(returncode=0, stderr="", stdout=json.dumps({
        "verification-results": {"success": True, "verified":22, "errors":0,
            "is-verifying-entire-crate":True},
        "verus-infer":dict(crate_success=True, interface_written=True, functions=functions,
            verification_scope=dict(native_collection_completed=True,
                complete_selected_scope_verified=True, checked_exec_bodies=12, checked_proof_bodies=0))}))


class Acceptance(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.output = Path(self.temporary.name)
        self.artifact = self.output / "provider.vir"
        self.artifact.write_bytes(b"fresh")

    def tearDown(self):
        self.temporary.cleanup()

    def accept(self, run, validator=fee.validate_case):
        return validator(run, "positive", self.artifact, total=22, bodies=12, phase="cold")

    def mutate(self, change):
        run=fixture()
        data=json.loads(run["stdout"])
        change(data)
        run["stdout"]=json.dumps(data)
        return run

    def test_valid_positive(self):
        self.accept(fixture())

    def test_each_mandatory_check_is_required(self):
        for name in ("soundness", "maximality", "exactness"):
            for change in ("missing", "incomplete", "not_required"):
                with self.subTest(name=name, change=change):
                    def mutate(data):
                        proofs=data["verus-infer"]["functions"][0]["proofs"]
                        if change=="missing":
                            del proofs[name]
                        else:
                            proofs[name]["status" if change=="incomplete" else "required"] = (
                                "incomplete" if change=="incomplete" else False)
                    with self.assertRaises(ValueError):
                        self.accept(self.mutate(mutate))

    def test_domain_cache_work_scope_and_body_counts_are_not_optional(self):
        mutations = [
            lambda data: data["verus-infer"]["functions"][0]["contract"].update(requires="false"),
            lambda data: data["verus-infer"]["functions"][0]["metrics"].update(cache="hit"),
            lambda data: data["verus-infer"]["functions"][0]["metrics"].update(solver_micros=0),
            lambda data: data["verus-infer"]["functions"][0].update(required_checks_passed="true"),
            lambda data: data["verification-results"].update(success=1),
            lambda data: data["verification-results"].update(verified=True),
            lambda data: data["verification-results"].update(**{"is-verifying-entire-crate":False}),
            lambda data: data["verus-infer"]["verification_scope"].update(complete_selected_scope_verified=False),
            lambda data: data["verus-infer"]["verification_scope"].update(checked_exec_bodies=11),
            lambda data: data["verus-infer"]["verification_scope"].update(checked_proof_bodies=1),
            lambda data: data["verus-infer"]["functions"][0].update(exported=False),
        ]
        for index, change in enumerate(mutations):
            with self.subTest(index=index), self.assertRaises(ValueError):
                self.accept(self.mutate(change))

    def test_missing_artifact_is_rejected(self):
        self.artifact.unlink()
        with self.assertRaises(ValueError):
            self.accept(fixture())

    def test_warm_phase_distinguishes_cache_hits_from_regenerated_records(self):
        run=fixture()
        data=json.loads(run["stdout"])
        functions=data["verus-infer"]["functions"]
        functions[0]["metrics"]["cache"]="hit"
        for function in functions[1:]:
            function["metrics"]["cache_policy"]="regenerated_state"
        run["stdout"]=json.dumps(data)
        fee.validate_case(run,"positive",self.artifact,total=22,bodies=12,phase="warm")
        functions[0]["metrics"].update(cache="miss",cache_policy="regenerated_state")
        run["stdout"]=json.dumps(data)
        with self.assertRaisesRegex(ValueError,"no observed candidate cache hit"):
            fee.validate_case(run,"positive",self.artifact,total=22,bodies=12,phase="warm")

    def test_cache_and_certificate_acceptance_checks_are_live(self):
        cache=self.output/"cache"
        cache.mkdir()
        with self.assertRaises(ValueError):
            fee.check_cache(cache)
        (cache/"entry.json").write_text(json.dumps(dict(version=CACHE_VERSION)))
        fee.check_cache(cache)
        (cache/"entry.json").write_text(json.dumps(dict(version="verus-infer-v75:old:4096")))
        with self.assertRaises(ValueError):
            fee.check_cache(cache)
        certificate=dict(schema=INTERFACE_SCHEMA,policy=INTERFACE_POLICY,soundness=True,maximality=True,
            exactness=True,summaries="a"*64,proofs=[[dict(path=dict(segments=[name])),
                dict(soundness=True,maximality=True,exactness=True)]
                for name in ("from_msats","fee","add_to","subtract_from")])
        with patch.object(fee,"metadata",return_value=(certificate,"a"*64)):
            fee.check_certificate(self.artifact,4)
            certificate["proofs"][0][1]["maximality"]=False
            with self.assertRaises(ValueError):
                fee.check_certificate(self.artifact,4)

    def test_semantic_and_safety_are_distinct(self):
        for category, marker in (("semantic","precondition not satisfied"),
                ("safety","postcondition not satisfied")):
            run=fixture()
            data=json.loads(run["stdout"])
            data["verification-results"].update(success=False,errors=1)
            inference=data["verus-infer"]
            inference.update(crate_success=False,interface_written=False,status="caller_proof_failed")
            for function in inference["functions"]:
                function["exported"]=False
            run.update(returncode=1,stderr=marker,stdout=json.dumps(data))
            self.artifact.unlink(missing_ok=True)
            with self.assertRaises(ValueError):
                fee.validate_case(run,category,self.artifact)

    def test_run_case_cannot_skip_validator(self):
        (self.output/"provider.rs").write_text("frozen source")
        def command(label, argv, env):
            run=fixture()
            (self.output/(label+".stdout")).write_text(run["stdout"])
            (self.output/(label+".stderr")).write_text("")
            self.artifact.write_bytes(b"fresh")
            (self.output/"libprovider.rlib").write_bytes(b"fresh native")
            return 0
        with patch.object(fee,"validate_case",side_effect=ValueError("acceptance sentinel")) as validator:
            with self.assertRaisesRegex(ValueError,"acceptance sentinel"):
                fee.run_case(command,{},self.output,Path("verus"),dict(cases=[]),"provider",
                    total=22,bodies=12,phase="cold")
            validator.assert_called_once()

    def test_independent_check_removal_campaign(self):
        gates = {
            "native domain was narrowed": lambda data:data["verus-infer"]["functions"][0]["contract"].update(requires="false"),
            "cache phase not observed": lambda data:data["verus-infer"]["functions"][0]["metrics"].update(cache="hit"),
            "fresh required-check solver work absent": lambda data:data["verus-infer"]["functions"][0]["metrics"].update(solver_micros=0),
            "selected or partial verification is not accepted": lambda data:data["verification-results"].update(**{"is-verifying-entire-crate":False}),
            "positive native scope incomplete": lambda data:data["verus-infer"]["verification_scope"].update(complete_selected_scope_verified=False),
            "positive native body coverage differs": lambda data:data["verus-infer"]["verification_scope"].update(checked_exec_bodies=11),
        }
        original=ast.parse(textwrap.dedent(inspect.getsource(fee.validate_case)))
        for message, change in gates.items():
            tree=copy.deepcopy(original)
            removed=[]
            class Remove(ast.NodeTransformer):
                def visit_Expr(self, node):
                    if isinstance(node.value,ast.Call) and isinstance(node.value.func,ast.Name):
                        if node.value.func.id=="require" and any(isinstance(argument,ast.Constant)
                                and argument.value==message for argument in node.value.args):
                            removed.append(message)
                            return ast.Pass()
                    return node
            tree=ast.fix_missing_locations(Remove().visit(tree))
            namespace=dict(fee.__dict__)
            exec(compile(tree,"removed-check","exec"),namespace)
            self.assertEqual(removed,[message])
            run=self.mutate(change)
            with self.subTest(message=message):
                with self.assertRaises(ValueError):
                    self.accept(run)
                self.accept(run,namespace["validate_case"])


if __name__=="__main__":
    unittest.main()
