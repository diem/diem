// ================================================================================
// Notation

// This files contains a Handlebars Rust template for the prover's Boogie prelude.
// The template language constructs allow the prelude to adjust the actual content
// to multiple options. We only use a few selected template constructs which are
// mostly self-explaining. See the handlebars crate documentation for more information.
//
// Currently, we use the following symbols from the template evaluation context (if you add any
// new ones, please document them here):
//
// - `native_equality: bool`: whether we should generate native instead of stratified
//   equality.
// - `type_requires: string`: expands either to `requires` or `free requires`.
//   This determines how we check argument types of procedures. With `free requires`
//   we assume type correctness, with `requires` we double check this.
// - `stratification_depth: int`: to which depth we generate stratified functions.
//   We use a custom helper #stratified which uses this information as well; see
//   usage below.
// - `aggressive_func_inline`: a string which either expands to `{:inline}` or empty.
//   This is used to mark larger functions for inlining.
// - `func_inline`: a string which expands to `{:inline}` or empty.
//   This is used to mark smaller (but non-trivial) functions for inlining.

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

type {:datatype} Path;
function {:constructor} Path(p: [int]int, size: int): Path;
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
function {:constructor} StrType() : TypeValue;
function {:constructor} VectorType(t: TypeValue) : TypeValue;
function {:constructor} StructType(name: TypeName, ps: TypeValueArray, ts: TypeValueArray) : TypeValue;
function {:constructor} $TypeType(): TypeValue;
function {:constructor} ErrorType() : TypeValue;

function {:inline} DefaultTypeValue() : TypeValue { ErrorType() }
function {:builtin "MapConst"} MapConstTypeValue(tv: TypeValue): [int]TypeValue;

type {:datatype} TypeValueArray;
function {:constructor} TypeValueArray(v: [int]TypeValue, l: int): TypeValueArray;
const EmptyTypeValueArray: TypeValueArray;
axiom l#TypeValueArray(EmptyTypeValueArray) == 0;
axiom v#TypeValueArray(EmptyTypeValueArray) == MapConstTypeValue(DefaultTypeValue());


// Values
// ------

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
function {:constructor} Vector(v: ValueArray): Value; // used to both represent move Struct and Vector
function {:constructor} $Range(lb: Value, ub: Value): Value;
function {:constructor} $Type(t: TypeValue): Value;
function {:constructor} Error(): Value;

function {:inline} DefaultValue(): Value { Error() }
function {:builtin "MapConst"} MapConstValue(v: Value): [int]Value;

function {:inline} $IsValidU8(v: Value): bool {
  is#Integer(v) && i#Integer(v) >= 0 && i#Integer(v) <= MAX_U8
}

function {:inline} $IsValidU8Vector(vec: Value): bool {
  $Vector_is_well_formed(vec)
  && (forall i: int :: {$vmap(vec)[i]} 0 <= i && i < $vlen(vec) ==> $IsValidU8($vmap(vec)[i]))
}

function {:inline} $IsValidU64(v: Value): bool {
  is#Integer(v) && i#Integer(v) >= 0 && i#Integer(v) <= MAX_U64
}

