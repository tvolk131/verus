from matrix import *
source=(ROOT/'source/tools/research/item_packing_goal/reuse.rs').read_text()
expect('review-packet-export',source,ok=True,verified=31,errors=0,export=True)
