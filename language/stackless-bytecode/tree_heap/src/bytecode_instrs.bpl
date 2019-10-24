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
