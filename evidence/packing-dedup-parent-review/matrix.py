from trials import *

def bools():
 require('bool-provider',app('bool-provider','provider',BOOL,goals=True),2)
 c=BOOL.replace('u64::from(b)','provider::all_bool(b)')
 require('bool-client',app('bool-client','client',c,[('provider',R/'bool-provider')]),1)
 require('false-client',app('false-client','false_client',c.replace('r==if b {1u64}else{0u64}','r==0u64'),[('provider',R/'bool-provider')]),0,1,False)
 require('goals-off',app('goals-off','off',BOOL),0,1,False)
 require('reexport',app('reexport','bridge',c.replace('fn all_bool','fn via'),[('provider',R/'bool-provider')]),1)
 require('transitive-client',app('transitive-client','third',c.replace('provider::all_bool','bridge::via'),[('provider',R/'bool-provider'),('bridge',R/'reexport')]),1)
 private=BOOL.replace('pub fn all_bool','fn all_bool')+'verus! {pub fn unrelated()->(r:u64) ensures r==9u64, {9}}'
 require('private-provider',app('private-provider','private',private,goals=True),3)
 require('private-client',app('private-client','private_client','use vstd::prelude::*; verus! {pub fn read()->(r:u64) ensures r==9u64, {private::unrelated()}}',[('private',R/'private-provider')]),1)

def observers():
 obs='''use vstd::prelude::*; use component::*; verus! {
 pub fn tag(t:BorTag,p:Permission,b:bool)->(r:Item)
 requires t.value() <= (u64::MAX >> 3),
 ensures item_word_wf(r.word()), r.word() & (u64::MAX >> 3)==t.value(), {Item::new(t,p,b)}
 pub fn permission(x:Item)->(r:Permission) ensures r==x.permission(), {x.perm()}
 pub fn protector(x:Item)->(r:bool) ensures r==(x.word() & 0x8000000000000000u64 > 0), {x.protected()}
 pub fn eq(a:&BorTag,b:&BorTag)->(r:bool) ensures r==(a.value()==b.value()), {compare_equal(a,b)}
 pub fn lookup_word(w:u64)->(r:u64) requires item_word_wf(w), ensures r==(w & (u64::MAX >> 3)), {lookup(w)}
 }'''
 for epoch in ['cold','warm']:
  require('observer-'+epoch,app('observer-'+epoch,'observer_'+epoch,obs,[('component',R/'stable-full'/epoch)]),5)
 require('observer-false',app('observer-false','observer_false',obs.replace('r==(x.word() & 0x8000000000000000u64 > 0)','r==false'),[('component',R/'stable-full/cold')]),4,1,False)
 # Constructor scope control: protector equality with input is not a public postcondition.
 limited='use vstd::prelude::*; use component::*; verus! {pub fn input_protector(t:BorTag,p:Permission,b:bool)->(r:Item) requires t.value() <= (u64::MAX >> 3), ensures r.protected()==b, {Item::new(t,p,b)}}'
 # protected() is exec, so use the existing closed word observation in the postcondition.
 limited=limited.replace('r.protected()==b','(r.word() & 0x8000000000000000u64 > 0)==b')
 require('constructor-protector-limit',app('constructor-protector-limit','limited',limited,[('component',R/'stable-full/cold')]),0,1,False)
 permission=limited.replace('input_protector','input_permission').replace('(r.word() & 0x8000000000000000u64 > 0)==b','r.permission()==p')
 require('constructor-permission-limit',app('constructor-permission-limit','limited_permission',permission,[('component',R/'stable-full/cold')]),0,1,False)

if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'use -B')
 if sys.argv[1]=='bool':bools()
 elif sys.argv[1]=='observers':observers()
