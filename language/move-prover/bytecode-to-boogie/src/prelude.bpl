// ================================================================================
// Domains

// Debug tracking
// --------------

// Debug tracking is used to inject information used for model analysis. The generated code emits statements
// like this:
//
//     assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_position, value);
//
// While those tracking assumptions are trivially true for the provers logic, the solver (at least Z3)
// will construct a function mapping which appears in the model, e.g.:
//
//     $DebugTrackLocal -> {
//         1 1 0 440 (Vector (ValueArray |T@[Int]Value!val!0| 0)) -> true
//         1 1 2 533 (Integer 1) -> true
//         ...
//         else -> true
//     }
//
// This information can then be read out from the model.


// Tracks debug information for a parameter, local or a return parameter. Return parameter indices start at
// the overall number of locals (including parameters).
function $DebugTrackLocal(module_idx: int, func_idx: int, var_idx: int, code_index: int, value: Value) : bool {
  true
}

// Tracks at which location a function was aborted.
function $DebugTrackAbort(module_idx: int, func_idx: int, code_index: int) : bool {
  true
}

// Path type
// ---------

type Edge = int; // both FieldName and vector index are mapped to int

type {:datatype} Path;
function {:constructor} Path(p: [int]Edge, size: int): Path;
const EmptyPath: Path;
axiom size#Path(EmptyPath) == 0;

function {:inline 1} path_index_at(p: Path, i: int): int {
    p#Path(p)[i]
}

// Type Values
// -----------

type TypeName;
type FieldName = int;
type LocalName;
type {:datatype} TypeValue;
function {:constructor} BooleanType() : TypeValue;
function {:constructor} IntegerType() : TypeValue;
function {:constructor} AddressType() : TypeValue;
function {:constructor} ByteArrayType() : TypeValue;
function {:constructor} StrType() : TypeValue;
function {:constructor} VectorType(t: TypeValue) : TypeValue;
function {:constructor} StructType(name: TypeName, ts: TypeValueArray) : TypeValue;
function {:constructor} ErrorType() : TypeValue;
const DefaultTypeValue: TypeValue;
axiom DefaultTypeValue == ErrorType();
function {:builtin "MapConst"} MapConstTypeValue(tv: TypeValue): [int]TypeValue;

type {:datatype} TypeValueArray;
function {:constructor} TypeValueArray(v: [int]TypeValue, l: int): TypeValueArray;
const EmptyTypeValueArray: TypeValueArray;
axiom l#TypeValueArray(EmptyTypeValueArray) == 0;
axiom v#TypeValueArray(EmptyTypeValueArray) == MapConstTypeValue(DefaultTypeValue);

