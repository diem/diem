type TypeName;
type FieldName;
type LocalName;
type Address = int;
type ByteArray;
type String;
type Location = int;
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
function {:constructor} Vector(v: [Edge]Value, l: int): Value;

const DefaultMap: [Edge]Value;

type {:datatype} Reference;
function {:constructor} Reference(rt: RefType, p: Path, l: Location): Reference;

type {:datatype} RefType;
function {:constructor} Global(a: Address, t: TypeName): RefType;
function {:constructor} Local(): RefType;

type {:datatype} TypeStore;
function {:constructor} TypeStore(domain: [TypeName]bool, contents: [TypeName]Location): TypeStore;

type {:datatype} GlobalStore;
function {:constructor} GlobalStore(domain: [Address]bool, contents: [Address]TypeStore): GlobalStore;

type {:datatype} Memory;
function {:constructor} Memory(domain: [Location]bool, contents: [Location]Value): Memory;

var gs : GlobalStore;
var m : Memory;
var m_size : int;

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
    ts := TypeStore(domain#TypeStore(ts)[t := true], contents#TypeStore(ts)[t := m_size]);
    gs := GlobalStore(domain#GlobalStore(gs), contents#GlobalStore(gs)[a := ts]);
    m := Memory(domain#Memory(m)[m_size := true], contents#Memory(m)[m_size := v]);
    m_size := m_size + 1;
}

procedure {:inline 1} MoveFrom(address: Value, t: TypeName) returns (dst: Value)
requires is#Address(address);
{
    var a: Address;
    var ts: TypeStore;
    var l: Location;
    a := a#Address(address);
    assert domain#GlobalStore(gs)[a];
    ts := contents#GlobalStore(gs)[a];
    assert domain#TypeStore(ts)[t];
    l := contents#TypeStore(ts)[t];
    assert domain#Memory(m)[l];
    dst := contents#Memory(m)[l];
    m := Memory(domain#Memory(m)[l := false], contents#Memory(m));
    ts := TypeStore(domain#TypeStore(ts)[t := false], contents#TypeStore(ts));
    gs := GlobalStore(domain#GlobalStore(gs), contents#GlobalStore(gs)[a := ts]);
}

procedure {:inline 1} BorrowGlobal(address: Value, t: TypeName) returns (dst: Reference)
requires is#Address(address);
{
    var a: Address;
    var v: Value;
    var ts: TypeStore;
    var l: Location;
    a := a#Address(address);
    assert domain#GlobalStore(gs)[a];
    ts := contents#GlobalStore(gs)[a];
    assert domain#TypeStore(ts)[t];
    l := contents#TypeStore(ts)[t];
    assert domain#Memory(m)[l];
    dst := Reference(Global(a, t), Path(DefaultPath, 0), l);
}

procedure {:inline 1} BorrowLoc(l: int) returns (dst: Reference)
{
    dst := Reference(Local(), Path(DefaultPath, 0), l);
}

procedure {:inline 1} BorrowField(src: Reference, f: FieldName) returns (dst: Reference)
{
    var p: Path;
    var size: int;
    p := p#Reference(src);
    size := size#Path(p);
	  p := Path(p#Path(p)[size := Field(f)], size+1);
    dst := Reference(rt#Reference(src), p, l#Reference(src));
}

procedure {:inline 1} WriteRef(to: Reference, new_v: Value)
{
    var a: Address;
    var ts: TypeStore;
    var t: TypeName;
    var v: Value;
    var l: Location;
    if (is#Global(rt#Reference(to))) {
        a := a#Global(rt#Reference(to));
        assert domain#GlobalStore(gs)[a];
        ts := contents#GlobalStore(gs)[a];
        t := t#Global(rt#Reference(to));
        assert domain#TypeStore(ts)[t];
        l := l#Reference(to);
        v := contents#Memory(m)[l];
        call v := UpdateValueMax(p#Reference(to), 0, v, new_v);
        m := Memory(domain#Memory(m), contents#Memory(m)[l := v]);
    } else {
        v := contents#Memory(m)[l#Reference(to)];
        call v := UpdateValueMax(p#Reference(to), 0, v, new_v);
        m := Memory(domain#Memory(m), contents#Memory(m)[l#Reference(to) := v]);
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
        call v := ReadValueMax(p#Reference(from), 0, contents#Memory(m)[contents#TypeStore(ts)[t]]);
    } else {
        call v := ReadValueMax(p#Reference(from), 0, contents#Memory(m)[l#Reference(from)]);
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

procedure {:inline 1} Eq(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Boolean(src1 == src2);
}

procedure {:inline 1} Neq(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Boolean(src1 != src2);
}

procedure {:inline 1} Eq_Vector_T(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Boolean(src1 == src2);
}

procedure {:inline 1} Neq_Vector_T(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Boolean(src1 != src2);
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

procedure {:inline 1} Vector_empty() returns (v: Value) {
    v := Vector(DefaultMap, 0);
}

procedure {:inline 1} Vector_is_empty(r: Reference) returns (b: Value) {
    var v: Value;
    call v := ReadRef(r);
    b := Boolean(l#Vector(v) == 0);
}

procedure {:inline 1} Vector_push_back(r: Reference, val: Value) {
    var old_v: Value;
    var old_len: int;
    call old_v := ReadRef(r);
    old_len := l#Vector(old_v);
    call WriteRef(r, Vector(v#Vector(old_v)[Index(old_len) := val], old_len+1));
}

procedure {:inline 1} Vector_pop_back(r: Reference) returns (e: Value){
    var v: Value;
    var old_len: int;
    call v := ReadRef(r);
    old_len := l#Vector(v);
    e := v#Vector(v)[Index(old_len-1)];
    call WriteRef(r, Vector(v#Vector(v), old_len-1));
}

procedure {:inline 1} Vector_append(r: Reference, other_v: Value) {
    var v: Value;
    var old_len: int;
    var other_len: int;
    var result: Value;
    call v := ReadRef(r);
    old_len := l#Vector(v);
    other_len := l#Vector(other_v);
    result := Vector(
        (lambda e:Edge ::
            if i#Index(e) < old_len then
		        v#Vector(v)[e]
		    else
		        v#Vector(other_v)[Index(i#Index(e) - old_len)]
        ),
	    old_len + other_len);
    call WriteRef(r, result);
}

procedure {:inline 1} Vector_reverse(r: Reference) {
    var v: Value;
    var result: Value;
    var len: int;

    call v := ReadRef(r);
    len := l#Vector(v);
    result := Vector(
        (lambda e:Edge ::
            v#Vector(v)[Index(len-i#Index(e)-1)]
        ),
	    len);
    call WriteRef(r, result);
}

procedure {:inline 1} Vector_length(r: Reference) returns (l: Value) {
    var v: Value;
    call v := ReadRef(r);
    l := Integer(l#Vector(v));
}

procedure {:inline 1} Vector_borrow(src: Reference, index: Value) returns (dst: Reference) {
    var p: Path;
    var size: int;
    p := p#Reference(src);
    size := size#Path(p);
	  p := Path(p#Path(p)[size := Index(i#Integer(index))], size+1);
    dst := Reference(rt#Reference(src), p, l#Reference(src));
}

procedure {:inline 1} Vector_borrow_mut(src: Reference, index: Value) returns (dst: Reference) {
    var p: Path;
    var size: int;
    p := p#Reference(src);
    size := size#Path(p);
	  p := Path(p#Path(p)[size := Index(i#Integer(index))], size+1);
    dst := Reference(rt#Reference(src), p, l#Reference(src));
}

procedure {:inline 1} Vector_destroy_empty(v: Value) {
    assert (l#Vector(v) == 0);
}

procedure {:inline 1} Vector_swap(src: Reference, i: Value, j: Value) {
    var i_val: Value;
    var j_val: Value;
    var i_ind: int;
    var j_ind: int;
    var v: Value;
    i_ind := i#Integer(i);
    j_ind := i#Integer(j);
    call v := ReadRef(src);
    assert (l#Vector(v) > i_ind && l#Vector(v) > j_ind);
    i_val := v#Vector(v)[Index(i_ind)];
    j_val := v#Vector(v)[Index(j_ind)];
    v := Vector(v#Vector(v)[Index(i_ind) := j_val][Index(j_ind) := i_val], l#Vector(v));
    call WriteRef(src, v);
}

procedure {:inline 1} Vector_get(src: Reference, i: Value) returns (e: Value) {
    var i_ind: int;
    var v: Value;
    call v := ReadRef(src);
    i_ind := i#Integer(i);
    assert (i_ind < l#Vector(v));
    e := v#Vector(v)[Index(i_ind)];
}

procedure {:inline 1} Vector_set(src: Reference, i: Value, e: Value) {
    var i_ind: int;
    var v: Value;
    i_ind := i#Integer(i);
    call v := ReadRef(src);
    assert (l#Vector(v) > i_ind);
    v := Vector(v#Vector(v)[Index(i_ind) := e], l#Vector(v));
    call WriteRef(src, v);
}


// everything below is auto generated

const unique Vector_T: TypeName;
const unique Option_T: TypeName;
const unique Option_T_v: FieldName;

procedure {:inline 1} Pack_Option_T(v0: Value) returns (v: Value)
{
    assert is#Vector(v0);
    v := Map(DefaultMap[Field(Option_T_v) := v0]);
}

procedure {:inline 1} Unpack_Option_T(v: Value) returns (v0: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(Option_T_v)];
}

procedure {:inline 1} Eq_Option_T(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_Vector_T(m#Map(v1)[Field(Option_T_v)], m#Map(v2)[Field(Option_T_v)]);
    res := Boolean(true && b#Boolean(b0));
}

procedure {:inline 1} Neq_Option_T(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_Option_T(v1, v2);
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

procedure {:inline 1} ReadValue1(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        e := p#Path(p)[i];
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := ReadValue0(p, i+1, v');
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
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := ReadValue1(p, i+1, v');
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

procedure {:inline 1} UpdateValue1(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        e := p#Path(p)[i];
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := UpdateValue0(p, i+1, v', new_v);
        if (is#Map(v)) { v' := Map(m#Map(v)[e := v']);}
        if (is#Vector(v)) { v' := Vector(v#Vector(v)[e := v'], l#Vector(v));}
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
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := UpdateValue1(p, i+1, v', new_v);
        if (is#Map(v)) { v' := Map(m#Map(v)[e := v']);}
        if (is#Vector(v)) { v' := Vector(v#Vector(v)[e := v'], l#Vector(v));}
    }
}

procedure {:inline 1} Option_none () returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // Vector_T
    var t1: Value; // Option_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 2;

    // bytecode translation starts here
    call t0 := Vector_empty();
    assume is#Vector(t0);

    m := Memory(domain#Memory(m)[old_size+0 := true], contents#Memory(m)[old_size+0 := t0]);

    assume is#Vector(contents#Memory(m)[old_size+0]);

    call tmp := Pack_Option_T(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+1];
    return;

}

procedure Option_none_verify () returns (ret0: Value)
{
    call ret0 := Option_none();
}

procedure {:inline 1} Option_some (arg0: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // typeparam
    var t1: Value; // Vector_T
    var t2: Value; // Vector_T
    var t3: Reference; // Vector_T_ref
    var t4: Value; // typeparam
    var t5: Value; // Vector_T
    var t6: Value; // Option_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 7;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call t2 := Vector_empty();
    assume is#Vector(t2);

    m := Memory(domain#Memory(m)[old_size+2 := true], contents#Memory(m)[old_size+2 := t2]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t3 := BorrowLoc(old_size+1);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call Vector_push_back(t3, contents#Memory(m)[old_size+4]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    assume is#Vector(contents#Memory(m)[old_size+5]);

    call tmp := Pack_Option_T(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+6];
    return;

}

procedure Option_some_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := Option_some(arg0);
}

procedure {:inline 1} Option_unwrap_or (arg0: Value, arg1: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // Option_T
    var t1: Value; // typeparam
    var t2: Value; // Vector_T
    var t3: Value; // Option_T
    var t4: Value; // Vector_T
    var t5: Reference; // Vector_T_ref
    var t6: Value; // bool
    var t7: Value; // typeparam
    var t8: Reference; // Vector_T_ref
    var t9: Value; // typeparam

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Map(arg0);

    old_size := m_size;
    m_size := m_size + 10;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t4 := Unpack_Option_T(contents#Memory(m)[old_size+3]);
    assume is#Vector(t4);

    m := Memory(domain#Memory(m)[old_size+4 := true], contents#Memory(m)[old_size+4 := t4]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t5 := BorrowLoc(old_size+2);

    call t6 := Vector_is_empty(t5);
    assume is#Boolean(t6);

    m := Memory(domain#Memory(m)[old_size+6 := true], contents#Memory(m)[old_size+6 := t6]);

    tmp := contents#Memory(m)[old_size + 6];
if (!b#Boolean(tmp)) { goto Label_8; }

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+7];
    return;

Label_8:
    call t8 := BorrowLoc(old_size+2);

    call t9 := Vector_pop_back(t8);

    m := Memory(domain#Memory(m)[old_size+9 := true], contents#Memory(m)[old_size+9 := t9]);

    ret0 := contents#Memory(m)[old_size+9];
    return;

}

procedure Option_unwrap_or_verify (arg0: Value, arg1: Value) returns (ret0: Value)
{
    call ret0 := Option_unwrap_or(arg0, arg1);
}

procedure {:inline 1} Option_test_u64 () returns ()
{
    // declare local variables
    var t0: Value; // Option_T
    var t1: Value; // Option_T
    var t2: Value; // Option_T
    var t3: Value; // int
    var t4: Value; // Option_T
    var t5: Value; // Option_T
    var t6: Value; // int
    var t7: Value; // int
    var t8: Value; // int
    var t9: Value; // bool
    var t10: Value; // bool
    var t11: Value; // int
    var t12: Value; // Option_T
    var t13: Value; // int
    var t14: Value; // int
    var t15: Value; // int
    var t16: Value; // bool
    var t17: Value; // bool
    var t18: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 19;

    // bytecode translation starts here
    call t2 := Option_none();
    assume is#Map(t2);

    m := Memory(domain#Memory(m)[old_size+2 := true], contents#Memory(m)[old_size+2 := t2]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t4 := Option_some(contents#Memory(m)[old_size+3]);
    assume is#Map(t4);

    m := Memory(domain#Memory(m)[old_size+4 := true], contents#Memory(m)[old_size+4 := t4]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call t7 := Option_unwrap_or(contents#Memory(m)[old_size+5], contents#Memory(m)[old_size+6]);
    assume is#Integer(t7);

    m := Memory(domain#Memory(m)[old_size+7 := true], contents#Memory(m)[old_size+7 := t7]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+7], contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 10];
if (!b#Boolean(tmp)) { goto Label_14; }

    call tmp := LdConst(10);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    assert false;

Label_14:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    call t14 := Option_unwrap_or(contents#Memory(m)[old_size+12], contents#Memory(m)[old_size+13]);
    assume is#Integer(t14);

    m := Memory(domain#Memory(m)[old_size+14 := true], contents#Memory(m)[old_size+14 := t14]);

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+14], contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+16]);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 17];
if (!b#Boolean(tmp)) { goto Label_23; }

    call tmp := LdConst(10);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    assert false;

Label_23:
    return;

}

procedure Option_test_u64_verify () returns ()
{
    call Option_test_u64();
}

procedure {:inline 1} Option_test_append_reverse () returns ()
{
    // declare local variables
    var t0: Value; // Vector_T
    var t1: Value; // Vector_T
    var t2: Value; // Vector_T
    var t3: Value; // Vector_T
    var t4: Reference; // Vector_T_ref
    var t5: Value; // Vector_T
    var t6: Reference; // Vector_T_ref
    var t7: Value; // bool
    var t8: Value; // bool
    var t9: Value; // int
    var t10: Value; // Vector_T
    var t11: Reference; // Vector_T_ref
    var t12: Value; // int
    var t13: Reference; // Vector_T_ref
    var t14: Value; // int
    var t15: Reference; // Vector_T_ref
    var t16: Reference; // Vector_T_ref
    var t17: Value; // int
    var t18: Reference; // Vector_T_ref
    var t19: Value; // int
    var t20: Reference; // Vector_T_ref
    var t21: Value; // Vector_T
    var t22: Reference; // Vector_T_ref
    var t23: Value; // bool
    var t24: Value; // bool
    var t25: Value; // bool
    var t26: Value; // int
    var t27: Reference; // Vector_T_ref
    var t28: Value; // int
    var t29: Value; // int
    var t30: Value; // bool
    var t31: Value; // bool
    var t32: Value; // int
    var t33: Reference; // Vector_T_ref
    var t34: Value; // int
    var t35: Reference; // int_ref
    var t36: Value; // int
    var t37: Value; // int
    var t38: Value; // bool
    var t39: Value; // bool
    var t40: Value; // int
    var t41: Reference; // Vector_T_ref
    var t42: Value; // int
    var t43: Reference; // int_ref
    var t44: Value; // int
    var t45: Value; // int
    var t46: Value; // bool
    var t47: Value; // bool
    var t48: Value; // int
    var t49: Reference; // Vector_T_ref
    var t50: Value; // int
    var t51: Reference; // int_ref
    var t52: Value; // int
    var t53: Value; // int
    var t54: Value; // bool
    var t55: Value; // bool
    var t56: Value; // int
    var t57: Reference; // Vector_T_ref
    var t58: Value; // int
    var t59: Reference; // int_ref
    var t60: Value; // int
    var t61: Value; // int
    var t62: Value; // bool
    var t63: Value; // bool
    var t64: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 65;

    // bytecode translation starts here
    call t2 := Vector_empty();
    assume is#Vector(t2);

    m := Memory(domain#Memory(m)[old_size+2 := true], contents#Memory(m)[old_size+2 := t2]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call t3 := Vector_empty();
    assume is#Vector(t3);

    m := Memory(domain#Memory(m)[old_size+3 := true], contents#Memory(m)[old_size+3 := t3]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t4 := BorrowLoc(old_size+0);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call Vector_append(t4, contents#Memory(m)[old_size+5]);

    call t6 := BorrowLoc(old_size+0);

    call t7 := Vector_is_empty(t6);
    assume is#Boolean(t7);

    m := Memory(domain#Memory(m)[old_size+7 := true], contents#Memory(m)[old_size+7 := t7]);

    call tmp := Not(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 8];
if (!b#Boolean(tmp)) { goto Label_13; }

    call tmp := LdConst(3);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    assert false;

Label_13:
    call t10 := Vector_empty();
    assume is#Vector(t10);

    m := Memory(domain#Memory(m)[old_size+10 := true], contents#Memory(m)[old_size+10 := t10]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+10]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t11 := BorrowLoc(old_size+1);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call Vector_push_back(t11, contents#Memory(m)[old_size+12]);

    call t13 := BorrowLoc(old_size+1);

    call tmp := LdConst(2);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call Vector_push_back(t13, contents#Memory(m)[old_size+14]);

    call t15 := BorrowLoc(old_size+1);

    call Vector_reverse(t15);

    call t16 := BorrowLoc(old_size+1);

    call tmp := LdConst(3);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    call Vector_push_back(t16, contents#Memory(m)[old_size+17]);

    call t18 := BorrowLoc(old_size+0);

    call tmp := LdConst(4);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    call Vector_push_back(t18, contents#Memory(m)[old_size+19]);

    call t20 := BorrowLoc(old_size+0);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    call Vector_append(t20, contents#Memory(m)[old_size+21]);

    call t22 := BorrowLoc(old_size+0);

    call t23 := Vector_is_empty(t22);
    assume is#Boolean(t23);

    m := Memory(domain#Memory(m)[old_size+23 := true], contents#Memory(m)[old_size+23 := t23]);

    call tmp := Not(contents#Memory(m)[old_size+23]);
    m := Memory(domain#Memory(m)[24+old_size := true], contents#Memory(m)[24+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+24]);
    m := Memory(domain#Memory(m)[25+old_size := true], contents#Memory(m)[25+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 25];
if (!b#Boolean(tmp)) { goto Label_39; }

    call tmp := LdConst(4);
    m := Memory(domain#Memory(m)[26+old_size := true], contents#Memory(m)[26+old_size := tmp]);

    assert false;

Label_39:
    call t27 := BorrowLoc(old_size+0);

    call t28 := Vector_length(t27);
    assume is#Integer(t28);

    m := Memory(domain#Memory(m)[old_size+28 := true], contents#Memory(m)[old_size+28 := t28]);

    call tmp := LdConst(4);
    m := Memory(domain#Memory(m)[29+old_size := true], contents#Memory(m)[29+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+28], contents#Memory(m)[old_size+29]);
    m := Memory(domain#Memory(m)[30+old_size := true], contents#Memory(m)[30+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+30]);
    m := Memory(domain#Memory(m)[31+old_size := true], contents#Memory(m)[31+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 31];
if (!b#Boolean(tmp)) { goto Label_47; }

    call tmp := LdConst(5);
    m := Memory(domain#Memory(m)[32+old_size := true], contents#Memory(m)[32+old_size := tmp]);

    assert false;

Label_47:
    call t33 := BorrowLoc(old_size+0);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[34+old_size := true], contents#Memory(m)[34+old_size := tmp]);

    call t35 := Vector_borrow(t33, contents#Memory(m)[old_size+34]);

    call tmp := ReadRef(t35);
    assume is#Integer(tmp);

    m := Memory(domain#Memory(m)[36+old_size := true], contents#Memory(m)[36+old_size := tmp]);

    call tmp := LdConst(4);
    m := Memory(domain#Memory(m)[37+old_size := true], contents#Memory(m)[37+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+36], contents#Memory(m)[old_size+37]);
    m := Memory(domain#Memory(m)[38+old_size := true], contents#Memory(m)[38+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+38]);
    m := Memory(domain#Memory(m)[39+old_size := true], contents#Memory(m)[39+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 39];
if (!b#Boolean(tmp)) { goto Label_57; }

    call tmp := LdConst(7);
    m := Memory(domain#Memory(m)[40+old_size := true], contents#Memory(m)[40+old_size := tmp]);

    assert false;

Label_57:
    call t41 := BorrowLoc(old_size+0);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[42+old_size := true], contents#Memory(m)[42+old_size := tmp]);

    call t43 := Vector_borrow(t41, contents#Memory(m)[old_size+42]);

    call tmp := ReadRef(t43);
    assume is#Integer(tmp);

    m := Memory(domain#Memory(m)[44+old_size := true], contents#Memory(m)[44+old_size := tmp]);

    call tmp := LdConst(2);
    m := Memory(domain#Memory(m)[45+old_size := true], contents#Memory(m)[45+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+44], contents#Memory(m)[old_size+45]);
    m := Memory(domain#Memory(m)[46+old_size := true], contents#Memory(m)[46+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+46]);
    m := Memory(domain#Memory(m)[47+old_size := true], contents#Memory(m)[47+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 47];
if (!b#Boolean(tmp)) { goto Label_67; }

    call tmp := LdConst(7);
    m := Memory(domain#Memory(m)[48+old_size := true], contents#Memory(m)[48+old_size := tmp]);

    assert false;

Label_67:
    call t49 := BorrowLoc(old_size+0);

    call tmp := LdConst(2);
    m := Memory(domain#Memory(m)[50+old_size := true], contents#Memory(m)[50+old_size := tmp]);

    call t51 := Vector_borrow(t49, contents#Memory(m)[old_size+50]);

    call tmp := ReadRef(t51);
    assume is#Integer(tmp);

    m := Memory(domain#Memory(m)[52+old_size := true], contents#Memory(m)[52+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[53+old_size := true], contents#Memory(m)[53+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+52], contents#Memory(m)[old_size+53]);
    m := Memory(domain#Memory(m)[54+old_size := true], contents#Memory(m)[54+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+54]);
    m := Memory(domain#Memory(m)[55+old_size := true], contents#Memory(m)[55+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 55];
if (!b#Boolean(tmp)) { goto Label_77; }

    call tmp := LdConst(8);
    m := Memory(domain#Memory(m)[56+old_size := true], contents#Memory(m)[56+old_size := tmp]);

    assert false;

Label_77:
    call t57 := BorrowLoc(old_size+0);

    call tmp := LdConst(3);
    m := Memory(domain#Memory(m)[58+old_size := true], contents#Memory(m)[58+old_size := tmp]);

    call t59 := Vector_borrow(t57, contents#Memory(m)[old_size+58]);

    call tmp := ReadRef(t59);
    assume is#Integer(tmp);

    m := Memory(domain#Memory(m)[60+old_size := true], contents#Memory(m)[60+old_size := tmp]);

    call tmp := LdConst(3);
    m := Memory(domain#Memory(m)[61+old_size := true], contents#Memory(m)[61+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+60], contents#Memory(m)[old_size+61]);
    m := Memory(domain#Memory(m)[62+old_size := true], contents#Memory(m)[62+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+62]);
    m := Memory(domain#Memory(m)[63+old_size := true], contents#Memory(m)[63+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 63];
if (!b#Boolean(tmp)) { goto Label_87; }

    call tmp := LdConst(9);
    m := Memory(domain#Memory(m)[64+old_size := true], contents#Memory(m)[64+old_size := tmp]);

    assert false;

Label_87:
    return;

}

procedure Option_test_append_reverse_verify () returns ()
{
    call Option_test_append_reverse();
}

procedure {:inline 1} Option_test_get_set_swap () returns ()
{
    // declare local variables
    var t0: Value; // Vector_T
    var t1: Value; // Vector_T
    var t2: Value; // Vector_T
    var t3: Value; // Vector_T
    var t4: Value; // Vector_T
    var t5: Reference; // Vector_T_ref
    var t6: Value; // int
    var t7: Reference; // Vector_T_ref
    var t8: Value; // int
    var t9: Reference; // Vector_T_ref
    var t10: Value; // int
    var t11: Value; // int
    var t12: Reference; // Vector_T_ref
    var t13: Value; // int
    var t14: Value; // int
    var t15: Value; // int
    var t16: Value; // bool
    var t17: Value; // bool
    var t18: Value; // int
    var t19: Reference; // Vector_T_ref
    var t20: Value; // int
    var t21: Reference; // Vector_T_ref
    var t22: Value; // int
    var t23: Value; // int
    var t24: Reference; // Vector_T_ref
    var t25: Value; // int
    var t26: Value; // int
    var t27: Value; // int
    var t28: Value; // bool
    var t29: Value; // bool
    var t30: Value; // int
    var t31: Reference; // Vector_T_ref
    var t32: Value; // int
    var t33: Value; // int
    var t34: Value; // int
    var t35: Value; // bool
    var t36: Value; // bool
    var t37: Value; // int
    var t38: Reference; // Vector_T_ref
    var t39: Value; // int
    var t40: Value; // int
    var t41: Reference; // Vector_T_ref
    var t42: Value; // int
    var t43: Value; // int
    var t44: Value; // int
    var t45: Value; // bool
    var t46: Value; // bool
    var t47: Value; // int
    var t48: Reference; // Vector_T_ref
    var t49: Value; // int
    var t50: Value; // int
    var t51: Value; // int
    var t52: Value; // bool
    var t53: Value; // bool
    var t54: Value; // int
    var t55: Reference; // Vector_T_ref
    var t56: Value; // int
    var t57: Value; // int
    var t58: Value; // bool
    var t59: Value; // bool
    var t60: Value; // int
    var t61: Reference; // Vector_T_ref
    var t62: Value; // int
    var t63: Value; // int
    var t64: Value; // bool
    var t65: Value; // bool
    var t66: Value; // int
    var t67: Reference; // Vector_T_ref
    var t68: Value; // int
    var t69: Value; // int
    var t70: Value; // bool
    var t71: Value; // bool
    var t72: Value; // int
    var t73: Reference; // Vector_T_ref
    var t74: Value; // bool
    var t75: Value; // bool
    var t76: Value; // int
    var t77: Value; // Vector_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 78;

    // bytecode translation starts here
    call t2 := Vector_empty();
    assume is#Vector(t2);

    m := Memory(domain#Memory(m)[old_size+2 := true], contents#Memory(m)[old_size+2 := t2]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call Vector_destroy_empty(contents#Memory(m)[old_size+3]);

    call t4 := Vector_empty();
    assume is#Vector(t4);

    m := Memory(domain#Memory(m)[old_size+4 := true], contents#Memory(m)[old_size+4 := t4]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t5 := BorrowLoc(old_size+1);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call Vector_push_back(t5, contents#Memory(m)[old_size+6]);

    call t7 := BorrowLoc(old_size+1);

    call tmp := LdConst(2);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call Vector_push_back(t7, contents#Memory(m)[old_size+8]);

    call t9 := BorrowLoc(old_size+1);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := LdConst(22);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call Vector_set(t9, contents#Memory(m)[old_size+10], contents#Memory(m)[old_size+11]);

    call t12 := BorrowLoc(old_size+1);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    call t14 := Vector_get(t12, contents#Memory(m)[old_size+13]);
    assume is#Integer(t14);

    m := Memory(domain#Memory(m)[old_size+14 := true], contents#Memory(m)[old_size+14 := t14]);

    call tmp := LdConst(22);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+14], contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+16]);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 17];
if (!b#Boolean(tmp)) { goto Label_25; }

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    assert false;

Label_25:
    call t19 := BorrowLoc(old_size+1);

    call tmp := LdConst(3);
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call Vector_push_back(t19, contents#Memory(m)[old_size+20]);

    call t21 := BorrowLoc(old_size+1);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[22+old_size := true], contents#Memory(m)[22+old_size := tmp]);

    call tmp := LdConst(2);
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    call Vector_swap(t21, contents#Memory(m)[old_size+22], contents#Memory(m)[old_size+23]);

    call t24 := BorrowLoc(old_size+1);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[25+old_size := true], contents#Memory(m)[25+old_size := tmp]);

    call t26 := Vector_get(t24, contents#Memory(m)[old_size+25]);
    assume is#Integer(t26);

    m := Memory(domain#Memory(m)[old_size+26 := true], contents#Memory(m)[old_size+26 := t26]);

    call tmp := LdConst(3);
    m := Memory(domain#Memory(m)[27+old_size := true], contents#Memory(m)[27+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+26], contents#Memory(m)[old_size+27]);
    m := Memory(domain#Memory(m)[28+old_size := true], contents#Memory(m)[28+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+28]);
    m := Memory(domain#Memory(m)[29+old_size := true], contents#Memory(m)[29+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 29];
if (!b#Boolean(tmp)) { goto Label_41; }

    call tmp := LdConst(43);
    m := Memory(domain#Memory(m)[30+old_size := true], contents#Memory(m)[30+old_size := tmp]);

    assert false;

Label_41:
    call t31 := BorrowLoc(old_size+1);

    call tmp := LdConst(2);
    m := Memory(domain#Memory(m)[32+old_size := true], contents#Memory(m)[32+old_size := tmp]);

    call t33 := Vector_get(t31, contents#Memory(m)[old_size+32]);
    assume is#Integer(t33);

    m := Memory(domain#Memory(m)[old_size+33 := true], contents#Memory(m)[old_size+33 := t33]);

    call tmp := LdConst(22);
    m := Memory(domain#Memory(m)[34+old_size := true], contents#Memory(m)[34+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+33], contents#Memory(m)[old_size+34]);
    m := Memory(domain#Memory(m)[35+old_size := true], contents#Memory(m)[35+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+35]);
    m := Memory(domain#Memory(m)[36+old_size := true], contents#Memory(m)[36+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 36];
if (!b#Boolean(tmp)) { goto Label_50; }

    call tmp := LdConst(44);
    m := Memory(domain#Memory(m)[37+old_size := true], contents#Memory(m)[37+old_size := tmp]);

    assert false;

Label_50:
    call t38 := BorrowLoc(old_size+1);

    call tmp := LdConst(2);
    m := Memory(domain#Memory(m)[39+old_size := true], contents#Memory(m)[39+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[40+old_size := true], contents#Memory(m)[40+old_size := tmp]);

    call Vector_swap(t38, contents#Memory(m)[old_size+39], contents#Memory(m)[old_size+40]);

    call t41 := BorrowLoc(old_size+1);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[42+old_size := true], contents#Memory(m)[42+old_size := tmp]);

    call t43 := Vector_get(t41, contents#Memory(m)[old_size+42]);
    assume is#Integer(t43);

    m := Memory(domain#Memory(m)[old_size+43 := true], contents#Memory(m)[old_size+43 := t43]);

    call tmp := LdConst(22);
    m := Memory(domain#Memory(m)[44+old_size := true], contents#Memory(m)[44+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+43], contents#Memory(m)[old_size+44]);
    m := Memory(domain#Memory(m)[45+old_size := true], contents#Memory(m)[45+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+45]);
    m := Memory(domain#Memory(m)[46+old_size := true], contents#Memory(m)[46+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 46];
if (!b#Boolean(tmp)) { goto Label_63; }

    call tmp := LdConst(45);
    m := Memory(domain#Memory(m)[47+old_size := true], contents#Memory(m)[47+old_size := tmp]);

    assert false;

Label_63:
    call t48 := BorrowLoc(old_size+1);

    call tmp := LdConst(2);
    m := Memory(domain#Memory(m)[49+old_size := true], contents#Memory(m)[49+old_size := tmp]);

    call t50 := Vector_get(t48, contents#Memory(m)[old_size+49]);
    assume is#Integer(t50);

    m := Memory(domain#Memory(m)[old_size+50 := true], contents#Memory(m)[old_size+50 := t50]);

    call tmp := LdConst(3);
    m := Memory(domain#Memory(m)[51+old_size := true], contents#Memory(m)[51+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+50], contents#Memory(m)[old_size+51]);
    m := Memory(domain#Memory(m)[52+old_size := true], contents#Memory(m)[52+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+52]);
    m := Memory(domain#Memory(m)[53+old_size := true], contents#Memory(m)[53+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 53];
if (!b#Boolean(tmp)) { goto Label_72; }

    call tmp := LdConst(46);
    m := Memory(domain#Memory(m)[54+old_size := true], contents#Memory(m)[54+old_size := tmp]);

    assert false;

Label_72:
    call t55 := BorrowLoc(old_size+1);

    call t56 := Vector_pop_back(t55);
    assume is#Integer(t56);

    m := Memory(domain#Memory(m)[old_size+56 := true], contents#Memory(m)[old_size+56 := t56]);

    call tmp := LdConst(3);
    m := Memory(domain#Memory(m)[57+old_size := true], contents#Memory(m)[57+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+56], contents#Memory(m)[old_size+57]);
    m := Memory(domain#Memory(m)[58+old_size := true], contents#Memory(m)[58+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+58]);
    m := Memory(domain#Memory(m)[59+old_size := true], contents#Memory(m)[59+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 59];
if (!b#Boolean(tmp)) { goto Label_80; }

    call tmp := LdConst(47);
    m := Memory(domain#Memory(m)[60+old_size := true], contents#Memory(m)[60+old_size := tmp]);

    assert false;

Label_80:
    call t61 := BorrowLoc(old_size+1);

    call t62 := Vector_pop_back(t61);
    assume is#Integer(t62);

    m := Memory(domain#Memory(m)[old_size+62 := true], contents#Memory(m)[old_size+62 := t62]);

    call tmp := LdConst(22);
    m := Memory(domain#Memory(m)[63+old_size := true], contents#Memory(m)[63+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+62], contents#Memory(m)[old_size+63]);
    m := Memory(domain#Memory(m)[64+old_size := true], contents#Memory(m)[64+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+64]);
    m := Memory(domain#Memory(m)[65+old_size := true], contents#Memory(m)[65+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 65];
if (!b#Boolean(tmp)) { goto Label_88; }

    call tmp := LdConst(48);
    m := Memory(domain#Memory(m)[66+old_size := true], contents#Memory(m)[66+old_size := tmp]);

    assert false;

Label_88:
    call t67 := BorrowLoc(old_size+1);

    call t68 := Vector_pop_back(t67);
    assume is#Integer(t68);

    m := Memory(domain#Memory(m)[old_size+68 := true], contents#Memory(m)[old_size+68 := t68]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[69+old_size := true], contents#Memory(m)[69+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+68], contents#Memory(m)[old_size+69]);
    m := Memory(domain#Memory(m)[70+old_size := true], contents#Memory(m)[70+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+70]);
    m := Memory(domain#Memory(m)[71+old_size := true], contents#Memory(m)[71+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 71];
if (!b#Boolean(tmp)) { goto Label_96; }

    call tmp := LdConst(49);
    m := Memory(domain#Memory(m)[72+old_size := true], contents#Memory(m)[72+old_size := tmp]);

    assert false;

Label_96:
    call t73 := BorrowLoc(old_size+1);

    call t74 := Vector_is_empty(t73);
    assume is#Boolean(t74);

    m := Memory(domain#Memory(m)[old_size+74 := true], contents#Memory(m)[old_size+74 := t74]);

    call tmp := Not(contents#Memory(m)[old_size+74]);
    m := Memory(domain#Memory(m)[75+old_size := true], contents#Memory(m)[75+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 75];
if (!b#Boolean(tmp)) { goto Label_102; }

    call tmp := LdConst(50);
    m := Memory(domain#Memory(m)[76+old_size := true], contents#Memory(m)[76+old_size := tmp]);

    assert false;

Label_102:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[77+old_size := true], contents#Memory(m)[77+old_size := tmp]);

    call Vector_destroy_empty(contents#Memory(m)[old_size+77]);

    return;

}

procedure Option_test_get_set_swap_verify () returns ()
{
    call Option_test_get_set_swap();
}
