//! Stage1 local carrier only. Separate from bool certificates and recognition.
//! No source call may be redirected until the actual full Function has passed
//! independent correspondence and its helper enters the ordinary body pipeline.
use rustc_middle::{mir, ty};
use vir::ast::{Fun, Function};

type Reject = &'static str;

struct ConcreteOrderingInstances<'tcx> {
    carrier: ty::Instance<'tcx>,
    get: ty::Instance<'tcx>,
    primitive: ty::Instance<'tcx>,
    nonzero: ty::Ty<'tcx>,
    ordering: ty::Ty<'tcx>,
}

/// No public constructor: this is an operational check result, not a proof.
struct CheckedExpansion {
    statement: usize,
    formal: mir::Local,
    result: mir::Local,
}

/// Must bind existing imported functions by semantic source/model identities.
struct OrderingModelBindings {
    get: Fun,
    spec_get: Fun,
    view: Fun,
    primitive: Fun,
    obeys_cmp_spec: Fun,
    cmp_spec: Fun,
}

/// An independently checked actual full ordinary VIR Function. Local only;
/// never convertible to the bool ForeignCertificate.
struct OrderingBinding {
    function: Function,
    expansions: [CheckedExpansion; 2],
    models: OrderingModelBindings,
}
