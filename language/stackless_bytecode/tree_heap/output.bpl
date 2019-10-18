type TypeName;
type FieldName;
type LocalName;
type Address = int;
type ByteArray;
type String;

type {:datatype} Edge;
function {:constructor} Field(f: FieldName): Edge;
function {:constructor} Index(i: int): Edge;
function {:constructor} String(s: String): Edge;

type {:datatype} Path;
function {:constructor} Path(p: [int]Edge, size: int): Path;

const DefaultPath: [int]Edge;

type {:datatype} Value;
function {:constructor} Boolean(b: bool): Value;
function {:constructor} Integer(i: int): Value;
function {:constructor} Address(a: Address): Value;
function {:constructor} ByteArray(b: ByteArray): Value;
function {:constructor} Str(a: String): Value;
function {:constructor} Map(m: [Edge]Value): Value;

const DefaultMap: [Edge]Value;

type {:datatype} Reference;
function {:constructor} Reference(rt: RefType, p: Path): Reference;

type {:datatype} RefType;
function {:constructor} Global(a: Address, t: TypeName): RefType;
function {:constructor} Local(l: int): RefType;

type {:datatype} TypeStore;
function {:constructor} TypeStore(domain: [TypeName]bool, contents: [TypeName]Value): TypeStore;

type {:datatype} GlobalStore;
function {:constructor} GlobalStore(domain: [Address]bool, contents: [Address]TypeStore): GlobalStore;

var gs : GlobalStore;
var ls : [int]Value;
var ls_size : int;

