"""Frozen final bundle, then necessary proofs. Does not accept old compiler imports."""
from trials import *
from stable import stable
from matrix import bools,observers
from fixtures import fixture as full_fixture

def main():
 need(sys.flags.dont_write_bytecode,'use -B')
 tools={n:sha(R/'bundle'/n) for n in TOOLS}
 write(R/'FINAL_TOOLS.json',dict(tools=tools,source=manifest(R/'src'),compiler_executing_bytes=tools['rust_verify']))
 full,_,fidelity=full_fixture();need(sha(R/'full.rs')==hashlib.sha256(full.encode()).hexdigest(),'full frozen fixture')
 for kind,source in [('full',full),('packet',(ROOT/'source/tools/research/item_packing_goal/reuse.rs').read_text())]:stable(kind,source)
 if 'ablation' in sys.argv:return
 packet=(ROOT/'source/tools/research/item_packing_goal/reuse.rs').read_text().replace('fn packet(','pub fn packet(').replace('const CHANNEL_MASK','pub const CHANNEL_MASK')
 stable('packet-public-api',packet)
 stable('bool',BOOL);bools();observers()
 # Explicit controllers execute in this process with this exact root.
 for script in ['packet_client.py','auxiliary.py','erasure.py']:
  path=HERE/script;exec(compile(path.read_text(),str(path),'exec'),{'__name__':'__main__','__file__':str(path)})
 need(tools=={n:sha(R/'bundle'/n) for n in TOOLS},'final bundle drift')
if __name__=='__main__':main()
