"""Pin goal requirements while allowing documentary progress updates."""
from hashlib import sha256

GOAL_REQUIREMENTS_SHA256 = '11014c3ac73ddf13c464769f5d0d1cbd24598ab52777718d31872c70b3b8f7b0'
START = b'## Required property and domain\n'
END = b'## Coordination\n'


def goal_scope_sha256(snapshot):
    if snapshot.count(START) != 1 or snapshot.count(END) != 1:
        raise ValueError('missing or ambiguous goal requirement boundaries')
    start, end = snapshot.index(START), snapshot.index(END)
    if start >= end:
        raise ValueError('goal requirement boundaries out of order')
    actual = sha256(snapshot[start:end]).hexdigest()
    if actual != GOAL_REQUIREMENTS_SHA256:
        raise ValueError('goal domain or completion requirements changed')
    return actual


def check_goal_snapshot(snapshot, retained_sha256, expected_scope_sha256):
    if expected_scope_sha256 != GOAL_REQUIREMENTS_SHA256:
        raise ValueError('missing or incorrect frozen goal requirements pin')
    if sha256(snapshot).hexdigest() != retained_sha256:
        raise ValueError('goal snapshot differs from retained input')
    return goal_scope_sha256(snapshot)
