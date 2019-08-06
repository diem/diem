type TypeName;
type FieldName;
type LocalName;
type Address;
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
function {:constructor} GlobalReference(a: Address, t: TypeName, p: Path, v: Value): Reference;
function {:constructor} LocalReference(c: CreationTime, l: LocalName, p: Path, v: Value): Reference;

type {:datatype} ResourceStore;
function {:constructor} ResourceStore(domain: [Address]bool, contents: [Address]Value): ResourceStore;

var senderAddress: Value;

procedure {:inline 1} DeepUpdateReference(src: Reference, dst: Reference) returns (dst': Reference)
{
    var isPrefix: bool;
    var v': Value;
    dst' := dst;
    if (is#LocalReference(src)) {
        if (is#LocalReference(dst) && c#LocalReference(src) == c#LocalReference(dst) && l#LocalReference(src) == l#LocalReference(dst)) {
            call isPrefix := IsPrefixMax(p#LocalReference(dst), p#LocalReference(src));
            if (isPrefix) {
                call v' := UpdateValueMax(p#LocalReference(src), v#LocalReference(src), p#LocalReference(dst), v#LocalReference(dst));
                dst' := LocalReference(c#LocalReference(dst), l#LocalReference(dst), p#LocalReference(dst), v');
            }
        }
    } else {
        if (is#GlobalReference(dst) && a#GlobalReference(src) == a#GlobalReference(dst) && t#GlobalReference(src) == t#GlobalReference(dst)) {
            call isPrefix := IsPrefixMax(p#GlobalReference(dst), p#GlobalReference(src));
            if (isPrefix) {
                call v' := UpdateValueMax(p#GlobalReference(src), v#GlobalReference(src), p#GlobalReference(dst), v#GlobalReference(dst));
                dst' := GlobalReference(a#GlobalReference(dst), t#GlobalReference(dst), p#GlobalReference(dst), v');
            }
        }
    }
}

procedure {:inline 1} DeepUpdateLocal(c: CreationTime, l: LocalName, src: Reference, dst: Value) returns (dst': Value)
{
    var v': Value;
    dst' := dst;
    if (is#LocalReference(src) && c == c#LocalReference(src) && l == l#LocalReference(src)) {
        call dst' := UpdateValueMax(p#LocalReference(src), v#LocalReference(src), Nil(), dst);
    }
}

procedure {:inline 1} DeepUpdateGlobal(t: TypeName, src: Reference, dst: ResourceStore) returns (dst': ResourceStore)
{
    var v': Value;
    dst' := dst;
    if (is#GlobalReference(src) && t == t#GlobalReference(src)) {
        call v' := UpdateValueMax(p#GlobalReference(src), v#GlobalReference(src), Nil(), contents#ResourceStore(dst)[a#GlobalReference(src)]);
        dst' := ResourceStore(domain#ResourceStore(dst), contents#ResourceStore(dst)[a#GlobalReference(src) := v']);
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
    a := a#Address(senderAddress);
    assert !domain#ResourceStore(rs)[a];
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
    assert domain#ResourceStore(rs)[a];
    v := contents#ResourceStore(rs)[a];
    dst := GlobalReference(a, t, Nil(), v);
}

procedure {:inline 1} BorrowLoc(c: CreationTime, l: LocalName, local: Value) returns (dst: Reference)
{
    dst := LocalReference(c, l, Nil(), local);
}

procedure {:inline 1} BorrowField(src: Reference, f: FieldName) returns (dst: Reference)
{
    if (is#GlobalReference(src)) {
        assert is#Map(v#GlobalReference(src));
        dst := GlobalReference(a#GlobalReference(src), t#GlobalReference(src), Cons(p#GlobalReference(src), Field(f)), m#Map(v#GlobalReference(src))[Field(f)]);
    } else {
        assert is#Map(v#LocalReference(src));
        dst := LocalReference(c#LocalReference(src), l#LocalReference(src), Cons(p#LocalReference(src), Field(f)), m#Map(v#LocalReference(src))[Field(f)]);
    }
}

procedure {:inline 1} WriteRef(to: Reference, v: Value) returns (to': Reference)
{
    if (is#GlobalReference(to)) {
        to' := GlobalReference(a#GlobalReference(to), t#GlobalReference(to), p#GlobalReference(to), v);
    } else {
        to' := LocalReference(c#LocalReference(to), l#LocalReference(to), p#LocalReference(to), v);
    }
}

procedure {:inline 1} ReadRef(from: Reference) returns (v: Value)
{
    if (is#GlobalReference(from)) {
        v := v#GlobalReference(from);
    } else {
        v := v#LocalReference(from);
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

procedure {:inline 1} Add(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    dst := Integer(i#Integer(src1) + i#Integer(src2));
}

procedure {:inline 1} Sub(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    dst := Integer(i#Integer(src1) - i#Integer(src2));
}

procedure {:inline 1} Mul(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    dst := Integer(i#Integer(src1) * i#Integer(src2));
}

procedure {:inline 1} Div(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
    dst := Integer(i#Integer(src1) div i#Integer(src2));
}

procedure {:inline 1} Mod(src1: Value, src2: Value) returns (dst: Value)
{
    assert is#Integer(src1) && is#Integer(src2);
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

// Special instruction
var Address_Exists: [Address]bool;
procedure {:inline 1} CreateAccount(addr_val: Value)
modifies Address_Exists;
{
  var addr: Address;
  addr := a#Address(addr_val);
  assert !Address_Exists[addr];
  Address_Exists[addr] := true;
}
