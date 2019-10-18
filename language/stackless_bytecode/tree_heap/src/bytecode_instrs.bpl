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
