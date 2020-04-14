// ================================================================================
// Domains

// Debug tracking
// --------------

// Debug tracking is used to inject information used for model analysis. The generated code emits statements
// like this:
//
//     assume $DebugTrackLocal(file_id, byte_index, var_idx, value);
//
// While those tracking assumptions are trivially true for the provers logic, the solver (at least Z3)
// will construct a function mapping which appears in the model, e.g.:
//
//     $DebugTrackLocal -> {
//         1 440 0 (Vector (ValueArray |T@[Int]Value!val!0| 0)) -> true
//         1 533 1 (Integer 1) -> true
//         ...
//         else -> true
//     }
//
// This information can then be read out from the model.


// Tracks debug information for a parameter, local or a return parameter. Return parameter indices start at
// the overall number of locals (including parameters).
function $DebugTrackLocal(file_id: int, byte_index:  int, var_idx: int, value: Value) : bool {
  true
}

// Tracks at which location a function was aborted.
function $DebugTrackAbort(file_id: int, byte_index: int) : bool {
  true
}

// Path type
// ---------

type Edge = int; // both FieldName and vector index are mapped to int

type {:datatype} Path;
function {:constructor} Path(p: [int]Edge, size: int): Path;
const EmptyPath: Path;
axiom size#Path(EmptyPath) == 0;

