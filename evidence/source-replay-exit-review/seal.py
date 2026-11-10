#!/usr/bin/env python3
"""Recheck every retained fresh product and raw command before final acceptance."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
import review as r
import finalize


def verify_products():
    r.guard()
    r.require(r.read(r.HERE/'seal-freeze.json')['sha256']==r.sha(Path(__file__)),'seal changed')
    count=0
    for short in r.LANES:
        receipt=r.read(r.RAW/short/'receipt.json')
        r.require(receipt['success'] is True,'unsuccessful replay')
        for p,facts in receipt['proof_artifacts'].items():
            r.require(r.sha(p)==facts['sha256'],'replay product changed '+p)
            count+=1
        independent=r.read(r.RAW/(short+'-independent.json'))
        for p,h in independent['products'].items():
            r.require(r.sha(p)==h,'independent product changed '+p)
            count+=1
    for p in r.RAW.glob('*-command.json'):
        command=r.read(p)
        r.require(command['exit_code']==0,'review command failed')
        for k in ['stdout','stderr']:
            r.require(r.sha(command[k]['path'])==command[k]['sha256'],'raw command log changed')
    return count


if __name__=='__main__':
    count=verify_products()
    finalize.final()
    r.require(verify_products()==count,'product inventory changed')
    r.write(r.RAW/'seal-receipt.json',{'success':True,'all_final_guards_before_success':True,
        'fresh_products_rechecked':count,'publication':False,'goal_complete':False,
        'final_receipt':r.pin(r.RAW/'final-receipt.json'),
        'preparation_failure':r.pin(r.RAW/'preparation-failure.json'),
        'source_hashes':{str(p.relative_to(r.ROOT)):r.sha(p) for p in r.HERE.iterdir()
            if p.is_file() and p.name not in ['SHA256SUMS','seal-receipt.json']}})
    r.write(r.HERE/'raw-receipts.json',{n:r.pin(r.RAW/n) for n in [
        'pretrial.json','history-authentication.json','constructor/receipt.json','pointer/receipt.json','exit/receipt.json',
        'constructor-independent.json','pointer-independent.json','exit-independent.json','final-receipt.json',
        'preparation-failure.json','seal-receipt.json']})
    print('all fresh sources, objects, individual outputs and raw command logs sealed')
