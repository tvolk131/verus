use vstd::prelude::*;
#[path = "../kernel-v4/kernel.rs"] mod kernel;
use kernel::*;
verus! {
pub open spec fn state_0(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}),custodian:0,stack:seq![],ok:true} }
pub open spec fn state_1(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead),custodian:0,stack:seq![],ok:true} }
pub open spec fn state_2(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead),custodian:0,stack:seq![],ok:true} }
pub open spec fn state_3(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead),custodian:0,stack:seq![],ok:true} }
pub open spec fn state_4(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}),custodian:0,stack:seq![],ok:true} }
pub open spec fn state_5(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(4,Val::Dead),custodian:0,stack:seq![],ok:true} }
pub open spec fn state_6(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(4,Val::Dead),custodian:0,stack:seq![],ok:true} }
pub open spec fn state_7(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}),custodian:0,stack:seq![],ok:true} }
pub open spec fn state_8(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}),custodian:1,stack:seq![0],ok:true} }
pub open spec fn state_9(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}),custodian:1,stack:seq![0],ok:true} }
pub open spec fn state_10(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_11(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Dead),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_12(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_13(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::Dead),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_14(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::Dead).insert(204,Val::Dead),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_15(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::Dead).insert(204,Val::Dead).insert(207,Val::Dead),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_16(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::Dead).insert(204,Val::Dead).insert(207,Val::Raw{p:Ptr{origin,loan,offset:0},meta:Some(128)}),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_17(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::Dead).insert(204,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:Some(128)}).insert(207,Val::Raw{p:Ptr{origin,loan,offset:0},meta:Some(128)}),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_18(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::Dead).insert(204,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:Some(128)}).insert(207,Val::Dead),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_19(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::Dead).insert(204,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:Some(128)}).insert(207,Val::Dead).insert(208,Val::Dead),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_20(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::Dead).insert(204,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:Some(128)}).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_21(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::Dead).insert(204,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:Some(128)}).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Raw{p:Ptr{origin,loan,offset:0},meta:Some(128)}),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_22(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::Dead).insert(204,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:Some(128)}).insert(207,Val::Dead).insert(208,Val::Raw{p:Ptr{origin,loan,offset:0},meta:None}).insert(209,Val::Raw{p:Ptr{origin,loan,offset:0},meta:Some(128)}),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_23(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::Dead).insert(204,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:Some(128)}).insert(207,Val::Dead).insert(208,Val::Raw{p:Ptr{origin,loan,offset:0},meta:None}).insert(209,Val::Dead),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_24(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:None}).insert(204,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:Some(128)}).insert(207,Val::Dead).insert(208,Val::Raw{p:Ptr{origin,loan,offset:0},meta:None}).insert(209,Val::Dead),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_25(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:None}).insert(204,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:Some(128)}).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_26(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:None}).insert(204,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_27(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:None}).insert(204,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_28(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:None}).insert(204,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_29(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:None}).insert(204,Val::Dead).insert(206,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_30(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:None}).insert(204,Val::Dead).insert(206,Val::Raw{p:Ptr{origin,loan,offset:0},meta:None}).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_31(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:None}).insert(204,Val::Dead).insert(205,Val::Raw{p:Ptr{origin,loan,offset:128},meta:None}).insert(206,Val::Raw{p:Ptr{origin,loan,offset:0},meta:None}).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_32(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:None}).insert(204,Val::Dead).insert(205,Val::Raw{p:Ptr{origin,loan,offset:128},meta:None}).insert(206,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_33(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(200,Val::Iter{start:Ptr{origin,loan,offset:0},end:Ptr{origin,loan,offset:128}}).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::NonNull{p:Ptr{origin,loan,offset:0},meta:None}).insert(204,Val::Dead).insert(205,Val::Raw{p:Ptr{origin,loan,offset:128},meta:None}).insert(206,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_34(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(200,Val::Iter{start:Ptr{origin,loan,offset:0},end:Ptr{origin,loan,offset:128}}).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Num(128)).insert(203,Val::Dead).insert(204,Val::Dead).insert(205,Val::Raw{p:Ptr{origin,loan,offset:128},meta:None}).insert(206,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_35(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(200,Val::Iter{start:Ptr{origin,loan,offset:0},end:Ptr{origin,loan,offset:128}}).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Dead).insert(203,Val::Dead).insert(204,Val::Dead).insert(205,Val::Raw{p:Ptr{origin,loan,offset:128},meta:None}).insert(206,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)),custodian:2,stack:seq![0,1],ok:true} }
pub open spec fn state_36(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(101,Val::Dead).insert(200,Val::Iter{start:Ptr{origin,loan,offset:0},end:Ptr{origin,loan,offset:128}}).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Dead).insert(203,Val::Dead).insert(204,Val::Dead).insert(205,Val::Raw{p:Ptr{origin,loan,offset:128},meta:None}).insert(206,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)),custodian:1,stack:seq![0],ok:true} }
pub open spec fn state_37(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(100,Val::Iter{start:Ptr{origin,loan,offset:0},end:Ptr{origin,loan,offset:128}}).insert(101,Val::Dead).insert(200,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Dead).insert(203,Val::Dead).insert(204,Val::Dead).insert(205,Val::Raw{p:Ptr{origin,loan,offset:128},meta:None}).insert(206,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)),custodian:1,stack:seq![0],ok:true} }
pub open spec fn state_38(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(100,Val::Iter{start:Ptr{origin,loan,offset:0},end:Ptr{origin,loan,offset:128}}).insert(101,Val::Dead).insert(200,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Dead).insert(203,Val::Dead).insert(204,Val::Dead).insert(205,Val::Raw{p:Ptr{origin,loan,offset:128},meta:None}).insert(206,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)),custodian:0,stack:seq![],ok:true} }
pub open spec fn state_39(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Iter{start:Ptr{origin,loan,offset:0},end:Ptr{origin,loan,offset:128}}).insert(3,Val::Dead).insert(4,Val::Dead).insert(100,Val::Dead).insert(101,Val::Dead).insert(200,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Dead).insert(203,Val::Dead).insert(204,Val::Dead).insert(205,Val::Raw{p:Ptr{origin,loan,offset:128},meta:None}).insert(206,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)),custodian:0,stack:seq![],ok:true} }
pub open spec fn state_40(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Iter{start:Ptr{origin,loan,offset:0},end:Ptr{origin,loan,offset:128}}).insert(3,Val::Dead).insert(4,Val::Dead).insert(100,Val::Dead).insert(101,Val::Dead).insert(200,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Dead).insert(203,Val::Dead).insert(204,Val::Dead).insert(205,Val::Raw{p:Ptr{origin,loan,offset:128},meta:None}).insert(206,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)),custodian:0,stack:seq![],ok:true} }
pub open spec fn state_41(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Iter{start:Ptr{origin,loan,offset:0},end:Ptr{origin,loan,offset:128}}).insert(3,Val::Dead).insert(4,Val::Dead).insert(100,Val::Dead).insert(101,Val::Dead).insert(200,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Dead).insert(203,Val::Dead).insert(204,Val::Dead).insert(205,Val::Raw{p:Ptr{origin,loan,offset:128},meta:None}).insert(206,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)).insert(10002,Val::Unit),custodian:0,stack:seq![],ok:true} }
pub open spec fn state_42(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(0,Val::Unit).insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Iter{start:Ptr{origin,loan,offset:0},end:Ptr{origin,loan,offset:128}}).insert(3,Val::Dead).insert(4,Val::Dead).insert(100,Val::Dead).insert(101,Val::Dead).insert(200,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Dead).insert(203,Val::Dead).insert(204,Val::Dead).insert(205,Val::Raw{p:Ptr{origin,loan,offset:128},meta:None}).insert(206,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)).insert(10002,Val::Unit),custodian:0,stack:seq![],ok:true} }
pub open spec fn state_43(bytes:Seq<u8>,origin:int,loan:int) -> State { State{bytes,origin,loan,regs:Map::<int,Val>::empty().insert(0,Val::Unit).insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}).insert(2,Val::Dead).insert(3,Val::Dead).insert(4,Val::Dead).insert(100,Val::Dead).insert(101,Val::Dead).insert(200,Val::Dead).insert(201,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:true}).insert(202,Val::Dead).insert(203,Val::Dead).insert(204,Val::Dead).insert(205,Val::Raw{p:Ptr{origin,loan,offset:128},meta:None}).insert(206,Val::Dead).insert(207,Val::Dead).insert(208,Val::Dead).insert(209,Val::Dead).insert(10001,Val::Num(0)).insert(10002,Val::Unit),custodian:0,stack:seq![],ok:true} }
pub proof fn edge_0(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_0(bytes,origin,loan),Op::Clear(2)) == state_1(bytes,origin,loan),
 wf(state_1(bytes,origin,loan)),
{}
pub proof fn edge_1(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_1(bytes,origin,loan),Op::Clear(3)) == state_2(bytes,origin,loan),
 wf(state_2(bytes,origin,loan)),
{}
pub proof fn edge_2(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_2(bytes,origin,loan),Op::Clear(4)) == state_3(bytes,origin,loan),
 wf(state_3(bytes,origin,loan)),
{}
pub proof fn edge_3(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_3(bytes,origin,loan),Op::Reborrow(4,1)) == state_4(bytes,origin,loan),
 wf(state_4(bytes,origin,loan)),
{}
pub proof fn edge_4(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_4(bytes,origin,loan),Op::Unsize(3,4)) == state_5(bytes,origin,loan),
 wf(state_5(bytes,origin,loan)),
{}
pub proof fn edge_5(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_5(bytes,origin,loan),Op::Clear(4)) == state_6(bytes,origin,loan),
 wf(state_6(bytes,origin,loan)),
{}
pub proof fn edge_6(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_6(bytes,origin,loan),Op::Move(101,3)) == state_7(bytes,origin,loan),
 wf(state_7(bytes,origin,loan)),
{}
pub proof fn edge_7(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_7(bytes,origin,loan),Op::Enter(1,101)) == state_8(bytes,origin,loan),
 wf(state_8(bytes,origin,loan)),
{}
pub proof fn edge_8(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_8(bytes,origin,loan),Op::Move(201,101)) == state_9(bytes,origin,loan),
 wf(state_9(bytes,origin,loan)),
{}
pub proof fn edge_9(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_9(bytes,origin,loan),Op::Enter(2,201)) == state_10(bytes,origin,loan),
 wf(state_10(bytes,origin,loan)),
{}
pub proof fn edge_10(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_10(bytes,origin,loan),Op::Clear(202)) == state_11(bytes,origin,loan),
 wf(state_11(bytes,origin,loan)),
{}
pub proof fn edge_11(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_11(bytes,origin,loan),Op::Metadata(202,201)) == state_12(bytes,origin,loan),
 wf(state_12(bytes,origin,loan)),
{}
pub proof fn edge_12(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_12(bytes,origin,loan),Op::Clear(203)) == state_13(bytes,origin,loan),
 wf(state_13(bytes,origin,loan)),
{}
pub proof fn edge_13(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_13(bytes,origin,loan),Op::Clear(204)) == state_14(bytes,origin,loan),
 wf(state_14(bytes,origin,loan)),
{}
pub proof fn edge_14(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_14(bytes,origin,loan),Op::Clear(207)) == state_15(bytes,origin,loan),
 wf(state_15(bytes,origin,loan)),
{}
pub proof fn edge_15(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_15(bytes,origin,loan),Op::Raw(207,201)) == state_16(bytes,origin,loan),
 wf(state_16(bytes,origin,loan)),
{}
pub proof fn edge_16(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_16(bytes,origin,loan),Op::Wrap(204,207)) == state_17(bytes,origin,loan),
 wf(state_17(bytes,origin,loan)),
{}
pub proof fn edge_17(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_17(bytes,origin,loan),Op::Clear(207)) == state_18(bytes,origin,loan),
 wf(state_18(bytes,origin,loan)),
{}
pub proof fn edge_18(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_18(bytes,origin,loan),Op::Clear(208)) == state_19(bytes,origin,loan),
 wf(state_19(bytes,origin,loan)),
{}
pub proof fn edge_19(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_19(bytes,origin,loan),Op::Clear(209)) == state_20(bytes,origin,loan),
 wf(state_20(bytes,origin,loan)),
{}
pub proof fn edge_20(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_20(bytes,origin,loan),Op::Unwrap(209,204)) == state_21(bytes,origin,loan),
 wf(state_21(bytes,origin,loan)),
{}
pub proof fn edge_21(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_21(bytes,origin,loan),Op::Thin(208,209)) == state_22(bytes,origin,loan),
 wf(state_22(bytes,origin,loan)),
{}
pub proof fn edge_22(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_22(bytes,origin,loan),Op::Clear(209)) == state_23(bytes,origin,loan),
 wf(state_23(bytes,origin,loan)),
{}
pub proof fn edge_23(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_23(bytes,origin,loan),Op::Wrap(203,208)) == state_24(bytes,origin,loan),
 wf(state_24(bytes,origin,loan)),
{}
pub proof fn edge_24(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_24(bytes,origin,loan),Op::Clear(208)) == state_25(bytes,origin,loan),
 wf(state_25(bytes,origin,loan)),
{}
pub proof fn edge_25(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_25(bytes,origin,loan),Op::Clear(204)) == state_26(bytes,origin,loan),
 wf(state_26(bytes,origin,loan)),
{}
pub proof fn edge_26(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_26(bytes,origin,loan),Op::Num(10001,0)) == state_27(bytes,origin,loan),
 wf(state_27(bytes,origin,loan)),
{}
pub proof fn edge_27(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_27(bytes,origin,loan),Op::Test(10001,0)) == state_28(bytes,origin,loan),
 wf(state_28(bytes,origin,loan)),
{}
pub proof fn edge_28(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_28(bytes,origin,loan),Op::Clear(206)) == state_29(bytes,origin,loan),
 wf(state_29(bytes,origin,loan)),
{}
pub proof fn edge_29(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_29(bytes,origin,loan),Op::Unwrap(206,203)) == state_30(bytes,origin,loan),
 wf(state_30(bytes,origin,loan)),
{}
pub proof fn edge_30(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_30(bytes,origin,loan),Op::Offset(205,206,202)) == state_31(bytes,origin,loan),
 wf(state_31(bytes,origin,loan)),
{}
pub proof fn edge_31(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_31(bytes,origin,loan),Op::Clear(206)) == state_32(bytes,origin,loan),
 wf(state_32(bytes,origin,loan)),
{}
pub proof fn edge_32(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_32(bytes,origin,loan),Op::Iter(200,203,205)) == state_33(bytes,origin,loan),
 wf(state_33(bytes,origin,loan)),
{}
pub proof fn edge_33(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_33(bytes,origin,loan),Op::Clear(203)) == state_34(bytes,origin,loan),
 wf(state_34(bytes,origin,loan)),
{}
pub proof fn edge_34(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_34(bytes,origin,loan),Op::Clear(202)) == state_35(bytes,origin,loan),
 wf(state_35(bytes,origin,loan)),
{}
pub proof fn edge_35(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_35(bytes,origin,loan),Op::Leave(2,200)) == state_36(bytes,origin,loan),
 wf(state_36(bytes,origin,loan)),
{}
pub proof fn edge_36(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_36(bytes,origin,loan),Op::Move(100,200)) == state_37(bytes,origin,loan),
 wf(state_37(bytes,origin,loan)),
{}
pub proof fn edge_37(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_37(bytes,origin,loan),Op::Leave(1,100)) == state_38(bytes,origin,loan),
 wf(state_38(bytes,origin,loan)),
{}
pub proof fn edge_38(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_38(bytes,origin,loan),Op::Move(2,100)) == state_39(bytes,origin,loan),
 wf(state_39(bytes,origin,loan)),
{}
pub proof fn edge_39(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_39(bytes,origin,loan),Op::Clear(3)) == state_40(bytes,origin,loan),
 wf(state_40(bytes,origin,loan)),
{}
pub proof fn edge_40(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_40(bytes,origin,loan),Op::Unit(10002)) == state_41(bytes,origin,loan),
 wf(state_41(bytes,origin,loan)),
{}
pub proof fn edge_41(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_41(bytes,origin,loan),Op::Copy(0,10002)) == state_42(bytes,origin,loan),
 wf(state_42(bytes,origin,loan)),
{}
pub proof fn edge_42(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_42(bytes,origin,loan),Op::Clear(2)) == state_43(bytes,origin,loan),
 wf(state_43(bytes,origin,loan)),
{}
pub proof fn checked_drop_trace(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(initial(bytes,origin,loan),program()) == state_43(bytes,origin,loan),
 state_43(bytes,origin,loan).bytes==bytes,
 state_43(bytes,origin,loan).custodian==0,
 state_43(bytes,origin,loan).stack.len()==0,
{
 hide(step);
 embedding(bytes,origin,loan);
 edge_0(bytes,origin,loan);
 edge_1(bytes,origin,loan);
 edge_2(bytes,origin,loan);
 edge_3(bytes,origin,loan);
 edge_4(bytes,origin,loan);
 edge_5(bytes,origin,loan);
 edge_6(bytes,origin,loan);
 edge_7(bytes,origin,loan);
 edge_8(bytes,origin,loan);
 edge_9(bytes,origin,loan);
 edge_10(bytes,origin,loan);
 edge_11(bytes,origin,loan);
 edge_12(bytes,origin,loan);
 edge_13(bytes,origin,loan);
 edge_14(bytes,origin,loan);
 edge_15(bytes,origin,loan);
 edge_16(bytes,origin,loan);
 edge_17(bytes,origin,loan);
 edge_18(bytes,origin,loan);
 edge_19(bytes,origin,loan);
 edge_20(bytes,origin,loan);
 edge_21(bytes,origin,loan);
 edge_22(bytes,origin,loan);
 edge_23(bytes,origin,loan);
 edge_24(bytes,origin,loan);
 edge_25(bytes,origin,loan);
 edge_26(bytes,origin,loan);
 edge_27(bytes,origin,loan);
 edge_28(bytes,origin,loan);
 edge_29(bytes,origin,loan);
 edge_30(bytes,origin,loan);
 edge_31(bytes,origin,loan);
 edge_32(bytes,origin,loan);
 edge_33(bytes,origin,loan);
 edge_34(bytes,origin,loan);
 edge_35(bytes,origin,loan);
 edge_36(bytes,origin,loan);
 edge_37(bytes,origin,loan);
 edge_38(bytes,origin,loan);
 edge_39(bytes,origin,loan);
 edge_40(bytes,origin,loan);
 edge_41(bytes,origin,loan);
 edge_42(bytes,origin,loan);
 reveal_with_fuel(run,45);
}
pub open spec fn program() -> Seq<Op> { seq![Op::Clear(2),Op::Clear(3),Op::Clear(4),Op::Reborrow(4,1),Op::Unsize(3,4),Op::Clear(4),Op::Move(101,3),Op::Enter(1,101),Op::Move(201,101),Op::Enter(2,201),Op::Clear(202),Op::Metadata(202,201),Op::Clear(203),Op::Clear(204),Op::Clear(207),Op::Raw(207,201),Op::Wrap(204,207),Op::Clear(207),Op::Clear(208),Op::Clear(209),Op::Unwrap(209,204),Op::Thin(208,209),Op::Clear(209),Op::Wrap(203,208),Op::Clear(208),Op::Clear(204),Op::Num(10001,0),Op::Test(10001,0),Op::Clear(206),Op::Unwrap(206,203),Op::Offset(205,206,202),Op::Clear(206),Op::Iter(200,203,205),Op::Clear(203),Op::Clear(202),Op::Leave(2,200),Op::Move(100,200),Op::Leave(1,100),Op::Move(2,100),Op::Clear(3),Op::Unit(10002),Op::Copy(0,10002),Op::Clear(2)] }
}
