// ================================================================================
// Domains

type TypeName;
type FieldName = int;
type LocalName;
type Address = int;
type ByteArray;
type String;
type {:datatype} Edge;
function {:constructor} Field(f: FieldName): Edge;
function {:constructor} Index(i: int): Edge;

type {:datatype} Path;
function {:constructor} Path(p: [int]Edge, size: int): Path;
const EmptyPath: Path;
axiom size#Path(EmptyPath) == 0;
function {:inline} vector_index(p: Path, i: int): int {
    if (is#Field(p#Path(p)[i]))
        then f#Field(p#Path(p)[i])
    else
        i#Index(p#Path(p)[i])
}

type {:datatype} TypeValue;
function {:constructor} BooleanType() : TypeValue;
function {:constructor} IntegerType() : TypeValue;
function {:constructor} AddressType() : TypeValue;
function {:constructor} ByteArrayType() : TypeValue;
function {:constructor} StrType() : TypeValue;
function {:constructor} VectorType(t: TypeValue) : TypeValue;
function {:constructor} StructType(name: TypeName, ts: TypeValueArray) : TypeValue;
const DefaultTypeValue: TypeValue;
function {:builtin "MapConst"} MapConstTypeValue(tv: TypeValue): [int]TypeValue;

type {:datatype} TypeValueArray;
function {:constructor} TypeValueArray(v: [int]TypeValue, l: int): TypeValueArray;
const EmptyTypeValueArray: TypeValueArray;
axiom l#TypeValueArray(EmptyTypeValueArray) == 0;
axiom v#TypeValueArray(EmptyTypeValueArray) == MapConstTypeValue(DefaultTypeValue);

type {:datatype} Value;
function {:constructor} Boolean(b: bool): Value;
function {:constructor} Integer(i: int): Value;
function {:constructor} Address(a: Address): Value;
function {:constructor} ByteArray(b: ByteArray): Value;
function {:constructor} Str(a: String): Value;
function {:constructor} Vector(v: ValueArray): Value;
const DefaultValue: Value;
function {:builtin "MapConst"} MapConstValue(v: Value): [int]Value;

type {:datatype} ValueArray;
function {:constructor} ValueArray(v: [int]Value, l: int): ValueArray;
const EmptyValueArray: ValueArray;
axiom l#ValueArray(EmptyValueArray) == 0;
axiom v#ValueArray(EmptyValueArray) == MapConstValue(DefaultValue);
function {:inline} AddValueArray(a: ValueArray, v: Value): ValueArray {
    ValueArray(v#ValueArray(a)[l#ValueArray(a) := v], l#ValueArray(a) + 1)
}
function {:inline} RemoveValueArray(a: ValueArray): ValueArray {
    ValueArray(v#ValueArray(a)[l#ValueArray(a) := DefaultValue], l#ValueArray(a) - 1)
}
function {:inline} ConcatValueArray(a1: ValueArray, a2: ValueArray): ValueArray {
    ValueArray(
        (lambda i: int :: if i < l#ValueArray(a1) then v#ValueArray(a1)[i] else v#ValueArray(a2)[i - l#ValueArray(a1)]),
        l#ValueArray(a1) + l#ValueArray(a2))
}
function {:inline} ReverseValueArray(a: ValueArray): ValueArray {
    ValueArray(
        (lambda i: int :: if 0 <= i && i < l#ValueArray(a) then v#ValueArray(a)[l#ValueArray(a) - i - 1] else DefaultValue),
        l#ValueArray(a)
    )
}
function {:inline} UpdateValueArray(a: ValueArray, i: int, elem: Value): ValueArray {
    ValueArray(v#ValueArray(a)[i := elem], l#ValueArray(a))
}
function {:inline} SwapValueArray(a: ValueArray, i: int, j: int): ValueArray {
    ValueArray(v#ValueArray(a)[i := v#ValueArray(a)[j]][j := v#ValueArray(a)[i]], l#ValueArray(a))
}
function {:inline} IsEmpty(a: ValueArray): bool {
    l#ValueArray(a) == 0
}

/*
function WellFormedValueArray(a: ValueArray): bool {
    0 <= l#ValueArray(a) &&
    (forall i: int :: (0 <= i && i < l#ValueArray(a)) || v#ValueArray(a)[i] == DefaultValue) &&
    (forall i: int :: 0 <= i && i < l#ValueArray(a) ==> WellFormedValue(v#ValueArray(a)[i]))
}
function WellFormedValue(v: Value) : bool {
    if (is#Vector(v))
        then WellFormedValueArray(v#Vector(v))
    else
        true
}
*/

function IsEqual(v1: Value, v2: Value): bool {
    (v1 == v2) ||
    (is#Vector(v1) &&
     is#Vector(v2) &&
     vlen(v1) == vlen(v2) &&
     (forall i: int :: 0 <= i && i < vlen(v1) ==> IsEqual(vmap(v1)[i], vmap(v2)[i])))
}

function ReadValue(p: Path, i: int, v: Value) : Value
{
    if (i == size#Path(p))
        then v
    else
        ReadValue(p, i+1, vmap(v)[vector_index(p, i)])
}

function UpdateValue(p: Path, i: int, v: Value, new_v: Value): Value
{
    if (i == size#Path(p))
        then new_v
    else
        update_vector(v, vector_index(p, i), UpdateValue(p, i+1, vmap(v)[vector_index(p, i)], new_v))
}

function {:inline} vmap(v: Value): [int]Value {
    v#ValueArray(v#Vector(v))
}
function {:inline} vlen(v: Value): int {
    l#ValueArray(v#Vector(v))
}
function {:inline} mk_vector(): Value {
    Vector(EmptyValueArray)
}
function {:inline} push_back_vector(v: Value, elem: Value): Value {
    Vector(AddValueArray(v#Vector(v), elem))
}
function {:inline} pop_back_vector(v: Value): Value {
    Vector(RemoveValueArray(v#Vector(v)))
}
function {:inline} append_vector(v1: Value, v2: Value): Value {
    Vector(ConcatValueArray(v#Vector(v1), v#Vector(v2)))
}
function {:inline} reverse_vector(v: Value): Value {
    Vector(ReverseValueArray(v#Vector(v)))
}
function {:inline} update_vector(v: Value, i: int, elem: Value): Value {
    Vector(UpdateValueArray(v#Vector(v), i, elem))
}
function {:inline} swap_vector(v: Value, i: int, j: int): Value {
    Vector(SwapValueArray(v#Vector(v), i, j))
}

type {:datatype} Location;
function {:constructor} Global(t: TypeValue, a: Address): Location;
function {:constructor} Local(i: int): Location;

type {:datatype} Reference;
function {:constructor} Reference(l: Location, p: Path): Reference;

type {:datatype} Memory;
function {:constructor} Memory(domain: [Location]bool, contents: [Location]Value): Memory;

var m : Memory;
var local_counter : int;

// ============================================================================================
// Instructions

procedure {:inline 1} Exists(address: Value, t: TypeValue) returns (dst: Value)
requires is#Address(address);
{
    dst := Boolean(domain#Memory(m)[Global(t, a#Address(address))]);
}

procedure {:inline 1} MoveToSender(ta: TypeValue, v: Value)
{
    var a: Address;
    var l: Location;

    a := sender#Transaction_cons(txn);
    l := Global(ta, a);
    assert !domain#Memory(m)[l];
    m := Memory(domain#Memory(m)[l := true], contents#Memory(m)[l := v]);
}

procedure {:inline 1} MoveFrom(address: Value, ta: TypeValue) returns (dst: Value)
requires is#Address(address);
{
    var a: Address;
    var l: Location;

    a := a#Address(address);
    l := Global(ta, a);
    assert domain#Memory(m)[l];
    dst := contents#Memory(m)[l];
    m := Memory(domain#Memory(m)[l := false], contents#Memory(m));
}

procedure {:inline 1} BorrowGlobal(address: Value, ta: TypeValue) returns (dst: Reference)
requires is#Address(address);
{
    var a: Address;
    var v: Value;
    var l: Location;

    a := a#Address(address);
    l := Global(ta, a);
    assert domain#Memory(m)[l];
    dst := Reference(l, EmptyPath);
}

procedure {:inline 1} BorrowLoc(l: int) returns (dst: Reference)
{
    dst := Reference(Local(l), EmptyPath);
}

procedure {:inline 1} BorrowField(src: Reference, f: FieldName) returns (dst: Reference)
{
    var p: Path;
    var size: int;

    p := p#Reference(src);
    size := size#Path(p);
	p := Path(p#Path(p)[size := Field(f)], size+1);
    dst := Reference(l#Reference(src), p);
}

procedure {:inline 1} WriteRef(to: Reference, new_v: Value)
{
    var l: Location;
    var v: Value;

    l := l#Reference(to);
    v := contents#Memory(m)[l];
    v := UpdateValue(p#Reference(to), 0, v, new_v);
    m := Memory(domain#Memory(m), contents#Memory(m)[l := v]);
}

procedure {:inline 1} ReadRef(from: Reference) returns (v: Value)
{
    v := ReadValue(p#Reference(from), 0, contents#Memory(m)[l#Reference(from)]);
}

procedure {:inline 1} CopyOrMoveRef(local: Reference) returns (dst: Reference)
{
    dst := local;
}

procedure {:inline 1} CopyOrMoveValue(local: Value) returns (dst: Value)
{
    dst := local;
}

procedure {:inline 1} FreezeRef(src: Reference) returns (dst: Reference)
{
    dst := src;
}

// Pack, and Unpack are auto-generated for each type T
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

// ==================================================================================
// Native Vector Type

function {:inline} Vector_T_type_value(tv: TypeValue): TypeValue {
    VectorType(tv)
}

procedure {:inline 1} Vector_empty(ta: TypeValue) returns (v: Value) {
    v := mk_vector();
}

procedure {:inline 1} Vector_is_empty(ta: TypeValue, r: Reference) returns (b: Value) {
    var v: Value;

    call v := ReadRef(r);
    b := Boolean(vlen(v) == 0);
}

procedure {:inline 1} Vector_push_back(ta: TypeValue, r: Reference, val: Value) {
    var old_v: Value;

    call old_v := ReadRef(r);
    call WriteRef(r, push_back_vector(old_v, val));
}

procedure {:inline 1} Vector_pop_back(ta: TypeValue, r: Reference) returns (e: Value){
    var v: Value;
    var old_len: int;

    call v := ReadRef(r);
    old_len := vlen(v);
    e := vmap(v)[old_len-1];
    call WriteRef(r, pop_back_vector(v));
}

procedure {:inline 1} Vector_append(ta: TypeValue, r: Reference, other_v: Value) {
    var v: Value;
    var old_len: int;
    var other_len: int;
    var result: Value;

    call v := ReadRef(r);
    old_len := vlen(v);
    other_len := vlen(other_v);
    result := append_vector(v, other_v);
    call WriteRef(r, result);
}

procedure {:inline 1} Vector_reverse(ta: TypeValue, r: Reference) {
    var v: Value;
    var result: Value;
    var len: int;

    call v := ReadRef(r);
    len := vlen(v);
    result := reverse_vector(v);
    call WriteRef(r, result);
}

procedure {:inline 1} Vector_length(ta: TypeValue, r: Reference) returns (l: Value) {
    var v: Value;

    call v := ReadRef(r);
    l := Integer(vlen(v));
}

procedure {:inline 1} Vector_borrow(ta: TypeValue, src: Reference, index: Value) returns (dst: Reference) {
    var p: Path;
    var size: int;

    p := p#Reference(src);
    size := size#Path(p);
	p := Path(p#Path(p)[size := Index(i#Integer(index))], size+1);
    dst := Reference(l#Reference(src), p);
}

procedure {:inline 1} Vector_borrow_mut(ta: TypeValue, src: Reference, index: Value) returns (dst: Reference) {
    var p: Path;
    var size: int;

    p := p#Reference(src);
    size := size#Path(p);
	  p := Path(p#Path(p)[size := Index(i#Integer(index))], size+1);
    dst := Reference(l#Reference(src), p);
}

procedure {:inline 1} Vector_destroy_empty(ta: TypeValue, v: Value) {
    assert (vlen(v) == 0);
}

procedure {:inline 1} Vector_swap(ta: TypeValue, src: Reference, i: Value, j: Value) {
    var i_ind: int;
    var j_ind: int;
    var v: Value;

    i_ind := i#Integer(i);
    j_ind := i#Integer(j);
    call v := ReadRef(src);
    assert (vlen(v) > i_ind && vlen(v) > j_ind);
    v := swap_vector(v, i_ind, j_ind);
    call WriteRef(src, v);
}

procedure {:inline 1} Vector_get(ta: TypeValue, src: Reference, i: Value) returns (e: Value) {
    var i_ind: int;
    var v: Value;

    call v := ReadRef(src);
    i_ind := i#Integer(i);
    assert (i_ind < vlen(v));
    e := vmap(v)[i_ind];
}

procedure {:inline 1} Vector_set(ta: TypeValue, src: Reference, i: Value, e: Value) {
    var i_ind: int;
    var v: Value;

    i_ind := i#Integer(i);
    call v := ReadRef(src);
    assert (vlen(v) > i_ind);
    v := update_vector(v, i_ind, e);
    call WriteRef(src, v);
}