function {:inline 1} ExtendTypeValueArray(ta: TypeValueArray, tv: TypeValue): TypeValueArray {
    TypeValueArray(v#TypeValueArray(ta)[l#TypeValueArray(ta) := tv], l#TypeValueArray(ta) + 1)
}


// Values
// ------

type Address = int;
type ByteArray;
type String;
type {:datatype} Value;

const MAX_U8: int;
axiom MAX_U8 == 255;
const MAX_U64: int;
axiom MAX_U64 == 9223372036854775807;
const MAX_U128: int;
axiom MAX_U128 == 340282366920938463463374607431768211456;

function {:constructor} Boolean(b: bool): Value;
function {:constructor} Integer(i: int): Value;
function {:constructor} Address(a: Address): Value;
function {:constructor} ByteArray(b: ByteArray): Value;
function {:constructor} Str(a: String): Value;
function {:constructor} Vector(v: ValueArray): Value; // used to both represent move Struct and Vector
function {:constructor} Error(): Value;
const DefaultValue: Value;
axiom DefaultValue == Error();
function {:builtin "MapConst"} MapConstValue(v: Value): [int]Value;

function {:inline} IsValidU8(v: Value): bool {
  is#Integer(v) && i#Integer(v) >= 0 && i#Integer(v) <= MAX_U8
}

function {:inline 1} max_u64(): Value {
  Integer(9223372036854775807)
}

function {:inline} IsValidInteger(v: Value): bool {
  is#Integer(v) && i#Integer(v) >= 0 && i#Integer(v) <= i#Integer(max_u64())
}

function {:inline} IsValidU64(v: Value): bool {
  is#Integer(v) && i#Integer(v) >= 0 && i#Integer(v) <= MAX_U64
}

function {:inline} IsValidU128(v: Value): bool {
  is#Integer(v) && i#Integer(v) >= 0 && i#Integer(v) <= MAX_U128
}

// Value Array
// -----------

type {:datatype} ValueArray;

function {:constructor} ValueArray(v: [int]Value, l: int): ValueArray;
const EmptyValueArray: ValueArray;

axiom l#ValueArray(EmptyValueArray) == 0;
axiom v#ValueArray(EmptyValueArray) == MapConstValue(DefaultValue);

function {:inline 1} AddValueArray(a: ValueArray, v: Value): ValueArray {
    ValueArray(v#ValueArray(a)[l#ValueArray(a) := v], l#ValueArray(a) + 1)
}

function {:inline 1} RemoveValueArray(a: ValueArray): ValueArray {
    ValueArray(v#ValueArray(a)[l#ValueArray(a) - 1 := DefaultValue], l#ValueArray(a) - 1)
}
function {:inline 1} ConcatValueArray(a1: ValueArray, a2: ValueArray): ValueArray {
    ValueArray(
        (lambda i: int :: if i < l#ValueArray(a1) then v#ValueArray(a1)[i] else v#ValueArray(a2)[i - l#ValueArray(a1)]),
        l#ValueArray(a1) + l#ValueArray(a2))
}
function {:inline 1} ReverseValueArray(a: ValueArray): ValueArray {
    ValueArray(
        (lambda i: int :: if 0 <= i && i < l#ValueArray(a) then v#ValueArray(a)[l#ValueArray(a) - i - 1] else DefaultValue),
        l#ValueArray(a)
    )
}
function {:inline 1} ExtendValueArray(a: ValueArray, elem: Value): ValueArray {
    ValueArray(v#ValueArray(a)[l#ValueArray(a) := elem], l#ValueArray(a) + 1)
}
function {:inline 1} UpdateValueArray(a: ValueArray, i: int, elem: Value): ValueArray {
    ValueArray(v#ValueArray(a)[i := elem], l#ValueArray(a))
}
function {:inline 1} SwapValueArray(a: ValueArray, i: int, j: int): ValueArray {
    ValueArray(v#ValueArray(a)[i := v#ValueArray(a)[j]][j := v#ValueArray(a)[i]], l#ValueArray(a))
}
function {:inline 1} IsEmpty(a: ValueArray): bool {
    l#ValueArray(a) == 0
}

// Stratified Functions on Values
// ------------------------------

// TODO: templatize this or move it back to the translator. For now we
//   prefer to handcode this so its easier to evolve the model independent of the
//   translator.

const StratificationDepth: int;
axiom StratificationDepth == 4;

function {:inline 1} IsEqual4(v1: Value, v2: Value): bool {
    v1 == v2
}
function {:inline 1} IsEqual3(v1: Value, v2: Value): bool {
    (v1 == v2) ||
    (is#Vector(v1) &&
     is#Vector(v2) &&
     vlen(v1) == vlen(v2) &&
     (forall i: int :: 0 <= i && i < vlen(v1) ==> IsEqual4(vmap(v1)[i], vmap(v2)[i])))
}
function {:inline 1} IsEqual2(v1: Value, v2: Value): bool {
    (v1 == v2) ||
    (is#Vector(v1) &&
     is#Vector(v2) &&
     vlen(v1) == vlen(v2) &&
     (forall i: int :: 0 <= i && i < vlen(v1) ==> IsEqual3(vmap(v1)[i], vmap(v2)[i])))
}
function {:inline 1} IsEqual1(v1: Value, v2: Value): bool {
    (v1 == v2) ||
    (is#Vector(v1) &&
     is#Vector(v2) &&
     vlen(v1) == vlen(v2) &&
     (forall i: int :: 0 <= i && i < vlen(v1) ==> IsEqual2(vmap(v1)[i], vmap(v2)[i])))
}
function {:inline 1} IsEqual(v1: Value, v2: Value): bool {
    IsEqual1(v1, v2)
}

function {:inline 1} ReadValue4(p: Path, v: Value): Value {
    v
}
function {:inline 1} ReadValue3(p: Path, v: Value) : Value {
    if (3 == size#Path(p)) then
        v
    else
        ReadValue4(p, vmap(v)[path_index_at(p, 3)])
}
function {:inline 1} ReadValue2(p: Path, v: Value) : Value {
    if (2 == size#Path(p)) then
        v
    else
        ReadValue3(p, vmap(v)[path_index_at(p, 2)])
}
function {:inline 1} ReadValue1(p: Path, v: Value) : Value {
    if (1 == size#Path(p)) then
        v
    else
        ReadValue2(p, vmap(v)[path_index_at(p, 1)])
}
function {:inline 1} ReadValue0(p: Path, v: Value) : Value {
    if (0 == size#Path(p)) then
        v
    else
        ReadValue1(p, vmap(v)[path_index_at(p, 0)])
}
function {:inline 1} ReadValue(p: Path, v: Value): Value {
    ReadValue0(p, v)
}

function {:inline 1} UpdateValue4(p: Path, v: Value, new_v: Value): Value {
    new_v
}
function {:inline 1} UpdateValue3(p: Path, v: Value, new_v: Value): Value {
    if (3 == size#Path(p)) then
        new_v
    else
        update_vector(v, path_index_at(p, 3), UpdateValue4(p, vmap(v)[path_index_at(p, 3)], new_v))
}
function {:inline 1} UpdateValue2(p: Path, v: Value, new_v: Value): Value {
    if (2 == size#Path(p)) then
        new_v
    else
        update_vector(v, path_index_at(p, 2), UpdateValue3(p, vmap(v)[path_index_at(p, 2)], new_v))
}
function {:inline 1} UpdateValue1(p: Path, v: Value, new_v: Value): Value {
    if (1 == size#Path(p)) then
        new_v
    else
        update_vector(v, path_index_at(p, 1), UpdateValue2(p, vmap(v)[path_index_at(p, 1)], new_v))
}
function {:inline 1} UpdateValue0(p: Path, v: Value, new_v: Value): Value {
    if (0 == size#Path(p)) then
        new_v
    else
        update_vector(v, path_index_at(p, 0), UpdateValue1(p, vmap(v)[path_index_at(p, 0)], new_v))
}
function {:inline 1} UpdateValue(p: Path, v: Value, new_v: Value): Value {
    UpdateValue0(p, v, new_v)
}

// Vector related functions on values
// ----------------------------------

function {:inline 1} vmap(v: Value): [int]Value {
    v#ValueArray(v#Vector(v))
}
function {:inline 1} vlen(v: Value): int {
    l#ValueArray(v#Vector(v))
}
function {:inline 1} mk_vector(): Value {
    Vector(EmptyValueArray)
}
function {:inline 1} push_back_vector(v: Value, elem: Value): Value {
    Vector(AddValueArray(v#Vector(v), elem))
}
function {:inline 1} pop_back_vector(v: Value): Value {
    Vector(RemoveValueArray(v#Vector(v)))
}
function {:inline 1} append_vector(v1: Value, v2: Value): Value {
    Vector(ConcatValueArray(v#Vector(v1), v#Vector(v2)))
}
function {:inline 1} reverse_vector(v: Value): Value {
    Vector(ReverseValueArray(v#Vector(v)))
}
function {:inline 1} update_vector(v: Value, i: int, elem: Value): Value {
    Vector(UpdateValueArray(v#Vector(v), i, elem))
}
function {:inline 1} swap_vector(v: Value, i: int, j: int): Value {
    Vector(SwapValueArray(v#Vector(v), i, j))
}

// ============================================================================================
// Memory

type {:datatype} Location;
function {:constructor} Global(t: TypeValue, a: Address): Location;
function {:constructor} Local(i: int): Location;

type {:datatype} Reference;
function {:constructor} Reference(l: Location, t: TypeValue, p: Path): Reference;
const DefaultReference: Reference;

function {:inline} RootReferenceType(r: Reference): TypeValue {
  t#Reference(r)
}

function {:inline} RootReference(r: Reference): Reference {
  Reference(l#Reference(r), t#Reference(r), EmptyPath)
}

function {:inline} IsValidReferenceParameter(m: Memory, local_counter: int, r: Reference): bool {
  // If the reference parameter is for a local, its index must be less than the current
  // local counter. This prevents any aliasing with locals which we create later.
  (is#Local(l#Reference(r)) ==> i#Local(l#Reference(r)) < local_counter)
  &&
  // The path must be in range of current stratification depth.
  (size#Path(p#Reference(r)) >= 0 && size#Path(p#Reference(r)) < StratificationDepth)
  &&
  // Each internal node in the memory tree must be a vector and each index in the path must be in range.
  (size#Path(p#Reference(r)) == 0 ||
   (is#Vector(contents#Memory(m)[l#Reference(r)]) && p#Path(p#Reference(r))[0] >= 0 &&
     p#Path(p#Reference(r))[0] < vlen(contents#Memory(m)[l#Reference(r)])))
  &&
  (size#Path(p#Reference(r)) <= 1 ||
    (is#Vector(ReadValue(Path(p#Path(p#Reference(r)), 1), contents#Memory(m)[l#Reference(r)]))
     && p#Path(p#Reference(r))[1] >= 0 &&
     p#Path(p#Reference(r))[1] < vlen(ReadValue(Path(p#Path(p#Reference(r)), 1), contents#Memory(m)[l#Reference(r)]))))
  &&
  (size#Path(p#Reference(r)) <= 2 ||
    (is#Vector(ReadValue(Path(p#Path(p#Reference(r)), 2), contents#Memory(m)[l#Reference(r)]))
     && p#Path(p#Reference(r))[1] >= 0 &&
     p#Path(p#Reference(r))[1] < vlen(ReadValue(Path(p#Path(p#Reference(r)), 2), contents#Memory(m)[l#Reference(r)]))))
}

type {:datatype} Memory;
function {:constructor} Memory(domain: [Location]bool, contents: [Location]Value): Memory;

function {:builtin "MapConst"} ConstMemoryDomain(v: bool): [Location]bool;
function {:builtin "MapConst"} ConstMemoryContent(v: Value): [Location]Value;

const EmptyMemory: Memory;
axiom domain#Memory(EmptyMemory) == ConstMemoryDomain(false);
axiom contents#Memory(EmptyMemory) == ConstMemoryContent(DefaultValue);

var __m : Memory;
var __local_counter : int;
var __abort_flag: bool;

function {:inline 1} GetLocal(m: Memory, idx: int): Value {
   contents#Memory(m)[Local(idx)]
}

function {:inline 1} UpdateLocal(m: Memory, idx: int, v: Value): Memory {
    Memory(domain#Memory(m)[Local(idx) := true], contents#Memory(m)[Local(idx) := v])
}

procedure {:inline 1} InitVerification() {
  // Set local counter to 0
  __local_counter := 0;
  // Assume sender account exists.
  assume ExistsTxnSenderAccount(__m, __txn);
}

// ============================================================================================
// Specifications

// TODO: unify some of this with instruction procedures to avoid duplication

// Tests whether resource exists.
function {:inline 1} ExistsResourceRaw(m: Memory, resource: TypeValue, addr: Address): bool {
    domain#Memory(m)[Global(resource, addr)]
}
function {:inline 1} ExistsResource(m: Memory, resource: TypeValue, addr: Address): Value {
    Boolean(ExistsResourceRaw(m, resource, addr))
}

// Obtains reference to the given resource.
function {:inline 1} GetResourceReference(resource: TypeValue, addr: Address): Reference {
    Reference(Global(resource, addr), resource, EmptyPath)
}

// Obtains reference to local.
function {:inline 1} GetLocalReference(frame_idx: int, idx: int, t: TypeValue): Reference {
    Reference(Local(frame_idx + idx), t, EmptyPath)
}

// Applies a field selection to the reference.
function {:inline 1} SelectFieldFromRef(ref: Reference, field: FieldName): Reference {
    Reference(
      l#Reference(ref),
      t#Reference(ref),
      Path(p#Path(p#Reference(ref))[size#Path(p#Reference(ref)) := field], size#Path(p#Reference(ref)) + 1)
    )
}

// Applies a field selection to a value.
function {:inline 1} SelectField(val: Value, field: FieldName): Value {
    vmap(val)[field]
}

// Dereferences a reference.
function {:inline 1} Dereference(m: Memory, ref: Reference): Value {
    ReadValue(p#Reference(ref), contents#Memory(m)[l#Reference(ref)])
}

// Checker whether sender account exists.
function {:inline 1} ExistsTxnSenderAccount(m: Memory, txn: Transaction): bool {
   domain#Memory(m)[Global(LibraAccount_T_type_value(), sender#Transaction(txn))]
}

// Forward declaration of type value of LibraAccount. This is declared so we can define
// ExistsTxnSenderAccount.
function LibraAccount_T_type_value(): TypeValue;

// Returns sender address.
function {:inline 1} TxnSenderAddress(txn: Transaction): Address {
  sender#Transaction(txn)
}


// ============================================================================================
// Instructions

procedure {:inline 1} Exists(address: Value, t: TypeValue) returns (dst: Value)
{
    assume is#Address(address);
    dst := ExistsResource(__m, t, a#Address(address));
}

procedure {:inline 1} MoveToSender(ta: TypeValue, v: Value)
{
    var a: Address;
    var l: Location;

    a := sender#Transaction(__txn);
    l := Global(ta, a);
    if (ExistsResourceRaw(__m, ta, a)) {
        __abort_flag := true;
        return;
    }
    __m := Memory(domain#Memory(__m)[l := true], contents#Memory(__m)[l := v]);
}

procedure {:inline 1} MoveFrom(address: Value, ta: TypeValue) returns (dst: Value)
{
    var a: Address;
    var l: Location;
    assume is#Address(address);
    a := a#Address(address);
    l := Global(ta, a);
    if (!ExistsResourceRaw(__m, ta, a)) {
        __abort_flag := true;
        return;
    }
    dst := contents#Memory(__m)[l];
    __m := Memory(domain#Memory(__m)[l := false], contents#Memory(__m)[l := DefaultValue]);
}

procedure {:inline 1} BorrowGlobal(address: Value, ta: TypeValue) returns (dst: Reference)
{
    var a: Address;
    var v: Value;
    var l: Location;
    assume is#Address(address);
    a := a#Address(address);
    l := Global(ta, a);
    if (!ExistsResourceRaw(__m, ta, a)) {
        __abort_flag := true;
        return;
    }
    dst := Reference(l, ta, EmptyPath);
}

procedure {:inline 1} BorrowLoc(l: int, t: TypeValue) returns (dst: Reference)
{
    dst := Reference(Local(l), t, EmptyPath);
}

procedure {:inline 1} BorrowField(src: Reference, f: FieldName) returns (dst: Reference)
{
    var p: Path;
    var size: int;

    p := p#Reference(src);
    size := size#Path(p);
	p := Path(p#Path(p)[size := f], size+1);
    dst := Reference(l#Reference(src), t#Reference(src), p);
}

procedure {:inline 1} WriteRef(to: Reference, new_v: Value)
{
    var l: Location;
    var v: Value;

    l := l#Reference(to);
    v := contents#Memory(__m)[l];
    v := UpdateValue(p#Reference(to), v, new_v);
    __m := Memory(domain#Memory(__m), contents#Memory(__m)[l := v]);
}

procedure {:inline 1} ReadRef(from: Reference) returns (v: Value)
{
    v := ReadValue(p#Reference(from), contents#Memory(__m)[l#Reference(from)]);
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

procedure {:inline 1} CastU8(src: Value) returns (dst: Value)
{
    assume is#Integer(src);
    if (i#Integer(src) > MAX_U8) {
        __abort_flag := true;
        return;
    }
    dst := src;
}

procedure {:inline 1} CastU64(src: Value) returns (dst: Value)
{
    assume is#Integer(src);
    if (i#Integer(src) > MAX_U64) {
        __abort_flag := true;
        return;
    }
    dst := src;
}

procedure {:inline 1} CastU128(src: Value) returns (dst: Value)
{
    assume is#Integer(src);
    if (i#Integer(src) > MAX_U128) {
        __abort_flag := true;
        return;
    }
    dst := src;
}

procedure {:inline 1} AddU8(src1: Value, src2: Value) returns (dst: Value)
{
    assume IsValidU8(src1) && IsValidU8(src2);
    if (i#Integer(src1) + i#Integer(src2) > MAX_U8) {
        __abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) + i#Integer(src2));
}

procedure {:inline 1} AddU64(src1: Value, src2: Value) returns (dst: Value)
{
    assume IsValidU64(src1) && IsValidU64(src2);
    if (i#Integer(src1) + i#Integer(src2) > MAX_U64) {
        __abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) + i#Integer(src2));
}

procedure {:inline 1} AddU128(src1: Value, src2: Value) returns (dst: Value)
{
    assume IsValidU128(src1) && IsValidU128(src2);
    if (i#Integer(src1) + i#Integer(src2) > MAX_U128) {
        __abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) + i#Integer(src2));
}

procedure {:inline 1} Sub(src1: Value, src2: Value) returns (dst: Value)
{
    assume is#Integer(src1) && is#Integer(src2);
    if (i#Integer(src1) < i#Integer(src2)) {
        __abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) - i#Integer(src2));
}

procedure {:inline 1} MulU8(src1: Value, src2: Value) returns (dst: Value)
{
    assume IsValidU8(src1) && IsValidU8(src2);
    if (i#Integer(src1) * i#Integer(src2) > MAX_U8) {
        __abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) * i#Integer(src2));
}

procedure {:inline 1} MulU64(src1: Value, src2: Value) returns (dst: Value)
{
    assume IsValidU64(src1) && IsValidU64(src2);
    if (i#Integer(src1) * i#Integer(src2) > MAX_U64) {
        __abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) * i#Integer(src2));
}

procedure {:inline 1} MulU128(src1: Value, src2: Value) returns (dst: Value)
{
    assume IsValidU128(src1) && IsValidU128(src2);
    if (i#Integer(src1) * i#Integer(src2) > MAX_U128) {
        __abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) * i#Integer(src2));
}

procedure {:inline 1} Div(src1: Value, src2: Value) returns (dst: Value)
{
    assume is#Integer(src1) && is#Integer(src2);
    if (i#Integer(src2) == 0) {
        __abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) div i#Integer(src2));
}

procedure {:inline 1} Mod(src1: Value, src2: Value) returns (dst: Value)
{
    assume is#Integer(src1) && is#Integer(src2);
    if (i#Integer(src2) == 0) {
        __abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) mod i#Integer(src2));
}

procedure {:inline 1} Lt(src1: Value, src2: Value) returns (dst: Value)
{
    assume is#Integer(src1) && is#Integer(src2);
    dst := Boolean(i#Integer(src1) < i#Integer(src2));
}

procedure {:inline 1} Gt(src1: Value, src2: Value) returns (dst: Value)
{
    assume is#Integer(src1) && is#Integer(src2);
    dst := Boolean(i#Integer(src1) > i#Integer(src2));
}

procedure {:inline 1} Le(src1: Value, src2: Value) returns (dst: Value)
{
    assume is#Integer(src1) && is#Integer(src2);
    dst := Boolean(i#Integer(src1) <= i#Integer(src2));
}

procedure {:inline 1} Ge(src1: Value, src2: Value) returns (dst: Value)
{
    assume is#Integer(src1) && is#Integer(src2);
    dst := Boolean(i#Integer(src1) >= i#Integer(src2));
}

procedure {:inline 1} And(src1: Value, src2: Value) returns (dst: Value)
{
    assume is#Boolean(src1) && is#Boolean(src2);
    dst := Boolean(b#Boolean(src1) && b#Boolean(src2));
}

procedure {:inline 1} Or(src1: Value, src2: Value) returns (dst: Value)
{
    assume is#Boolean(src1) && is#Boolean(src2);
    dst := Boolean(b#Boolean(src1) || b#Boolean(src2));
}

procedure {:inline 1} Not(src: Value) returns (dst: Value)
{
    assume is#Boolean(src);
    dst := Boolean(!b#Boolean(src));
}

// Pack and Unpack are auto-generated for each type T

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
// --------------------------------

type {:datatype} Transaction;
var __txn: Transaction;
function {:constructor} Transaction(
  gas_unit_price: int, max_gas_units: int, public_key: ByteArray,
  sender: Address, sequence_number: int, gas_remaining: int) : Transaction;


const some_key: ByteArray;

procedure {:inline 1} InitTransaction(sender: Address) {
  __txn := Transaction(1, 1000, some_key, sender, 0, 1000);
}

procedure {:inline 1} GetGasRemaining() returns (ret_gas_remaining: Value)
{
  ret_gas_remaining := Integer(gas_remaining#Transaction(__txn));
}

procedure {:inline 1} GetTxnSequenceNumber() returns (ret_sequence_number: Value)
{
  ret_sequence_number := Integer(sequence_number#Transaction(__txn));
}

procedure {:inline 1} GetTxnPublicKey() returns (ret_public_key: Value)
{
  ret_public_key := ByteArray(public_key#Transaction(__txn));
}

procedure {:inline 1} GetTxnSenderAddress() returns (ret_sender: Value)
{
  ret_sender := Address(sender#Transaction(__txn));
}

procedure {:inline 1} GetTxnMaxGasUnits() returns (ret_max_gas_units: Value)
{
  ret_max_gas_units := Integer(max_gas_units#Transaction(__txn));
}

procedure {:inline 1} GetTxnGasUnitPrice() returns (ret_gas_unit_price: Value)
{
  ret_gas_unit_price := Integer(gas_unit_price#Transaction(__txn));
}

// ==================================================================================
// Native Vector Type

function {:inline} Vector_T_type_value(tv: TypeValue): TypeValue {
    VectorType(tv)
}

function {:inline} $Vector_T_is_well_formed(v: Value): bool {
    is#Vector(v)
}

procedure {:inline 1} Vector_empty(ta: TypeValue) returns (v: Value) {
    v := mk_vector();
}

procedure {:inline 1} Vector_is_empty(ta: TypeValue, r: Reference) returns (b: Value) {
    var v: Value;
    v := Dereference(__m, r);
    assume is#Vector(v);
    assume IsValidReferenceParameter(__m, __local_counter, r);
    b := Boolean(vlen(v) == 0);
}

procedure {:inline 1} Vector_push_back(ta: TypeValue, r: Reference, val: Value) {
    var v: Value;
    v := Dereference(__m, r);
    assume is#Vector(v);
    assume IsValidReferenceParameter(__m, __local_counter, r);
    call WriteRef(r, push_back_vector(v, val));
}

procedure {:inline 1} Vector_pop_back(ta: TypeValue, r: Reference) returns (e: Value) {
    var v: Value;
    var len: int;
    v := Dereference(__m, r);
    assume is#Vector(v);
    assume IsValidReferenceParameter(__m, __local_counter, r);
    len := vlen(v);
    if (len == 0) {
        __abort_flag := true;
        return;
    }
    e := vmap(v)[len-1];
    call WriteRef(r, pop_back_vector(v));
}

procedure {:inline 1} Vector_append(ta: TypeValue, r: Reference, other: Value) {
    var v: Value;
    v := Dereference(__m, r);
    assume is#Vector(v);
    assume IsValidReferenceParameter(__m, __local_counter, r);
    assume is#Vector(other);
    call WriteRef(r, append_vector(v, other));
}

procedure {:inline 1} Vector_reverse(ta: TypeValue, r: Reference) {
    var v: Value;
    v := Dereference(__m, r);
    assume is#Vector(v);
    assume IsValidReferenceParameter(__m, __local_counter, r);
    call WriteRef(r, reverse_vector(v));
}

procedure {:inline 1} Vector_length(ta: TypeValue, r: Reference) returns (l: Value) {
    var v: Value;
    v := Dereference(__m, r);
    assume is#Vector(v);
    assume IsValidReferenceParameter(__m, __local_counter, r);
    l := Integer(vlen(v));
}

procedure {:inline 1} Vector_borrow(ta: TypeValue, src: Reference, index: Value) returns (dst: Reference) {
    call dst := Vector_borrow_mut(ta, src, index);
}

procedure {:inline 1} Vector_borrow_mut(ta: TypeValue, src: Reference, index: Value) returns (dst: Reference) {
    var p: Path;
    var size: int;
    var i_ind: int;
    var v: Value;

    assume is#Integer(index);
    i_ind := i#Integer(index);
    v := Dereference(__m, src);
    assume is#Vector(v);
    assume IsValidReferenceParameter(__m, __local_counter, src);
    if (i_ind < 0 || i_ind >= vlen(v)) {
        __abort_flag := true;
        return;
    }

    p := p#Reference(src);
    size := size#Path(p);
	p := Path(p#Path(p)[size := i_ind], size+1);
    dst := Reference(l#Reference(src), t#Reference(src), p);
}

procedure {:inline 1} Vector_destroy_empty(ta: TypeValue, v: Value) {
    if (vlen(v) != 0) {
      __abort_flag := true;
    }
}

procedure {:inline 1} Vector_swap(ta: TypeValue, src: Reference, i: Value, j: Value) {
    var i_ind: int;
    var j_ind: int;
    var v: Value;
    assume is#Integer(i);
    assume is#Integer(j);
    i_ind := i#Integer(i);
    j_ind := i#Integer(j);
    v := Dereference(__m, src);
    assume is#Vector(v);
    assume IsValidReferenceParameter(__m, __local_counter, src);
    if (i_ind >= vlen(v) || j_ind >= vlen(v) || i_ind < 0 || j_ind < 0) {
        __abort_flag := true;
        return;
    }
    v := swap_vector(v, i_ind, j_ind);
    call WriteRef(src, v);
}

procedure {:inline 1} Vector_get(ta: TypeValue, src: Reference, i: Value) returns (e: Value) {
    var i_ind: int;
    var v: Value;

    assume is#Integer(i);
    i_ind := i#Integer(i);
    v := Dereference(__m, src);
    assume is#Vector(v);
    assume IsValidReferenceParameter(__m, __local_counter, src);
    if (i_ind < 0 || i_ind >= vlen(v)) {
        __abort_flag := true;
        return;
    }
    e := vmap(v)[i_ind];
}

procedure {:inline 1} Vector_set(ta: TypeValue, src: Reference, i: Value, e: Value) {
    var i_ind: int;
    var v: Value;

    assume is#Integer(i);
    i_ind := i#Integer(i);
    v := Dereference(__m, src);
    assume is#Vector(v);
    assume IsValidReferenceParameter(__m, __local_counter, src);
    if (i_ind < 0 || i_ind >= vlen(v)) {
        __abort_flag := true;
        return;
    }
    v := update_vector(v, i_ind, e);
    call WriteRef(src, v);
}
