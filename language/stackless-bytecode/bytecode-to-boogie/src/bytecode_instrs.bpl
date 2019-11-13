type TypeName;
type FieldName;
type LocalName;
type Address = int;
type ByteArray;
type String;
type CreationTime = int;

type {:datatype} Edge;
function {:constructor} Field(f: FieldName): Edge;
function {:constructor} Index(i: int): Edge;
function {:constructor} String(s: String): Edge;

type {:datatype} Path;
function {:constructor} Nil(): Path;
function {:constructor} Cons(p: Path, e: Edge): Path;

type {:datatype} Value;
function {:constructor} Boolean(b: bool): Value;
function {:constructor} Integer(i: int): Value;
function {:constructor} Address(a: Address): Value;
function {:constructor} ByteArray(b: ByteArray): Value;
function {:constructor} Str(a: String): Value;
function {:constructor} Map(m: [Edge]Value): Value;

const DefaultMap: [Edge]Value;

type {:datatype} Reference;
function {:constructor} Reference(rt: RefType, p: Path, v: Value): Reference;

type {:datatype} RefType;
function {:constructor} Global(a: Address, t: TypeName): RefType;
function {:constructor} Local(c: CreationTime, l: LocalName): RefType;

type {:datatype} ResourceStore;
function {:constructor} ResourceStore(domain: [Address]bool, contents: [Address]Value): ResourceStore;

var senderAddress: Value;

procedure {:inline 1} DeepUpdateReference(src: Reference, dst: Reference) returns (dst': Reference)
{
    var isPrefix: bool;
    var v': Value;
    dst' := dst;
    if (rt#Reference(src) == rt#Reference(dst)) {
        call isPrefix := IsPrefixMax(p#Reference(dst), p#Reference(src));
        if (isPrefix) {
            call v' := UpdateValueMax(p#Reference(src), v#Reference(src), p#Reference(dst), v#Reference(dst));
            dst' := Reference(rt#Reference(dst), p#Reference(dst), v');
        }
    }
}

procedure {:inline 1} DeepUpdateLocal(c: CreationTime, l: LocalName, src: Reference, dst: Value) returns (dst': Value)
{
    var v': Value;
    dst' := dst;
    if (is#Local(rt#Reference(src)) && c == c#Local(rt#Reference(src)) && l == l#Local(rt#Reference(src))) {
        call dst' := UpdateValueMax(p#Reference(src), v#Reference(src), Nil(), dst);
    }
}

procedure {:inline 1} DeepUpdateGlobal(t: TypeName, src: Reference, dst: ResourceStore) returns (dst': ResourceStore)
{
    var v': Value;
    var a: Address;
    dst' := dst;
    if (is#Global(rt#Reference(src)) && t == t#Global(rt#Reference(src))) {
        a := a#Global(rt#Reference(src));
        call v' := UpdateValueMax(p#Reference(src), v#Reference(src), Nil(), contents#ResourceStore(dst)[a]);
        dst' := ResourceStore(domain#ResourceStore(dst), contents#ResourceStore(dst)[a := v']);
    }
}

procedure {:inline 1} Exists(address: Value, rs: ResourceStore) returns (dst: Value)
{
    assert is#Address(address);
    dst := Boolean(domain#ResourceStore(rs)[a#Address(address)]);
}

procedure {:inline 1} MoveToSender(rs: ResourceStore, v: Value) returns (rs': ResourceStore)
{
    var a: Address;
    a := sender#Transaction_cons(txn);
    if (domain#ResourceStore(rs)[a]) {
        // sender already has the resource
        abort_flag := true;
    }
    rs' := ResourceStore(domain#ResourceStore(rs)[a := true], contents#ResourceStore(rs)[a := v]);
}

procedure {:inline 1} MoveFrom(address: Value, rs: ResourceStore) returns (dst: Value, rs': ResourceStore)
{
    var a: Address;
    assert is#Address(address);
    a := a#Address(address);
    assert domain#ResourceStore(rs)[a];
    dst := contents#ResourceStore(rs)[a];
    rs' := ResourceStore(domain#ResourceStore(rs)[a := false], contents#ResourceStore(rs));
}

procedure {:inline 1} BorrowGlobal(address: Value, t: TypeName, rs: ResourceStore) returns (dst: Reference)
{
    var a: Address;
    var v: Value;
    a := a#Address(address);
    if (!domain#ResourceStore(rs)[a]) {
        abort_flag := true;
    }
    v := contents#ResourceStore(rs)[a];
    dst := Reference(Global(a, t), Nil(), v);
}

procedure {:inline 1} BorrowLoc(c: CreationTime, l: LocalName, local: Value) returns (dst: Reference)
{
    dst := Reference(Local(c, l), Nil(), local);
}

procedure {:inline 1} BorrowField(src: Reference, f: FieldName) returns (dst: Reference)
{
    assert is#Map(v#Reference(src));
    dst := Reference(rt#Reference(src), Cons(p#Reference(src), Field(f)), m#Map(v#Reference(src))[Field(f)]);
}

procedure {:inline 1} WriteRef(to: Reference, v: Value) returns (to': Reference)
{
    to' := Reference(rt#Reference(to), p#Reference(to), v);
}

procedure {:inline 1} ReadRef(from: Reference) returns (v: Value)
{
    v := v#Reference(from);
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