procedure {:inline 1} Exists(address: Value, t: TypeName) returns (dst: Value)
requires is#Address(address);
{
    dst := Boolean(domain#GlobalStore(gs)[a#Address(address)] && domain#TypeStore(contents#GlobalStore(gs)[a#Address(address)])[t]);
}

procedure {:inline 1} MoveToSender(t: TypeName, v: Value)
{
    var a: Address;
    var ts: TypeStore;
    a := sender#Transaction_cons(txn);
    assert domain#GlobalStore(gs)[a];
    ts := contents#GlobalStore(gs)[a];
    assert !domain#TypeStore(ts)[t];
    ts := TypeStore(domain#TypeStore(ts)[t := true], contents#TypeStore(ts)[t := v]);
    gs := GlobalStore(domain#GlobalStore(gs), contents#GlobalStore(gs)[a := ts]);
}

procedure {:inline 1} MoveFrom(address: Value, t: TypeName) returns (dst: Value)
requires is#Address(address);
{
    var a: Address;
    var ts: TypeStore;
    a := a#Address(address);
    assert domain#GlobalStore(gs)[a];
    ts := contents#GlobalStore(gs)[a];
    assert domain#TypeStore(ts)[t];
    dst := contents#TypeStore(ts)[t];
    ts := TypeStore(domain#TypeStore(ts)[t := false], contents#TypeStore(ts));
    gs := GlobalStore(domain#GlobalStore(gs), contents#GlobalStore(gs)[a := ts]);
}

procedure {:inline 1} BorrowGlobal(address: Value, t: TypeName) returns (dst: Reference)
requires is#Address(address);
{
    var a: Address;
    var v: Value;
    var ts: TypeStore;
    a := a#Address(address);
    assert domain#GlobalStore(gs)[a];
    ts := contents#GlobalStore(gs)[a];
    assert domain#TypeStore(ts)[t];
    dst := Reference(Global(a, t), Path(DefaultPath, 0));
}

procedure {:inline 1} BorrowLoc(l: int) returns (dst: Reference)
{
    dst := Reference(Local(l), Path(DefaultPath, 0));
}

procedure {:inline 1} BorrowField(src: Reference, f: FieldName) returns (dst: Reference)
{
    var p: Path;
    var size: int;
    p := p#Reference(src);
    size := size#Path(p);
	p := Path(p#Path(p)[size := Field(f)], size+1);
    dst := Reference(rt#Reference(src), p);

}

procedure {:inline 1} WriteRef(to: Reference, new_v: Value)
{
    var a: Address;
    var ts: TypeStore;
    var t: TypeName;
    var v: Value;
    if (is#Global(rt#Reference(to))) {
        a := a#Global(rt#Reference(to));
        assert domain#GlobalStore(gs)[a];
        ts := contents#GlobalStore(gs)[a];
        t := t#Global(rt#Reference(to));
        assert domain#TypeStore(ts)[t];
        v := contents#TypeStore(ts)[t];
        call v := UpdateValueMax(p#Reference(to), 0, v, new_v);
        ts := TypeStore(domain#TypeStore(ts), contents#TypeStore(ts)[t := v]);
        gs := GlobalStore(domain#GlobalStore(gs), contents#GlobalStore(gs)[a := ts]);
    } else {
        v := ls[l#Local(rt#Reference(to))];
        call v := UpdateValueMax(p#Reference(to), 0, v, new_v);
        ls := ls[l#Local(rt#Reference(to)) := v];
    }
}

procedure {:inline 1} ReadRef(from: Reference) returns (v: Value)
{
    var a: Address;
    var ts: TypeStore;
    var t: TypeName;
    if (is#Global(rt#Reference(from))) {
        a := a#Global(rt#Reference(from));
        assert domain#GlobalStore(gs)[a];
        ts := contents#GlobalStore(gs)[a];
        t := t#Global(rt#Reference(from));
        assert domain#TypeStore(ts)[t];
        call v := ReadValueMax(p#Reference(from), 0, contents#TypeStore(ts)[t]);
    } else {
        call v := ReadValueMax(p#Reference(from), 0, ls[l#Local(rt#Reference(from))]);
    }
}


procedure {:inline 1} CopyOrMoveRef(local: Reference) returns (dst: Reference)
{
    dst := local;
}

procedure {:inline 1} CopyOrMoveValue(local: Value) returns (dst: Value)
{
    dst := local;
}

procedure {:inline 1} FreezeRef(src: Reference) returns (dest: Reference)
{
    dest := src;
}

// Eq, Pack, and Unpack are auto-generated for each type T
const MAX_U64: int;
axiom MAX_U64 == 9223372036854775807;
var abort_flag: bool;

procedure {:inline 1} Add(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    if (i#Integer(src1) + i#Integer(src2) > MAX_U64) {
        abort_flag := true;
    }
    dst := Integer(i#Integer(src1) + i#Integer(src2));
}

procedure {:inline 1} Sub(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    if (i#Integer(src1) < i#Integer(src2)) {
        abort_flag := true;
    }
    dst := Integer(i#Integer(src1) - i#Integer(src2));
}

procedure {:inline 1} Mul(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    if (i#Integer(src1) * i#Integer(src2) > MAX_U64) {
        abort_flag := true;
    }
    dst := Integer(i#Integer(src1) * i#Integer(src2));
}

procedure {:inline 1} Div(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    if (i#Integer(src2) == 0) {
        abort_flag := true;
    }
    dst := Integer(i#Integer(src1) div i#Integer(src2));
}

procedure {:inline 1} Mod(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    if (i#Integer(src2) == 0) {
        abort_flag := true;
    }
    dst := Integer(i#Integer(src1) mod i#Integer(src2));
}

procedure {:inline 1} Lt(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    dst := Boolean(i#Integer(src1) < i#Integer(src2));
}

procedure {:inline 1} Gt(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    dst := Boolean(i#Integer(src1) > i#Integer(src2));
}

procedure {:inline 1} Le(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    dst := Boolean(i#Integer(src1) <= i#Integer(src2));
}

procedure {:inline 1} Ge(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    dst := Boolean(i#Integer(src1) >= i#Integer(src2));
}

procedure {:inline 1} And(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Boolean(src1) && is#Boolean(src2);
    dst := Boolean(b#Boolean(src1) && b#Boolean(src2));
}

procedure {:inline 1} Or(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Boolean(src1) && is#Boolean(src2);
    dst := Boolean(b#Boolean(src1) || b#Boolean(src2));
}

procedure {:inline 1} Not(src: Value) returns (dst: Value)
{
    assert is#Boolean(src);
    dst := Boolean(!b#Boolean(src));
}

procedure {:inline 1} Eq_int(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    dst := Boolean(i#Integer(src1) == i#Integer(src2));
}

procedure {:inline 1} Neq_int(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    dst := Boolean(i#Integer(src1) != i#Integer(src2));
}

procedure {:inline 1} Eq_bool(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Boolean(src1) && is#Boolean(src2);
    dst := Boolean(b#Boolean(src1) == b#Boolean(src2));
}

procedure {:inline 1} Neq_bool(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Boolean(src1) && is#Boolean(src2);
    dst := Boolean(b#Boolean(src1) != b#Boolean(src2));
}

procedure {:inline 1} Eq_address(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Address(src1) && is#Address(src2);
    dst := Boolean(a#Address(src1) == a#Address(src2));
}

procedure {:inline 1} Neq_address(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Address(src1) && is#Address(src2);
    dst := Boolean(a#Address(src1) != a#Address(src2));
}

procedure {:inline 1} Eq_bytearray(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#ByteArray(src1) && is#ByteArray(src2);
    dst := Boolean(b#ByteArray(src1) == b#ByteArray(src2));
}

procedure {:inline 1} Neq_bytearray(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#ByteArray(src1) && is#ByteArray(src2);
    dst := Boolean(b#ByteArray(src1) != b#ByteArray(src2));
}

procedure {:inline 1} LdConst(val: int) returns (ret: Value)
{
    ret := Integer(val);
}

procedure {:inline 1} LdAddr(val: Address) returns (ret: Value)
{
    ret := Address(val);
}

procedure {:inline 1} LdByteArray(val: ByteArray) returns (ret: Value)
{
    ret := ByteArray(val);
}

procedure {:inline 1} LdStr(val: String) returns (ret: Value)
{
    ret := Str(val);
}

procedure {:inline 1} LdTrue() returns (ret: Value)
{
    ret := Boolean(true);
}

procedure {:inline 1} LdFalse() returns (ret: Value)
{
    ret := Boolean(false);
}

// Transaction builtin instructions
type {:datatype} Transaction;
var txn: Transaction;
function {:constructor} Transaction_cons(
  gas_unit_price: int, max_gas_units: int, public_key: ByteArray,
  sender: Address, sequence_number: int, gas_remaining: int) : Transaction;

procedure {:inline 1} GetGasRemaining() returns (ret_gas_remaining: Value)
{
  ret_gas_remaining := Integer(gas_remaining#Transaction_cons(txn));
}

procedure {:inline 1} GetTxnSequenceNumber() returns (ret_sequence_number: Value)
{
  ret_sequence_number := Integer(sequence_number#Transaction_cons(txn));
}

procedure {:inline 1} GetTxnPublicKey() returns (ret_public_key: Value)
{
  ret_public_key := ByteArray(public_key#Transaction_cons(txn));
}

procedure {:inline 1} GetTxnSenderAddress() returns (ret_sender: Value)
{
  ret_sender := Address(sender#Transaction_cons(txn));
}

procedure {:inline 1} GetTxnMaxGasUnits() returns (ret_max_gas_units: Value)
{
  ret_max_gas_units := Integer(max_gas_units#Transaction_cons(txn));
}

procedure {:inline 1} GetTxnGasUnitPrice() returns (ret_gas_unit_price: Value)
{
  ret_gas_unit_price := Integer(gas_unit_price#Transaction_cons(txn));
}


// everything below is auto generated

const unique Test3_T: TypeName;
const unique Test3_T_f: FieldName;
const unique Test3_T_g: FieldName;

procedure {:inline 1} Pack_Test3_T(v0: Value, v1: Value) returns (v: Value)
{
    assert is#Integer(v0);
    assert is#Integer(v1);
    v := Map(DefaultMap[Field(Test3_T_f) := v0][Field(Test3_T_g) := v1]);
}

procedure {:inline 1} Unpack_Test3_T(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(Test3_T_f)];
    v1 := m#Map(v)[Field(Test3_T_g)];
}

procedure {:inline 1} Eq_Test3_T(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_int(m#Map(v1)[Field(Test3_T_f)], m#Map(v2)[Field(Test3_T_f)]);
    call b1 := Eq_int(m#Map(v1)[Field(Test3_T_g)], m#Map(v2)[Field(Test3_T_g)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1));
}

procedure {:inline 1} Neq_Test3_T(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_Test3_T(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

procedure {:inline 1} ReadValue0(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        assert false;
    }
}

procedure {:inline 1} ReadValueMax(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        e := p#Path(p)[i];
        v' := m#Map(v)[e];
        call v' := ReadValue0(p, i+1, v');
    }
}

procedure {:inline 1} UpdateValue0(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        assert false;
    }
}

procedure {:inline 1} UpdateValueMax(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        e := p#Path(p)[i];
        v' := m#Map(v)[e];
        call v' := UpdateValue0(p, i+1, v', new_v);
        v' := Map(m#Map(v)[e := v']);
    }
}

procedure Test3_test3 (arg0: Value) returns ()
{
    // declare local variables
    var t0: Value; // bool
    var t1: Value; // Test3_T
    var t2: Reference; // Test3_T_ref
    var t3: Reference; // int_ref
    var t4: Reference; // int_ref
    var t5: Reference; // int_ref
    var t6: Reference; // int_ref
    var t7: Value; // int
    var t8: Value; // int
    var t9: Value; // int
    var t10: Value; // int
    var t11: Value; // Test3_T
    var t12: Reference; // Test3_T_ref
    var t13: Value; // bool
    var t14: Reference; // Test3_T_ref
    var t15: Reference; // int_ref
    var t16: Reference; // Test3_T_ref
    var t17: Reference; // int_ref
    var t18: Value; // int
    var t19: Reference; // int_ref
    var t20: Value; // bool
    var t21: Value; // bool
    var t22: Reference; // Test3_T_ref
    var t23: Reference; // int_ref
    var t24: Reference; // Test3_T_ref
    var t25: Reference; // int_ref
    var t26: Value; // int
    var t27: Reference; // int_ref
    var t28: Reference; // Test3_T_ref
    var t29: Reference; // int_ref
    var t30: Reference; // Test3_T_ref
    var t31: Reference; // int_ref
    var t32: Reference; // int_ref
    var t33: Value; // int
    var t34: Reference; // int_ref
    var t35: Value; // int
    var t36: Value; // bool
    var t37: Value; // int
    var t38: Value; // int
    var t39: Value; // bool
    var t40: Value; // bool
    var t41: Value; // int
    var t42: Value; // int
    var t43: Value; // int
    var t44: Value; // bool
    var t45: Value; // bool
    var t46: Value; // int
    var t47: Value; // int
    var t48: Value; // int
    var t49: Value; // bool
    var t50: Value; // bool
    var t51: Value; // int
    var t52: Value; // int
    var t53: Value; // int
    var t54: Value; // bool
    var t55: Value; // bool
    var t56: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Boolean(arg0);

    old_size := ls_size;
    ls_size := ls_size + 57;
    ls[old_size+0] := arg0;

    // bytecode translation starts here
    call tmp := LdConst(0);
    ls[old_size+9] := tmp;

    call tmp := LdConst(0);
    ls[old_size+10] := tmp;

    assume is#Integer(ls[old_size+9]);

    assume is#Integer(ls[old_size+10]);

    call tmp := Pack_Test3_T(ls[old_size+9], ls[old_size+10]);
    ls[old_size+11] := tmp;

    call tmp := CopyOrMoveValue(ls[old_size+11]);
    ls[old_size+1] := tmp;

    call t12 := BorrowLoc(old_size+1);

    call t2 := CopyOrMoveRef(t12);

    call tmp := CopyOrMoveValue(ls[old_size+0]);
    ls[old_size+13] := tmp;

    tmp := ls[old_size + 13];
    if (!b#Boolean(tmp)) { goto Label_12; }

    call t14 := CopyOrMoveRef(t2);

    call t15 := BorrowField(t14, Test3_T_f);

    call t3 := CopyOrMoveRef(t15);

    goto Label_15;

Label_12:
    call t16 := CopyOrMoveRef(t2);

    call t17 := BorrowField(t16, Test3_T_g);

    call t3 := CopyOrMoveRef(t17);

Label_15:
    call tmp := LdConst(10);
    ls[old_size+18] := tmp;

    call t19 := CopyOrMoveRef(t3);

    call WriteRef(t19, ls[old_size+18]);

    call tmp := CopyOrMoveValue(ls[old_size+0]);
    ls[old_size+20] := tmp;

    call tmp := Not(ls[old_size+20]);
    ls[old_size+21] := tmp;

    tmp := ls[old_size + 21];
    if (!b#Boolean(tmp)) { goto Label_25; }

    call t22 := CopyOrMoveRef(t2);

    call t23 := BorrowField(t22, Test3_T_f);

    call t4 := CopyOrMoveRef(t23);

    goto Label_28;

Label_25:
    call t24 := CopyOrMoveRef(t2);

    call t25 := BorrowField(t24, Test3_T_g);

    call t4 := CopyOrMoveRef(t25);

Label_28:
    call tmp := LdConst(20);
    ls[old_size+26] := tmp;

    call t27 := CopyOrMoveRef(t4);

    call WriteRef(t27, ls[old_size+26]);

    call t28 := CopyOrMoveRef(t2);

    call t29 := BorrowField(t28, Test3_T_f);

    call t5 := CopyOrMoveRef(t29);

    call t30 := CopyOrMoveRef(t2);

    call t31 := BorrowField(t30, Test3_T_g);

    call t6 := CopyOrMoveRef(t31);

    call t32 := CopyOrMoveRef(t5);

    call tmp := ReadRef(t32);
    assume is#Integer(tmp);

    ls[old_size+33] := tmp;

    call tmp := CopyOrMoveValue(ls[old_size+33]);
    ls[old_size+7] := tmp;

    call t34 := CopyOrMoveRef(t6);

    call tmp := ReadRef(t34);
    assume is#Integer(tmp);

    ls[old_size+35] := tmp;

    call tmp := CopyOrMoveValue(ls[old_size+35]);
    ls[old_size+8] := tmp;

    call tmp := CopyOrMoveValue(ls[old_size+0]);
    ls[old_size+36] := tmp;

    tmp := ls[old_size + 36];
    if (!b#Boolean(tmp)) { goto Label_60; }

    call tmp := CopyOrMoveValue(ls[old_size+7]);
    ls[old_size+37] := tmp;

    call tmp := LdConst(10);
    ls[old_size+38] := tmp;

    call tmp := Eq_int(ls[old_size+37], ls[old_size+38]);
    ls[old_size+39] := tmp;

    call tmp := Not(ls[old_size+39]);
    ls[old_size+40] := tmp;

    tmp := ls[old_size + 40];
    if (!b#Boolean(tmp)) { goto Label_52; }

    call tmp := LdConst(42);
    ls[old_size+41] := tmp;

    assert false;

Label_52:
    call tmp := CopyOrMoveValue(ls[old_size+8]);
    ls[old_size+42] := tmp;

    call tmp := LdConst(20);
    ls[old_size+43] := tmp;

    call tmp := Eq_int(ls[old_size+42], ls[old_size+43]);
    ls[old_size+44] := tmp;

    call tmp := Not(ls[old_size+44]);
    ls[old_size+45] := tmp;

    tmp := ls[old_size + 45];
    if (!b#Boolean(tmp)) { goto Label_59; }

    call tmp := LdConst(42);
    ls[old_size+46] := tmp;

    assert false;

Label_59:
    goto Label_74;

Label_60:
    call tmp := CopyOrMoveValue(ls[old_size+7]);
    ls[old_size+47] := tmp;

    call tmp := LdConst(20);
    ls[old_size+48] := tmp;

    call tmp := Eq_int(ls[old_size+47], ls[old_size+48]);
    ls[old_size+49] := tmp;

    call tmp := Not(ls[old_size+49]);
    ls[old_size+50] := tmp;

    tmp := ls[old_size + 50];
    if (!b#Boolean(tmp)) { goto Label_67; }

    call tmp := LdConst(42);
    ls[old_size+51] := tmp;

    assert false;

Label_67:
    call tmp := CopyOrMoveValue(ls[old_size+8]);
    ls[old_size+52] := tmp;

    call tmp := LdConst(10);
    ls[old_size+53] := tmp;

    call tmp := Eq_int(ls[old_size+52], ls[old_size+53]);
    ls[old_size+54] := tmp;

    call tmp := Not(ls[old_size+54]);
    ls[old_size+55] := tmp;

    tmp := ls[old_size + 55];
    if (!b#Boolean(tmp)) { goto Label_74; }

    call tmp := LdConst(42);
    ls[old_size+56] := tmp;

    assert false;

Label_74:
    return;

}
