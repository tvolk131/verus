"""Expected current engine identity for live transfer receipt acceptance."""

CACHE_VERSION = "verus-infer-v83:integrated-profiles-and-checked-foreign:4096"
INTERFACE_POLICY = "candidate-bound-evidence-v79"
INTERFACE_SCHEMA = 40
ENGINE_IDENTITY = (83, 79, 40)


def require_no_foreign_dependencies(certificate):
    """These registered slices have no checked foreign-body dependency."""
    from infer_experiment import require
    require(certificate.get("foreign_dependencies") == [],
            "unexpected or missing checked foreign dependencies")
