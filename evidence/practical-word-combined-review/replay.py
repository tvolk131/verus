"""Run the unchanged existing combined replay with the pinned build environment."""
import os,sys,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];O=Path('/tmp/practical-word-combined-review-5ecea5fe')
env=dict(os.environ,**json.loads((O/'env.json').read_text()),PYTHONDONTWRITEBYTECODE='1')
args=[sys.executable,'-B',str(ROOT/'source/tools/research/practical_redb_fedimint/common_set/replay_s1.py'),'--bundle',str(O/'bundle'),'--output',str(O/'combined-replay-fresh'),'--combined','--policy','78']
os.execve(sys.executable,args,env)
