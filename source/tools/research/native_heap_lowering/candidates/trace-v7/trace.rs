use vstd::prelude::*;
#[path = "../kernel-v5/kernel.rs"] mod kernel;
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
{
 assert(step(state_0(bytes,origin,loan),Op::Clear(2)).regs =~= state_1(bytes,origin,loan).regs);
 assert(step(state_0(bytes,origin,loan),Op::Clear(2)).stack =~= state_1(bytes,origin,loan).stack);
}
pub proof fn edge_1(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_1(bytes,origin,loan),Op::Clear(3)) == state_2(bytes,origin,loan),
 wf(state_2(bytes,origin,loan)),
{
 assert(step(state_1(bytes,origin,loan),Op::Clear(3)).regs =~= state_2(bytes,origin,loan).regs);
 assert(step(state_1(bytes,origin,loan),Op::Clear(3)).stack =~= state_2(bytes,origin,loan).stack);
}
pub proof fn edge_2(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_2(bytes,origin,loan),Op::Clear(4)) == state_3(bytes,origin,loan),
 wf(state_3(bytes,origin,loan)),
{
 assert(step(state_2(bytes,origin,loan),Op::Clear(4)).regs =~= state_3(bytes,origin,loan).regs);
 assert(step(state_2(bytes,origin,loan),Op::Clear(4)).stack =~= state_3(bytes,origin,loan).stack);
}
pub proof fn edge_3(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_3(bytes,origin,loan),Op::Reborrow(4,1)) == state_4(bytes,origin,loan),
 wf(state_4(bytes,origin,loan)),
{
 assert(step(state_3(bytes,origin,loan),Op::Reborrow(4,1)).regs =~= state_4(bytes,origin,loan).regs);
 assert(step(state_3(bytes,origin,loan),Op::Reborrow(4,1)).stack =~= state_4(bytes,origin,loan).stack);
}
pub proof fn edge_4(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_4(bytes,origin,loan),Op::Unsize(3,4)) == state_5(bytes,origin,loan),
 wf(state_5(bytes,origin,loan)),
{
 assert(step(state_4(bytes,origin,loan),Op::Unsize(3,4)).regs =~= state_5(bytes,origin,loan).regs);
 assert(step(state_4(bytes,origin,loan),Op::Unsize(3,4)).stack =~= state_5(bytes,origin,loan).stack);
}
pub proof fn edge_5(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_5(bytes,origin,loan),Op::Clear(4)) == state_6(bytes,origin,loan),
 wf(state_6(bytes,origin,loan)),
{
 assert(step(state_5(bytes,origin,loan),Op::Clear(4)).regs =~= state_6(bytes,origin,loan).regs);
 assert(step(state_5(bytes,origin,loan),Op::Clear(4)).stack =~= state_6(bytes,origin,loan).stack);
}
pub proof fn edge_6(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_6(bytes,origin,loan),Op::Move(101,3)) == state_7(bytes,origin,loan),
 wf(state_7(bytes,origin,loan)),
{
 assert(step(state_6(bytes,origin,loan),Op::Move(101,3)).regs =~= state_7(bytes,origin,loan).regs);
 assert(step(state_6(bytes,origin,loan),Op::Move(101,3)).stack =~= state_7(bytes,origin,loan).stack);
}
pub proof fn edge_7(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_7(bytes,origin,loan),Op::Enter(1,101)) == state_8(bytes,origin,loan),
 wf(state_8(bytes,origin,loan)),
{
 assert(step(state_7(bytes,origin,loan),Op::Enter(1,101)).regs =~= state_8(bytes,origin,loan).regs);
 assert(step(state_7(bytes,origin,loan),Op::Enter(1,101)).stack =~= state_8(bytes,origin,loan).stack);
}
pub proof fn edge_8(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_8(bytes,origin,loan),Op::Move(201,101)) == state_9(bytes,origin,loan),
 wf(state_9(bytes,origin,loan)),
{
 assert(step(state_8(bytes,origin,loan),Op::Move(201,101)).regs =~= state_9(bytes,origin,loan).regs);
 assert(step(state_8(bytes,origin,loan),Op::Move(201,101)).stack =~= state_9(bytes,origin,loan).stack);
}
pub proof fn edge_9(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_9(bytes,origin,loan),Op::Enter(2,201)) == state_10(bytes,origin,loan),
 wf(state_10(bytes,origin,loan)),
{
 assert(step(state_9(bytes,origin,loan),Op::Enter(2,201)).regs =~= state_10(bytes,origin,loan).regs);
 assert(step(state_9(bytes,origin,loan),Op::Enter(2,201)).stack =~= state_10(bytes,origin,loan).stack);
}
pub proof fn edge_10(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_10(bytes,origin,loan),Op::Clear(202)) == state_11(bytes,origin,loan),
 wf(state_11(bytes,origin,loan)),
{
 assert(step(state_10(bytes,origin,loan),Op::Clear(202)).regs =~= state_11(bytes,origin,loan).regs);
 assert(step(state_10(bytes,origin,loan),Op::Clear(202)).stack =~= state_11(bytes,origin,loan).stack);
}
pub proof fn edge_11(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_11(bytes,origin,loan),Op::Metadata(202,201)) == state_12(bytes,origin,loan),
 wf(state_12(bytes,origin,loan)),
{
 assert(step(state_11(bytes,origin,loan),Op::Metadata(202,201)).regs =~= state_12(bytes,origin,loan).regs);
 assert(step(state_11(bytes,origin,loan),Op::Metadata(202,201)).stack =~= state_12(bytes,origin,loan).stack);
}
pub proof fn edge_12(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_12(bytes,origin,loan),Op::Clear(203)) == state_13(bytes,origin,loan),
 wf(state_13(bytes,origin,loan)),
{
 assert(step(state_12(bytes,origin,loan),Op::Clear(203)).regs =~= state_13(bytes,origin,loan).regs);
 assert(step(state_12(bytes,origin,loan),Op::Clear(203)).stack =~= state_13(bytes,origin,loan).stack);
}
pub proof fn edge_13(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_13(bytes,origin,loan),Op::Clear(204)) == state_14(bytes,origin,loan),
 wf(state_14(bytes,origin,loan)),
{
 assert(step(state_13(bytes,origin,loan),Op::Clear(204)).regs =~= state_14(bytes,origin,loan).regs);
 assert(step(state_13(bytes,origin,loan),Op::Clear(204)).stack =~= state_14(bytes,origin,loan).stack);
}
pub proof fn edge_14(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_14(bytes,origin,loan),Op::Clear(207)) == state_15(bytes,origin,loan),
 wf(state_15(bytes,origin,loan)),
{
 assert(step(state_14(bytes,origin,loan),Op::Clear(207)).regs =~= state_15(bytes,origin,loan).regs);
 assert(step(state_14(bytes,origin,loan),Op::Clear(207)).stack =~= state_15(bytes,origin,loan).stack);
}
pub proof fn edge_15(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_15(bytes,origin,loan),Op::Raw(207,201)) == state_16(bytes,origin,loan),
 wf(state_16(bytes,origin,loan)),
{
 assert(step(state_15(bytes,origin,loan),Op::Raw(207,201)).regs =~= state_16(bytes,origin,loan).regs);
 assert(step(state_15(bytes,origin,loan),Op::Raw(207,201)).stack =~= state_16(bytes,origin,loan).stack);
}
pub proof fn edge_16(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_16(bytes,origin,loan),Op::Wrap(204,207)) == state_17(bytes,origin,loan),
 wf(state_17(bytes,origin,loan)),
{
 assert(step(state_16(bytes,origin,loan),Op::Wrap(204,207)).regs =~= state_17(bytes,origin,loan).regs);
 assert(step(state_16(bytes,origin,loan),Op::Wrap(204,207)).stack =~= state_17(bytes,origin,loan).stack);
}
pub proof fn edge_17(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_17(bytes,origin,loan),Op::Clear(207)) == state_18(bytes,origin,loan),
 wf(state_18(bytes,origin,loan)),
{
 assert(step(state_17(bytes,origin,loan),Op::Clear(207)).regs =~= state_18(bytes,origin,loan).regs);
 assert(step(state_17(bytes,origin,loan),Op::Clear(207)).stack =~= state_18(bytes,origin,loan).stack);
}
pub proof fn edge_18(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_18(bytes,origin,loan),Op::Clear(208)) == state_19(bytes,origin,loan),
 wf(state_19(bytes,origin,loan)),
{
 assert(step(state_18(bytes,origin,loan),Op::Clear(208)).regs =~= state_19(bytes,origin,loan).regs);
 assert(step(state_18(bytes,origin,loan),Op::Clear(208)).stack =~= state_19(bytes,origin,loan).stack);
}
pub proof fn edge_19(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_19(bytes,origin,loan),Op::Clear(209)) == state_20(bytes,origin,loan),
 wf(state_20(bytes,origin,loan)),
{
 assert(step(state_19(bytes,origin,loan),Op::Clear(209)).regs =~= state_20(bytes,origin,loan).regs);
 assert(step(state_19(bytes,origin,loan),Op::Clear(209)).stack =~= state_20(bytes,origin,loan).stack);
}
pub proof fn edge_20(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_20(bytes,origin,loan),Op::Unwrap(209,204)) == state_21(bytes,origin,loan),
 wf(state_21(bytes,origin,loan)),
{
 assert(step(state_20(bytes,origin,loan),Op::Unwrap(209,204)).regs =~= state_21(bytes,origin,loan).regs);
 assert(step(state_20(bytes,origin,loan),Op::Unwrap(209,204)).stack =~= state_21(bytes,origin,loan).stack);
}
pub proof fn edge_21(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_21(bytes,origin,loan),Op::Thin(208,209)) == state_22(bytes,origin,loan),
 wf(state_22(bytes,origin,loan)),
{
 assert(step(state_21(bytes,origin,loan),Op::Thin(208,209)).regs =~= state_22(bytes,origin,loan).regs);
 assert(step(state_21(bytes,origin,loan),Op::Thin(208,209)).stack =~= state_22(bytes,origin,loan).stack);
}
pub proof fn edge_22(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_22(bytes,origin,loan),Op::Clear(209)) == state_23(bytes,origin,loan),
 wf(state_23(bytes,origin,loan)),
{
 assert(step(state_22(bytes,origin,loan),Op::Clear(209)).regs =~= state_23(bytes,origin,loan).regs);
 assert(step(state_22(bytes,origin,loan),Op::Clear(209)).stack =~= state_23(bytes,origin,loan).stack);
}
pub proof fn edge_23(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_23(bytes,origin,loan),Op::Wrap(203,208)) == state_24(bytes,origin,loan),
 wf(state_24(bytes,origin,loan)),
{
 assert(step(state_23(bytes,origin,loan),Op::Wrap(203,208)).regs =~= state_24(bytes,origin,loan).regs);
 assert(step(state_23(bytes,origin,loan),Op::Wrap(203,208)).stack =~= state_24(bytes,origin,loan).stack);
}
pub proof fn edge_24(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_24(bytes,origin,loan),Op::Clear(208)) == state_25(bytes,origin,loan),
 wf(state_25(bytes,origin,loan)),
{
 assert(step(state_24(bytes,origin,loan),Op::Clear(208)).regs =~= state_25(bytes,origin,loan).regs);
 assert(step(state_24(bytes,origin,loan),Op::Clear(208)).stack =~= state_25(bytes,origin,loan).stack);
}
pub proof fn edge_25(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_25(bytes,origin,loan),Op::Clear(204)) == state_26(bytes,origin,loan),
 wf(state_26(bytes,origin,loan)),
{
 assert(step(state_25(bytes,origin,loan),Op::Clear(204)).regs =~= state_26(bytes,origin,loan).regs);
 assert(step(state_25(bytes,origin,loan),Op::Clear(204)).stack =~= state_26(bytes,origin,loan).stack);
}
pub proof fn edge_26(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_26(bytes,origin,loan),Op::Num(10001,0)) == state_27(bytes,origin,loan),
 wf(state_27(bytes,origin,loan)),
{
 assert(step(state_26(bytes,origin,loan),Op::Num(10001,0)).regs =~= state_27(bytes,origin,loan).regs);
 assert(step(state_26(bytes,origin,loan),Op::Num(10001,0)).stack =~= state_27(bytes,origin,loan).stack);
}
pub proof fn edge_27(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_27(bytes,origin,loan),Op::Test(10001,0)) == state_28(bytes,origin,loan),
 wf(state_28(bytes,origin,loan)),
{
 assert(step(state_27(bytes,origin,loan),Op::Test(10001,0)).regs =~= state_28(bytes,origin,loan).regs);
 assert(step(state_27(bytes,origin,loan),Op::Test(10001,0)).stack =~= state_28(bytes,origin,loan).stack);
}
pub proof fn edge_28(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_28(bytes,origin,loan),Op::Clear(206)) == state_29(bytes,origin,loan),
 wf(state_29(bytes,origin,loan)),
{
 assert(step(state_28(bytes,origin,loan),Op::Clear(206)).regs =~= state_29(bytes,origin,loan).regs);
 assert(step(state_28(bytes,origin,loan),Op::Clear(206)).stack =~= state_29(bytes,origin,loan).stack);
}
pub proof fn edge_29(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_29(bytes,origin,loan),Op::Unwrap(206,203)) == state_30(bytes,origin,loan),
 wf(state_30(bytes,origin,loan)),
{
 assert(step(state_29(bytes,origin,loan),Op::Unwrap(206,203)).regs =~= state_30(bytes,origin,loan).regs);
 assert(step(state_29(bytes,origin,loan),Op::Unwrap(206,203)).stack =~= state_30(bytes,origin,loan).stack);
}
pub proof fn edge_30(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_30(bytes,origin,loan),Op::Offset(205,206,202)) == state_31(bytes,origin,loan),
 wf(state_31(bytes,origin,loan)),
{
 assert(step(state_30(bytes,origin,loan),Op::Offset(205,206,202)).regs =~= state_31(bytes,origin,loan).regs);
 assert(step(state_30(bytes,origin,loan),Op::Offset(205,206,202)).stack =~= state_31(bytes,origin,loan).stack);
}
pub proof fn edge_31(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_31(bytes,origin,loan),Op::Clear(206)) == state_32(bytes,origin,loan),
 wf(state_32(bytes,origin,loan)),
{
 assert(step(state_31(bytes,origin,loan),Op::Clear(206)).regs =~= state_32(bytes,origin,loan).regs);
 assert(step(state_31(bytes,origin,loan),Op::Clear(206)).stack =~= state_32(bytes,origin,loan).stack);
}
pub proof fn edge_32(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_32(bytes,origin,loan),Op::Iter(200,203,205)) == state_33(bytes,origin,loan),
 wf(state_33(bytes,origin,loan)),
{
 assert(step(state_32(bytes,origin,loan),Op::Iter(200,203,205)).regs =~= state_33(bytes,origin,loan).regs);
 assert(step(state_32(bytes,origin,loan),Op::Iter(200,203,205)).stack =~= state_33(bytes,origin,loan).stack);
}
pub proof fn edge_33(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_33(bytes,origin,loan),Op::Clear(203)) == state_34(bytes,origin,loan),
 wf(state_34(bytes,origin,loan)),
{
 assert(step(state_33(bytes,origin,loan),Op::Clear(203)).regs =~= state_34(bytes,origin,loan).regs);
 assert(step(state_33(bytes,origin,loan),Op::Clear(203)).stack =~= state_34(bytes,origin,loan).stack);
}
pub proof fn edge_34(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_34(bytes,origin,loan),Op::Clear(202)) == state_35(bytes,origin,loan),
 wf(state_35(bytes,origin,loan)),
{
 assert(step(state_34(bytes,origin,loan),Op::Clear(202)).regs =~= state_35(bytes,origin,loan).regs);
 assert(step(state_34(bytes,origin,loan),Op::Clear(202)).stack =~= state_35(bytes,origin,loan).stack);
}
pub proof fn edge_35(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_35(bytes,origin,loan),Op::Leave(2,200)) == state_36(bytes,origin,loan),
 wf(state_36(bytes,origin,loan)),
{
 assert(step(state_35(bytes,origin,loan),Op::Leave(2,200)).regs =~= state_36(bytes,origin,loan).regs);
 assert(step(state_35(bytes,origin,loan),Op::Leave(2,200)).stack =~= state_36(bytes,origin,loan).stack);
}
pub proof fn edge_36(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_36(bytes,origin,loan),Op::Move(100,200)) == state_37(bytes,origin,loan),
 wf(state_37(bytes,origin,loan)),
{
 assert(step(state_36(bytes,origin,loan),Op::Move(100,200)).regs =~= state_37(bytes,origin,loan).regs);
 assert(step(state_36(bytes,origin,loan),Op::Move(100,200)).stack =~= state_37(bytes,origin,loan).stack);
}
pub proof fn edge_37(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_37(bytes,origin,loan),Op::Leave(1,100)) == state_38(bytes,origin,loan),
 wf(state_38(bytes,origin,loan)),
{
 assert(step(state_37(bytes,origin,loan),Op::Leave(1,100)).regs =~= state_38(bytes,origin,loan).regs);
 assert(step(state_37(bytes,origin,loan),Op::Leave(1,100)).stack =~= state_38(bytes,origin,loan).stack);
}
pub proof fn edge_38(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_38(bytes,origin,loan),Op::Move(2,100)) == state_39(bytes,origin,loan),
 wf(state_39(bytes,origin,loan)),
{
 assert(step(state_38(bytes,origin,loan),Op::Move(2,100)).regs =~= state_39(bytes,origin,loan).regs);
 assert(step(state_38(bytes,origin,loan),Op::Move(2,100)).stack =~= state_39(bytes,origin,loan).stack);
}
pub proof fn edge_39(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_39(bytes,origin,loan),Op::Clear(3)) == state_40(bytes,origin,loan),
 wf(state_40(bytes,origin,loan)),
{
 assert(step(state_39(bytes,origin,loan),Op::Clear(3)).regs =~= state_40(bytes,origin,loan).regs);
 assert(step(state_39(bytes,origin,loan),Op::Clear(3)).stack =~= state_40(bytes,origin,loan).stack);
}
pub proof fn edge_40(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_40(bytes,origin,loan),Op::Unit(10002)) == state_41(bytes,origin,loan),
 wf(state_41(bytes,origin,loan)),
{
 assert(step(state_40(bytes,origin,loan),Op::Unit(10002)).regs =~= state_41(bytes,origin,loan).regs);
 assert(step(state_40(bytes,origin,loan),Op::Unit(10002)).stack =~= state_41(bytes,origin,loan).stack);
}
pub proof fn edge_41(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_41(bytes,origin,loan),Op::Copy(0,10002)) == state_42(bytes,origin,loan),
 wf(state_42(bytes,origin,loan)),
{
 assert(step(state_41(bytes,origin,loan),Op::Copy(0,10002)).regs =~= state_42(bytes,origin,loan).regs);
 assert(step(state_41(bytes,origin,loan),Op::Copy(0,10002)).stack =~= state_42(bytes,origin,loan).stack);
}
pub proof fn edge_42(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures step(state_42(bytes,origin,loan),Op::Clear(2)) == state_43(bytes,origin,loan),
 wf(state_43(bytes,origin,loan)),
{
 assert(step(state_42(bytes,origin,loan),Op::Clear(2)).regs =~= state_43(bytes,origin,loan).regs);
 assert(step(state_42(bytes,origin,loan),Op::Clear(2)).stack =~= state_43(bytes,origin,loan).stack);
}
pub proof fn start(bytes:Seq<u8>,origin:int,loan:int) requires bytes.len()==128, ensures initial(bytes,origin,loan)==state_0(bytes,origin,loan), {}
pub proof fn finish(bytes:Seq<u8>,origin:int,loan:int) ensures state_43(bytes,origin,loan).bytes==bytes, state_43(bytes,origin,loan).custodian==0, state_43(bytes,origin,loan).stack.len()==0, {}
pub closed spec fn tail_43() -> Seq<Op> { seq![] }
pub proof fn path_43(bytes:Seq<u8>,origin:int,loan:int) ensures run(state_43(bytes,origin,loan),tail_43())==state_43(bytes,origin,loan), { reveal(tail_43); }
pub closed spec fn tail_42() -> Seq<Op> { seq![Op::Clear(2)]+tail_43() }
pub proof fn path_42(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_42(bytes,origin,loan),tail_42())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_42);
 hide(state_43);
 reveal(tail_42);
 edge_42(bytes,origin,loan);
 path_43(bytes,origin,loan);
 run_cons(state_42(bytes,origin,loan),Op::Clear(2),tail_43());
}
pub closed spec fn tail_41() -> Seq<Op> { seq![Op::Copy(0,10002)]+tail_42() }
pub proof fn path_41(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_41(bytes,origin,loan),tail_41())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_41);
 hide(state_42);
 hide(state_43);
 reveal(tail_41);
 edge_41(bytes,origin,loan);
 path_42(bytes,origin,loan);
 run_cons(state_41(bytes,origin,loan),Op::Copy(0,10002),tail_42());
}
pub closed spec fn tail_40() -> Seq<Op> { seq![Op::Unit(10002)]+tail_41() }
pub proof fn path_40(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_40(bytes,origin,loan),tail_40())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_40);
 hide(state_41);
 hide(state_43);
 reveal(tail_40);
 edge_40(bytes,origin,loan);
 path_41(bytes,origin,loan);
 run_cons(state_40(bytes,origin,loan),Op::Unit(10002),tail_41());
}
pub closed spec fn tail_39() -> Seq<Op> { seq![Op::Clear(3)]+tail_40() }
pub proof fn path_39(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_39(bytes,origin,loan),tail_39())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_39);
 hide(state_40);
 hide(state_43);
 reveal(tail_39);
 edge_39(bytes,origin,loan);
 path_40(bytes,origin,loan);
 run_cons(state_39(bytes,origin,loan),Op::Clear(3),tail_40());
}
pub closed spec fn tail_38() -> Seq<Op> { seq![Op::Move(2,100)]+tail_39() }
pub proof fn path_38(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_38(bytes,origin,loan),tail_38())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_38);
 hide(state_39);
 hide(state_43);
 reveal(tail_38);
 edge_38(bytes,origin,loan);
 path_39(bytes,origin,loan);
 run_cons(state_38(bytes,origin,loan),Op::Move(2,100),tail_39());
}
pub closed spec fn tail_37() -> Seq<Op> { seq![Op::Leave(1,100)]+tail_38() }
pub proof fn path_37(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_37(bytes,origin,loan),tail_37())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_37);
 hide(state_38);
 hide(state_43);
 reveal(tail_37);
 edge_37(bytes,origin,loan);
 path_38(bytes,origin,loan);
 run_cons(state_37(bytes,origin,loan),Op::Leave(1,100),tail_38());
}
pub closed spec fn tail_36() -> Seq<Op> { seq![Op::Move(100,200)]+tail_37() }
pub proof fn path_36(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_36(bytes,origin,loan),tail_36())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_36);
 hide(state_37);
 hide(state_43);
 reveal(tail_36);
 edge_36(bytes,origin,loan);
 path_37(bytes,origin,loan);
 run_cons(state_36(bytes,origin,loan),Op::Move(100,200),tail_37());
}
pub closed spec fn tail_35() -> Seq<Op> { seq![Op::Leave(2,200)]+tail_36() }
pub proof fn path_35(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_35(bytes,origin,loan),tail_35())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_35);
 hide(state_36);
 hide(state_43);
 reveal(tail_35);
 edge_35(bytes,origin,loan);
 path_36(bytes,origin,loan);
 run_cons(state_35(bytes,origin,loan),Op::Leave(2,200),tail_36());
}
pub closed spec fn tail_34() -> Seq<Op> { seq![Op::Clear(202)]+tail_35() }
pub proof fn path_34(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_34(bytes,origin,loan),tail_34())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_34);
 hide(state_35);
 hide(state_43);
 reveal(tail_34);
 edge_34(bytes,origin,loan);
 path_35(bytes,origin,loan);
 run_cons(state_34(bytes,origin,loan),Op::Clear(202),tail_35());
}
pub closed spec fn tail_33() -> Seq<Op> { seq![Op::Clear(203)]+tail_34() }
pub proof fn path_33(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_33(bytes,origin,loan),tail_33())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_33);
 hide(state_34);
 hide(state_43);
 reveal(tail_33);
 edge_33(bytes,origin,loan);
 path_34(bytes,origin,loan);
 run_cons(state_33(bytes,origin,loan),Op::Clear(203),tail_34());
}
pub closed spec fn tail_32() -> Seq<Op> { seq![Op::Iter(200,203,205)]+tail_33() }
pub proof fn path_32(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_32(bytes,origin,loan),tail_32())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_32);
 hide(state_33);
 hide(state_43);
 reveal(tail_32);
 edge_32(bytes,origin,loan);
 path_33(bytes,origin,loan);
 run_cons(state_32(bytes,origin,loan),Op::Iter(200,203,205),tail_33());
}
pub closed spec fn tail_31() -> Seq<Op> { seq![Op::Clear(206)]+tail_32() }
pub proof fn path_31(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_31(bytes,origin,loan),tail_31())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_31);
 hide(state_32);
 hide(state_43);
 reveal(tail_31);
 edge_31(bytes,origin,loan);
 path_32(bytes,origin,loan);
 run_cons(state_31(bytes,origin,loan),Op::Clear(206),tail_32());
}
pub closed spec fn tail_30() -> Seq<Op> { seq![Op::Offset(205,206,202)]+tail_31() }
pub proof fn path_30(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_30(bytes,origin,loan),tail_30())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_30);
 hide(state_31);
 hide(state_43);
 reveal(tail_30);
 edge_30(bytes,origin,loan);
 path_31(bytes,origin,loan);
 run_cons(state_30(bytes,origin,loan),Op::Offset(205,206,202),tail_31());
}
pub closed spec fn tail_29() -> Seq<Op> { seq![Op::Unwrap(206,203)]+tail_30() }
pub proof fn path_29(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_29(bytes,origin,loan),tail_29())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_29);
 hide(state_30);
 hide(state_43);
 reveal(tail_29);
 edge_29(bytes,origin,loan);
 path_30(bytes,origin,loan);
 run_cons(state_29(bytes,origin,loan),Op::Unwrap(206,203),tail_30());
}
pub closed spec fn tail_28() -> Seq<Op> { seq![Op::Clear(206)]+tail_29() }
pub proof fn path_28(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_28(bytes,origin,loan),tail_28())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_28);
 hide(state_29);
 hide(state_43);
 reveal(tail_28);
 edge_28(bytes,origin,loan);
 path_29(bytes,origin,loan);
 run_cons(state_28(bytes,origin,loan),Op::Clear(206),tail_29());
}
pub closed spec fn tail_27() -> Seq<Op> { seq![Op::Test(10001,0)]+tail_28() }
pub proof fn path_27(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_27(bytes,origin,loan),tail_27())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_27);
 hide(state_28);
 hide(state_43);
 reveal(tail_27);
 edge_27(bytes,origin,loan);
 path_28(bytes,origin,loan);
 run_cons(state_27(bytes,origin,loan),Op::Test(10001,0),tail_28());
}
pub closed spec fn tail_26() -> Seq<Op> { seq![Op::Num(10001,0)]+tail_27() }
pub proof fn path_26(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_26(bytes,origin,loan),tail_26())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_26);
 hide(state_27);
 hide(state_43);
 reveal(tail_26);
 edge_26(bytes,origin,loan);
 path_27(bytes,origin,loan);
 run_cons(state_26(bytes,origin,loan),Op::Num(10001,0),tail_27());
}
pub closed spec fn tail_25() -> Seq<Op> { seq![Op::Clear(204)]+tail_26() }
pub proof fn path_25(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_25(bytes,origin,loan),tail_25())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_25);
 hide(state_26);
 hide(state_43);
 reveal(tail_25);
 edge_25(bytes,origin,loan);
 path_26(bytes,origin,loan);
 run_cons(state_25(bytes,origin,loan),Op::Clear(204),tail_26());
}
pub closed spec fn tail_24() -> Seq<Op> { seq![Op::Clear(208)]+tail_25() }
pub proof fn path_24(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_24(bytes,origin,loan),tail_24())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_24);
 hide(state_25);
 hide(state_43);
 reveal(tail_24);
 edge_24(bytes,origin,loan);
 path_25(bytes,origin,loan);
 run_cons(state_24(bytes,origin,loan),Op::Clear(208),tail_25());
}
pub closed spec fn tail_23() -> Seq<Op> { seq![Op::Wrap(203,208)]+tail_24() }
pub proof fn path_23(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_23(bytes,origin,loan),tail_23())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_23);
 hide(state_24);
 hide(state_43);
 reveal(tail_23);
 edge_23(bytes,origin,loan);
 path_24(bytes,origin,loan);
 run_cons(state_23(bytes,origin,loan),Op::Wrap(203,208),tail_24());
}
pub closed spec fn tail_22() -> Seq<Op> { seq![Op::Clear(209)]+tail_23() }
pub proof fn path_22(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_22(bytes,origin,loan),tail_22())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_22);
 hide(state_23);
 hide(state_43);
 reveal(tail_22);
 edge_22(bytes,origin,loan);
 path_23(bytes,origin,loan);
 run_cons(state_22(bytes,origin,loan),Op::Clear(209),tail_23());
}
pub closed spec fn tail_21() -> Seq<Op> { seq![Op::Thin(208,209)]+tail_22() }
pub proof fn path_21(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_21(bytes,origin,loan),tail_21())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_21);
 hide(state_22);
 hide(state_43);
 reveal(tail_21);
 edge_21(bytes,origin,loan);
 path_22(bytes,origin,loan);
 run_cons(state_21(bytes,origin,loan),Op::Thin(208,209),tail_22());
}
pub closed spec fn tail_20() -> Seq<Op> { seq![Op::Unwrap(209,204)]+tail_21() }
pub proof fn path_20(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_20(bytes,origin,loan),tail_20())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_20);
 hide(state_21);
 hide(state_43);
 reveal(tail_20);
 edge_20(bytes,origin,loan);
 path_21(bytes,origin,loan);
 run_cons(state_20(bytes,origin,loan),Op::Unwrap(209,204),tail_21());
}
pub closed spec fn tail_19() -> Seq<Op> { seq![Op::Clear(209)]+tail_20() }
pub proof fn path_19(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_19(bytes,origin,loan),tail_19())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_19);
 hide(state_20);
 hide(state_43);
 reveal(tail_19);
 edge_19(bytes,origin,loan);
 path_20(bytes,origin,loan);
 run_cons(state_19(bytes,origin,loan),Op::Clear(209),tail_20());
}
pub closed spec fn tail_18() -> Seq<Op> { seq![Op::Clear(208)]+tail_19() }
pub proof fn path_18(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_18(bytes,origin,loan),tail_18())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_18);
 hide(state_19);
 hide(state_43);
 reveal(tail_18);
 edge_18(bytes,origin,loan);
 path_19(bytes,origin,loan);
 run_cons(state_18(bytes,origin,loan),Op::Clear(208),tail_19());
}
pub closed spec fn tail_17() -> Seq<Op> { seq![Op::Clear(207)]+tail_18() }
pub proof fn path_17(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_17(bytes,origin,loan),tail_17())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_17);
 hide(state_18);
 hide(state_43);
 reveal(tail_17);
 edge_17(bytes,origin,loan);
 path_18(bytes,origin,loan);
 run_cons(state_17(bytes,origin,loan),Op::Clear(207),tail_18());
}
pub closed spec fn tail_16() -> Seq<Op> { seq![Op::Wrap(204,207)]+tail_17() }
pub proof fn path_16(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_16(bytes,origin,loan),tail_16())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_16);
 hide(state_17);
 hide(state_43);
 reveal(tail_16);
 edge_16(bytes,origin,loan);
 path_17(bytes,origin,loan);
 run_cons(state_16(bytes,origin,loan),Op::Wrap(204,207),tail_17());
}
pub closed spec fn tail_15() -> Seq<Op> { seq![Op::Raw(207,201)]+tail_16() }
pub proof fn path_15(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_15(bytes,origin,loan),tail_15())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_15);
 hide(state_16);
 hide(state_43);
 reveal(tail_15);
 edge_15(bytes,origin,loan);
 path_16(bytes,origin,loan);
 run_cons(state_15(bytes,origin,loan),Op::Raw(207,201),tail_16());
}
pub closed spec fn tail_14() -> Seq<Op> { seq![Op::Clear(207)]+tail_15() }
pub proof fn path_14(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_14(bytes,origin,loan),tail_14())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_14);
 hide(state_15);
 hide(state_43);
 reveal(tail_14);
 edge_14(bytes,origin,loan);
 path_15(bytes,origin,loan);
 run_cons(state_14(bytes,origin,loan),Op::Clear(207),tail_15());
}
pub closed spec fn tail_13() -> Seq<Op> { seq![Op::Clear(204)]+tail_14() }
pub proof fn path_13(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_13(bytes,origin,loan),tail_13())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_13);
 hide(state_14);
 hide(state_43);
 reveal(tail_13);
 edge_13(bytes,origin,loan);
 path_14(bytes,origin,loan);
 run_cons(state_13(bytes,origin,loan),Op::Clear(204),tail_14());
}
pub closed spec fn tail_12() -> Seq<Op> { seq![Op::Clear(203)]+tail_13() }
pub proof fn path_12(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_12(bytes,origin,loan),tail_12())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_12);
 hide(state_13);
 hide(state_43);
 reveal(tail_12);
 edge_12(bytes,origin,loan);
 path_13(bytes,origin,loan);
 run_cons(state_12(bytes,origin,loan),Op::Clear(203),tail_13());
}
pub closed spec fn tail_11() -> Seq<Op> { seq![Op::Metadata(202,201)]+tail_12() }
pub proof fn path_11(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_11(bytes,origin,loan),tail_11())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_11);
 hide(state_12);
 hide(state_43);
 reveal(tail_11);
 edge_11(bytes,origin,loan);
 path_12(bytes,origin,loan);
 run_cons(state_11(bytes,origin,loan),Op::Metadata(202,201),tail_12());
}
pub closed spec fn tail_10() -> Seq<Op> { seq![Op::Clear(202)]+tail_11() }
pub proof fn path_10(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_10(bytes,origin,loan),tail_10())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_10);
 hide(state_11);
 hide(state_43);
 reveal(tail_10);
 edge_10(bytes,origin,loan);
 path_11(bytes,origin,loan);
 run_cons(state_10(bytes,origin,loan),Op::Clear(202),tail_11());
}
pub closed spec fn tail_9() -> Seq<Op> { seq![Op::Enter(2,201)]+tail_10() }
pub proof fn path_9(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_9(bytes,origin,loan),tail_9())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_9);
 hide(state_10);
 hide(state_43);
 reveal(tail_9);
 edge_9(bytes,origin,loan);
 path_10(bytes,origin,loan);
 run_cons(state_9(bytes,origin,loan),Op::Enter(2,201),tail_10());
}
pub closed spec fn tail_8() -> Seq<Op> { seq![Op::Move(201,101)]+tail_9() }
pub proof fn path_8(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_8(bytes,origin,loan),tail_8())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_8);
 hide(state_9);
 hide(state_43);
 reveal(tail_8);
 edge_8(bytes,origin,loan);
 path_9(bytes,origin,loan);
 run_cons(state_8(bytes,origin,loan),Op::Move(201,101),tail_9());
}
pub closed spec fn tail_7() -> Seq<Op> { seq![Op::Enter(1,101)]+tail_8() }
pub proof fn path_7(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_7(bytes,origin,loan),tail_7())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_7);
 hide(state_8);
 hide(state_43);
 reveal(tail_7);
 edge_7(bytes,origin,loan);
 path_8(bytes,origin,loan);
 run_cons(state_7(bytes,origin,loan),Op::Enter(1,101),tail_8());
}
pub closed spec fn tail_6() -> Seq<Op> { seq![Op::Move(101,3)]+tail_7() }
pub proof fn path_6(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_6(bytes,origin,loan),tail_6())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_6);
 hide(state_7);
 hide(state_43);
 reveal(tail_6);
 edge_6(bytes,origin,loan);
 path_7(bytes,origin,loan);
 run_cons(state_6(bytes,origin,loan),Op::Move(101,3),tail_7());
}
pub closed spec fn tail_5() -> Seq<Op> { seq![Op::Clear(4)]+tail_6() }
pub proof fn path_5(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_5(bytes,origin,loan),tail_5())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_5);
 hide(state_6);
 hide(state_43);
 reveal(tail_5);
 edge_5(bytes,origin,loan);
 path_6(bytes,origin,loan);
 run_cons(state_5(bytes,origin,loan),Op::Clear(4),tail_6());
}
pub closed spec fn tail_4() -> Seq<Op> { seq![Op::Unsize(3,4)]+tail_5() }
pub proof fn path_4(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_4(bytes,origin,loan),tail_4())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_4);
 hide(state_5);
 hide(state_43);
 reveal(tail_4);
 edge_4(bytes,origin,loan);
 path_5(bytes,origin,loan);
 run_cons(state_4(bytes,origin,loan),Op::Unsize(3,4),tail_5());
}
pub closed spec fn tail_3() -> Seq<Op> { seq![Op::Reborrow(4,1)]+tail_4() }
pub proof fn path_3(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_3(bytes,origin,loan),tail_3())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_3);
 hide(state_4);
 hide(state_43);
 reveal(tail_3);
 edge_3(bytes,origin,loan);
 path_4(bytes,origin,loan);
 run_cons(state_3(bytes,origin,loan),Op::Reborrow(4,1),tail_4());
}
pub closed spec fn tail_2() -> Seq<Op> { seq![Op::Clear(4)]+tail_3() }
pub proof fn path_2(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_2(bytes,origin,loan),tail_2())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_2);
 hide(state_3);
 hide(state_43);
 reveal(tail_2);
 edge_2(bytes,origin,loan);
 path_3(bytes,origin,loan);
 run_cons(state_2(bytes,origin,loan),Op::Clear(4),tail_3());
}
pub closed spec fn tail_1() -> Seq<Op> { seq![Op::Clear(3)]+tail_2() }
pub proof fn path_1(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_1(bytes,origin,loan),tail_1())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_1);
 hide(state_2);
 hide(state_43);
 reveal(tail_1);
 edge_1(bytes,origin,loan);
 path_2(bytes,origin,loan);
 run_cons(state_1(bytes,origin,loan),Op::Clear(3),tail_2());
}
pub closed spec fn tail_0() -> Seq<Op> { seq![Op::Clear(2)]+tail_1() }
pub proof fn path_0(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(state_0(bytes,origin,loan),tail_0())==state_43(bytes,origin,loan),
{
 hide(step);
 hide(wf);
 hide(state_0);
 hide(state_1);
 hide(state_43);
 reveal(tail_0);
 edge_0(bytes,origin,loan);
 path_1(bytes,origin,loan);
 run_cons(state_0(bytes,origin,loan),Op::Clear(2),tail_1());
}
pub proof fn checked_drop_trace(bytes:Seq<u8>,origin:int,loan:int)
 requires bytes.len()==128,
 ensures run(initial(bytes,origin,loan),tail_0()) == state_43(bytes,origin,loan),
 state_43(bytes,origin,loan).bytes==bytes,
 state_43(bytes,origin,loan).custodian==0,
 state_43(bytes,origin,loan).stack.len()==0,
{
 hide(step);
 hide(state_0);
 hide(state_43);
 start(bytes,origin,loan);
 finish(bytes,origin,loan);
 path_0(bytes,origin,loan);
}
}