function {:inline} path_index_at(p: Path, i: int): int {
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

function {:inline} ExtendTypeValueArray(ta: TypeValueArray, tv: TypeValue): TypeValueArray {
    TypeValueArray(v#TypeValueArray(ta)[l#TypeValueArray(ta) := tv], l#TypeValueArray(ta) + 1)
}


// Values
// ------

type ByteArray;
type String;
type {:datatype} Value;

const MAX_U8: int;
axiom MAX_U8 == 255;
const MAX_U64: int;
axiom MAX_U64 == 18446744073709551615;
const MAX_U128: int;
axiom MAX_U128 == 340282366920938463463374607431768211455;

function {:constructor} Boolean(b: bool): Value;
function {:constructor} Integer(i: int): Value;
function {:constructor} Address(a: int): Value;
function {:constructor} ByteArray(b: ByteArray): Value;
function {:constructor} Str(a: String): Value;
function {:constructor} Vector(v: ValueArray): Value; // used to both represent move Struct and Vector
function {:constructor} $Range(lb: Value, ub: Value): Value;
function {:constructor} Error(): Value;
const DefaultValue: Value;
axiom DefaultValue == Error();
function {:builtin "MapConst"} MapConstValue(v: Value): [int]Value;

function {:inline} $IsValidU8(v: Value): bool {
  is#Integer(v) && i#Integer(v) >= 0 && i#Integer(v) <= MAX_U8
}

function {:inline} $IsValidU8Vector(vec: Value): bool {
  $Vector_is_well_formed(vec)
  && (forall i: int :: 0 <= i && i < $vlen(vec) ==> $IsValidU8($vmap(vec)[i]))
}

function {:inline} $IsValidU64(v: Value): bool {
  is#Integer(v) && i#Integer(v) >= 0 && i#Integer(v) <= MAX_U64
}

function {:inline} $IsValidU64Vector(vec: Value): bool {
  $Vector_is_well_formed(vec)
  && (forall i: int :: 0 <= i && i < $vlen(vec) ==> $IsValidU64($vmap(vec)[i]))
}


function {:inline} $IsValidU128(v: Value): bool {
  is#Integer(v) && i#Integer(v) >= 0 && i#Integer(v) <= MAX_U128
}

function {:inline} $IsValidU128Vector(vec: Value): bool {
  $Vector_is_well_formed(vec)
  && (forall i: int :: 0 <= i && i < $vlen(vec) ==> $IsValidU128($vmap(vec)[i]))
}

function {:inline} $IsValidNum(v: Value): bool {
  is#Integer(v)
}



// Value Array
// -----------

type {:datatype} ValueArray;

function {:constructor} ValueArray(v: [int]Value, l: int): ValueArray;
const EmptyValueArray: ValueArray;

axiom l#ValueArray(EmptyValueArray) == 0;
axiom v#ValueArray(EmptyValueArray) == MapConstValue(DefaultValue);

function {:inline} RemoveValueArray(a: ValueArray): ValueArray {
    ValueArray(v#ValueArray(a)[l#ValueArray(a) - 1 := DefaultValue], l#ValueArray(a) - 1)
}
function {:inline} RemoveIndexValueArray(a: ValueArray, i: int): ValueArray {
    ValueArray(
        (lambda j: int :: if j < i then v#ValueArray(a)[j] else v#ValueArray(a)[j+1]),
        l#ValueArray(a) - 1)
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
function {:inline} SliceValueArray(a: ValueArray, i: int, j: int): ValueArray { // return the sliced vector of a for the range [i, j)
    ValueArray((lambda k:int :: if 0 <= k && k < j-i then v#ValueArray(a)[i+k] else DefaultValue), (if j-i < 0 then 0 else j-i))
}
function {:inline} ExtendValueArray(a: ValueArray, elem: Value): ValueArray {
    ValueArray(v#ValueArray(a)[l#ValueArray(a) := elem], l#ValueArray(a) + 1)
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

// Stratified Functions on Values
// ------------------------------

// TODO: templatize this or move it back to the translator. For now we
//   prefer to handcode this so its easier to evolve the model independent of the
//   translator.

const StratificationDepth: int;
axiom StratificationDepth == 4;

function {:inline} IsEqual4(v1: Value, v2: Value): bool {
    v1 == v2
}
function {:inline} IsEqual3(v1: Value, v2: Value): bool {
    (v1 == v2) ||
    (is#Vector(v1) &&
     is#Vector(v2) &&
     $vlen(v1) == $vlen(v2) &&
     (forall i: int :: 0 <= i && i < $vlen(v1) ==> IsEqual4($vmap(v1)[i], $vmap(v2)[i])))
}
function {:inline} IsEqual2(v1: Value, v2: Value): bool {
    (v1 == v2) ||
    (is#Vector(v1) &&
     is#Vector(v2) &&
     $vlen(v1) == $vlen(v2) &&
     (forall i: int :: 0 <= i && i < $vlen(v1) ==> IsEqual3($vmap(v1)[i], $vmap(v2)[i])))
}
function {:inline} IsEqual1(v1: Value, v2: Value): bool {
    (v1 == v2) ||
    (is#Vector(v1) &&
     is#Vector(v2) &&
     $vlen(v1) == $vlen(v2) &&
     (forall i: int :: 0 <= i && i < $vlen(v1) ==> IsEqual2($vmap(v1)[i], $vmap(v2)[i])))
}
function {:inline} IsEqual(v1: Value, v2: Value): bool {
    IsEqual1(v1, v2)
}

function {:inline} $ReadValue4(p: Path, v: Value): Value {
    v
}
function {:inline} $ReadValue3(p: Path, v: Value) : Value {
    if (3 == size#Path(p)) then
        v
    else
        $ReadValue4(p, $vmap(v)[path_index_at(p, 3)])
}
function {:inline} $ReadValue2(p: Path, v: Value) : Value {
    if (2 == size#Path(p)) then
        v
    else
        $ReadValue3(p, $vmap(v)[path_index_at(p, 2)])
}
function {:inline} $ReadValue1(p: Path, v: Value) : Value {
    if (1 == size#Path(p)) then
        v
    else
        $ReadValue2(p, $vmap(v)[path_index_at(p, 1)])
}
function {:inline} $ReadValue0(p: Path, v: Value) : Value {
    if (0 == size#Path(p)) then
        v
    else
        $ReadValue1(p, $vmap(v)[path_index_at(p, 0)])
}
function {:inline} $ReadValue(p: Path, v: Value): Value {
    $ReadValue0(p, v)
}

function {:inline} UpdateValue4(p: Path, v: Value, new_v: Value): Value {
    new_v
}
function {:inline} UpdateValue3(p: Path, v: Value, new_v: Value): Value {
    if (3 == size#Path(p)) then
        new_v
    else
        $update_vector(v, path_index_at(p, 3), UpdateValue4(p, $vmap(v)[path_index_at(p, 3)], new_v))
}
function {:inline} UpdateValue2(p: Path, v: Value, new_v: Value): Value {
    if (2 == size#Path(p)) then
        new_v
    else
        $update_vector(v, path_index_at(p, 2), UpdateValue3(p, $vmap(v)[path_index_at(p, 2)], new_v))
}
function {:inline} UpdateValue1(p: Path, v: Value, new_v: Value): Value {
    if (1 == size#Path(p)) then
        new_v
    else
        $update_vector(v, path_index_at(p, 1), UpdateValue2(p, $vmap(v)[path_index_at(p, 1)], new_v))
}
function {:inline} UpdateValue0(p: Path, v: Value, new_v: Value): Value {
    if (0 == size#Path(p)) then
        new_v
    else
        $update_vector(v, path_index_at(p, 0), UpdateValue1(p, $vmap(v)[path_index_at(p, 0)], new_v))
}
function {:inline} UpdateValue(p: Path, v: Value, new_v: Value): Value {
    UpdateValue0(p, v, new_v)
}

// Vector related functions on values
// ----------------------------------

function {:inline} $vmap(v: Value): [int]Value {
    v#ValueArray(v#Vector(v))
}
function {:inline} $vlen(v: Value): int {
    l#ValueArray(v#Vector(v))
}

// All invalid elements of array are DefaultValue. This is useful in specialized
// cases
function {:inline} IsNormalizedMap(va: [int]Value, len: int): bool {
    (forall i: int :: i < 0 || i >= len ==> va[i] == DefaultValue)
}

// Check that all invalid elements of vector are DefaultValue
function {:inline} $is_normalized_vector(v: Value): bool {
    IsNormalizedMap($vmap(v), $vlen(v))
}

// Sometimes, we need the length as a Value, not an int.
function {:inline} $vlen_value(v: Value): Value {
    Integer($vlen(v))
}
function {:inline} $mk_vector(): Value {
    Vector(EmptyValueArray)
}
function {:inline} $push_back_vector(v: Value, elem: Value): Value {
    Vector(ExtendValueArray(v#Vector(v), elem))
}
function {:inline} $pop_back_vector(v: Value): Value {
    Vector(RemoveValueArray(v#Vector(v)))
}
function {:inline} $append_vector(v1: Value, v2: Value): Value {
    Vector(ConcatValueArray(v#Vector(v1), v#Vector(v2)))
}
function {:inline} $reverse_vector(v: Value): Value {
    Vector(ReverseValueArray(v#Vector(v)))
}
function {:inline} $update_vector(v: Value, i: int, elem: Value): Value {
    Vector(UpdateValueArray(v#Vector(v), i, elem))
}
// $update_vector_by_value requires index to be a Value, not int.
function {:inline} $update_vector_by_value(v: Value, i: Value, elem: Value): Value {
    Vector(UpdateValueArray(v#Vector(v), i#Integer(i), elem))
}
function {:inline} $select_vector(v: Value, i: int) : Value {
    $vmap(v)[i]
}
// $select_vector_by_value requires index to be a Value, not int.
function {:inline} $select_vector_by_value(v: Value, i: Value) : Value {
    $vmap(v)[i#Integer(i)]
}
function {:inline} $swap_vector(v: Value, i: int, j: int): Value {
    Vector(SwapValueArray(v#Vector(v), i, j))
}
function {:inline} $slice_vector(v: Value, r: Value) : Value {
    Vector(SliceValueArray(v#Vector(v), i#Integer(lb#$Range(r)), i#Integer(ub#$Range(r))))
}
function {:inline} $InVectorRange(v: Value, i: int): bool {
    i >= 0 && i < $vlen(v)
}
function {:inline} $remove_vector(v: Value, i:int): Value {
    Vector(RemoveIndexValueArray(v#Vector(v), i))
}
function {:inline} $contains_vector(v: Value, e: Value): bool {
    (exists i:int :: 0<=i && i<$vlen(v) && IsEqual($vmap(v)[i], e))
}

function {:inline} $InRange(r: Value, i: int): bool {
   i#Integer(lb#$Range(r)) <= i && i < i#Integer(ub#$Range(r))
}


// ============================================================================================
// Memory

type {:datatype} Location;
function {:constructor} Global(t: TypeValue, a: int): Location;
function {:constructor} Local(i: int): Location;
function {:constructor} Param(i: int): Location;

type {:datatype} Reference;
function {:constructor} Reference(l: Location, p: Path): Reference;
const DefaultReference: Reference;

type {:datatype} Memory;
function {:constructor} Memory(domain: [Location]bool, contents: [Location]Value): Memory;

function {:builtin "MapConst"} ConstMemoryDomain(v: bool): [Location]bool;
function {:builtin "MapConst"} ConstMemoryContent(v: Value): [Location]Value;

const EmptyMemory: Memory;
axiom domain#Memory(EmptyMemory) == ConstMemoryDomain(false);
axiom contents#Memory(EmptyMemory) == ConstMemoryContent(DefaultValue);

var $m : Memory;
var $local_counter : int;
var $abort_flag: bool;

function {:inline} $GetLocal(m: Memory, idx: int): Value {
   contents#Memory(m)[Local(idx)]
}

function {:inline} $UpdateLocal(m: Memory, idx: int, v: Value): Memory {
    Memory(domain#Memory(m)[Local(idx) := true], contents#Memory(m)[Local(idx) := v])
}

procedure {:inline 1} $InitVerification() {
  // Set local counter to 0
  $local_counter := 0;
  // Assume sender account exists.
  assume $ExistsTxnSenderAccount($m, $txn);
}

// ============================================================================================
// Specifications

// TODO: unify some of this with instruction procedures to avoid duplication

// Tests whether resource exists.
function {:inline} $ResourceExistsRaw(m: Memory, resource: TypeValue, addr: int): bool {
    domain#Memory(m)[Global(resource, addr)]
}
function {:inline} $ResourceExists(m: Memory, resource: TypeValue, address: Value): Value {
    Boolean($ResourceExistsRaw(m, resource, a#Address(address)))
}

// Obtains reference to the given resource.
function {:inline} $GetResourceReference(resource: TypeValue, addr: int): Reference {
    Reference(Global(resource, addr), EmptyPath)
}

// Obtains value of given resource.
function {:inline} $ResourceValue(m: Memory, resource: TypeValue, address: Value): Value {
  contents#Memory(m)[Global(resource, a#Address(address))]
}

// Applies a field selection to a value.
function {:inline} $SelectField(val: Value, field: FieldName): Value {
    $vmap(val)[field]
}

// Dereferences a reference.
function {:inline} $Dereference(m: Memory, ref: Reference): Value {
    $ReadValue(p#Reference(ref), contents#Memory(m)[l#Reference(ref)])
}

// Check whether sender account exists.
function {:inline} $ExistsTxnSenderAccount(m: Memory, txn: Transaction): bool {
   domain#Memory(m)[Global(LibraAccount_T_type_value(), sender#Transaction(txn))]
}

function {:inline} $TxnSender(txn: Transaction): Value {
    Address(sender#Transaction(txn))
}

// Forward declaration of type value of LibraAccount. This is declared so we can define
// ExistsTxnSenderAccount.
function LibraAccount_T_type_value(): TypeValue;

// Returns sender address.
function {:inline} TxnSenderAddress(txn: Transaction): int {
  sender#Transaction(txn)
}


// ============================================================================================
// Instructions

procedure {:inline 1} Exists(address: Value, t: TypeValue) returns (dst: Value)
requires is#Address(address);
{
    dst := $ResourceExists($m, t, address);
}

procedure {:inline 1} MoveToSender(ta: TypeValue, v: Value)
{
    var a: int;
    var l: Location;

    a := sender#Transaction($txn);
    l := Global(ta, a);
    if ($ResourceExistsRaw($m, ta, a)) {
        $abort_flag := true;
        return;
    }
    $m := Memory(domain#Memory($m)[l := true], contents#Memory($m)[l := v]);
}

procedure {:inline 1} MoveFrom(address: Value, ta: TypeValue) returns (dst: Value)
requires is#Address(address);
{
    var a: int;
    var l: Location;
    a := a#Address(address);
    l := Global(ta, a);
    if (!$ResourceExistsRaw($m, ta, a)) {
        $abort_flag := true;
        return;
    }
    dst := contents#Memory($m)[l];
    $m := Memory(domain#Memory($m)[l := false], contents#Memory($m)[l := DefaultValue]);
}

procedure {:inline 1} BorrowGlobal(address: Value, ta: TypeValue) returns (dst: Reference)
requires is#Address(address);
{
    var a: int;
    var l: Location;
    a := a#Address(address);
    l := Global(ta, a);
    if (!$ResourceExistsRaw($m, ta, a)) {
        $abort_flag := true;
        return;
    }
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
    p := Path(p#Path(p)[size := f], size+1);
    dst := Reference(l#Reference(src), p);
}

procedure {:inline 1} WriteRef(to: Reference, new_v: Value)
{
    var l: Location;
    var v: Value;

    l := l#Reference(to);
    v := contents#Memory($m)[l];
    v := UpdateValue(p#Reference(to), v, new_v);
    $m := Memory(domain#Memory($m), contents#Memory($m)[l := v]);
}

procedure {:inline 1} ReadRef(from: Reference) returns (v: Value)
{
    v := $ReadValue(p#Reference(from), contents#Memory($m)[l#Reference(from)]);
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
requires is#Integer(src);
{
    if (i#Integer(src) > MAX_U8) {
        $abort_flag := true;
        return;
    }
    dst := src;
}

procedure {:inline 1} CastU64(src: Value) returns (dst: Value)
requires is#Integer(src);
{
    if (i#Integer(src) > MAX_U64) {
        $abort_flag := true;
        return;
    }
    dst := src;
}

procedure {:inline 1} CastU128(src: Value) returns (dst: Value)
requires is#Integer(src);
{
    if (i#Integer(src) > MAX_U128) {
        $abort_flag := true;
        return;
    }
    dst := src;
}

procedure {:inline 1} AddU8(src1: Value, src2: Value) returns (dst: Value)
requires $IsValidU8(src1) && $IsValidU8(src2);
{
    if (i#Integer(src1) + i#Integer(src2) > MAX_U8) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) + i#Integer(src2));
}

procedure {:inline 1} AddU64(src1: Value, src2: Value) returns (dst: Value)
requires $IsValidU64(src1) && $IsValidU64(src2);
{
    if (i#Integer(src1) + i#Integer(src2) > MAX_U64) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) + i#Integer(src2));
}

procedure {:inline 1} AddU128(src1: Value, src2: Value) returns (dst: Value)
requires $IsValidU128(src1) && $IsValidU128(src2);
{
    if (i#Integer(src1) + i#Integer(src2) > MAX_U128) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) + i#Integer(src2));
}

procedure {:inline 1} Sub(src1: Value, src2: Value) returns (dst: Value)
requires is#Integer(src1) && is#Integer(src2);
{
    if (i#Integer(src1) < i#Integer(src2)) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) - i#Integer(src2));
}

procedure {:inline 1} Shl(src1: Value, src2: Value) returns (dst: Value)
requires is#Integer(src1) && is#Integer(src2);
{
    // TOOD: implement
    assert false;
}

procedure {:inline 1} Shr(src1: Value, src2: Value) returns (dst: Value)
requires is#Integer(src1) && is#Integer(src2);
{
    // TOOD: implement
    assert false;
}

procedure {:inline 1} MulU8(src1: Value, src2: Value) returns (dst: Value)
requires $IsValidU8(src1) && $IsValidU8(src2);
{
    if (i#Integer(src1) * i#Integer(src2) > MAX_U8) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) * i#Integer(src2));
}

procedure {:inline 1} MulU64(src1: Value, src2: Value) returns (dst: Value)
requires $IsValidU64(src1) && $IsValidU64(src2);
{
    if (i#Integer(src1) * i#Integer(src2) > MAX_U64) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) * i#Integer(src2));
}

procedure {:inline 1} MulU128(src1: Value, src2: Value) returns (dst: Value)
requires $IsValidU128(src1) && $IsValidU128(src2);
{
    if (i#Integer(src1) * i#Integer(src2) > MAX_U128) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) * i#Integer(src2));
}

procedure {:inline 1} Div(src1: Value, src2: Value) returns (dst: Value)
requires is#Integer(src1) && is#Integer(src2);
{
    if (i#Integer(src2) == 0) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) div i#Integer(src2));
}

procedure {:inline 1} Mod(src1: Value, src2: Value) returns (dst: Value)
requires is#Integer(src1) && is#Integer(src2);
{
    if (i#Integer(src2) == 0) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) mod i#Integer(src2));
}

procedure {:inline 1} ArithBinaryUnimplemented(src1: Value, src2: Value) returns (dst: Value);
requires is#Integer(src1) && is#Integer(src2);
ensures is#Integer(dst);

procedure {:inline 1} Lt(src1: Value, src2: Value) returns (dst: Value)
requires is#Integer(src1) && is#Integer(src2);
{
    dst := Boolean(i#Integer(src1) < i#Integer(src2));
}

procedure {:inline 1} Gt(src1: Value, src2: Value) returns (dst: Value)
requires is#Integer(src1) && is#Integer(src2);
{
    dst := Boolean(i#Integer(src1) > i#Integer(src2));
}

procedure {:inline 1} Le(src1: Value, src2: Value) returns (dst: Value)
requires is#Integer(src1) && is#Integer(src2);
{
    dst := Boolean(i#Integer(src1) <= i#Integer(src2));
}

procedure {:inline 1} Ge(src1: Value, src2: Value) returns (dst: Value)
requires is#Integer(src1) && is#Integer(src2);
{
    dst := Boolean(i#Integer(src1) >= i#Integer(src2));
}

procedure {:inline 1} And(src1: Value, src2: Value) returns (dst: Value)
requires is#Boolean(src1) && is#Boolean(src2);
{
    dst := Boolean(b#Boolean(src1) && b#Boolean(src2));
}

procedure {:inline 1} Or(src1: Value, src2: Value) returns (dst: Value)
requires is#Boolean(src1) && is#Boolean(src2);
{
    dst := Boolean(b#Boolean(src1) || b#Boolean(src2));
}

procedure {:inline 1} Not(src: Value) returns (dst: Value)
requires is#Boolean(src);
{
    dst := Boolean(!b#Boolean(src));
}

// Pack and Unpack are auto-generated for each type T


// Transaction
// -----------

type {:datatype} Transaction;
var $txn: Transaction;
function {:constructor} Transaction(
  gas_unit_price: int, max_gas_units: int, public_key: ByteArray,
  sender: int, sequence_number: int, gas_remaining: int) : Transaction;


const some_key: ByteArray;


// ==================================================================================
// Native Vector Type

function {:inline} $Vector_type_value(tv: TypeValue): TypeValue {
    VectorType(tv)
}

function {:inline} $Vector_is_well_formed(v: Value): bool {
    is#Vector(v) && $vlen(v) >= 0 &&
    (
        var va := v#Vector(v);
        0 <= l#ValueArray(va) &&
        (forall x: int :: (0 <= x && x < l#ValueArray(va)) || v#ValueArray(va)[x] == DefaultValue)
    )
}

procedure {:inline 1} $Vector_empty(ta: TypeValue) returns (v: Value) {
    v := $mk_vector();
}

procedure {:inline 1} $Vector_is_empty(ta: TypeValue, r: Reference) returns (b: Value) {
    var v: Value;
    v := $Dereference($m, r);
    assume is#Vector(v);
    b := Boolean($vlen(v) == 0);
}

procedure {:inline 1} $Vector_push_back(ta: TypeValue, r: Reference, val: Value) {
    var v: Value;
    v := $Dereference($m, r);
    assume is#Vector(v);
    call WriteRef(r, $push_back_vector(v, val));
}

procedure {:inline 1} $Vector_pop_back(ta: TypeValue, r: Reference) returns (e: Value) {
    var v: Value;
    var len: int;
    v := $Dereference($m, r);
    assume is#Vector(v);
    len := $vlen(v);
    if (len == 0) {
        $abort_flag := true;
        return;
    }
    e := $vmap(v)[len-1];
    call WriteRef(r, $pop_back_vector(v));
}

procedure {:inline 1} $Vector_append(ta: TypeValue, r: Reference, other: Value) {
    var v: Value;
    v := $Dereference($m, r);
    assume is#Vector(v);
    assume is#Vector(other);
    call WriteRef(r, $append_vector(v, other));
}

procedure {:inline 1} $Vector_reverse(ta: TypeValue, r: Reference) {
    var v: Value;
    v := $Dereference($m, r);
    assume is#Vector(v);
    call WriteRef(r, $reverse_vector(v));
}

procedure {:inline 1} $Vector_length(ta: TypeValue, r: Reference) returns (l: Value) {
    var v: Value;
    v := $Dereference($m, r);
    assume is#Vector(v);
    l := Integer($vlen(v));
}

procedure {:inline 1} $Vector_borrow(ta: TypeValue, src: Reference, index: Value) returns (dst: Reference) {
    call dst := $Vector_borrow_mut(ta, src, index);
}

procedure {:inline 1} $Vector_borrow_mut(ta: TypeValue, src: Reference, index: Value) returns (dst: Reference)
requires is#Integer(index);
{
    var p: Path;
    var size: int;
    var i_ind: int;
    var v: Value;

    i_ind := i#Integer(index);
    v := $Dereference($m, src);
    assume is#Vector(v);
    if (i_ind < 0 || i_ind >= $vlen(v)) {
        $abort_flag := true;
        return;
    }

    p := p#Reference(src);
    size := size#Path(p);
    p := Path(p#Path(p)[size := i_ind], size+1);
    dst := Reference(l#Reference(src), p);
}

procedure {:inline 1} $Vector_destroy_empty(ta: TypeValue, v: Value) {
    if ($vlen(v) != 0) {
      $abort_flag := true;
    }
}

procedure {:inline 1} $Vector_swap(ta: TypeValue, src: Reference, i: Value, j: Value)
requires is#Integer(i) && is#Integer(j);
{
    var i_ind: int;
    var j_ind: int;
    var v: Value;
    i_ind := i#Integer(i);
    j_ind := i#Integer(j);
    v := $Dereference($m, src);
    assume is#Vector(v);
    if (i_ind >= $vlen(v) || j_ind >= $vlen(v) || i_ind < 0 || j_ind < 0) {
        $abort_flag := true;
        return;
    }
    v := $swap_vector(v, i_ind, j_ind);
    call WriteRef(src, v);
}

procedure {:inline 1} $Vector_get(ta: TypeValue, src: Reference, i: Value) returns (e: Value)
requires is#Integer(i);
{
    var i_ind: int;
    var v: Value;

    i_ind := i#Integer(i);
    v := $Dereference($m, src);
    assume is#Vector(v);
    if (i_ind < 0 || i_ind >= $vlen(v)) {
        $abort_flag := true;
        return;
    }
    e := $vmap(v)[i_ind];
}

procedure {:inline 1} $Vector_set(ta: TypeValue, src: Reference, i: Value, e: Value)
requires is#Integer(i);
{
    var i_ind: int;
    var v: Value;

    i_ind := i#Integer(i);
    v := $Dereference($m, src);
    assume is#Vector(v);
    if (i_ind < 0 || i_ind >= $vlen(v)) {
        $abort_flag := true;
        return;
    }
    v := $update_vector(v, i_ind, e);
    call WriteRef(src, v);
}

procedure {:inline 1} $Vector_remove(ta: TypeValue, r: Reference, i: Value) returns (e: Value)
requires is#Integer(i);
{
    var i_ind: int;
    var v: Value;

    i_ind := i#Integer(i);

    v := $Dereference($m, r);
    assume is#Vector(v);
    if (i_ind < 0 || i_ind >= $vlen(v)) {
        $abort_flag := true;
        return;
    }
    e := $vmap(v)[i_ind];
    call WriteRef(r, $remove_vector(v, i_ind));
}

procedure {:inline 1} $Vector_swap_remove(ta: TypeValue, r: Reference, i: Value) returns (e: Value)
requires is#Integer(i);
{
    var i_ind: int;
    var v: Value;
    var len: int;

    i_ind := i#Integer(i);

    v := $Dereference($m, r);
    assume is#Vector(v);
    len := $vlen(v);
    if (i_ind < 0 || i_ind >= len) {
        $abort_flag := true;
        return;
    }
    e := $vmap(v)[i_ind];
    call WriteRef(r, $pop_back_vector($swap_vector(v, i_ind, len-1)));
}

procedure {:inline 1} $Vector_contains(ta: TypeValue, vr: Reference, er: Reference) returns (res: Value)  {
    var v: Value;
    var e: Value;

    v := $Dereference($m, vr);
    e := $Dereference($m, er);
    assume is#Vector(v);

    res := Boolean($contains_vector(v, e));
}



// ==================================================================================
// Native hash

// Hash is modeled as an otherwise uninterpreted injection.
// In truth, it is not an injection since the domain has greater cardinality
// (arbitrary length vectors) than the co-domain (vectors of length 32).  But it is
// common to assume in code there are no hash collisions in practice.  Fortunately,
// Boogie is not smart enough to recognized that there is an inconsistency.
// FIXME: If we were using a reliable extensional theory of arrays, and if we could use ==
// instead of IsEqual, we might be able to avoid so many quantified formulas by
// using a sha2_inverse function in the ensures conditions of Hash_sha2_256 to
// assert that sha2/3 are injections without using global quantified axioms.

function $Hash_sha2(val: Value) : Value;

// This says that Hash_sha2 respects isEquals (this would be automatic if we had an
// extensional theory of arrays and used ==, which has the substitution property
// for functions).
axiom (forall v1,v2: Value :: $Vector_is_well_formed(v1) && $Vector_is_well_formed(v2)
       && IsEqual(v1, v2) ==> IsEqual($Hash_sha2(v1), $Hash_sha2(v2)));

// This says that Hash_sha2 is an injection
axiom (forall v1,v2: Value :: $Vector_is_well_formed(v1) && $Vector_is_well_formed(v2)
        && IsEqual($Hash_sha2(v1), $Hash_sha2(v2)) ==> IsEqual(v1, v2));

// This procedure has no body. We want Boogie to just use its requires
// and ensures properties when verifying code that calls it.
procedure $Hash_sha2_256(val: Value) returns (res: Value);
// It will still work without this, but this helps verifier find more reasonable counterexamples.
// requires $IsValidU8Vector(val);  // FIXME: Generated calling code does not ensure validity.
ensures res == $Hash_sha2(val);     // returns Hash_sha2 value
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.
ensures $vlen(res) == 32;               // result is 32 bytes.

// similarly for Hash_sha3
function $Hash_sha3(val: Value) : Value;

axiom (forall v1,v2: Value :: $Vector_is_well_formed(v1) && $Vector_is_well_formed(v2)
       && IsEqual(v1, v2) ==> IsEqual($Hash_sha3(v1), $Hash_sha3(v2)));

axiom (forall v1,v2: Value :: $Vector_is_well_formed(v1) && $Vector_is_well_formed(v2)
        && IsEqual($Hash_sha3(v1), $Hash_sha3(v2)) ==> IsEqual(v1, v2));

procedure $Hash_sha3_256(val: Value) returns (res: Value);
ensures res == $Hash_sha3(val);     // returns Hash_sha3 value
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.
ensures $vlen(res) == 32;               // result is 32 bytes.

// ==================================================================================
// Native libra_account

// TODO: implement the below methods

procedure {:inline 1} $LibraAccount_save_account(ta: TypeValue, balance: Value, account: Value, addr: Value) {
    assert false; // $LibraAccount_save_account not implemented
}

procedure {:inline 1} $LibraAccount_write_to_event_store(ta: TypeValue, guid: Value, count: Value, msg: Value) {
    // This function is modeled as a no-op because the actual side effect of this native function is not observable from the Move side.
}

// ==================================================================================
// Native lcs

// TODO: implement the below methods

procedure {:inline 1} $Signature_ed25519_verify(signature: Value, public_key: Value, message: Value) returns (res: Value) {
    assert false; // $Signature_ed25519_verify not implemented
}

procedure {:inline 1} Signature_ed25519_threshold_verify(bitmap: Value, signature: Value, public_key: Value, message: Value) returns (res: Value) {
    assert false; // Signature_ed25519_threshold_verify not implemented
}

// ==================================================================================
// Native signature

// TODO: implement the below methods

// ==================================================================================
// Native LCS::serialize

// native public fun serialize<MoveValue>(v: &MoveValue): vector<u8>;

// Serialize is modeled as an uninterpreted function, with an additional
// axiom to say it's an injection.

function {:inline} $LCS_serialize($m: Memory, ta: TypeValue, v: Value): Value {
    $LCS_serialize_core(ta, v)
}

function $LCS_serialize_core(ta: TypeValue, v: Value): Value;

// This says that $serialize respects isEquals (substitution property)
// Without this, Boogie will get false positives where v1, v2 differ at invalid
// indices.
axiom (forall ta: TypeValue ::
       (forall v1,v2: Value :: IsEqual(v1, v2) ==> IsEqual($LCS_serialize_core(ta, v1), $LCS_serialize_core(ta, v2))));


// This says that serialize is an injection
axiom (forall ta1, ta2: TypeValue ::
       (forall v1, v2: Value :: IsEqual($LCS_serialize_core(ta1, v1), $LCS_serialize_core(ta2, v2))
           ==> IsEqual(v1, v2) && (ta1 == ta2)));

procedure $LCS_to_bytes(ta: TypeValue, r: Reference) returns (res: Value);
ensures res == $LCS_serialize_core(ta, $Dereference($m, r));
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.
ensures $vlen(res) > 0;
