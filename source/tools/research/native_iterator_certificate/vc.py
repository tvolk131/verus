"""Conditional scalar VCs derived from the checked normal-step value summary.

These integer implications assume the state invariant. They neither establish
that invariant at native construction nor provide pointer/ownership semantics.
"""
def smt(expression):
    if type(expression) is int: return str(expression)
    op, args = expression['op'], expression['args']
    if op in ('index', 'length') and args == []: return op
    if op == 'Add' and len(args) == 2: return '(+ ' + ' '.join(smt(a) for a in args) + ')'
    raise ValueError('unsupported scalar VC expression')

def generate(inspection):
    summaries = inspection['conditional_native_value_summaries']
    if len(summaries) != 2: raise ValueError('native value summaries incomplete')
    if summaries[0]['new_index_expression'] != summaries[1]['new_index_expression']:
        raise ValueError('native wrapper/specialized index disagreement')
    step = smt(summaries[0]['new_index_expression'])
    header = '''; Conditional on invariant; NOT a native memory/refinement proof.
(set-logic QF_LIA)
(set-option :timeout 5000)
(declare-const index Int)
(declare-const length Int)
(declare-const key_length Int)
(assert (and (<= 0 index) (< index length) (<= length 128)
             (<= length key_length) (<= 0 key_length)))
'''
    return {
        'native_step_scalar_range_given_invariant': header +
            f'(assert (not (and (<= 0 {step}) (<= {step} length) (<= {step} 18446744073709551615))))\n(check-sat)\n',
        'native_getter_scalar_bounds_given_invariant': header +
            '(assert (not (and (< index 128) (< index key_length))))\n(check-sat)\n',
    }

def require_unsat(returncode, stdout, stderr):
    if returncode != 0 or stdout.strip() != 'unsat' or stderr.strip():
        raise ValueError('failed_incomplete_or_resource_limited_backend_proof')