function {:inline} $IsValidU128(v: Value): bool {
  is#Integer(v) && i#Integer(v) >= 0 && i#Integer(v) <= MAX_U128
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
axiom v#ValueArray(EmptyValueArray) == MapConstValue(Error());

function {{func_inline}} RemoveValueArray(a: ValueArray): ValueArray {
    (
        var l := l#ValueArray(a) - 1;
        ValueArray(
            (lambda i: int ::
                if i >= 0 && i < l then v#ValueArray(a)[i] else DefaultValue()),
            l
        )
    )
}
function {{func_inline}} RemoveIndexValueArray(a: ValueArray, i: int): ValueArray {
    (
        var l := l#ValueArray(a) - 1;
        ValueArray(
            (lambda j: int ::
                if j >= 0 && j < l then
                    if j < i then v#ValueArray(a)[j] else v#ValueArray(a)[j+1]
                else DefaultValue()),
            l
        )
    )
}
function {{func_inline}} ConcatValueArray(a1: ValueArray, a2: ValueArray): ValueArray {
    (
        var l1, l2 := l#ValueArray(a1), l#ValueArray(a2);
        ValueArray(
            (lambda i: int ::
                if i >= 0 && i < l1 + l2 then
                    if i < l1 then v#ValueArray(a1)[i] else v#ValueArray(a2)[i - l1]
                else
                    DefaultValue()),
            l1 + l2)
    )
}
function {{func_inline}} ReverseValueArray(a: ValueArray): ValueArray {
    (
        var l := l#ValueArray(a);
        ValueArray(
            (lambda i: int :: if 0 <= i && i < l then v#ValueArray(a)[l - i - 1] else DefaultValue()),
            l
        )
    )
}
function {{func_inline}} SliceValueArray(a: ValueArray, i: int, j: int): ValueArray { // return the sliced vector of a for the range [i, j)
    ValueArray((lambda k:int :: if 0 <= k && k < j-i then v#ValueArray(a)[i+k] else DefaultValue()), (if j-i < 0 then 0 else j-i))
}
function {{func_inline}} ExtendValueArray(a: ValueArray, elem: Value): ValueArray {
    (var len := l#ValueArray(a);
     ValueArray(v#ValueArray(a)[len := elem], len + 1))
}
function {{func_inline}} UpdateValueArray(a: ValueArray, i: int, elem: Value): ValueArray {
    ValueArray(v#ValueArray(a)[i := elem], l#ValueArray(a))
}
function {{func_inline}} SwapValueArray(a: ValueArray, i: int, j: int): ValueArray {
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
axiom StratificationDepth == {{stratification_depth}};

{{#if native_equality}}

// Map IsEqual to native Boogie equality. This only works with extensional arrays as provided
// by the array theory.
function {:inline} IsEqual(v1: Value, v2: Value): bool {
    v1 == v2
}

{{else}}

// Generate a stratified version of IsEqual for depth of {{stratification_depth}}.
{{#stratified}}
function {{aggressive_func_inline}} IsEqual_{{@this_suffix}}(v1: Value, v2: Value): bool {
    (v1 == v2) ||
    (is#Vector(v1) &&
     is#Vector(v2) &&
     $vlen(v1) == $vlen(v2) &&
     (forall i: int :: 0 <= i && i < $vlen(v1) ==> IsEqual_{{@next_suffix}}($vmap(v1)[i], $vmap(v2)[i])))
}
{{else}}
function {:inline} IsEqual_{{@this_suffix}}(v1: Value, v2: Value): bool {
    v1 == v2
}
{{/stratified}}

function {:inline} IsEqual(v1: Value, v2: Value): bool {
    IsEqual_stratified(v1, v2)
}

{{/if}}

// Generate stratified ReadValue for the depth of {{stratification_depth}}.

{{#stratified}}
function {{aggressive_func_inline}} $ReadValue_{{@this_suffix}}(p: Path, v: Value) : Value {
    if ({{@this_level}} == size#Path(p)) then
        v
    else
        $ReadValue_{{@next_suffix}}(p, $vmap(v)[path_index_at(p, {{@this_level}})])
}
{{else}}
function {:inline} $ReadValue_{{@this_suffix}}(p: Path, v: Value): Value {
    v
}
{{/stratified}}

function {:inline} $ReadValue(p: Path, v: Value): Value {
    $ReadValue_stratified(p, v)
}

// Generate stratified $UpdateValue for the depth of {{stratification_depth}}.

{{#stratified}}
function {{aggressive_func_inline}} $UpdateValue_{{@this_suffix}}(p: Path, offset: int, v: Value, new_v: Value): Value {
    (var poffset := offset + {{@this_level}};
    if (poffset == size#Path(p)) then
        new_v
    else
        $update_vector(v, path_index_at(p, poffset),
                       $UpdateValue_{{@next_suffix}}(p, offset, $vmap(v)[path_index_at(p, poffset)], new_v)))
}
{{else}}
function {:inline} $UpdateValue_{{@this_suffix}}(p: Path, offset: int, v: Value, new_v: Value): Value {
    new_v
}
{{/stratified}}

function {:inline} $UpdateValue(p: Path, offset: int, v: Value, new_v: Value): Value {
    $UpdateValue_stratified(p, offset, v, new_v)
}

// Generate stratified $IsPathPrefix for the depth of {{stratification_depth}}.

{{#stratified}}
function {{aggressive_func_inline}} $IsPathPrefix_{{@this_suffix}}(p1: Path, p2: Path): bool {
    if ({{@this_level}} == size#Path(p1)) then
        true
    else if (p#Path(p1)[{{@this_level}}] == p#Path(p2)[{{@this_level}}]) then
        $IsPathPrefix_{{@next_suffix}}(p1, p2)
    else
        false
}
{{else}}
function {:inline} $IsPathPrefix_{{@this_suffix}}(p1: Path, p2: Path): bool {
    true
}
{{/stratified}}

function {:inline} $IsPathPrefix(p1: Path, p2: Path): bool {
    $IsPathPrefix_stratified(p1, p2)
}

// Generate stratified $ConcatPath for the depth of {{stratification_depth}}.

{{#stratified}}
function {{aggressive_func_inline}} $ConcatPath_{{@this_suffix}}(p1: Path, p2: Path): Path {
    if ({{@this_level}} == size#Path(p2)) then
        p1
    else
        $ConcatPath_{{@next_suffix}}(Path(p#Path(p1)[size#Path(p1) := p#Path(p2)[{{@this_level}}]], size#Path(p1) + 1), p2)
}
{{else}}
function {:inline} $ConcatPath_{{@this_suffix}}(p1: Path, p2: Path): Path {
    p1
}
{{/stratified}}

function {:inline} $ConcatPath(p1: Path, p2: Path): Path {
    $ConcatPath_stratified(p1, p2)
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
    (forall i: int :: i < 0 || i >= len ==> va[i] == DefaultValue())
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
function {:constructor} Reference(l: Location, p: Path, v: Value): Reference;
const DefaultReference: Reference;

type {:datatype} Memory;
function {:constructor} Memory(domain: [Location]bool, contents: [Location]Value): Memory;

function {:builtin "MapConst"} ConstMemoryDomain(v: bool): [Location]bool;
function {:builtin "MapConst"} ConstMemoryContent(v: Value): [Location]Value;

const EmptyMemory: Memory;
axiom domain#Memory(EmptyMemory) == ConstMemoryDomain(false);
axiom contents#Memory(EmptyMemory) == ConstMemoryContent(DefaultValue());

var $m: Memory;
var $abort_flag: bool;

procedure {:inline 1} $InitVerification() {
  // Set abort_flag to false
  $abort_flag := false;
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

// Obtains value of given resource.
function {:inline} $ResourceValue(m: Memory, resource: TypeValue, address: Value): Value {
  contents#Memory(m)[Global(resource, a#Address(address))]
}

// Applies a field selection to a value.
function {:inline} $SelectField(val: Value, field: FieldName): Value {
    $vmap(val)[field]
}

// Dereferences a reference.
function {:inline} $Dereference(ref: Reference): Value {
    v#Reference(ref)
}

// Check whether sender account exists.
function {:inline} $ExistsTxnSenderAccount(m: Memory, txn: Transaction): bool {
   domain#Memory(m)[Global($LibraAccount_T_type_value(), sender#Transaction(txn))]
}

function {:inline} $TxnSender(txn: Transaction): Value {
    Address(sender#Transaction(txn))
}

// Forward declaration of type value of LibraAccount. This is declared so we can define
// $ExistsTxnSenderAccount and $LibraAccount_save_account
function $LibraAccount_T_type_value(): TypeValue;

function $LibraAccount_Balance_type_value(tv: TypeValue): TypeValue;

// ============================================================================================
// Instructions

procedure {:inline 1} $Exists(address: Value, t: TypeValue) returns (dst: Value)
{{type_requires}} is#Address(address);
{
    dst := $ResourceExists($m, t, address);
}

procedure {:inline 1} $MoveToSender(ta: TypeValue, v: Value)
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

procedure {:inline 1} $MoveFrom(address: Value, ta: TypeValue) returns (dst: Value)
{{type_requires}} is#Address(address);
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
    $m := Memory(domain#Memory($m)[l := false], contents#Memory($m)[l := DefaultValue()]);
}

procedure {:inline 1} $BorrowGlobal(address: Value, ta: TypeValue) returns (dst: Reference)
{{type_requires}} is#Address(address);
{
    var a: int;
    var l: Location;
    a := a#Address(address);
    l := Global(ta, a);
    if (!$ResourceExistsRaw($m, ta, a)) {
        $abort_flag := true;
        return;
    }
    dst := Reference(l, EmptyPath, contents#Memory($m)[l]);
}

procedure {:inline 1} $BorrowLoc(l: int, v: Value) returns (dst: Reference)
{
    dst := Reference(Local(l), EmptyPath, v);
}

procedure {:inline 1} $BorrowField(src: Reference, f: FieldName) returns (dst: Reference)
{
    var p: Path;
    var size: int;

    p := p#Reference(src);
    size := size#Path(p);
    p := Path(p#Path(p)[size := f], size+1);
    dst := Reference(l#Reference(src), p, $vmap(v#Reference(src))[f]);
}

procedure {:inline 1} $GetGlobal(address: Value, ta: TypeValue) returns (dst: Value)
{{type_requires}} is#Address(address);
{
    var r: Reference;

    call r := $BorrowGlobal(address, ta);
    call dst := $ReadRef(r);
}

procedure {:inline 1} $GetFieldFromReference(src: Reference, f: FieldName) returns (dst: Value)
{
    var r: Reference;

    call r := $BorrowField(src, f);
    call dst := $ReadRef(r);
}

procedure {:inline 1} $GetFieldFromValue(src: Value, f: FieldName) returns (dst: Value)
{
    dst := $vmap(src)[f];
}

procedure {:inline 1} $WriteRef(to: Reference, new_v: Value) returns (to': Reference)
{
    to' := Reference(l#Reference(to), p#Reference(to), new_v);
}

procedure {:inline 1} $ReadRef(from: Reference) returns (v: Value)
{
    v := v#Reference(from);
}

procedure {:inline 1} $CopyOrMoveRef(local: Reference) returns (dst: Reference)
{
    dst := local;
}

procedure {:inline 1} $CopyOrMoveValue(local: Value) returns (dst: Value)
{
    dst := local;
}

procedure {:inline 1} WritebackToGlobal(src: Reference)
{
    var l: Location;
    var v: Value;

    l := l#Reference(src);
    if (is#Global(l)) {
        v := $UpdateValue(p#Reference(src), 0, contents#Memory($m)[l], v#Reference(src));
        $m := Memory(domain#Memory($m), contents#Memory($m)[l := v]);
    }
}

procedure {:inline 1} WritebackToValue(src: Reference, idx: int, vdst: Value) returns (vdst': Value)
{
    if (l#Reference(src) == Local(idx)) {
        vdst' := $UpdateValue(p#Reference(src), 0, vdst, v#Reference(src));
    } else {
        vdst' := vdst;
    }
}

procedure {:inline 1} WritebackToReference(src: Reference, dst: Reference) returns (dst': Reference)
{
    var srcPath, dstPath: Path;

    srcPath := p#Reference(src);
    dstPath := p#Reference(dst);
    if (l#Reference(dst) == l#Reference(src) && size#Path(dstPath) <= size#Path(srcPath) && $IsPathPrefix(dstPath, srcPath)) {
        dst' := Reference(
                    l#Reference(dst),
                    dstPath,
                    $UpdateValue(srcPath, size#Path(dstPath), v#Reference(dst), v#Reference(src)));
    } else {
        dst' := dst;
    }
}

procedure {:inline 1} Splice1(idx1: int, src1:Reference, dst: Reference) returns (dst': Reference) {
    assume l#Reference(dst) == Local(idx1);
    dst' := Reference(l#Reference(dst), $ConcatPath(p#Reference(src1), p#Reference(dst)), v#Reference(dst));
}

procedure {:inline 1} $CastU8(src: Value) returns (dst: Value)
{{type_requires}} is#Integer(src);
{
    if (i#Integer(src) > MAX_U8) {
        $abort_flag := true;
        return;
    }
    dst := src;
}

procedure {:inline 1} $CastU64(src: Value) returns (dst: Value)
{{type_requires}} is#Integer(src);
{
    if (i#Integer(src) > MAX_U64) {
        $abort_flag := true;
        return;
    }
    dst := src;
}

procedure {:inline 1} $CastU128(src: Value) returns (dst: Value)
{{type_requires}} is#Integer(src);
{
    if (i#Integer(src) > MAX_U128) {
        $abort_flag := true;
        return;
    }
    dst := src;
}

procedure {:inline 1} $AddU8(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} $IsValidU8(src1) && $IsValidU8(src2);
{
    if (i#Integer(src1) + i#Integer(src2) > MAX_U8) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) + i#Integer(src2));
}

procedure {:inline 1} $AddU64(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} $IsValidU64(src1) && $IsValidU64(src2);
{
    if (i#Integer(src1) + i#Integer(src2) > MAX_U64) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) + i#Integer(src2));
}

procedure {:inline 1} $AddU128(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} $IsValidU128(src1) && $IsValidU128(src2);
{
    if (i#Integer(src1) + i#Integer(src2) > MAX_U128) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) + i#Integer(src2));
}

procedure {:inline 1} $Sub(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} is#Integer(src1) && is#Integer(src2);
{
    if (i#Integer(src1) < i#Integer(src2)) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) - i#Integer(src2));
}

// This deals only with narrow special cases. Src2 must be constant
// 32 or 64, which is what we use now.  Obviously, it could be extended
// to src2 == any integer value from 0..127.
// Left them out for brevity
function power_of_2 (power:Value): int {
    (var p := i#Integer(power);
     if p == 32 then 4294967296
     else if p == 64 then 18446744073709551616
     // value is undefined, otherwise.
     else -1
     )
}

procedure {:inline 1} $Shl(src1: Value, src2: Value) returns (dst: Value)
requires is#Integer(src1) && is#Integer(src2);
{
    var po2: int;
    po2 := power_of_2(src2);
    assert po2 >= 1;   // po2 < 0 if src2 not 32 or 63
    dst := Integer(i#Integer(src2) * po2);
}

procedure {:inline 1} $Shr(src1: Value, src2: Value) returns (dst: Value)
requires is#Integer(src1) && is#Integer(src2);
{
    var po2: int;
    po2 := power_of_2(src2);
    assert po2 >= 1;   // po2 < 0 if src2 not 32 or 63
    dst := Integer(i#Integer(src2) div po2);
}

procedure {:inline 1} $MulU8(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} $IsValidU8(src1) && $IsValidU8(src2);
{
    if (i#Integer(src1) * i#Integer(src2) > MAX_U8) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) * i#Integer(src2));
}

procedure {:inline 1} $MulU64(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} $IsValidU64(src1) && $IsValidU64(src2);
{
    if (i#Integer(src1) * i#Integer(src2) > MAX_U64) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) * i#Integer(src2));
}

procedure {:inline 1} $MulU128(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} $IsValidU128(src1) && $IsValidU128(src2);
{
    if (i#Integer(src1) * i#Integer(src2) > MAX_U128) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) * i#Integer(src2));
}

procedure {:inline 1} $Div(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} is#Integer(src1) && is#Integer(src2);
{
    if (i#Integer(src2) == 0) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) div i#Integer(src2));
}

procedure {:inline 1} $Mod(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} is#Integer(src1) && is#Integer(src2);
{
    if (i#Integer(src2) == 0) {
        $abort_flag := true;
        return;
    }
    dst := Integer(i#Integer(src1) mod i#Integer(src2));
}

procedure {:inline 1} $ArithBinaryUnimplemented(src1: Value, src2: Value) returns (dst: Value);
{{type_requires}} is#Integer(src1) && is#Integer(src2);
ensures is#Integer(dst);

procedure {:inline 1} $Lt(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} is#Integer(src1) && is#Integer(src2);
{
    dst := Boolean(i#Integer(src1) < i#Integer(src2));
}

procedure {:inline 1} $Gt(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} is#Integer(src1) && is#Integer(src2);
{
    dst := Boolean(i#Integer(src1) > i#Integer(src2));
}

procedure {:inline 1} $Le(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} is#Integer(src1) && is#Integer(src2);
{
    dst := Boolean(i#Integer(src1) <= i#Integer(src2));
}

procedure {:inline 1} $Ge(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} is#Integer(src1) && is#Integer(src2);
{
    dst := Boolean(i#Integer(src1) >= i#Integer(src2));
}

procedure {:inline 1} $And(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} is#Boolean(src1) && is#Boolean(src2);
{
    dst := Boolean(b#Boolean(src1) && b#Boolean(src2));
}

procedure {:inline 1} $Or(src1: Value, src2: Value) returns (dst: Value)
{{type_requires}} is#Boolean(src1) && is#Boolean(src2);
{
    dst := Boolean(b#Boolean(src1) || b#Boolean(src2));
}

procedure {:inline 1} $Not(src: Value) returns (dst: Value)
{{type_requires}} is#Boolean(src);
{
    dst := Boolean(!b#Boolean(src));
}

// Pack and Unpack are auto-generated for each type T


// Transaction
// -----------

type {:datatype} Transaction;
var $txn: Transaction;
function {:constructor} Transaction(sender: int) : Transaction;


// ==================================================================================
// Native Vector Type

function {:inline} $Vector_type_value(tv: TypeValue): TypeValue {
    VectorType(tv)
}

function {:inline} $Vector_is_well_formed(v: Value): bool {
    is#Vector(v) &&
    (
        var va := v#Vector(v);
        (
            var l := l#ValueArray(va);
            0 <= l &&
            (forall x: int :: {v#ValueArray(va)[x]} x < 0 || x >= l ==> v#ValueArray(va)[x] == DefaultValue())
        )
    )
}

procedure {:inline 1} $Vector_empty(ta: TypeValue) returns (v: Value) {
    v := $mk_vector();
}

procedure {:inline 1} $Vector_is_empty(ta: TypeValue, v: Value) returns (b: Value) {
    assume is#Vector(v);
    b := Boolean($vlen(v) == 0);
}

procedure {:inline 1} $Vector_push_back(ta: TypeValue, v: Value, val: Value) returns (v': Value) {
    assume is#Vector(v);
    v' := $push_back_vector(v, val);
}

procedure {:inline 1} $Vector_pop_back(ta: TypeValue, v: Value) returns (e: Value, v': Value) {
    var len: int;
    assume is#Vector(v);
    len := $vlen(v);
    if (len == 0) {
        $abort_flag := true;
        return;
    }
    e := $vmap(v)[len-1];
    v' := $pop_back_vector(v);
}

procedure {:inline 1} $Vector_append(ta: TypeValue, v: Value, other: Value) returns (v': Value) {
    assume is#Vector(v);
    assume is#Vector(other);
    v' := $append_vector(v, other);
}

procedure {:inline 1} $Vector_reverse(ta: TypeValue, v: Value) returns (v': Value) {
    assume is#Vector(v);
    v' := $reverse_vector(v);
}

procedure {:inline 1} $Vector_length(ta: TypeValue, v: Value) returns (l: Value) {
    assume is#Vector(v);
    l := Integer($vlen(v));
}

procedure {:inline 1} $Vector_borrow(ta: TypeValue, src: Value, i: Value) returns (dst: Value) {
    var i_ind: int;

    assume is#Vector(src);
    assume is#Integer(i);
    i_ind := i#Integer(i);
    if (i_ind < 0 || i_ind >= $vlen(src)) {
        $abort_flag := true;
        return;
    }
    dst := $vmap(src)[i_ind];
}

procedure {:inline 1} $Vector_borrow_mut(ta: TypeValue, v: Value, index: Value) returns (dst: Reference, v': Value)
{{type_requires}} is#Integer(index);
{
    var i_ind: int;

    i_ind := i#Integer(index);
    assume is#Vector(v);
    if (i_ind < 0 || i_ind >= $vlen(v)) {
        $abort_flag := true;
        return;
    }
    dst := Reference(Local(0), Path(p#Path(EmptyPath)[0 := i_ind], 1), $vmap(v)[i_ind]);
    v' := v;
}

procedure {:inline 1} $Vector_destroy_empty(ta: TypeValue, v: Value) {
    if ($vlen(v) != 0) {
      $abort_flag := true;
    }
}

procedure {:inline 1} $Vector_swap(ta: TypeValue, v: Value, i: Value, j: Value) returns (v': Value)
{{type_requires}} is#Integer(i) && is#Integer(j);
{
    var i_ind: int;
    var j_ind: int;
    assume is#Vector(v);
    i_ind := i#Integer(i);
    j_ind := i#Integer(j);
    if (i_ind >= $vlen(v) || j_ind >= $vlen(v) || i_ind < 0 || j_ind < 0) {
        $abort_flag := true;
        return;
    }
    v' := $swap_vector(v, i_ind, j_ind);
}

procedure {:inline 1} $Vector_remove(ta: TypeValue, v: Value, i: Value) returns (e: Value, v': Value)
{{type_requires}} is#Integer(i);
{
    var i_ind: int;

    assume is#Vector(v);
    i_ind := i#Integer(i);
    if (i_ind < 0 || i_ind >= $vlen(v)) {
        $abort_flag := true;
        return;
    }
    e := $vmap(v)[i_ind];
    v' := $remove_vector(v, i_ind);
}

procedure {:inline 1} $Vector_swap_remove(ta: TypeValue, v: Value, i: Value) returns (e: Value, v': Value)
{{type_requires}} is#Integer(i);
{
    var i_ind: int;
    var len: int;

    assume is#Vector(v);
    i_ind := i#Integer(i);
    len := $vlen(v);
    if (i_ind < 0 || i_ind >= len) {
        $abort_flag := true;
        return;
    }
    e := $vmap(v)[i_ind];
    v' := $pop_back_vector($swap_vector(v, i_ind, len-1));
}

procedure {:inline 1} $Vector_contains(ta: TypeValue, v: Value, e: Value) returns (res: Value)  {
    assume is#Vector(v);
    res := Boolean($contains_vector(v, e));
}

// FIXME: This procedure sometimes (not always) make the test (performance_200511) very slow (> 10 mins) or hang
// although this is not used in the test script (performance_200511). The test finishes in 20 secs when it works fine.
procedure {:inline 1} $Vector_index_of(ta: TypeValue, v: Value, e: Value) returns (res1: Value, res2: Value);
requires is#Vector(v);
ensures is#Boolean(res1);
ensures is#Integer(res2);
ensures 0 <= i#Integer(res2) && i#Integer(res2) < $vlen(v);
ensures res1 == Boolean($contains_vector(v, e));
ensures b#Boolean(res1) ==> IsEqual($vmap(v)[i#Integer(res2)], e);
ensures b#Boolean(res1) ==> (forall i:int :: 0<=i && i<i#Integer(res2) ==> !IsEqual($vmap(v)[i], e));
ensures !b#Boolean(res1) ==> i#Integer(res2) == 0;

// FIXME: This alternative definition has the same issue as the other one above.
// TODO: Delete this when unnecessary
//procedure {:inline 1} $Vector_index_of(ta: TypeValue, v: Value, e: Value) returns (res1: Value, res2: Value) {
//    var b: bool;
//    var i: int;
//    assume is#Vector(v);
//    b := $contains_vector(v, e);
//    if (b) {
//        havoc i;
//        assume 0 <= i && i < $vlen(v);
//        assume IsEqual($vmap(v)[i], e);
//        assume (forall j:int :: 0<=j && j<i ==> !IsEqual($vmap(v)[j], e));
//    }
//    else {
//        i := 0;
//    }
//    res1 := Boolean(b);
//    res2 := Integer(i);
//}

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


function {:inline} $Hash_sha2($m: Memory, $txn: Transaction, val: Value): Value {
    $Hash_sha2_core(val)
}

function $Hash_sha2_core(val: Value): Value;

// This says that Hash_sha2 respects isEquals (this would be automatic if we had an
// extensional theory of arrays and used ==, which has the substitution property
// for functions).
axiom (forall v1,v2: Value :: $Vector_is_well_formed(v1) && $Vector_is_well_formed(v2)
       && IsEqual(v1, v2) ==> IsEqual($Hash_sha2_core(v1), $Hash_sha2_core(v2)));

// This says that Hash_sha2 is an injection
axiom (forall v1,v2: Value :: $Vector_is_well_formed(v1) && $Vector_is_well_formed(v2)
        && IsEqual($Hash_sha2_core(v1), $Hash_sha2_core(v2)) ==> IsEqual(v1, v2));

// This procedure has no body. We want Boogie to just use its requires
// and ensures properties when verifying code that calls it.
procedure $Hash_sha2_256(val: Value) returns (res: Value);
// It will still work without this, but this helps verifier find more reasonable counterexamples.
{{type_requires}} $IsValidU8Vector(val);
ensures res == $Hash_sha2_core(val);     // returns Hash_sha2 value
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.
ensures $vlen(res) == 32;               // result is 32 bytes.

// similarly for Hash_sha3
function {:inline} $Hash_sha3($m: Memory, $txn: Transaction, val: Value): Value {
    $Hash_sha3_core(val)
}
function $Hash_sha3_core(val: Value): Value;

axiom (forall v1,v2: Value :: $Vector_is_well_formed(v1) && $Vector_is_well_formed(v2)
       && IsEqual(v1, v2) ==> IsEqual($Hash_sha3_core(v1), $Hash_sha3_core(v2)));

axiom (forall v1,v2: Value :: $Vector_is_well_formed(v1) && $Vector_is_well_formed(v2)
        && IsEqual($Hash_sha3_core(v1), $Hash_sha3_core(v2)) ==> IsEqual(v1, v2));

procedure $Hash_sha3_256(val: Value) returns (res: Value);
ensures res == $Hash_sha3_core(val);     // returns Hash_sha3 value
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.
ensures $vlen(res) == 32;               // result is 32 bytes.

// ==================================================================================
// Native libra_account

// TODO: this function clashes with a similar version in older libraries. This is solved by a hack where the
// translator appends _OLD to the name when encountering this. The hack shall be removed once old library
// sources are not longer used.
procedure {:inline 1} $LibraAccount_save_account_OLD(ta: TypeValue, balance: Value, account: Value, addr: Value) {
    var a: int;
    var t_T: TypeValue;
    var l_T: Location;
    var t_Balance: TypeValue;
    var l_Balance: Location;

    a := a#Address(addr);
    t_T := $LibraAccount_T_type_value();
    if ($ResourceExistsRaw($m, t_T, a)) {
        $abort_flag := true;
        return;
    }

    t_Balance := $LibraAccount_Balance_type_value(ta);
    if ($ResourceExistsRaw($m, t_Balance, a)) {
        $abort_flag := true;
        return;
    }

    l_T := Global(t_T, a);
    l_Balance := Global(t_Balance, a);
    $m := Memory(domain#Memory($m)[l_T := true][l_Balance := true], contents#Memory($m)[l_T := account][l_Balance := balance]);
}

procedure {:inline 1} $LibraAccount_save_account(
       t_Token: TypeValue, t_AT: TypeValue, account_type: Value, balance: Value,
       account: Value, event_generator: Value, addr: Value) {
    // TODO: implement this
    assert false;
}

procedure {:inline 1} $LibraAccount_write_to_event_store(ta: TypeValue, guid: Value, count: Value, msg: Value) {
    // TODO: this is used in old library sources, remove it once those sources are not longer used in tests.
    // This function is modeled as a no-op because the actual side effect of this native function is not observable from the Move side.
}

procedure {:inline 1} $Event_write_to_event_store(ta: TypeValue, guid: Value, count: Value, msg: Value) {
    // This function is modeled as a no-op because the actual side effect of this native function is not observable from the Move side.
}

// ==================================================================================
// Native signer

procedure {:inline 1} $Signer_borrow_address(signer: Value) returns (res: Value)
    {{type_requires}} is#Address(signer);
{
    // A signer is currently identical to an address.
    res := signer;
}

// TODO: implement the below methods
// ==================================================================================
// Native signature

// TODO: implement the below methods

procedure {:inline 1} $Signature_ed25519_validate_pubkey(public_key: Value) returns (res: Value) {
    assert false; // $Signature_ed25519_validate_pubkey not implemented
}

procedure {:inline 1} $Signature_ed25519_verify(signature: Value, public_key: Value, message: Value) returns (res: Value) {
    assert false; // $Signature_ed25519_verify not implemented
}

procedure {:inline 1} Signature_ed25519_threshold_verify(bitmap: Value, signature: Value, public_key: Value, message: Value) returns (res: Value) {
    assert false; // Signature_ed25519_threshold_verify not implemented
}

// ==================================================================================
// Native LCS::serialize

// native define serialize<MoveValue>(v: &MoveValue): vector<u8>;

// Serialize is modeled as an uninterpreted function, with an additional
// axiom to say it's an injection.

function {:inline} $LCS_serialize($m: Memory, $txn: Transaction, ta: TypeValue, v: Value): Value {
    $LCS_serialize_core(v)
}

function $LCS_serialize_core(v: Value): Value;
function $LCS_serialize_core_inv(v: Value): Value;
// Needed only because IsEqual(v1, v2) is weaker than v1 == v2 in case there is a vector nested inside v1 or v2.
axiom (forall v1, v2: Value :: IsEqual(v1, v2) ==> $LCS_serialize_core(v1) == $LCS_serialize_core(v2));
// Injectivity
axiom (forall v: Value :: $LCS_serialize_core_inv($LCS_serialize_core(v)) == v);

// This says that serialize returns a non-empty vec<u8>
{{#if (eq serialize_bound 0)}}
axiom (forall v: Value :: ( var r := $LCS_serialize_core(v); $IsValidU8Vector(r) && $vlen(r) > 0 ));
{{else}}
axiom (forall v: Value :: ( var r := $LCS_serialize_core(v); $IsValidU8Vector(r) && $vlen(r) > 0 && $vlen(r) <= {{serialize_bound}} ));
{{/if}}

procedure $LCS_to_bytes(ta: TypeValue, v: Value) returns (res: Value);
ensures res == $LCS_serialize($m, $txn, ta, v);
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.
