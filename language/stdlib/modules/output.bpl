
// ** prelude from <inline-prelude>

// ================================================================================
// Notation

// This files contains a Handlebars Rust template for the prover's Boogie prelude.
// The template language constructs allow the prelude to adjust the actual content
// to multiple options. We only use a few selected template constructs which are
// mostly self-explaining. See the handlebars crate documentation for more information.
//
// The object passed in as context for templates is the struct cli::Options and its
// sub-structs.

// ================================================================================
// Domains

// Debug tracking
// --------------

// Debug tracking is used to inject information used for model analysis. The generated code emits statements
// like this:
//
//     assume $DebugTrackLocal(file_id, byte_index, var_idx, $Value);
//
// While those tracking assumptions are trivially true for the provers logic, the solver (at least Z3)
// will construct a function mapping which appears in the model, e.g.:
//
//     $DebugTrackLocal -> {
//         1 440 0 (Vector (ValueArray |T@[Int]$Value!val!0| 0)) -> true
//         1 533 1 ($Integer 1) -> true
//         ...
//         else -> true
//     }
//
// This information can then be read out from the model.


// Tracks debug information for a parameter, local or a return parameter. Return parameter indices start at
// the overall number of locals (including parameters).
function $DebugTrackLocal(file_id: int, byte_index:  int, var_idx: int, $Value: $Value) : bool {
  true
}

// Tracks at which location a function was aborted.
function $DebugTrackAbort(file_id: int, byte_index: int) : bool {
  true
}

// Tracks the $Value of a specification (sub-)expression.
function $DebugTrackExp(module_id: int, node_id: int, $Value: $Value) : $Value { $Value }


// Path type
// ---------

type {:datatype} $Path;
function {:constructor} $Path(p: [int]int, size: int): $Path;
const $EmptyPath: $Path;
axiom size#$Path($EmptyPath) == 0;

function {:inline} $path_index_at(p: $Path, i: int): int {
    p#$Path(p)[i]
}

// Type Values
// -----------

type $TypeName;
type $FieldName = int;
type $LocalName;
type {:datatype} $TypeValue;
function {:constructor} $BooleanType() : $TypeValue;
function {:constructor} $IntegerType() : $TypeValue;
function {:constructor} $AddressType() : $TypeValue;
function {:constructor} $StrType() : $TypeValue;
function {:constructor} $VectorType(t: $TypeValue) : $TypeValue;
function {:constructor} $StructType(name: $TypeName, ps: $TypeValueArray, ts: $TypeValueArray) : $TypeValue;
function {:constructor} $TypeType(): $TypeValue;
function {:constructor} $ErrorType() : $TypeValue;

function {:inline} $DefaultTypeValue() : $TypeValue { $ErrorType() }
function {:builtin "MapConst"} $MapConstTypeValue(tv: $TypeValue): [int]$TypeValue;

type {:datatype} $TypeValueArray;
function {:constructor} $TypeValueArray(v: [int]$TypeValue, l: int): $TypeValueArray;
const $EmptyTypeValueArray: $TypeValueArray;
axiom l#$TypeValueArray($EmptyTypeValueArray) == 0;
axiom v#$TypeValueArray($EmptyTypeValueArray) == $MapConstTypeValue($DefaultTypeValue());


// Values
// ------

type {:datatype} $Value;

const $MAX_U8: int;
axiom $MAX_U8 == 255;
const $MAX_U64: int;
axiom $MAX_U64 == 18446744073709551615;
const $MAX_U128: int;
axiom $MAX_U128 == 340282366920938463463374607431768211455;

function {:constructor} $Boolean(b: bool): $Value;
function {:constructor} $Integer(i: int): $Value;
function {:constructor} $Address(a: int): $Value;
function {:constructor} $Vector(v: $ValueArray): $Value; // used to both represent move Struct and Vector
function {:constructor} $Range(lb: $Value, ub: $Value): $Value;
function {:constructor} $Type(t: $TypeValue): $Value;
function {:constructor} $Error(): $Value;

function {:inline} $DefaultValue(): $Value { $Error() }
function {:builtin "MapConst"} $MapConstValue(v: $Value): [int]$Value;

function {:inline} $IsValidU8(v: $Value): bool {
  is#$Integer(v) && i#$Integer(v) >= 0 && i#$Integer(v) <= $MAX_U8
}

function {:inline} $IsValidU8Vector(vec: $Value): bool {
  $Vector_is_well_formed(vec)
  && (forall i: int :: {$select_vector(vec, i)} 0 <= i && i < $vlen(vec) ==> $IsValidU8($select_vector(vec, i)))
}

function {:inline} $IsValidU64(v: $Value): bool {
  is#$Integer(v) && i#$Integer(v) >= 0 && i#$Integer(v) <= $MAX_U64
}

function {:inline} $IsValidU128(v: $Value): bool {
  is#$Integer(v) && i#$Integer(v) >= 0 && i#$Integer(v) <= $MAX_U128
}

function {:inline} $IsValidNum(v: $Value): bool {
  is#$Integer(v)
}


// Value Array
// -----------





// This is the implementation of $ValueArray using integer maps

type {:datatype} $ValueArray;

function {:constructor} $ValueArray(v: [int]$Value, l: int): $ValueArray;

function $EmptyValueArray(): $ValueArray;
axiom l#$ValueArray($EmptyValueArray()) == 0;
axiom v#$ValueArray($EmptyValueArray()) == $MapConstValue($Error());

function {:inline} $ReadValueArray(a: $ValueArray, i: int): $Value {
    (
        v#$ValueArray(a)[i]
    )
}

function {:inline} $LenValueArray(a: $ValueArray): int {
    (
        l#$ValueArray(a)
    )
}

function {:inline} $RemoveValueArray(a: $ValueArray): $ValueArray {
    (
        var l := l#$ValueArray(a) - 1;
        $ValueArray(
            (lambda i: int ::
                if i >= 0 && i < l then v#$ValueArray(a)[i] else $DefaultValue()),
            l
        )
    )
}

function {:inline} $RemoveIndexValueArray(a: $ValueArray, i: int): $ValueArray {
    (
        var l := l#$ValueArray(a) - 1;
        $ValueArray(
            (lambda j: int ::
                if j >= 0 && j < l then
                    if j < i then v#$ValueArray(a)[j] else v#$ValueArray(a)[j+1]
                else $DefaultValue()),
            l
        )
    )
}

function {:inline} $ConcatValueArray(a1: $ValueArray, a2: $ValueArray): $ValueArray {
    (
        var l1, l2 := l#$ValueArray(a1), l#$ValueArray(a2);
        $ValueArray(
            (lambda i: int ::
                if i >= 0 && i < l1 + l2 then
                    if i < l1 then v#$ValueArray(a1)[i] else v#$ValueArray(a2)[i - l1]
                else
                    $DefaultValue()),
            l1 + l2)
    )
}

function {:inline} $ReverseValueArray(a: $ValueArray): $ValueArray {
    (
        var l := l#$ValueArray(a);
        $ValueArray(
            (lambda i: int :: if 0 <= i && i < l then v#$ValueArray(a)[l - i - 1] else $DefaultValue()),
            l
        )
    )
}

function {:inline} $SliceValueArray(a: $ValueArray, i: int, j: int): $ValueArray { // return the sliced vector of a for the range [i, j)
    $ValueArray((lambda k:int :: if 0 <= k && k < j-i then v#$ValueArray(a)[i+k] else $DefaultValue()), (if j-i < 0 then 0 else j-i))
}

function {:inline} $ExtendValueArray(a: $ValueArray, elem: $Value): $ValueArray {
    (var len := l#$ValueArray(a);
     $ValueArray(v#$ValueArray(a)[len := elem], len + 1))
}

function {:inline} $UpdateValueArray(a: $ValueArray, i: int, elem: $Value): $ValueArray {
    $ValueArray(v#$ValueArray(a)[i := elem], l#$ValueArray(a))
}

function {:inline} $SwapValueArray(a: $ValueArray, i: int, j: int): $ValueArray {
    $ValueArray(v#$ValueArray(a)[i := v#$ValueArray(a)[j]][j := v#$ValueArray(a)[i]], l#$ValueArray(a))
}

function {:inline} $IsEmpty(a: $ValueArray): bool {
    l#$ValueArray(a) == 0
}

// All invalid elements of array are DefaultValue. This is useful in specialized
// cases. This is used to defined normalization for $Vector
function {:inline} $IsNormalizedValueArray(a: $ValueArray, len: int): bool {
    (forall i: int :: i < 0 || i >= len ==> v#$ValueArray(a)[i] == $DefaultValue())
}


 //end of backend.vector_using_sequences


// Stratified Functions on Values
// ------------------------------

// TODO: templatize this or move it back to the translator. For now we
//   prefer to handcode this so its easier to evolve the model independent of the
//   translator.

const $StratificationDepth: int;
axiom $StratificationDepth == 4;



// Generate a stratified version of IsEqual for depth of 4.

function  $IsEqual_stratified(v1: $Value, v2: $Value): bool {
    (v1 == v2) ||
    (is#$Vector(v1) &&
     is#$Vector(v2) &&
     $vlen(v1) == $vlen(v2) &&
     (forall i: int :: 0 <= i && i < $vlen(v1) ==> $IsEqual_level1($select_vector(v1,i), $select_vector(v2,i))))
}

function  $IsEqual_level1(v1: $Value, v2: $Value): bool {
    (v1 == v2) ||
    (is#$Vector(v1) &&
     is#$Vector(v2) &&
     $vlen(v1) == $vlen(v2) &&
     (forall i: int :: 0 <= i && i < $vlen(v1) ==> $IsEqual_level2($select_vector(v1,i), $select_vector(v2,i))))
}

function  $IsEqual_level2(v1: $Value, v2: $Value): bool {
    (v1 == v2) ||
    (is#$Vector(v1) &&
     is#$Vector(v2) &&
     $vlen(v1) == $vlen(v2) &&
     (forall i: int :: 0 <= i && i < $vlen(v1) ==> $IsEqual_level3($select_vector(v1,i), $select_vector(v2,i))))
}

function {:inline} $IsEqual_level3(v1: $Value, v2: $Value): bool {
    v1 == v2
}


function {:inline} $IsEqual(v1: $Value, v2: $Value): bool {
    $IsEqual_stratified(v1, v2)
}



// Generate stratified ReadValue for the depth of 4.


function  $ReadValue_stratified(p: $Path, v: $Value) : $Value {
    if (0 == size#$Path(p)) then
        v
    else
        $ReadValue_level1(p, $select_vector(v,$path_index_at(p, 0)))
}

function  $ReadValue_level1(p: $Path, v: $Value) : $Value {
    if (1 == size#$Path(p)) then
        v
    else
        $ReadValue_level2(p, $select_vector(v,$path_index_at(p, 1)))
}

function  $ReadValue_level2(p: $Path, v: $Value) : $Value {
    if (2 == size#$Path(p)) then
        v
    else
        $ReadValue_level3(p, $select_vector(v,$path_index_at(p, 2)))
}

function {:inline} $ReadValue_level3(p: $Path, v: $Value): $Value {
    v
}


function {:inline} $ReadValue(p: $Path, v: $Value): $Value {
    $ReadValue_stratified(p, v)
}

// Generate stratified $UpdateValue for the depth of 4.


function  $UpdateValue_stratified(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    (var poffset := offset + 0;
    if (poffset == size#$Path(p)) then
        new_v
    else
        $update_vector(v, $path_index_at(p, poffset),
                       $UpdateValue_level1(p, offset, $select_vector(v,$path_index_at(p, poffset)), new_v)))
}

function  $UpdateValue_level1(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    (var poffset := offset + 1;
    if (poffset == size#$Path(p)) then
        new_v
    else
        $update_vector(v, $path_index_at(p, poffset),
                       $UpdateValue_level2(p, offset, $select_vector(v,$path_index_at(p, poffset)), new_v)))
}

function  $UpdateValue_level2(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    (var poffset := offset + 2;
    if (poffset == size#$Path(p)) then
        new_v
    else
        $update_vector(v, $path_index_at(p, poffset),
                       $UpdateValue_level3(p, offset, $select_vector(v,$path_index_at(p, poffset)), new_v)))
}

function {:inline} $UpdateValue_level3(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    new_v
}


function {:inline} $UpdateValue(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    $UpdateValue_stratified(p, offset, v, new_v)
}

// Generate stratified $IsPathPrefix for the depth of 4.


function  $IsPathPrefix_stratified(p1: $Path, p2: $Path): bool {
    if (0 == size#$Path(p1)) then
        true
    else if (p#$Path(p1)[0] == p#$Path(p2)[0]) then
        $IsPathPrefix_level1(p1, p2)
    else
        false
}

function  $IsPathPrefix_level1(p1: $Path, p2: $Path): bool {
    if (1 == size#$Path(p1)) then
        true
    else if (p#$Path(p1)[1] == p#$Path(p2)[1]) then
        $IsPathPrefix_level2(p1, p2)
    else
        false
}

function  $IsPathPrefix_level2(p1: $Path, p2: $Path): bool {
    if (2 == size#$Path(p1)) then
        true
    else if (p#$Path(p1)[2] == p#$Path(p2)[2]) then
        $IsPathPrefix_level3(p1, p2)
    else
        false
}

function {:inline} $IsPathPrefix_level3(p1: $Path, p2: $Path): bool {
    true
}


function {:inline} $IsPathPrefix(p1: $Path, p2: $Path): bool {
    $IsPathPrefix_stratified(p1, p2)
}

// Generate stratified $ConcatPath for the depth of 4.


function  $ConcatPath_stratified(p1: $Path, p2: $Path): $Path {
    if (0 == size#$Path(p2)) then
        p1
    else
        $ConcatPath_level1($Path(p#$Path(p1)[size#$Path(p1) := p#$Path(p2)[0]], size#$Path(p1) + 1), p2)
}

function  $ConcatPath_level1(p1: $Path, p2: $Path): $Path {
    if (1 == size#$Path(p2)) then
        p1
    else
        $ConcatPath_level2($Path(p#$Path(p1)[size#$Path(p1) := p#$Path(p2)[1]], size#$Path(p1) + 1), p2)
}

function  $ConcatPath_level2(p1: $Path, p2: $Path): $Path {
    if (2 == size#$Path(p2)) then
        p1
    else
        $ConcatPath_level3($Path(p#$Path(p1)[size#$Path(p1) := p#$Path(p2)[2]], size#$Path(p1) + 1), p2)
}

function {:inline} $ConcatPath_level3(p1: $Path, p2: $Path): $Path {
    p1
}


function {:inline} $ConcatPath(p1: $Path, p2: $Path): $Path {
    $ConcatPath_stratified(p1, p2)
}

// Vector related functions on Values
// ----------------------------------

function {:inline} $vlen(v: $Value): int {
    $LenValueArray(v#$Vector(v))
}

// Check that all invalid elements of vector are DefaultValue
function {:inline} $is_normalized_vector(v: $Value): bool {
    $IsNormalizedValueArray(v#$Vector(v), $vlen(v))
}

// Sometimes, we need the length as a Value, not an int.
function {:inline} $vlen_value(v: $Value): $Value {
    $Integer($vlen(v))
}
function {:inline} $mk_vector(): $Value {
    $Vector($EmptyValueArray())
}
function {:inline} $push_back_vector(v: $Value, elem: $Value): $Value {
    $Vector($ExtendValueArray(v#$Vector(v), elem))
}
function {:inline} $pop_back_vector(v: $Value): $Value {
    $Vector($RemoveValueArray(v#$Vector(v)))
}
function {:inline} $append_vector(v1: $Value, v2: $Value): $Value {
    $Vector($ConcatValueArray(v#$Vector(v1), v#$Vector(v2)))
}
function {:inline} $reverse_vector(v: $Value): $Value {
    $Vector($ReverseValueArray(v#$Vector(v)))
}
function {:inline} $update_vector(v: $Value, i: int, elem: $Value): $Value {
    $Vector($UpdateValueArray(v#$Vector(v), i, elem))
}
// $update_vector_by_value requires index to be a Value, not int.
function {:inline} $update_vector_by_value(v: $Value, i: $Value, elem: $Value): $Value {
    $Vector($UpdateValueArray(v#$Vector(v), i#$Integer(i), elem))
}
function {:inline} $select_vector(v: $Value, i: int) : $Value {
    $ReadValueArray(v#$Vector(v), i)
}
// $select_vector_by_value requires index to be a Value, not int.
function {:inline} $select_vector_by_value(v: $Value, i: $Value) : $Value {
    $select_vector(v, i#$Integer(i))
}
function {:inline} $swap_vector(v: $Value, i: int, j: int): $Value {
    $Vector($SwapValueArray(v#$Vector(v), i, j))
}
function {:inline} $slice_vector(v: $Value, r: $Value) : $Value {
    $Vector($SliceValueArray(v#$Vector(v), i#$Integer(lb#$Range(r)), i#$Integer(ub#$Range(r))))
}
function {:inline} $InVectorRange(v: $Value, i: int): bool {
    i >= 0 && i < $vlen(v)
}
function {:inline} $remove_vector(v: $Value, i:int): $Value {
    $Vector($RemoveIndexValueArray(v#$Vector(v), i))
}
function {:inline} $contains_vector(v: $Value, e: $Value): bool {
    (exists i:int :: 0 <= i && i < $vlen(v) && $IsEqual($select_vector(v,i), e))
}

function {:inline} $InRange(r: $Value, i: int): bool {
   i#$Integer(lb#$Range(r)) <= i && i < i#$Integer(ub#$Range(r))
}


// ============================================================================================
// Memory

type {:datatype} $Location;
function {:constructor} $Global(t: $TypeValue, a: int): $Location;
function {:constructor} $Local(i: int): $Location;
function {:constructor} $Param(i: int): $Location;

type {:datatype} $Reference;
function {:constructor} $Reference(l: $Location, p: $Path, v: $Value): $Reference;
const $DefaultReference: $Reference;

type {:datatype} $Memory;
function {:constructor} $Memory(domain: [$Location]bool, contents: [$Location]$Value): $Memory;

function $Memory__is_well_formed(m: $Memory): bool;

function {:builtin "MapConst"} $ConstMemoryDomain(v: bool): [$Location]bool;
function {:builtin "MapConst"} $ConstMemoryContent(v: $Value): [$Location]$Value;

const $EmptyMemory: $Memory;
axiom domain#$Memory($EmptyMemory) == $ConstMemoryDomain(false);
axiom contents#$Memory($EmptyMemory) == $ConstMemoryContent($DefaultValue());

var $m: $Memory;
var $abort_flag: bool;

procedure {:inline 1} $InitVerification() {
  // Set abort_flag to false
  $abort_flag := false;
}

// ============================================================================================
// Specifications

// TODO: unify some of this with instruction procedures to avoid duplication

// Tests whether resource exists.
function {:inline} $ResourceExistsRaw(m: $Memory, resource: $TypeValue, addr: int): bool {
    domain#$Memory(m)[$Global(resource, addr)]
}
function {:inline} $ResourceExists(m: $Memory, resource: $TypeValue, address: $Value): $Value {
    $Boolean($ResourceExistsRaw(m, resource, a#$Address(address)))
}

// Obtains Value of given resource.
function {:inline} $ResourceValue(m: $Memory, resource: $TypeValue, address: $Value): $Value {
  contents#$Memory(m)[$Global(resource, a#$Address(address))]
}

// Applies a field selection to a Value.
function {:inline} $SelectField(val: $Value, field: $FieldName): $Value { //breaks abstracts, we don't know $Fieldname = int
    $select_vector(val, field)
}

// Dereferences a reference.
function {:inline} $Dereference(ref: $Reference): $Value {
    v#$Reference(ref)
}

// Check whether sender account exists.
function {:inline} $ExistsTxnSenderAccount(m: $Memory, txn: $Transaction): bool {
   domain#$Memory(m)[$Global($LibraAccount_T_type_value(), sender#$Transaction(txn))]
}

function {:inline} $TxnSender(txn: $Transaction): $Value {
    $Address(sender#$Transaction(txn))
}

// Forward declaration of type Value of LibraAccount. This is declared so we can define
// $ExistsTxnSenderAccount and $LibraAccount_save_account
const unique $LibraAccount_T: $TypeName;
function $LibraAccount_T_type_value(): $TypeValue;
axiom is#$StructType($LibraAccount_T_type_value()) && name#$StructType($LibraAccount_T_type_value()) == $LibraAccount_T;
function $LibraAccount_Balance_type_value(tv: $TypeValue): $TypeValue;

// ============================================================================================
// Instructions

procedure {:inline 1} $Exists(address: $Value, t: $TypeValue) returns (dst: $Value)
free requires is#$Address(address);
{
    dst := $ResourceExists($m, t, address);
}

procedure {:inline 1} $MoveToRaw(ta: $TypeValue, a: int, v: $Value)
{
    var l: $Location;

    l := $Global(ta, a);
    if ($ResourceExistsRaw($m, ta, a)) {
        $abort_flag := true;
        return;
    }
    $m := $Memory(domain#$Memory($m)[l := true], contents#$Memory($m)[l := v]);
}

procedure {:inline 1} $MoveTo(ta: $TypeValue, v: $Value, signer: $Value)
{
    var addr: $Value;

    call addr := $Signer_borrow_address(signer);
    call $MoveToRaw(ta, a#$Address(addr), v);
}

procedure {:inline 1} $MoveToSender(ta: $TypeValue, v: $Value)
{
    call $MoveToRaw(ta, sender#$Transaction($txn), v);
}

procedure {:inline 1} $MoveFrom(address: $Value, ta: $TypeValue) returns (dst: $Value)
free requires is#$Address(address);
{
    var a: int;
    var l: $Location;
    a := a#$Address(address);
    l := $Global(ta, a);
    if (!$ResourceExistsRaw($m, ta, a)) {
        $abort_flag := true;
        return;
    }
    dst := contents#$Memory($m)[l];
    $m := $Memory(domain#$Memory($m)[l := false], contents#$Memory($m)[l := $DefaultValue()]);
}

procedure {:inline 1} $BorrowGlobal(address: $Value, ta: $TypeValue) returns (dst: $Reference)
free requires is#$Address(address);
{
    var a: int;
    var l: $Location;
    a := a#$Address(address);
    l := $Global(ta, a);
    if (!$ResourceExistsRaw($m, ta, a)) {
        $abort_flag := true;
        return;
    }
    dst := $Reference(l, $EmptyPath, contents#$Memory($m)[l]);
}

procedure {:inline 1} $BorrowLoc(l: int, v: $Value) returns (dst: $Reference)
{
    dst := $Reference($Local(l), $EmptyPath, v);
}

procedure {:inline 1} $BorrowField(src: $Reference, f: $FieldName) returns (dst: $Reference)
{
    var p: $Path;
    var size: int;

    p := p#$Reference(src);
    size := size#$Path(p);
    p := $Path(p#$Path(p)[size := f], size+1);
    dst := $Reference(l#$Reference(src), p, $select_vector(v#$Reference(src), f)); //breaks abstraction
}

procedure {:inline 1} $GetGlobal(address: $Value, ta: $TypeValue) returns (dst: $Value)
free requires is#$Address(address);
{
    var r: $Reference;

    call r := $BorrowGlobal(address, ta);
    call dst := $ReadRef(r);
}

procedure {:inline 1} $GetFieldFromReference(src: $Reference, f: $FieldName) returns (dst: $Value)
{
    var r: $Reference;

    call r := $BorrowField(src, f);
    call dst := $ReadRef(r);
}

procedure {:inline 1} $GetFieldFromValue(src: $Value, f: $FieldName) returns (dst: $Value)
{
    dst := $select_vector(src, f); //breaks abstraction
}

procedure {:inline 1} $WriteRef(to: $Reference, new_v: $Value) returns (to': $Reference)
{
    to' := $Reference(l#$Reference(to), p#$Reference(to), new_v);
}

procedure {:inline 1} $ReadRef(from: $Reference) returns (v: $Value)
{
    v := v#$Reference(from);
}

procedure {:inline 1} $CopyOrMoveRef(local: $Reference) returns (dst: $Reference)
{
    dst := local;
}

procedure {:inline 1} $CopyOrMoveValue(local: $Value) returns (dst: $Value)
{
    dst := local;
}

procedure {:inline 1} $WritebackToGlobal(src: $Reference)
{
    var l: $Location;
    var v: $Value;

    l := l#$Reference(src);
    if (is#$Global(l)) {
        v := $UpdateValue(p#$Reference(src), 0, contents#$Memory($m)[l], v#$Reference(src));
        $m := $Memory(domain#$Memory($m), contents#$Memory($m)[l := v]);
    }
}

procedure {:inline 1} $WritebackToValue(src: $Reference, idx: int, vdst: $Value) returns (vdst': $Value)
{
    if (l#$Reference(src) == $Local(idx)) {
        vdst' := $UpdateValue(p#$Reference(src), 0, vdst, v#$Reference(src));
    } else {
        vdst' := vdst;
    }
}

procedure {:inline 1} $WritebackToReference(src: $Reference, dst: $Reference) returns (dst': $Reference)
{
    var srcPath, dstPath: $Path;

    srcPath := p#$Reference(src);
    dstPath := p#$Reference(dst);
    if (l#$Reference(dst) == l#$Reference(src) && size#$Path(dstPath) <= size#$Path(srcPath) && $IsPathPrefix(dstPath, srcPath)) {
        dst' := $Reference(
                    l#$Reference(dst),
                    dstPath,
                    $UpdateValue(srcPath, size#$Path(dstPath), v#$Reference(dst), v#$Reference(src)));
    } else {
        dst' := dst;
    }
}

procedure {:inline 1} $Splice1(idx1: int, src1: $Reference, dst: $Reference) returns (dst': $Reference) {
    assume l#$Reference(dst) == $Local(idx1);
    dst' := $Reference(l#$Reference(dst), $ConcatPath(p#$Reference(src1), p#$Reference(dst)), v#$Reference(dst));
}

procedure {:inline 1} $CastU8(src: $Value) returns (dst: $Value)
free requires is#$Integer(src);
{
    if (i#$Integer(src) > $MAX_U8) {
        $abort_flag := true;
        return;
    }
    dst := src;
}

procedure {:inline 1} $CastU64(src: $Value) returns (dst: $Value)
free requires is#$Integer(src);
{
    if (i#$Integer(src) > $MAX_U64) {
        $abort_flag := true;
        return;
    }
    dst := src;
}

procedure {:inline 1} $CastU128(src: $Value) returns (dst: $Value)
free requires is#$Integer(src);
{
    if (i#$Integer(src) > $MAX_U128) {
        $abort_flag := true;
        return;
    }
    dst := src;
}

procedure {:inline 1} $AddU8(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU8(src1) && $IsValidU8(src2);
{
    if (i#$Integer(src1) + i#$Integer(src2) > $MAX_U8) {
        $abort_flag := true;
        return;
    }
    dst := $Integer(i#$Integer(src1) + i#$Integer(src2));
}

procedure {:inline 1} $AddU64(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU64(src1) && $IsValidU64(src2);
{
    if (i#$Integer(src1) + i#$Integer(src2) > $MAX_U64) {
        $abort_flag := true;
        return;
    }
    dst := $Integer(i#$Integer(src1) + i#$Integer(src2));
}

procedure {:inline 1} $AddU128(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU128(src1) && $IsValidU128(src2);
{
    if (i#$Integer(src1) + i#$Integer(src2) > $MAX_U128) {
        $abort_flag := true;
        return;
    }
    dst := $Integer(i#$Integer(src1) + i#$Integer(src2));
}

procedure {:inline 1} $Sub(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Integer(src1) && is#$Integer(src2);
{
    if (i#$Integer(src1) < i#$Integer(src2)) {
        $abort_flag := true;
        return;
    }
    dst := $Integer(i#$Integer(src1) - i#$Integer(src2));
}

// This deals only with narrow special cases. Src2 must be constant
// 32 or 64, which is what we use now.  Obviously, it could be extended
// to src2 == any integer Value from 0..127.
// Left them out for brevity
function $power_of_2(power: $Value): int {
    (var p := i#$Integer(power);
     if p == 32 then 4294967296
     else if p == 64 then 18446744073709551616
     // Value is undefined, otherwise.
     else -1
     )
}

procedure {:inline 1} $Shl(src1: $Value, src2: $Value) returns (dst: $Value)
requires is#$Integer(src1) && is#$Integer(src2);
{
    var po2: int;
    po2 := $power_of_2(src2);
    assert po2 >= 1;   // po2 < 0 if src2 not 32 or 63
    dst := $Integer(i#$Integer(src2) * po2);
}

procedure {:inline 1} $Shr(src1: $Value, src2: $Value) returns (dst: $Value)
requires is#$Integer(src1) && is#$Integer(src2);
{
    var po2: int;
    po2 := $power_of_2(src2);
    assert po2 >= 1;   // po2 < 0 if src2 not 32 or 63
    dst := $Integer(i#$Integer(src2) div po2);
}

procedure {:inline 1} $MulU8(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU8(src1) && $IsValidU8(src2);
{
    if (i#$Integer(src1) * i#$Integer(src2) > $MAX_U8) {
        $abort_flag := true;
        return;
    }
    dst := $Integer(i#$Integer(src1) * i#$Integer(src2));
}

procedure {:inline 1} $MulU64(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU64(src1) && $IsValidU64(src2);
{
    if (i#$Integer(src1) * i#$Integer(src2) > $MAX_U64) {
        $abort_flag := true;
        return;
    }
    dst := $Integer(i#$Integer(src1) * i#$Integer(src2));
}

procedure {:inline 1} $MulU128(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU128(src1) && $IsValidU128(src2);
{
    if (i#$Integer(src1) * i#$Integer(src2) > $MAX_U128) {
        $abort_flag := true;
        return;
    }
    dst := $Integer(i#$Integer(src1) * i#$Integer(src2));
}

procedure {:inline 1} $Div(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Integer(src1) && is#$Integer(src2);
{
    if (i#$Integer(src2) == 0) {
        $abort_flag := true;
        return;
    }
    dst := $Integer(i#$Integer(src1) div i#$Integer(src2));
}

procedure {:inline 1} $Mod(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Integer(src1) && is#$Integer(src2);
{
    if (i#$Integer(src2) == 0) {
        $abort_flag := true;
        return;
    }
    dst := $Integer(i#$Integer(src1) mod i#$Integer(src2));
}

procedure {:inline 1} $ArithBinaryUnimplemented(src1: $Value, src2: $Value) returns (dst: $Value);
free requires is#$Integer(src1) && is#$Integer(src2);
ensures is#$Integer(dst);

procedure {:inline 1} $Lt(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Integer(src1) && is#$Integer(src2);
{
    dst := $Boolean(i#$Integer(src1) < i#$Integer(src2));
}

procedure {:inline 1} $Gt(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Integer(src1) && is#$Integer(src2);
{
    dst := $Boolean(i#$Integer(src1) > i#$Integer(src2));
}

procedure {:inline 1} $Le(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Integer(src1) && is#$Integer(src2);
{
    dst := $Boolean(i#$Integer(src1) <= i#$Integer(src2));
}

procedure {:inline 1} $Ge(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Integer(src1) && is#$Integer(src2);
{
    dst := $Boolean(i#$Integer(src1) >= i#$Integer(src2));
}

procedure {:inline 1} $And(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Boolean(src1) && is#$Boolean(src2);
{
    dst := $Boolean(b#$Boolean(src1) && b#$Boolean(src2));
}

procedure {:inline 1} $Or(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Boolean(src1) && is#$Boolean(src2);
{
    dst := $Boolean(b#$Boolean(src1) || b#$Boolean(src2));
}

procedure {:inline 1} $Not(src: $Value) returns (dst: $Value)
free requires is#$Boolean(src);
{
    dst := $Boolean(!b#$Boolean(src));
}

// Pack and Unpack are auto-generated for each type T


// Transaction
// -----------

type {:datatype} $Transaction;
var $txn: $Transaction;
function {:constructor} $Transaction(sender: int) : $Transaction;


// ==================================================================================
// Native Vector Type

function {:inline} $Vector_type_value(tv: $TypeValue): $TypeValue {
    $VectorType(tv)
}



// This is uses the implementation of $ValueArray using integer maps
function {:inline} $Vector_is_well_formed(v: $Value): bool {
    is#$Vector(v) &&
    (
        var va := v#$Vector(v);
        (
            var l := l#$ValueArray(va);
            0 <= l && l <= $MAX_U64 &&
            (forall x: int :: {v#$ValueArray(va)[x]} x < 0 || x >= l ==> v#$ValueArray(va)[x] == $DefaultValue())
        )
    )
}



procedure {:inline 1} $Vector_empty(ta: $TypeValue) returns (v: $Value) {
    v := $mk_vector();
}

procedure {:inline 1} $Vector_is_empty(ta: $TypeValue, v: $Value) returns (b: $Value) {
    assume is#$Vector(v);
    b := $Boolean($vlen(v) == 0);
}

procedure {:inline 1} $Vector_push_back(ta: $TypeValue, v: $Value, val: $Value) returns (v': $Value) {
    assume is#$Vector(v);
    v' := $push_back_vector(v, val);
}

procedure {:inline 1} $Vector_pop_back(ta: $TypeValue, v: $Value) returns (e: $Value, v': $Value) {
    var len: int;
    assume is#$Vector(v);
    len := $vlen(v);
    if (len == 0) {
        $abort_flag := true;
        return;
    }
    e := $select_vector(v, len-1);
    v' := $pop_back_vector(v);
}

procedure {:inline 1} $Vector_append(ta: $TypeValue, v: $Value, other: $Value) returns (v': $Value) {
    assume is#$Vector(v);
    assume is#$Vector(other);
    v' := $append_vector(v, other);
}

procedure {:inline 1} $Vector_reverse(ta: $TypeValue, v: $Value) returns (v': $Value) {
    assume is#$Vector(v);
    v' := $reverse_vector(v);
}

procedure {:inline 1} $Vector_length(ta: $TypeValue, v: $Value) returns (l: $Value) {
    assume is#$Vector(v);
    l := $Integer($vlen(v));
}

procedure {:inline 1} $Vector_borrow(ta: $TypeValue, src: $Value, i: $Value) returns (dst: $Value) {
    var i_ind: int;

    assume is#$Vector(src);
    assume is#$Integer(i);
    i_ind := i#$Integer(i);
    if (i_ind < 0 || i_ind >= $vlen(src)) {
        $abort_flag := true;
        return;
    }
    dst := $select_vector(src, i_ind);
}

procedure {:inline 1} $Vector_borrow_mut(ta: $TypeValue, v: $Value, index: $Value) returns (dst: $Reference, v': $Value)
free requires is#$Integer(index);
{
    var i_ind: int;

    i_ind := i#$Integer(index);
    assume is#$Vector(v);
    if (i_ind < 0 || i_ind >= $vlen(v)) {
        $abort_flag := true;
        return;
    }
    dst := $Reference($Local(0), $Path(p#$Path($EmptyPath)[0 := i_ind], 1), $select_vector(v, i_ind));
    v' := v;
}

procedure {:inline 1} $Vector_destroy_empty(ta: $TypeValue, v: $Value) {
    if ($vlen(v) != 0) {
      $abort_flag := true;
    }
}

procedure {:inline 1} $Vector_swap(ta: $TypeValue, v: $Value, i: $Value, j: $Value) returns (v': $Value)
free requires is#$Integer(i) && is#$Integer(j);
{
    var i_ind: int;
    var j_ind: int;
    assume is#$Vector(v);
    i_ind := i#$Integer(i);
    j_ind := i#$Integer(j);
    if (i_ind >= $vlen(v) || j_ind >= $vlen(v) || i_ind < 0 || j_ind < 0) {
        $abort_flag := true;
        return;
    }
    v' := $swap_vector(v, i_ind, j_ind);
}

procedure {:inline 1} $Vector_remove(ta: $TypeValue, v: $Value, i: $Value) returns (e: $Value, v': $Value)
free requires is#$Integer(i);
{
    var i_ind: int;

    assume is#$Vector(v);
    i_ind := i#$Integer(i);
    if (i_ind < 0 || i_ind >= $vlen(v)) {
        $abort_flag := true;
        return;
    }
    e := $select_vector(v, i_ind);
    v' := $remove_vector(v, i_ind);
}

procedure {:inline 1} $Vector_swap_remove(ta: $TypeValue, v: $Value, i: $Value) returns (e: $Value, v': $Value)
free requires is#$Integer(i);
{
    var i_ind: int;
    var len: int;

    assume is#$Vector(v);
    i_ind := i#$Integer(i);
    len := $vlen(v);
    if (i_ind < 0 || i_ind >= len) {
        $abort_flag := true;
        return;
    }
    e := $select_vector(v, i_ind);
    v' := $pop_back_vector($swap_vector(v, i_ind, len-1));
}

procedure {:inline 1} $Vector_contains(ta: $TypeValue, v: $Value, e: $Value) returns (res: $Value)  {
    assume is#$Vector(v);
    res := $Boolean($contains_vector(v, e));
}

// FIXME: This procedure sometimes (not always) make the test (performance_200511) very slow (> 10 mins) or hang
// although this is not used in the test script (performance_200511). The test finishes in 20 secs when it works fine.
procedure {:inline 1} $Vector_index_of(ta: $TypeValue, v: $Value, e: $Value) returns (res1: $Value, res2: $Value);
requires is#$Vector(v);
ensures is#$Boolean(res1);
ensures is#$Integer(res2);
ensures 0 <= i#$Integer(res2) && i#$Integer(res2) < $vlen(v);
ensures res1 == $Boolean($contains_vector(v, e));
ensures b#$Boolean(res1) ==> $IsEqual($select_vector(v,i#$Integer(res2)), e);
ensures b#$Boolean(res1) ==> (forall i:int :: 0<=i && i<i#$Integer(res2) ==> !$IsEqual($select_vector(v,i), e));
ensures !b#$Boolean(res1) ==> i#$Integer(res2) == 0;

// FIXME: This alternative definition has the same issue as the other one above.
// TODO: Delete this when unnecessary
//procedure {:inline 1} $Vector_index_of(ta: $TypeValue, v: $Value, e: $Value) returns (res1: $Value, res2: $Value) {
//    var b: bool;
//    var i: int;
//    assume is#$Vector(v);
//    b := $contains_vector(v, e);
//    if (b) {
//        havoc i;
//        assume 0 <= i && i < $vlen(v);
//        assume $IsEqual($select_vector(v,i), e);
//        assume (forall j:int :: 0<=j && j<i ==> !$IsEqual($select_vector(v,j), e));
//    }
//    else {
//        i := 0;
//    }
//    res1 := $Boolean(b);
//    res2 := $Integer(i);
//}

// ==================================================================================
// Native hash

// Hash is modeled as an otherwise uninterpreted injection.
// In truth, it is not an injection since the domain has greater cardinality
// (arbitrary length vectors) than the co-domain (vectors of length 32).  But it is
// common to assume in code there are no hash collisions in practice.  Fortunately,
// Boogie is not smart enough to recognized that there is an inconsistency.
// FIXME: If we were using a reliable extensional theory of arrays, and if we could use ==
// instead of $IsEqual, we might be able to avoid so many quantified formulas by
// using a sha2_inverse function in the ensures conditions of Hash_sha2_256 to
// assert that sha2/3 are injections without using global quantified axioms.


function {:inline} $Hash_sha2($m: $Memory, $txn: $Transaction, val: $Value): $Value {
    $Hash_sha2_core(val)
}

function $Hash_sha2_core(val: $Value): $Value;

// This says that Hash_sha2 respects isEquals (this would be automatic if we had an
// extensional theory of arrays and used ==, which has the substitution property
// for functions).
axiom (forall v1,v2: $Value :: $Vector_is_well_formed(v1) && $Vector_is_well_formed(v2)
       && $IsEqual(v1, v2) ==> $IsEqual($Hash_sha2_core(v1), $Hash_sha2_core(v2)));

// This says that Hash_sha2 is an injection
axiom (forall v1,v2: $Value :: $Vector_is_well_formed(v1) && $Vector_is_well_formed(v2)
        && $IsEqual($Hash_sha2_core(v1), $Hash_sha2_core(v2)) ==> $IsEqual(v1, v2));

// This procedure has no body. We want Boogie to just use its requires
// and ensures properties when verifying code that calls it.
procedure $Hash_sha2_256(val: $Value) returns (res: $Value);
// It will still work without this, but this helps verifier find more reasonable counterexamples.
free requires $IsValidU8Vector(val);
ensures res == $Hash_sha2_core(val);     // returns Hash_sha2 Value
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.
ensures $vlen(res) == 32;               // result is 32 bytes.

// similarly for Hash_sha3
function {:inline} $Hash_sha3($m: $Memory, $txn: $Transaction, val: $Value): $Value {
    $Hash_sha3_core(val)
}
function $Hash_sha3_core(val: $Value): $Value;

axiom (forall v1,v2: $Value :: $Vector_is_well_formed(v1) && $Vector_is_well_formed(v2)
       && $IsEqual(v1, v2) ==> $IsEqual($Hash_sha3_core(v1), $Hash_sha3_core(v2)));

axiom (forall v1,v2: $Value :: $Vector_is_well_formed(v1) && $Vector_is_well_formed(v2)
        && $IsEqual($Hash_sha3_core(v1), $Hash_sha3_core(v2)) ==> $IsEqual(v1, v2));

procedure $Hash_sha3_256(val: $Value) returns (res: $Value);
ensures res == $Hash_sha3_core(val);     // returns Hash_sha3 Value
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.
ensures $vlen(res) == 32;               // result is 32 bytes.

// ==================================================================================
// Native libra_account

// TODO: this function clashes with a similar version in older libraries. This is solved by a hack where the
// translator appends _OLD to the name when encountering this. The hack shall be removed once old library
// sources are not longer used.
procedure {:inline 1} $LibraAccount_save_account_OLD(ta: $TypeValue, balance: $Value, account: $Value, addr: $Value) {
    var a: int;
    var t_T: $TypeValue;
    var l_T: $Location;
    var t_Balance: $TypeValue;
    var l_Balance: $Location;

    a := a#$Address(addr);
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

    l_T := $Global(t_T, a);
    l_Balance := $Global(t_Balance, a);
    $m := $Memory(domain#$Memory($m)[l_T := true][l_Balance := true], contents#$Memory($m)[l_T := account][l_Balance := balance]);
}

procedure {:inline 1} $LibraAccount_save_account(
       t_Token: $TypeValue, t_AT: $TypeValue, account_type: $Value, balance: $Value,
       account: $Value, event_generator: $Value, addr: $Value) {
    // TODO: implement this
    assert false;
}

procedure {:inline 1} $LibraAccount_create_signer(
  addr: $Value
) returns (signer: $Value) {
    // A signer is currently identical to an address.
    signer := addr;
}

procedure {:inline 1} $LibraAccount_destroy_signer(
  signer: $Value
) {
  return;
}

procedure {:inline 1} $LibraAccount_write_to_event_store(ta: $TypeValue, guid: $Value, count: $Value, msg: $Value) {
    // TODO: this is used in old library sources, remove it once those sources are not longer used in tests.
    // This function is modeled as a no-op because the actual side effect of this native function is not observable from the Move side.
}

procedure {:inline 1} $Event_write_to_event_store(ta: $TypeValue, guid: $Value, count: $Value, msg: $Value) {
    // This function is modeled as a no-op because the actual side effect of this native function is not observable from the Move side.
}

// ==================================================================================
// Native signer

procedure {:inline 1} $Signer_borrow_address(signer: $Value) returns (res: $Value)
    free requires is#$Address(signer);
{
    res := signer;
}

// TODO: implement the below methods
// ==================================================================================
// Native signature

// TODO: implement the below methods

procedure {:inline 1} $Signature_ed25519_validate_pubkey(public_key: $Value) returns (res: $Value) {
    assert false; // $Signature_ed25519_validate_pubkey not implemented
}

procedure {:inline 1} $Signature_ed25519_verify(signature: $Value, public_key: $Value, message: $Value) returns (res: $Value) {
    assert false; // $Signature_ed25519_verify not implemented
}

procedure {:inline 1} Signature_ed25519_threshold_verify(bitmap: $Value, signature: $Value, public_key: $Value, message: $Value) returns (res: $Value) {
    assert false; // Signature_ed25519_threshold_verify not implemented
}

// ==================================================================================
// Native LCS::serialize

// native define serialize<MoveValue>(v: &MoveValue): vector<u8>;

// Serialize is modeled as an uninterpreted function, with an additional
// axiom to say it's an injection.

function {:inline} $LCS_serialize($m: $Memory, $txn: $Transaction, ta: $TypeValue, v: $Value): $Value {
    $LCS_serialize_core(v)
}

function $LCS_serialize_core(v: $Value): $Value;
function $LCS_serialize_core_inv(v: $Value): $Value;
// Needed only because IsEqual(v1, v2) is weaker than v1 == v2 in case there is a vector nested inside v1 or v2.
axiom (forall v1, v2: $Value :: $IsEqual(v1, v2) ==> $LCS_serialize_core(v1) == $LCS_serialize_core(v2));
// Injectivity
axiom (forall v: $Value :: $LCS_serialize_core_inv($LCS_serialize_core(v)) == v);

// This says that serialize returns a non-empty vec<u8>

axiom (forall v: $Value :: ( var r := $LCS_serialize_core(v); $IsValidU8Vector(r) && $vlen(r) > 0 &&
                            $vlen(r) <= 4 ));


// Serialized addresses should have the same length
const $serialized_address_len: int;
axiom (forall v: $Value :: (var r := $LCS_serialize_core(v); is#$Address(v) ==> $vlen(r) == $serialized_address_len));

procedure $LCS_to_bytes(ta: $TypeValue, v: $Value) returns (res: $Value);
ensures res == $LCS_serialize($m, $txn, ta, v);
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.

// ==================================================================================
// Native Signer::get_address
function $Signer_get_address($m: $Memory, $txn: $Transaction, signer: $Value): $Value
{
    // A signer is currently identical to an address.
    signer
}



// ** spec vars of module Signer



// ** spec funs of module Signer



// ** structs of module Signer



// ** functions of module Signer

procedure {:inline 1} $Signer_address_of (s: $Value) returns ($ret0: $Value)
free requires is#$Address(s);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 407, 0, s); }

    // bytecode translation starts here
    // $t4 := move(s)
    call $tmp := $CopyOrMoveValue(s);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := Signer::borrow_address($t1)
    call $t2 := $Signer_borrow_address($t1);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 324);
      goto Abort;
    }
    assume is#$Address($t2);


    // $t3 := move($t2)
    call $tmp := $CopyOrMoveValue($t2);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(14, 460, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}



// ** spec vars of module CoreAddresses



// ** spec funs of module CoreAddresses

function {:inline} $CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS(): $Value {
    $Address(173345816)
}

function {:inline} $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS(): $Value {
    $Address(173345816)
}

function {:inline} $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS(): $Value {
    $Address(186537453)
}

function {:inline} $CoreAddresses_SPEC_VM_RESERVED_ADDRESS(): $Value {
    $Address(0)
}

function {:inline} $CoreAddresses_SPEC_TRANSACTION_FEE_ADDRESS(): $Value {
    $Address(4078)
}

function {:inline} $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS(): $Value {
    $Address(989845)
}



// ** structs of module CoreAddresses



// ** functions of module CoreAddresses

procedure {:inline 1} $CoreAddresses_ASSOCIATION_ROOT_ADDRESS () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0xa550c18
    $tmp := $Address(173345816);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(4, 336, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $CoreAddresses_CURRENCY_INFO_ADDRESS () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0xa550c18
    $tmp := $Address(173345816);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(4, 914, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $CoreAddresses_DEFAULT_CONFIG_ADDRESS () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0xf1a95
    $tmp := $Address(989845);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(4, 2810, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $CoreAddresses_TRANSACTION_FEE_ADDRESS () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0xfee
    $tmp := $Address(4078);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(4, 2372, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $CoreAddresses_TREASURY_COMPLIANCE_ADDRESS () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0xb1e55ed
    $tmp := $Address(186537453);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(4, 1382, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $CoreAddresses_VM_RESERVED_ADDRESS () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0x0
    $tmp := $Address(0);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(4, 1918, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}



// ** spec vars of module LibraTimestamp



// ** spec funs of module LibraTimestamp

function {:inline} $LibraTimestamp_spec_is_genesis($m: $Memory, $txn: $Transaction): $Value {
    $Boolean(!b#$Boolean($ResourceExists($m, $LibraTimestamp_TimeHasStarted_type_value(), $CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS())))
}

function {:inline} $LibraTimestamp_root_ctm_initialized($m: $Memory, $txn: $Transaction): $Value {
    $ResourceExists($m, $LibraTimestamp_CurrentTimeMicroseconds_type_value(), $CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS())
}

function {:inline} $LibraTimestamp_assoc_unix_time($m: $Memory, $txn: $Transaction): $Value {
    $SelectField($ResourceValue($m, $LibraTimestamp_CurrentTimeMicroseconds_type_value(), $CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS()), $LibraTimestamp_CurrentTimeMicroseconds_microseconds)
}



// ** structs of module LibraTimestamp

const unique $LibraTimestamp_CurrentTimeMicroseconds: $TypeName;
const $LibraTimestamp_CurrentTimeMicroseconds_microseconds: $FieldName;
axiom $LibraTimestamp_CurrentTimeMicroseconds_microseconds == 0;
function $LibraTimestamp_CurrentTimeMicroseconds_type_value(): $TypeValue {
    $StructType($LibraTimestamp_CurrentTimeMicroseconds, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $IntegerType()], 1))
}
function {:inline} $LibraTimestamp_CurrentTimeMicroseconds_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $IsValidU64($SelectField($this, $LibraTimestamp_CurrentTimeMicroseconds_microseconds))
}
function {:inline} $LibraTimestamp_CurrentTimeMicroseconds_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $IsValidU64($SelectField($this, $LibraTimestamp_CurrentTimeMicroseconds_microseconds))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $LibraTimestamp_CurrentTimeMicroseconds_is_well_formed($ResourceValue(m, $LibraTimestamp_CurrentTimeMicroseconds_type_value(), a))
);

procedure {:inline 1} $LibraTimestamp_CurrentTimeMicroseconds_pack($file_id: int, $byte_index: int, $var_idx: int, microseconds: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(microseconds);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := microseconds], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $LibraTimestamp_CurrentTimeMicroseconds_unpack($struct: $Value) returns (microseconds: $Value)
{
    assume is#$Vector($struct);
    microseconds := $SelectField($struct, $LibraTimestamp_CurrentTimeMicroseconds_microseconds);
    assume $IsValidU64(microseconds);
}

const unique $LibraTimestamp_TimeHasStarted: $TypeName;
const $LibraTimestamp_TimeHasStarted_dummy_field: $FieldName;
axiom $LibraTimestamp_TimeHasStarted_dummy_field == 0;
function $LibraTimestamp_TimeHasStarted_type_value(): $TypeValue {
    $StructType($LibraTimestamp_TimeHasStarted, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $LibraTimestamp_TimeHasStarted_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $LibraTimestamp_TimeHasStarted_dummy_field))
}
function {:inline} $LibraTimestamp_TimeHasStarted_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $LibraTimestamp_TimeHasStarted_dummy_field))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $LibraTimestamp_TimeHasStarted_is_well_formed($ResourceValue(m, $LibraTimestamp_TimeHasStarted_type_value(), a))
);

procedure {:inline 1} $LibraTimestamp_TimeHasStarted_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $LibraTimestamp_TimeHasStarted_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $LibraTimestamp_TimeHasStarted_dummy_field);
    assume is#$Boolean(dummy_field);
}



// ** functions of module LibraTimestamp

procedure {:inline 1} $LibraTimestamp_assert_is_genesis () returns ()
requires $ExistsTxnSenderAccount($m, $txn);
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn))))) ==> b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($LibraTimestamp_root_ctm_initialized($m, $txn))) ==> b#$Boolean($LibraTimestamp_root_ctm_initialized($m, $txn)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($LibraTimestamp_root_ctm_initialized($m, $txn))) ==> b#$Boolean($Boolean(i#$Integer(old($LibraTimestamp_assoc_unix_time($m, $txn))) <= i#$Integer($LibraTimestamp_assoc_unix_time($m, $txn)))))));
{
    // declare local variables
    var $t0: $Value; // $BooleanType()
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t2 := LibraTimestamp::is_genesis()
    call $t2 := $LibraTimestamp_is_genesis();
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 3287);
      goto Abort;
    }
    assume is#$Boolean($t2);


    // $t0 := $t2
    call $tmp := $CopyOrMoveValue($t2);
    $t0 := $tmp;
    if (true) { assume $DebugTrackLocal(16, 3554, 0, $tmp); }

    // if ($t0) goto L0 else goto L1
    $tmp := $t0;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // return ()
    return;

    // L2:
L2:

    // $t4 := 2
    $tmp := $Integer(2);
    $t4 := $tmp;

    // abort($t4)
    if (true) { assume $DebugTrackAbort(16, 3554); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure $LibraTimestamp_assert_is_genesis_verify () returns ()
{
    call $InitVerification();
    assume $Memory__is_well_formed($m);
    assume $ExistsTxnSenderAccount($m, $txn);
    call $LibraTimestamp_assert_is_genesis();
}

procedure {:inline 1} $LibraTimestamp_initialize (association: $Value) returns ()
free requires is#$Address(association);
requires $ExistsTxnSenderAccount($m, $txn);
ensures b#$Boolean(old($Boolean(!$IsEqual($Signer_get_address($m, $txn, association), $CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS())))) ==> $abort_flag;
ensures b#$Boolean(old($LibraTimestamp_root_ctm_initialized($m, $txn))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(!$IsEqual($Signer_get_address($m, $txn, association), $CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS())))))
    || b#$Boolean(old(($LibraTimestamp_root_ctm_initialized($m, $txn)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn))))) ==> b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($LibraTimestamp_root_ctm_initialized($m, $txn))) ==> b#$Boolean($LibraTimestamp_root_ctm_initialized($m, $txn)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($LibraTimestamp_root_ctm_initialized($m, $txn))) ==> b#$Boolean($Boolean(i#$Integer(old($LibraTimestamp_assoc_unix_time($m, $txn))) <= i#$Integer($LibraTimestamp_assoc_unix_time($m, $txn)))))));
ensures !$abort_flag ==> (b#$Boolean($LibraTimestamp_root_ctm_initialized($m, $txn)));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($LibraTimestamp_assoc_unix_time($m, $txn), $Integer(0)))));
{
    // declare local variables
    var timer: $Value; // $LibraTimestamp_CurrentTimeMicroseconds_type_value()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $LibraTimestamp_CurrentTimeMicroseconds_type_value()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(16, 1066, 0, association); }

    // bytecode translation starts here
    // $t14 := move(association)
    call $tmp := $CopyOrMoveValue(association);
    $t14 := $tmp;

    // $t4 := copy($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t4 := $tmp;

    // $t5 := Signer::address_of($t4)
    call $t5 := $Signer_address_of($t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 1211);
      goto Abort;
    }
    assume is#$Address($t5);


    // $t6 := CoreAddresses::ASSOCIATION_ROOT_ADDRESS()
    call $t6 := $CoreAddresses_ASSOCIATION_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 1253);
      goto Abort;
    }
    assume is#$Address($t6);


    // $t7 := ==($t5, $t6)
    $tmp := $Boolean($IsEqual($t5, $t6));
    $t7 := $tmp;

    // $t2 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(16, 1196, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t9 := move($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t9 := $tmp;

    // $t10 := 0
    $tmp := $Integer(0);
    $t10 := $tmp;

    // $t11 := pack LibraTimestamp::CurrentTimeMicroseconds($t10)
    call $tmp := $LibraTimestamp_CurrentTimeMicroseconds_pack(0, 0, 0, $t10);
    $t11 := $tmp;

    // move_to<LibraTimestamp::CurrentTimeMicroseconds>($t11, $t9)
    call $MoveTo($LibraTimestamp_CurrentTimeMicroseconds_type_value(), $t11, $t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 1430);
      goto Abort;
    }

    // return ()
    return;

    // L2:
L2:

    // $t12 := move($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t12 := $tmp;

    // destroy($t12)

    // $t13 := 1
    $tmp := $Integer(1);
    $t13 := $tmp;

    // abort($t13)
    if (true) { assume $DebugTrackAbort(16, 1196); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure $LibraTimestamp_initialize_verify (association: $Value) returns ()
free requires is#$Address(association);
{
    call $InitVerification();
    assume $Memory__is_well_formed($m);
    assume $ExistsTxnSenderAccount($m, $txn);
    call $LibraTimestamp_initialize(association);
}

procedure {:inline 1} $LibraTimestamp_is_genesis () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(false)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn))))) ==> b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($LibraTimestamp_root_ctm_initialized($m, $txn))) ==> b#$Boolean($LibraTimestamp_root_ctm_initialized($m, $txn)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($LibraTimestamp_root_ctm_initialized($m, $txn))) ==> b#$Boolean($Boolean(i#$Integer(old($LibraTimestamp_assoc_unix_time($m, $txn))) <= i#$Integer($LibraTimestamp_assoc_unix_time($m, $txn)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $LibraTimestamp_spec_is_genesis($m, $txn)))));
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $BooleanType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::ASSOCIATION_ROOT_ADDRESS()
    call $t0 := $CoreAddresses_ASSOCIATION_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 3355);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := exists<LibraTimestamp::TimeHasStarted>($t0)
    call $tmp := $Exists($t0, $LibraTimestamp_TimeHasStarted_type_value());
    $t1 := $tmp;

    // $t2 := !($t1)
    call $tmp := $Not($t1);
    $t2 := $tmp;

    // return $t2
    $ret0 := $t2;
    if (true) { assume $DebugTrackLocal(16, 3316, 3, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure $LibraTimestamp_is_genesis_verify () returns ($ret0: $Value)
{
    call $InitVerification();
    assume $Memory__is_well_formed($m);
    assume $ExistsTxnSenderAccount($m, $txn);
    call $ret0 := $LibraTimestamp_is_genesis();
}

procedure {:inline 1} $LibraTimestamp_now_microseconds () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($m, $LibraTimestamp_CurrentTimeMicroseconds_type_value(), $CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS()))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(!b#$Boolean($ResourceExists($m, $LibraTimestamp_CurrentTimeMicroseconds_type_value(), $CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS())))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn))))) ==> b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($LibraTimestamp_root_ctm_initialized($m, $txn))) ==> b#$Boolean($LibraTimestamp_root_ctm_initialized($m, $txn)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($LibraTimestamp_root_ctm_initialized($m, $txn))) ==> b#$Boolean($Boolean(i#$Integer(old($LibraTimestamp_assoc_unix_time($m, $txn))) <= i#$Integer($LibraTimestamp_assoc_unix_time($m, $txn)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $LibraTimestamp_assoc_unix_time($m, $txn)))));
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $LibraTimestamp_CurrentTimeMicroseconds_type_value()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::ASSOCIATION_ROOT_ADDRESS()
    call $t0 := $CoreAddresses_ASSOCIATION_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 3148);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := get_global<LibraTimestamp::CurrentTimeMicroseconds>($t0)
    call $tmp := $GetGlobal($t0, $LibraTimestamp_CurrentTimeMicroseconds_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 3094);
      goto Abort;
    }
    assume $LibraTimestamp_CurrentTimeMicroseconds_is_well_formed($tmp);
    $t1 := $tmp;

    // $t2 := get_field<LibraTimestamp::CurrentTimeMicroseconds>.microseconds($t1)
    call $tmp := $GetFieldFromValue($t1, $LibraTimestamp_CurrentTimeMicroseconds_microseconds);
    assume $IsValidU64($tmp);
    $t2 := $tmp;

    // $t3 := move($t2)
    call $tmp := $CopyOrMoveValue($t2);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(16, 3094, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure $LibraTimestamp_now_microseconds_verify () returns ($ret0: $Value)
{
    call $InitVerification();
    assume $Memory__is_well_formed($m);
    assume $ExistsTxnSenderAccount($m, $txn);
    call $ret0 := $LibraTimestamp_now_microseconds();
}

procedure {:inline 1} $LibraTimestamp_set_time_has_started (association: $Value) returns ()
free requires is#$Address(association);
requires $ExistsTxnSenderAccount($m, $txn);
ensures b#$Boolean(old($Boolean(!$IsEqual($Signer_get_address($m, $txn, association), $CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS())))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($LibraTimestamp_root_ctm_initialized($m, $txn))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!$IsEqual($LibraTimestamp_assoc_unix_time($m, $txn), $Integer(0))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(!$IsEqual($Signer_get_address($m, $txn, association), $CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS())))))
    || b#$Boolean(old(($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn))))))
    || b#$Boolean(old(($Boolean(!b#$Boolean($LibraTimestamp_root_ctm_initialized($m, $txn))))))
    || b#$Boolean(old(($Boolean(!$IsEqual($LibraTimestamp_assoc_unix_time($m, $txn), $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn))))) ==> b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($LibraTimestamp_root_ctm_initialized($m, $txn))) ==> b#$Boolean($LibraTimestamp_root_ctm_initialized($m, $txn)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($LibraTimestamp_root_ctm_initialized($m, $txn))) ==> b#$Boolean($Boolean(i#$Integer(old($LibraTimestamp_assoc_unix_time($m, $txn))) <= i#$Integer($LibraTimestamp_assoc_unix_time($m, $txn)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))));
{
    // declare local variables
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $BooleanType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $IntegerType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $BooleanType()
    var $t18: $Value; // $BooleanType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $BooleanType()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $BooleanType()
    var $t23: $Value; // $LibraTimestamp_TimeHasStarted_type_value()
    var $t24: $Value; // $AddressType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(16, 1570, 0, association); }

    // bytecode translation starts here
    // $t26 := move(association)
    call $tmp := $CopyOrMoveValue(association);
    $t26 := $tmp;

    // $t6 := copy($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t6 := $tmp;

    // $t7 := Signer::address_of($t6)
    call $t7 := $Signer_address_of($t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 1682);
      goto Abort;
    }
    assume is#$Address($t7);


    // $t8 := CoreAddresses::ASSOCIATION_ROOT_ADDRESS()
    call $t8 := $CoreAddresses_ASSOCIATION_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 1724);
      goto Abort;
    }
    assume is#$Address($t8);


    // $t9 := ==($t7, $t8)
    $tmp := $Boolean($IsEqual($t7, $t8));
    $t9 := $tmp;

    // $t1 := $t9
    call $tmp := $CopyOrMoveValue($t9);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(16, 1667, 1, $tmp); }

    // if ($t1) goto L0 else goto L1
    $tmp := $t1;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t11 := CoreAddresses::ASSOCIATION_ROOT_ADDRESS()
    call $t11 := $CoreAddresses_ASSOCIATION_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 1884);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := exists<LibraTimestamp::CurrentTimeMicroseconds>($t11)
    call $tmp := $Exists($t11, $LibraTimestamp_CurrentTimeMicroseconds_type_value());
    $t12 := $tmp;

    // if ($t12) goto L3 else goto L4
    $tmp := $t12;
    if (b#$Boolean($tmp)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L5
    goto L5;

    // L2:
L2:

    // $t13 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t13 := $tmp;

    // destroy($t13)

    // $t14 := 1
    $tmp := $Integer(1);
    $t14 := $tmp;

    // abort($t14)
    if (true) { assume $DebugTrackAbort(16, 1667); }
    goto Abort;

    // L3:
L3:

    // $t15 := LibraTimestamp::now_microseconds()
    call $t15 := $LibraTimestamp_now_microseconds();
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 3027);
      goto Abort;
    }
    assume $IsValidU64($t15);


    // $t16 := 0
    $tmp := $Integer(0);
    $t16 := $tmp;

    // $t17 := ==($t15, $t16)
    $tmp := $Boolean($IsEqual($t15, $t16));
    $t17 := $tmp;

    // $t5 := $t17
    call $tmp := $CopyOrMoveValue($t17);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(16, 1824, 5, $tmp); }

    // goto L6
    goto L6;

    // L5:
L5:

    // $t18 := false
    $tmp := $Boolean(false);
    $t18 := $tmp;

    // $t5 := $t18
    call $tmp := $CopyOrMoveValue($t18);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(16, 1824, 5, $tmp); }

    // goto L6
    goto L6;

    // L6:
L6:

    // $t3 := $t5
    call $tmp := $CopyOrMoveValue($t5);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(16, 1817, 3, $tmp); }

    // if ($t3) goto L7 else goto L8
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L7; } else { goto L8; }

    // L8:
L8:

    // goto L9
    goto L9;

    // L7:
L7:

    // $t21 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t21 := $tmp;

    // $t22 := false
    $tmp := $Boolean(false);
    $t22 := $tmp;

    // $t23 := pack LibraTimestamp::TimeHasStarted($t22)
    call $tmp := $LibraTimestamp_TimeHasStarted_pack(0, 0, 0, $t22);
    $t23 := $tmp;

    // move_to<LibraTimestamp::TimeHasStarted>($t23, $t21)
    call $MoveTo($LibraTimestamp_TimeHasStarted_type_value(), $t23, $t21);
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 1973);
      goto Abort;
    }

    // return ()
    return;

    // L9:
L9:

    // $t24 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t24 := $tmp;

    // destroy($t24)

    // $t25 := 2
    $tmp := $Integer(2);
    $t25 := $tmp;

    // abort($t25)
    if (true) { assume $DebugTrackAbort(16, 1817); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure $LibraTimestamp_set_time_has_started_verify (association: $Value) returns ()
free requires is#$Address(association);
{
    call $InitVerification();
    assume $Memory__is_well_formed($m);
    assume $ExistsTxnSenderAccount($m, $txn);
    call $LibraTimestamp_set_time_has_started(association);
}

procedure {:inline 1} $LibraTimestamp_update_global_time (account: $Value, proposer: $Value, timestamp: $Value) returns ()
free requires is#$Address(account);
free requires is#$Address(proposer);
free requires $IsValidU64(timestamp);
requires $ExistsTxnSenderAccount($m, $txn);
ensures b#$Boolean(old($Boolean(!$IsEqual($Signer_get_address($m, $txn, account), $CoreAddresses_SPEC_VM_RESERVED_ADDRESS())))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($LibraTimestamp_root_ctm_initialized($m, $txn))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual(proposer, $CoreAddresses_SPEC_VM_RESERVED_ADDRESS()))) && b#$Boolean($Boolean(!$IsEqual(timestamp, $LibraTimestamp_assoc_unix_time($m, $txn))))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean(!$IsEqual(proposer, $CoreAddresses_SPEC_VM_RESERVED_ADDRESS()))) && b#$Boolean($Boolean(!b#$Boolean($Boolean(i#$Integer(timestamp) > i#$Integer($LibraTimestamp_assoc_unix_time($m, $txn))))))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(!$IsEqual($Signer_get_address($m, $txn, account), $CoreAddresses_SPEC_VM_RESERVED_ADDRESS())))))
    || b#$Boolean(old(($Boolean(!b#$Boolean($LibraTimestamp_root_ctm_initialized($m, $txn))))))
    || b#$Boolean(old(($Boolean(b#$Boolean($Boolean($IsEqual(proposer, $CoreAddresses_SPEC_VM_RESERVED_ADDRESS()))) && b#$Boolean($Boolean(!$IsEqual(timestamp, $LibraTimestamp_assoc_unix_time($m, $txn))))))))
    || b#$Boolean(old(($Boolean(b#$Boolean($Boolean(!$IsEqual(proposer, $CoreAddresses_SPEC_VM_RESERVED_ADDRESS()))) && b#$Boolean($Boolean(!b#$Boolean($Boolean(i#$Integer(timestamp) > i#$Integer($LibraTimestamp_assoc_unix_time($m, $txn)))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn))))) ==> b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($LibraTimestamp_root_ctm_initialized($m, $txn))) ==> b#$Boolean($LibraTimestamp_root_ctm_initialized($m, $txn)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($LibraTimestamp_root_ctm_initialized($m, $txn))) ==> b#$Boolean($Boolean(i#$Integer(old($LibraTimestamp_assoc_unix_time($m, $txn))) <= i#$Integer($LibraTimestamp_assoc_unix_time($m, $txn)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($LibraTimestamp_assoc_unix_time($m, $txn), timestamp))));
{
    // declare local variables
    var global_timer: $Reference; // ReferenceType($LibraTimestamp_CurrentTimeMicroseconds_type_value())
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $AddressType()
    var $t16: $Reference; // ReferenceType($LibraTimestamp_CurrentTimeMicroseconds_type_value())
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $IntegerType()
    var $t22: $Reference; // ReferenceType($LibraTimestamp_CurrentTimeMicroseconds_type_value())
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $IntegerType()
    var $t25: $Value; // $BooleanType()
    var $t26: $Value; // $BooleanType()
    var $t27: $Reference; // ReferenceType($LibraTimestamp_CurrentTimeMicroseconds_type_value())
    var $t28: $Value; // $IntegerType()
    var $t29: $Reference; // ReferenceType($LibraTimestamp_CurrentTimeMicroseconds_type_value())
    var $t30: $Value; // $IntegerType()
    var $t31: $Value; // $IntegerType()
    var $t32: $Value; // $IntegerType()
    var $t33: $Value; // $BooleanType()
    var $t34: $Value; // $BooleanType()
    var $t35: $Reference; // ReferenceType($LibraTimestamp_CurrentTimeMicroseconds_type_value())
    var $t36: $Value; // $IntegerType()
    var $t37: $Value; // $IntegerType()
    var $t38: $Reference; // ReferenceType($LibraTimestamp_CurrentTimeMicroseconds_type_value())
    var $t39: $Reference; // ReferenceType($IntegerType())
    var $t40: $Value; // $AddressType()
    var $t41: $Value; // $AddressType()
    var $t42: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(16, 2139, 0, account); }
    if (true) { assume $DebugTrackLocal(16, 2139, 1, proposer); }
    if (true) { assume $DebugTrackLocal(16, 2139, 2, timestamp); }

    // bytecode translation starts here
    // $t40 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t40 := $tmp;

    // $t41 := move(proposer)
    call $tmp := $CopyOrMoveValue(proposer);
    $t41 := $tmp;

    // $t42 := move(timestamp)
    call $tmp := $CopyOrMoveValue(timestamp);
    $t42 := $tmp;

    // $t10 := move($t40)
    call $tmp := $CopyOrMoveValue($t40);
    $t10 := $tmp;

    // $t11 := Signer::address_of($t10)
    call $t11 := $Signer_address_of($t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 2363);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := CoreAddresses::VM_RESERVED_ADDRESS()
    call $t12 := $CoreAddresses_VM_RESERVED_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 2401);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := ==($t11, $t12)
    $tmp := $Boolean($IsEqual($t11, $t12));
    $t13 := $tmp;

    // $t4 := $t13
    call $tmp := $CopyOrMoveValue($t13);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(16, 2348, 4, $tmp); }

    // if ($t4) goto L0 else goto L1
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t15 := CoreAddresses::ASSOCIATION_ROOT_ADDRESS()
    call $t15 := $CoreAddresses_ASSOCIATION_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 2515);
      goto Abort;
    }
    assume is#$Address($t15);


    // $t16 := borrow_global<LibraTimestamp::CurrentTimeMicroseconds>($t15)
    call $t16 := $BorrowGlobal($t15, $LibraTimestamp_CurrentTimeMicroseconds_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 2457);
      goto Abort;
    }
    assume $LibraTimestamp_CurrentTimeMicroseconds_is_well_formed($Dereference($t16));

    // UnpackRef($t16)

    // global_timer := $t16
    call global_timer := $CopyOrMoveRef($t16);
    if (true) { assume $DebugTrackLocal(16, 2442, 3, $Dereference(global_timer)); }

    // $t18 := CoreAddresses::VM_RESERVED_ADDRESS()
    call $t18 := $CoreAddresses_VM_RESERVED_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 2583);
      goto Abort;
    }
    assume is#$Address($t18);


    // $t19 := ==($t41, $t18)
    $tmp := $Boolean($IsEqual($t41, $t18));
    $t19 := $tmp;

    // if ($t19) goto L3 else goto L4
    $tmp := $t19;
    if (b#$Boolean($tmp)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L5
    goto L5;

    // L2:
L2:

    // $t20 := 33
    $tmp := $Integer(33);
    $t20 := $tmp;

    // abort($t20)
    if (true) { assume $DebugTrackAbort(16, 2348); }
    goto Abort;

    // L3:
L3:

    // $t22 := copy(global_timer)
    call $t22 := $CopyOrMoveRef(global_timer);

    // $t23 := get_field<LibraTimestamp::CurrentTimeMicroseconds>.microseconds($t22)
    call $tmp := $GetFieldFromReference($t22, $LibraTimestamp_CurrentTimeMicroseconds_microseconds);
    assume $IsValidU64($tmp);
    $t23 := $tmp;

    // Reference(global_timer) <- $t22
    call global_timer := $WritebackToReference($t22, global_timer);

    // $t24 := move($t23)
    call $tmp := $CopyOrMoveValue($t23);
    $t24 := $tmp;

    // $t25 := ==($t42, $t24)
    $tmp := $Boolean($IsEqual($t42, $t24));
    $t25 := $tmp;

    // $t6 := $t25
    call $tmp := $CopyOrMoveValue($t25);
    $t6 := $tmp;
    if (true) { assume $DebugTrackLocal(16, 2701, 6, $tmp); }

    // if ($t6) goto L6 else goto L7
    $tmp := $t6;
    if (b#$Boolean($tmp)) { goto L6; } else { goto L7; }

    // L7:
L7:

    // goto L8
    goto L8;

    // L6:
L6:

    // goto L9
    goto L9;

    // L8:
L8:

    // $t27 := move(global_timer)
    call $t27 := $CopyOrMoveRef(global_timer);

    // destroy($t27)

    // LibraTimestamp::CurrentTimeMicroseconds <- $t27
    call $WritebackToGlobal($t27);

    // PackRef($t27)

    // $t28 := 5001
    $tmp := $Integer(5001);
    $t28 := $tmp;

    // abort($t28)
    if (true) { assume $DebugTrackAbort(16, 2701); }
    goto Abort;

    // L5:
L5:

    // $t29 := copy(global_timer)
    call $t29 := $CopyOrMoveRef(global_timer);

    // $t30 := get_field<LibraTimestamp::CurrentTimeMicroseconds>.microseconds($t29)
    call $tmp := $GetFieldFromReference($t29, $LibraTimestamp_CurrentTimeMicroseconds_microseconds);
    assume $IsValidU64($tmp);
    $t30 := $tmp;

    // Reference(global_timer) <- $t29
    call global_timer := $WritebackToReference($t29, global_timer);

    // $t31 := move($t30)
    call $tmp := $CopyOrMoveValue($t30);
    $t31 := $tmp;

    // $t33 := <($t31, $t42)
    call $tmp := $Lt($t31, $t42);
    $t33 := $tmp;

    // $t8 := $t33
    call $tmp := $CopyOrMoveValue($t33);
    $t8 := $tmp;
    if (true) { assume $DebugTrackLocal(16, 2831, 8, $tmp); }

    // if ($t8) goto L10 else goto L11
    $tmp := $t8;
    if (b#$Boolean($tmp)) { goto L10; } else { goto L11; }

    // L11:
L11:

    // goto L12
    goto L12;

    // L10:
L10:

    // goto L9
    goto L9;

    // L12:
L12:

    // $t35 := move(global_timer)
    call $t35 := $CopyOrMoveRef(global_timer);

    // destroy($t35)

    // LibraTimestamp::CurrentTimeMicroseconds <- $t35
    call $WritebackToGlobal($t35);

    // PackRef($t35)

    // $t36 := 5001
    $tmp := $Integer(5001);
    $t36 := $tmp;

    // abort($t36)
    if (true) { assume $DebugTrackAbort(16, 2831); }
    goto Abort;

    // L9:
L9:

    // $t38 := move(global_timer)
    call $t38 := $CopyOrMoveRef(global_timer);

    // $t39 := borrow_field<LibraTimestamp::CurrentTimeMicroseconds>.microseconds($t38)
    call $t39 := $BorrowField($t38, $LibraTimestamp_CurrentTimeMicroseconds_microseconds);
    assume $IsValidU64($Dereference($t39));

    // LibraTimestamp::CurrentTimeMicroseconds <- $t38
    call $WritebackToGlobal($t38);

    // UnpackRef($t39)

    // write_ref($t39, $t42)
    call $t39 := $WriteRef($t39, $t42);
    if (true) { assume $DebugTrackLocal(16, 2903, 3, $Dereference(global_timer)); }

    // LibraTimestamp::CurrentTimeMicroseconds <- $t39
    call $WritebackToGlobal($t39);

    // Reference($t38) <- $t39
    call $t38 := $WritebackToReference($t39, $t38);

    // PackRef($t38)

    // PackRef($t39)

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure $LibraTimestamp_update_global_time_verify (account: $Value, proposer: $Value, timestamp: $Value) returns ()
free requires is#$Address(account);
free requires is#$Address(proposer);
free requires $IsValidU64(timestamp);
{
    call $InitVerification();
    assume $Memory__is_well_formed($m);
    assume $ExistsTxnSenderAccount($m, $txn);
    call $LibraTimestamp_update_global_time(account, proposer, timestamp);
}



// ** spec vars of module Roles



// ** spec funs of module Roles

function {:inline} $Roles_spec_has_role_id($m: $Memory, $txn: $Transaction, addr: $Value): $Value {
    $ResourceExists($m, $Roles_RoleId_type_value(), addr)
}

function {:inline} $Roles_spec_get_role_id($m: $Memory, $txn: $Transaction, addr: $Value): $Value {
    $SelectField($ResourceValue($m, $Roles_RoleId_type_value(), addr), $Roles_RoleId_role_id)
}

function {:inline} $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID(): $Value {
    $Integer(0)
}

function {:inline} $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID(): $Value {
    $Integer(1)
}

function {:inline} $Roles_SPEC_DESIGNATED_DEALER_ROLE_ID(): $Value {
    $Integer(2)
}

function {:inline} $Roles_SPEC_VALIDATOR_ROLE_ID(): $Value {
    $Integer(3)
}

function {:inline} $Roles_SPEC_VALIDATOR_OPERATOR_ROLE_ID(): $Value {
    $Integer(4)
}

function {:inline} $Roles_SPEC_PARENT_VASP_ROLE_ID(): $Value {
    $Integer(5)
}

function {:inline} $Roles_SPEC_CHILD_VASP_ROLE_ID(): $Value {
    $Integer(6)
}

function {:inline} $Roles_SPEC_UNHOSTED_ROLE_ID(): $Value {
    $Integer(7)
}



// ** structs of module Roles

const unique $Roles_AssociationRootRole: $TypeName;
const $Roles_AssociationRootRole_dummy_field: $FieldName;
axiom $Roles_AssociationRootRole_dummy_field == 0;
function $Roles_AssociationRootRole_type_value(): $TypeValue {
    $StructType($Roles_AssociationRootRole, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $Roles_AssociationRootRole_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_AssociationRootRole_dummy_field))
}
function {:inline} $Roles_AssociationRootRole_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_AssociationRootRole_dummy_field))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Roles_AssociationRootRole_is_well_formed($ResourceValue(m, $Roles_AssociationRootRole_type_value(), a))
);

procedure {:inline 1} $Roles_AssociationRootRole_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Roles_AssociationRootRole_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $Roles_AssociationRootRole_dummy_field);
    assume is#$Boolean(dummy_field);
}

const unique $Roles_Capability: $TypeName;
const $Roles_Capability_owner_address: $FieldName;
axiom $Roles_Capability_owner_address == 0;
function $Roles_Capability_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Roles_Capability, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $AddressType()], 1))
}
function {:inline} $Roles_Capability_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Address($SelectField($this, $Roles_Capability_owner_address))
}
function {:inline} $Roles_Capability_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Address($SelectField($this, $Roles_Capability_owner_address))
}

axiom (forall m: $Memory, a: $Value, $tv0: $TypeValue :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Roles_Capability_is_well_formed($ResourceValue(m, $Roles_Capability_type_value($tv0), a))
);

procedure {:inline 1} $Roles_Capability_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, owner_address: $Value) returns ($struct: $Value)
{
    assume is#$Address(owner_address);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := owner_address], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Roles_Capability_unpack($tv0: $TypeValue, $struct: $Value) returns (owner_address: $Value)
{
    assume is#$Vector($struct);
    owner_address := $SelectField($struct, $Roles_Capability_owner_address);
    assume is#$Address(owner_address);
}

const unique $Roles_Privilege: $TypeName;
const $Roles_Privilege_witness: $FieldName;
axiom $Roles_Privilege_witness == 0;
const $Roles_Privilege_is_extracted: $FieldName;
axiom $Roles_Privilege_is_extracted == 1;
function $Roles_Privilege_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Roles_Privilege, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0][1 := $BooleanType()], 2))
}
function {:inline} $Roles_Privilege_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && is#$Boolean($SelectField($this, $Roles_Privilege_is_extracted))
}
function {:inline} $Roles_Privilege_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && is#$Boolean($SelectField($this, $Roles_Privilege_is_extracted))
}

axiom (forall m: $Memory, a: $Value, $tv0: $TypeValue :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Roles_Privilege_is_well_formed($ResourceValue(m, $Roles_Privilege_type_value($tv0), a))
);

procedure {:inline 1} $Roles_Privilege_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, witness: $Value, is_extracted: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(is_extracted);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := witness][1 := is_extracted], 2));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Roles_Privilege_unpack($tv0: $TypeValue, $struct: $Value) returns (witness: $Value, is_extracted: $Value)
{
    assume is#$Vector($struct);
    witness := $SelectField($struct, $Roles_Privilege_witness);
    is_extracted := $SelectField($struct, $Roles_Privilege_is_extracted);
    assume is#$Boolean(is_extracted);
}

const unique $Roles_ChildVASPRole: $TypeName;
const $Roles_ChildVASPRole_dummy_field: $FieldName;
axiom $Roles_ChildVASPRole_dummy_field == 0;
function $Roles_ChildVASPRole_type_value(): $TypeValue {
    $StructType($Roles_ChildVASPRole, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $Roles_ChildVASPRole_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_ChildVASPRole_dummy_field))
}
function {:inline} $Roles_ChildVASPRole_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_ChildVASPRole_dummy_field))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Roles_ChildVASPRole_is_well_formed($ResourceValue(m, $Roles_ChildVASPRole_type_value(), a))
);

procedure {:inline 1} $Roles_ChildVASPRole_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Roles_ChildVASPRole_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $Roles_ChildVASPRole_dummy_field);
    assume is#$Boolean(dummy_field);
}

const unique $Roles_DesignatedDealerRole: $TypeName;
const $Roles_DesignatedDealerRole_dummy_field: $FieldName;
axiom $Roles_DesignatedDealerRole_dummy_field == 0;
function $Roles_DesignatedDealerRole_type_value(): $TypeValue {
    $StructType($Roles_DesignatedDealerRole, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $Roles_DesignatedDealerRole_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_DesignatedDealerRole_dummy_field))
}
function {:inline} $Roles_DesignatedDealerRole_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_DesignatedDealerRole_dummy_field))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Roles_DesignatedDealerRole_is_well_formed($ResourceValue(m, $Roles_DesignatedDealerRole_type_value(), a))
);

procedure {:inline 1} $Roles_DesignatedDealerRole_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Roles_DesignatedDealerRole_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $Roles_DesignatedDealerRole_dummy_field);
    assume is#$Boolean(dummy_field);
}

const unique $Roles_ParentVASPRole: $TypeName;
const $Roles_ParentVASPRole_dummy_field: $FieldName;
axiom $Roles_ParentVASPRole_dummy_field == 0;
function $Roles_ParentVASPRole_type_value(): $TypeValue {
    $StructType($Roles_ParentVASPRole, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $Roles_ParentVASPRole_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_ParentVASPRole_dummy_field))
}
function {:inline} $Roles_ParentVASPRole_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_ParentVASPRole_dummy_field))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Roles_ParentVASPRole_is_well_formed($ResourceValue(m, $Roles_ParentVASPRole_type_value(), a))
);

procedure {:inline 1} $Roles_ParentVASPRole_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Roles_ParentVASPRole_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $Roles_ParentVASPRole_dummy_field);
    assume is#$Boolean(dummy_field);
}

const unique $Roles_RoleId: $TypeName;
const $Roles_RoleId_role_id: $FieldName;
axiom $Roles_RoleId_role_id == 0;
function $Roles_RoleId_type_value(): $TypeValue {
    $StructType($Roles_RoleId, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $IntegerType()], 1))
}
function {:inline} $Roles_RoleId_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $IsValidU64($SelectField($this, $Roles_RoleId_role_id))
}
function {:inline} $Roles_RoleId_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $IsValidU64($SelectField($this, $Roles_RoleId_role_id))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Roles_RoleId_is_well_formed($ResourceValue(m, $Roles_RoleId_type_value(), a))
);

procedure {:inline 1} $Roles_RoleId_pack($file_id: int, $byte_index: int, $var_idx: int, role_id: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(role_id);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := role_id], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Roles_RoleId_unpack($struct: $Value) returns (role_id: $Value)
{
    assume is#$Vector($struct);
    role_id := $SelectField($struct, $Roles_RoleId_role_id);
    assume $IsValidU64(role_id);
}

const unique $Roles_TreasuryComplianceRole: $TypeName;
const $Roles_TreasuryComplianceRole_dummy_field: $FieldName;
axiom $Roles_TreasuryComplianceRole_dummy_field == 0;
function $Roles_TreasuryComplianceRole_type_value(): $TypeValue {
    $StructType($Roles_TreasuryComplianceRole, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $Roles_TreasuryComplianceRole_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_TreasuryComplianceRole_dummy_field))
}
function {:inline} $Roles_TreasuryComplianceRole_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_TreasuryComplianceRole_dummy_field))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Roles_TreasuryComplianceRole_is_well_formed($ResourceValue(m, $Roles_TreasuryComplianceRole_type_value(), a))
);

procedure {:inline 1} $Roles_TreasuryComplianceRole_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Roles_TreasuryComplianceRole_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $Roles_TreasuryComplianceRole_dummy_field);
    assume is#$Boolean(dummy_field);
}

const unique $Roles_UnhostedRole: $TypeName;
const $Roles_UnhostedRole_dummy_field: $FieldName;
axiom $Roles_UnhostedRole_dummy_field == 0;
function $Roles_UnhostedRole_type_value(): $TypeValue {
    $StructType($Roles_UnhostedRole, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $Roles_UnhostedRole_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_UnhostedRole_dummy_field))
}
function {:inline} $Roles_UnhostedRole_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_UnhostedRole_dummy_field))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Roles_UnhostedRole_is_well_formed($ResourceValue(m, $Roles_UnhostedRole_type_value(), a))
);

procedure {:inline 1} $Roles_UnhostedRole_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Roles_UnhostedRole_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $Roles_UnhostedRole_dummy_field);
    assume is#$Boolean(dummy_field);
}

const unique $Roles_ValidatorOperatorRole: $TypeName;
const $Roles_ValidatorOperatorRole_dummy_field: $FieldName;
axiom $Roles_ValidatorOperatorRole_dummy_field == 0;
function $Roles_ValidatorOperatorRole_type_value(): $TypeValue {
    $StructType($Roles_ValidatorOperatorRole, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $Roles_ValidatorOperatorRole_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_ValidatorOperatorRole_dummy_field))
}
function {:inline} $Roles_ValidatorOperatorRole_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_ValidatorOperatorRole_dummy_field))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Roles_ValidatorOperatorRole_is_well_formed($ResourceValue(m, $Roles_ValidatorOperatorRole_type_value(), a))
);

procedure {:inline 1} $Roles_ValidatorOperatorRole_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Roles_ValidatorOperatorRole_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $Roles_ValidatorOperatorRole_dummy_field);
    assume is#$Boolean(dummy_field);
}

const unique $Roles_ValidatorRole: $TypeName;
const $Roles_ValidatorRole_dummy_field: $FieldName;
axiom $Roles_ValidatorRole_dummy_field == 0;
function $Roles_ValidatorRole_type_value(): $TypeValue {
    $StructType($Roles_ValidatorRole, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $Roles_ValidatorRole_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_ValidatorRole_dummy_field))
}
function {:inline} $Roles_ValidatorRole_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Roles_ValidatorRole_dummy_field))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Roles_ValidatorRole_is_well_formed($ResourceValue(m, $Roles_ValidatorRole_type_value(), a))
);

procedure {:inline 1} $Roles_ValidatorRole_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Roles_ValidatorRole_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $Roles_ValidatorRole_dummy_field);
    assume is#$Boolean(dummy_field);
}



// ** functions of module Roles

procedure {:inline 1} $Roles_ASSOCIATION_ROOT_ROLE_ID () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0
    $tmp := $Integer(0);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(13, 2025, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_CHILD_VASP_ROLE_ID () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 6
    $tmp := $Integer(6);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(13, 2289, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_DESIGNATED_DEALER_ROLE_ID () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 2
    $tmp := $Integer(2);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(13, 2121, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_PARENT_VASP_ROLE_ID () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 5
    $tmp := $Integer(5);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(13, 2249, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_TREASURY_COMPLIANCE_ROLE_ID () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 1
    $tmp := $Integer(1);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(13, 2074, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_UNHOSTED_ROLE_ID () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 7
    $tmp := $Integer(7);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(13, 2327, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_VALIDATOR_OPERATOR_ROLE_ID () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 4
    $tmp := $Integer(4);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(13, 2208, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_VALIDATOR_ROLE_ID () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 3
    $tmp := $Integer(3);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(13, 2160, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_add_privilege_to_account ($tv0: $TypeValue, account: $Value, witness: $Value, role_id: $Value) returns ()
free requires is#$Address(account);
free requires $IsValidU64(role_id);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var account_role: $Value; // $Roles_RoleId_type_value()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $Roles_RoleId_type_value()
    var $t9: $Value; // $Roles_RoleId_type_value()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $tv0
    var $t17: $Value; // $BooleanType()
    var $t18: $Value; // $Roles_Privilege_type_value($tv0)
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $tv0
    var $t23: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 4669, 0, account); }
    if (true) { assume $DebugTrackLocal(13, 4669, 1, witness); }
    if (true) { assume $DebugTrackLocal(13, 4669, 2, role_id); }

    // bytecode translation starts here
    // $t21 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t21 := $tmp;

    // $t22 := move(witness)
    call $tmp := $CopyOrMoveValue(witness);
    $t22 := $tmp;

    // $t23 := move(role_id)
    call $tmp := $CopyOrMoveValue(role_id);
    $t23 := $tmp;

    // $t6 := copy($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t6 := $tmp;

    // $t7 := Signer::address_of($t6)
    call $t7 := $Signer_address_of($t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 4867);
      goto Abort;
    }
    assume is#$Address($t7);


    // $t8 := get_global<Roles::RoleId>($t7)
    call $tmp := $GetGlobal($t7, $Roles_RoleId_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 4837);
      goto Abort;
    }
    assume $Roles_RoleId_is_well_formed($tmp);
    $t8 := $tmp;

    // account_role := $t8
    call $tmp := $CopyOrMoveValue($t8);
    account_role := $tmp;
    if (true) { assume $DebugTrackLocal(13, 4822, 3, $tmp); }

    // $t9 := move(account_role)
    call $tmp := $CopyOrMoveValue(account_role);
    $t9 := $tmp;

    // $t10 := get_field<Roles::RoleId>.role_id($t9)
    call $tmp := $GetFieldFromValue($t9, $Roles_RoleId_role_id);
    assume $IsValidU64($tmp);
    $t10 := $tmp;

    // $t11 := move($t10)
    call $tmp := $CopyOrMoveValue($t10);
    $t11 := $tmp;

    // $t13 := ==($t11, $t23)
    $tmp := $Boolean($IsEqual($t11, $t23));
    $t13 := $tmp;

    // $t4 := $t13
    call $tmp := $CopyOrMoveValue($t13);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 4897, 4, $tmp); }

    // if ($t4) goto L0 else goto L1
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t15 := move($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t15 := $tmp;

    // $t17 := false
    $tmp := $Boolean(false);
    $t17 := $tmp;

    // $t18 := pack Roles::Privilege<#0>($t22, $t17)
    call $tmp := $Roles_Privilege_pack(0, 0, 0, $tv0, $t22, $t17);
    $t18 := $tmp;

    // move_to<Roles::Privilege<#0>>($t18, $t15)
    call $MoveTo($Roles_Privilege_type_value($tv0), $t18, $t15);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 4949);
      goto Abort;
    }

    // return ()
    return;

    // L2:
L2:

    // $t19 := move($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t19 := $tmp;

    // destroy($t19)

    // $t20 := 0
    $tmp := $Integer(0);
    $t20 := $tmp;

    // abort($t20)
    if (true) { assume $DebugTrackAbort(13, 4897); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_add_privilege_to_account_association_root_role ($tv0: $TypeValue, account: $Value, witness: $Value) returns ()
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $tv0
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $tv0
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 5259, 0, account); }
    if (true) { assume $DebugTrackLocal(13, 5259, 1, witness); }

    // bytecode translation starts here
    // $t5 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t5 := $tmp;

    // $t6 := move(witness)
    call $tmp := $CopyOrMoveValue(witness);
    $t6 := $tmp;

    // $t2 := move($t5)
    call $tmp := $CopyOrMoveValue($t5);
    $t2 := $tmp;

    // $t4 := Roles::ASSOCIATION_ROOT_ROLE_ID()
    call $t4 := $Roles_ASSOCIATION_ROOT_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 1991);
      goto Abort;
    }
    assume $IsValidU64($t4);


    // Roles::add_privilege_to_account<#0>($t2, $t6, $t4)
    call $Roles_add_privilege_to_account($tv0, $t2, $t6, $t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 4673);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_add_privilege_to_account_child_vasp_role ($tv0: $TypeValue, account: $Value, witness: $Value) returns ()
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $tv0
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $tv0
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 6567, 0, account); }
    if (true) { assume $DebugTrackLocal(13, 6567, 1, witness); }

    // bytecode translation starts here
    // $t5 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t5 := $tmp;

    // $t6 := move(witness)
    call $tmp := $CopyOrMoveValue(witness);
    $t6 := $tmp;

    // $t2 := move($t5)
    call $tmp := $CopyOrMoveValue($t5);
    $t2 := $tmp;

    // $t4 := Roles::CHILD_VASP_ROLE_ID()
    call $t4 := $Roles_CHILD_VASP_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2261);
      goto Abort;
    }
    assume $IsValidU64($t4);


    // Roles::add_privilege_to_account<#0>($t2, $t6, $t4)
    call $Roles_add_privilege_to_account($tv0, $t2, $t6, $t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 4673);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_add_privilege_to_account_designated_dealer_role ($tv0: $TypeValue, account: $Value, witness: $Value) returns ()
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $tv0
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $tv0
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 5705, 0, account); }
    if (true) { assume $DebugTrackLocal(13, 5705, 1, witness); }

    // bytecode translation starts here
    // $t5 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t5 := $tmp;

    // $t6 := move(witness)
    call $tmp := $CopyOrMoveValue(witness);
    $t6 := $tmp;

    // $t2 := move($t5)
    call $tmp := $CopyOrMoveValue($t5);
    $t2 := $tmp;

    // $t4 := Roles::DESIGNATED_DEALER_ROLE_ID()
    call $t4 := $Roles_DESIGNATED_DEALER_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2086);
      goto Abort;
    }
    assume $IsValidU64($t4);


    // Roles::add_privilege_to_account<#0>($t2, $t6, $t4)
    call $Roles_add_privilege_to_account($tv0, $t2, $t6, $t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 4673);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_add_privilege_to_account_parent_vasp_role ($tv0: $TypeValue, account: $Value, witness: $Value) returns ()
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $tv0
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $tv0
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 6357, 0, account); }
    if (true) { assume $DebugTrackLocal(13, 6357, 1, witness); }

    // bytecode translation starts here
    // $t5 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t5 := $tmp;

    // $t6 := move(witness)
    call $tmp := $CopyOrMoveValue(witness);
    $t6 := $tmp;

    // $t2 := move($t5)
    call $tmp := $CopyOrMoveValue($t5);
    $t2 := $tmp;

    // $t4 := Roles::PARENT_VASP_ROLE_ID()
    call $t4 := $Roles_PARENT_VASP_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2220);
      goto Abort;
    }
    assume $IsValidU64($t4);


    // Roles::add_privilege_to_account<#0>($t2, $t6, $t4)
    call $Roles_add_privilege_to_account($tv0, $t2, $t6, $t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 4673);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_add_privilege_to_account_treasury_compliance_role ($tv0: $TypeValue, account: $Value, witness: $Value) returns ()
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $tv0
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $tv0
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 5479, 0, account); }
    if (true) { assume $DebugTrackLocal(13, 5479, 1, witness); }

    // bytecode translation starts here
    // $t5 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t5 := $tmp;

    // $t6 := move(witness)
    call $tmp := $CopyOrMoveValue(witness);
    $t6 := $tmp;

    // $t2 := move($t5)
    call $tmp := $CopyOrMoveValue($t5);
    $t2 := $tmp;

    // $t4 := Roles::TREASURY_COMPLIANCE_ROLE_ID()
    call $t4 := $Roles_TREASURY_COMPLIANCE_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2037);
      goto Abort;
    }
    assume $IsValidU64($t4);


    // Roles::add_privilege_to_account<#0>($t2, $t6, $t4)
    call $Roles_add_privilege_to_account($tv0, $t2, $t6, $t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 4673);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_add_privilege_to_account_unhosted_role ($tv0: $TypeValue, account: $Value, witness: $Value) returns ()
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $tv0
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $tv0
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 6775, 0, account); }
    if (true) { assume $DebugTrackLocal(13, 6775, 1, witness); }

    // bytecode translation starts here
    // $t5 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t5 := $tmp;

    // $t6 := move(witness)
    call $tmp := $CopyOrMoveValue(witness);
    $t6 := $tmp;

    // $t2 := move($t5)
    call $tmp := $CopyOrMoveValue($t5);
    $t2 := $tmp;

    // $t4 := Roles::UNHOSTED_ROLE_ID()
    call $t4 := $Roles_UNHOSTED_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2301);
      goto Abort;
    }
    assume $IsValidU64($t4);


    // Roles::add_privilege_to_account<#0>($t2, $t6, $t4)
    call $Roles_add_privilege_to_account($tv0, $t2, $t6, $t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 4673);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_add_privilege_to_account_validator_operator_role ($tv0: $TypeValue, account: $Value, witness: $Value) returns ()
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $tv0
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $tv0
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 6133, 0, account); }
    if (true) { assume $DebugTrackLocal(13, 6133, 1, witness); }

    // bytecode translation starts here
    // $t5 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t5 := $tmp;

    // $t6 := move(witness)
    call $tmp := $CopyOrMoveValue(witness);
    $t6 := $tmp;

    // $t2 := move($t5)
    call $tmp := $CopyOrMoveValue($t5);
    $t2 := $tmp;

    // $t4 := Roles::VALIDATOR_OPERATOR_ROLE_ID()
    call $t4 := $Roles_VALIDATOR_OPERATOR_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2172);
      goto Abort;
    }
    assume $IsValidU64($t4);


    // Roles::add_privilege_to_account<#0>($t2, $t6, $t4)
    call $Roles_add_privilege_to_account($tv0, $t2, $t6, $t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 4673);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_add_privilege_to_account_validator_role ($tv0: $TypeValue, account: $Value, witness: $Value) returns ()
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $tv0
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $tv0
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 5927, 0, account); }
    if (true) { assume $DebugTrackLocal(13, 5927, 1, witness); }

    // bytecode translation starts here
    // $t5 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t5 := $tmp;

    // $t6 := move(witness)
    call $tmp := $CopyOrMoveValue(witness);
    $t6 := $tmp;

    // $t2 := move($t5)
    call $tmp := $CopyOrMoveValue($t5);
    $t2 := $tmp;

    // $t4 := Roles::VALIDATOR_ROLE_ID()
    call $t4 := $Roles_VALIDATOR_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2133);
      goto Abort;
    }
    assume $IsValidU64($t4);


    // Roles::add_privilege_to_account<#0>($t2, $t6, $t4)
    call $Roles_add_privilege_to_account($tv0, $t2, $t6, $t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 4673);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_extract_privilege_to_capability ($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var owner_address: $Value; // $AddressType()
    var priv: $Reference; // ReferenceType($Roles_Privilege_type_value($tv0))
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $BooleanType()
    var $t11: $Value; // $BooleanType()
    var $t12: $Value; // $AddressType()
    var $t13: $Reference; // ReferenceType($Roles_Privilege_type_value($tv0))
    var $t14: $Reference; // ReferenceType($Roles_Privilege_type_value($tv0))
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $BooleanType()
    var $t17: $Value; // $BooleanType()
    var $t18: $Value; // $BooleanType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $BooleanType()
    var $t21: $Reference; // ReferenceType($Roles_Privilege_type_value($tv0))
    var $t22: $Reference; // ReferenceType($BooleanType())
    var $t23: $Value; // $AddressType()
    var $t24: $Value; // $Roles_Capability_type_value($tv0)
    var $t25: $Reference; // ReferenceType($Roles_Privilege_type_value($tv0))
    var $t26: $Value; // $IntegerType()
    var $t27: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 16397, 0, account); }

    // bytecode translation starts here
    // $t27 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t27 := $tmp;

    // $t7 := move($t27)
    call $tmp := $CopyOrMoveValue($t27);
    $t7 := $tmp;

    // $t8 := Signer::address_of($t7)
    call $t8 := $Signer_address_of($t7);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 16553);
      goto Abort;
    }
    assume is#$Address($t8);


    // owner_address := $t8
    call $tmp := $CopyOrMoveValue($t8);
    owner_address := $tmp;
    if (true) { assume $DebugTrackLocal(13, 16529, 1, $tmp); }

    // $t10 := exists<Roles::Privilege<#0>>(owner_address)
    call $tmp := $Exists(owner_address, $Roles_Privilege_type_value($tv0));
    $t10 := $tmp;

    // $t3 := $t10
    call $tmp := $CopyOrMoveValue($t10);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 16617, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t13 := borrow_global<Roles::Privilege<#0>>(owner_address)
    call $t13 := $BorrowGlobal(owner_address, $Roles_Privilege_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 16687);
      goto Abort;
    }
    assume $Roles_Privilege_is_well_formed($Dereference($t13));

    // UnpackRef($t13)

    // priv := $t13
    call priv := $CopyOrMoveRef($t13);
    if (true) { assume $DebugTrackLocal(13, 16680, 2, $Dereference(priv)); }

    // $t14 := copy(priv)
    call $t14 := $CopyOrMoveRef(priv);

    // $t15 := get_field<Roles::Privilege<#0>>.is_extracted($t14)
    call $tmp := $GetFieldFromReference($t14, $Roles_Privilege_is_extracted);
    assume is#$Boolean($tmp);
    $t15 := $tmp;

    // Reference(priv) <- $t14
    call priv := $WritebackToReference($t14, priv);

    // $t16 := move($t15)
    call $tmp := $CopyOrMoveValue($t15);
    $t16 := $tmp;

    // $t17 := !($t16)
    call $tmp := $Not($t16);
    $t17 := $tmp;

    // $t5 := $t17
    call $tmp := $CopyOrMoveValue($t17);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 16811, 5, $tmp); }

    // if ($t5) goto L3 else goto L4
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L5
    goto L5;

    // L2:
L2:

    // $t19 := 3
    $tmp := $Integer(3);
    $t19 := $tmp;

    // abort($t19)
    if (true) { assume $DebugTrackAbort(13, 16617); }
    goto Abort;

    // L3:
L3:

    // $t20 := true
    $tmp := $Boolean(true);
    $t20 := $tmp;

    // $t21 := move(priv)
    call $t21 := $CopyOrMoveRef(priv);

    // $t22 := borrow_field<Roles::Privilege<#0>>.is_extracted($t21)
    call $t22 := $BorrowField($t21, $Roles_Privilege_is_extracted);
    assume is#$Boolean($Dereference($t22));

    // Roles::Privilege <- $t21
    call $WritebackToGlobal($t21);

    // UnpackRef($t22)

    // write_ref($t22, $t20)
    call $t22 := $WriteRef($t22, $t20);
    if (true) { assume $DebugTrackLocal(13, 16901, 2, $Dereference(priv)); }

    // Roles::Privilege <- $t22
    call $WritebackToGlobal($t22);

    // Reference($t21) <- $t22
    call $t21 := $WritebackToReference($t22, $t21);

    // PackRef($t21)

    // PackRef($t22)

    // $t24 := pack Roles::Capability<#0>(owner_address)
    call $tmp := $Roles_Capability_pack(0, 0, 0, $tv0, owner_address);
    $t24 := $tmp;

    // return $t24
    $ret0 := $t24;
    if (true) { assume $DebugTrackLocal(13, 16935, 28, $ret0); }
    return;

    // L5:
L5:

    // $t25 := move(priv)
    call $t25 := $CopyOrMoveRef(priv);

    // destroy($t25)

    // Roles::Privilege <- $t25
    call $WritebackToGlobal($t25);

    // PackRef($t25)

    // $t26 := 4
    $tmp := $Integer(4);
    $t26 := $tmp;

    // abort($t26)
    if (true) { assume $DebugTrackAbort(13, 16811); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_grant_root_association_role (association: $Value) returns ()
free requires is#$Address(association);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var owner_address: $Value; // $AddressType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $Roles_RoleId_type_value()
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $BooleanType()
    var $t21: $Value; // $Roles_AssociationRootRole_type_value()
    var $t22: $Value; // $BooleanType()
    var $t23: $Value; // $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value())
    var $t24: $Value; // $AddressType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 7427, 0, association); }

    // bytecode translation starts here
    // $t26 := move(association)
    call $tmp := $CopyOrMoveValue(association);
    $t26 := $tmp;

    // $t6 := LibraTimestamp::is_genesis()
    call $t6 := $LibraTimestamp_is_genesis();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 7536);
      goto Abort;
    }
    assume is#$Boolean($t6);


    // $t2 := $t6
    call $tmp := $CopyOrMoveValue($t6);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 7513, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t8 := copy($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t8 := $tmp;

    // $t9 := Signer::address_of($t8)
    call $t9 := $Signer_address_of($t8);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 7590);
      goto Abort;
    }
    assume is#$Address($t9);


    // owner_address := $t9
    call $tmp := $CopyOrMoveValue($t9);
    owner_address := $tmp;
    if (true) { assume $DebugTrackLocal(13, 7566, 1, $tmp); }

    // $t11 := CoreAddresses::ASSOCIATION_ROOT_ADDRESS()
    call $t11 := $CoreAddresses_ASSOCIATION_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 7662);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := ==(owner_address, $t11)
    $tmp := $Boolean($IsEqual(owner_address, $t11));
    $t12 := $tmp;

    // $t4 := $t12
    call $tmp := $CopyOrMoveValue($t12);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 7623, 4, $tmp); }

    // if ($t4) goto L3 else goto L4
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L5
    goto L5;

    // L2:
L2:

    // $t14 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t14 := $tmp;

    // destroy($t14)

    // $t15 := 0
    $tmp := $Integer(0);
    $t15 := $tmp;

    // abort($t15)
    if (true) { assume $DebugTrackAbort(13, 7513); }
    goto Abort;

    // L3:
L3:

    // $t16 := copy($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t16 := $tmp;

    // $t17 := Roles::ASSOCIATION_ROOT_ROLE_ID()
    call $t17 := $Roles_ASSOCIATION_ROOT_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 1991);
      goto Abort;
    }
    assume $IsValidU64($t17);


    // $t18 := pack Roles::RoleId($t17)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t17);
    $t18 := $tmp;

    // move_to<Roles::RoleId>($t18, $t16)
    call $MoveTo($Roles_RoleId_type_value(), $t18, $t16);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 7760);
      goto Abort;
    }

    // $t19 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t19 := $tmp;

    // $t20 := false
    $tmp := $Boolean(false);
    $t20 := $tmp;

    // $t21 := pack Roles::AssociationRootRole($t20)
    call $tmp := $Roles_AssociationRootRole_pack(0, 0, 0, $t20);
    $t21 := $tmp;

    // $t22 := false
    $tmp := $Boolean(false);
    $t22 := $tmp;

    // $t23 := pack Roles::Privilege<Roles::AssociationRootRole>($t21, $t22)
    call $tmp := $Roles_Privilege_pack(0, 0, 0, $Roles_AssociationRootRole_type_value(), $t21, $t22);
    $t23 := $tmp;

    // move_to<Roles::Privilege<Roles::AssociationRootRole>>($t23, $t19)
    call $MoveTo($Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), $t23, $t19);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 7838);
      goto Abort;
    }

    // return ()
    return;

    // L5:
L5:

    // $t24 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t24 := $tmp;

    // destroy($t24)

    // $t25 := 0
    $tmp := $Integer(0);
    $t25 := $tmp;

    // abort($t25)
    if (true) { assume $DebugTrackAbort(13, 7623); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_grant_treasury_compliance_role (treasury_compliance_account: $Value, _: $Value) returns ()
free requires is#$Address(treasury_compliance_account);
free requires $Roles_Capability_is_well_formed(_);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var owner_address: $Value; // $AddressType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $Roles_RoleId_type_value()
    var $t20: $Value; // $AddressType()
    var $t21: $Value; // $BooleanType()
    var $t22: $Value; // $Roles_TreasuryComplianceRole_type_value()
    var $t23: $Value; // $BooleanType()
    var $t24: $Value; // $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value())
    var $t25: $Value; // $AddressType()
    var $t26: $Value; // $BooleanType()
    var $t27: $Value; // $Roles_AssociationRootRole_type_value()
    var $t28: $Value; // $BooleanType()
    var $t29: $Value; // $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value())
    var $t30: $Value; // $AddressType()
    var $t31: $Value; // $IntegerType()
    var $t32: $Value; // $AddressType()
    var $t33: $Value; // $Roles_Capability_type_value($Roles_AssociationRootRole_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 8125, 0, treasury_compliance_account); }
    if (true) { assume $DebugTrackLocal(13, 8125, 1, _); }

    // bytecode translation starts here
    // $t32 := move(treasury_compliance_account)
    call $tmp := $CopyOrMoveValue(treasury_compliance_account);
    $t32 := $tmp;

    // $t33 := move(_)
    call $tmp := $CopyOrMoveValue(_);
    $t33 := $tmp;

    // $t7 := LibraTimestamp::is_genesis()
    call $t7 := $LibraTimestamp_is_genesis();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 8298);
      goto Abort;
    }
    assume is#$Boolean($t7);


    // $t3 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 8275, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t9 := copy($t32)
    call $tmp := $CopyOrMoveValue($t32);
    $t9 := $tmp;

    // $t10 := Signer::address_of($t9)
    call $t10 := $Signer_address_of($t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 8352);
      goto Abort;
    }
    assume is#$Address($t10);


    // owner_address := $t10
    call $tmp := $CopyOrMoveValue($t10);
    owner_address := $tmp;
    if (true) { assume $DebugTrackLocal(13, 8328, 2, $tmp); }

    // $t12 := CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()
    call $t12 := $CoreAddresses_TREASURY_COMPLIANCE_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 8440);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := ==(owner_address, $t12)
    $tmp := $Boolean($IsEqual(owner_address, $t12));
    $t13 := $tmp;

    // $t5 := $t13
    call $tmp := $CopyOrMoveValue($t13);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 8401, 5, $tmp); }

    // if ($t5) goto L3 else goto L4
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L5
    goto L5;

    // L2:
L2:

    // $t15 := move($t32)
    call $tmp := $CopyOrMoveValue($t32);
    $t15 := $tmp;

    // destroy($t15)

    // $t16 := 0
    $tmp := $Integer(0);
    $t16 := $tmp;

    // abort($t16)
    if (true) { assume $DebugTrackAbort(13, 8275); }
    goto Abort;

    // L3:
L3:

    // $t17 := copy($t32)
    call $tmp := $CopyOrMoveValue($t32);
    $t17 := $tmp;

    // $t18 := Roles::TREASURY_COMPLIANCE_ROLE_ID()
    call $t18 := $Roles_TREASURY_COMPLIANCE_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2037);
      goto Abort;
    }
    assume $IsValidU64($t18);


    // $t19 := pack Roles::RoleId($t18)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t18);
    $t19 := $tmp;

    // move_to<Roles::RoleId>($t19, $t17)
    call $MoveTo($Roles_RoleId_type_value(), $t19, $t17);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 8547);
      goto Abort;
    }

    // $t20 := copy($t32)
    call $tmp := $CopyOrMoveValue($t32);
    $t20 := $tmp;

    // $t21 := false
    $tmp := $Boolean(false);
    $t21 := $tmp;

    // $t22 := pack Roles::TreasuryComplianceRole($t21)
    call $tmp := $Roles_TreasuryComplianceRole_pack(0, 0, 0, $t21);
    $t22 := $tmp;

    // $t23 := false
    $tmp := $Boolean(false);
    $t23 := $tmp;

    // $t24 := pack Roles::Privilege<Roles::TreasuryComplianceRole>($t22, $t23)
    call $tmp := $Roles_Privilege_pack(0, 0, 0, $Roles_TreasuryComplianceRole_type_value(), $t22, $t23);
    $t24 := $tmp;

    // move_to<Roles::Privilege<Roles::TreasuryComplianceRole>>($t24, $t20)
    call $MoveTo($Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), $t24, $t20);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 8644);
      goto Abort;
    }

    // $t25 := move($t32)
    call $tmp := $CopyOrMoveValue($t32);
    $t25 := $tmp;

    // $t26 := false
    $tmp := $Boolean(false);
    $t26 := $tmp;

    // $t27 := pack Roles::AssociationRootRole($t26)
    call $tmp := $Roles_AssociationRootRole_pack(0, 0, 0, $t26);
    $t27 := $tmp;

    // $t28 := false
    $tmp := $Boolean(false);
    $t28 := $tmp;

    // $t29 := pack Roles::Privilege<Roles::AssociationRootRole>($t27, $t28)
    call $tmp := $Roles_Privilege_pack(0, 0, 0, $Roles_AssociationRootRole_type_value(), $t27, $t28);
    $t29 := $tmp;

    // move_to<Roles::Privilege<Roles::AssociationRootRole>>($t29, $t25)
    call $MoveTo($Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), $t29, $t25);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 8947);
      goto Abort;
    }

    // return ()
    return;

    // L5:
L5:

    // $t30 := move($t32)
    call $tmp := $CopyOrMoveValue($t32);
    $t30 := $tmp;

    // destroy($t30)

    // $t31 := 0
    $tmp := $Integer(0);
    $t31 := $tmp;

    // abort($t31)
    if (true) { assume $DebugTrackAbort(13, 8401); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_new_child_vasp_role (creating_account: $Value, new_account: $Value) returns ()
free requires is#$Address(creating_account);
free requires is#$Address(new_account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var calling_role: $Value; // $Roles_RoleId_type_value()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $Roles_RoleId_type_value()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $Roles_RoleId_type_value()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $BooleanType()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $Roles_RoleId_type_value()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $AddressType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $Roles_RoleId_type_value()
    var $t27: $Value; // $AddressType()
    var $t28: $Value; // $BooleanType()
    var $t29: $Value; // $Roles_ChildVASPRole_type_value()
    var $t30: $Value; // $BooleanType()
    var $t31: $Value; // $Roles_Privilege_type_value($Roles_ChildVASPRole_type_value())
    var $t32: $Value; // $AddressType()
    var $t33: $Value; // $IntegerType()
    var $t34: $Value; // $AddressType()
    var $t35: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 13420, 0, creating_account); }
    if (true) { assume $DebugTrackLocal(13, 13420, 1, new_account); }

    // bytecode translation starts here
    // $t34 := move(creating_account)
    call $tmp := $CopyOrMoveValue(creating_account);
    $t34 := $tmp;

    // $t35 := move(new_account)
    call $tmp := $CopyOrMoveValue(new_account);
    $t35 := $tmp;

    // $t7 := move($t34)
    call $tmp := $CopyOrMoveValue($t34);
    $t7 := $tmp;

    // $t8 := Signer::address_of($t7)
    call $t8 := $Signer_address_of($t7);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 13598);
      goto Abort;
    }
    assume is#$Address($t8);


    // $t9 := get_global<Roles::RoleId>($t8)
    call $tmp := $GetGlobal($t8, $Roles_RoleId_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 13568);
      goto Abort;
    }
    assume $Roles_RoleId_is_well_formed($tmp);
    $t9 := $tmp;

    // calling_role := $t9
    call $tmp := $CopyOrMoveValue($t9);
    calling_role := $tmp;
    if (true) { assume $DebugTrackLocal(13, 13553, 2, $tmp); }

    // $t10 := copy($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t10 := $tmp;

    // $t11 := Signer::address_of($t10)
    call $t11 := $Signer_address_of($t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 13741);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := exists<Roles::RoleId>($t11)
    call $tmp := $Exists($t11, $Roles_RoleId_type_value());
    $t12 := $tmp;

    // $t13 := !($t12)
    call $tmp := $Not($t12);
    $t13 := $tmp;

    // $t3 := $t13
    call $tmp := $CopyOrMoveValue($t13);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 13710, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t15 := move(calling_role)
    call $tmp := $CopyOrMoveValue(calling_role);
    $t15 := $tmp;

    // $t16 := get_field<Roles::RoleId>.role_id($t15)
    call $tmp := $GetFieldFromValue($t15, $Roles_RoleId_role_id);
    assume $IsValidU64($tmp);
    $t16 := $tmp;

    // $t17 := move($t16)
    call $tmp := $CopyOrMoveValue($t16);
    $t17 := $tmp;

    // $t18 := Roles::PARENT_VASP_ROLE_ID()
    call $t18 := $Roles_PARENT_VASP_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2220);
      goto Abort;
    }
    assume $IsValidU64($t18);


    // $t19 := ==($t17, $t18)
    $tmp := $Boolean($IsEqual($t17, $t18));
    $t19 := $tmp;

    // $t5 := $t19
    call $tmp := $CopyOrMoveValue($t19);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 13779, 5, $tmp); }

    // if ($t5) goto L3 else goto L4
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L5
    goto L5;

    // L2:
L2:

    // $t21 := move($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t21 := $tmp;

    // destroy($t21)

    // $t22 := move(calling_role)
    call $tmp := $CopyOrMoveValue(calling_role);
    $t22 := $tmp;

    // destroy($t22)

    // $t23 := 1
    $tmp := $Integer(1);
    $t23 := $tmp;

    // abort($t23)
    if (true) { assume $DebugTrackAbort(13, 13710); }
    goto Abort;

    // L3:
L3:

    // $t24 := copy($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t24 := $tmp;

    // $t25 := Roles::CHILD_VASP_ROLE_ID()
    call $t25 := $Roles_CHILD_VASP_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2261);
      goto Abort;
    }
    assume $IsValidU64($t25);


    // $t26 := pack Roles::RoleId($t25)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t25);
    $t26 := $tmp;

    // move_to<Roles::RoleId>($t26, $t24)
    call $MoveTo($Roles_RoleId_type_value(), $t26, $t24);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 13845);
      goto Abort;
    }

    // $t27 := move($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t27 := $tmp;

    // $t28 := false
    $tmp := $Boolean(false);
    $t28 := $tmp;

    // $t29 := pack Roles::ChildVASPRole($t28)
    call $tmp := $Roles_ChildVASPRole_pack(0, 0, 0, $t28);
    $t29 := $tmp;

    // $t30 := false
    $tmp := $Boolean(false);
    $t30 := $tmp;

    // $t31 := pack Roles::Privilege<Roles::ChildVASPRole>($t29, $t30)
    call $tmp := $Roles_Privilege_pack(0, 0, 0, $Roles_ChildVASPRole_type_value(), $t29, $t30);
    $t31 := $tmp;

    // move_to<Roles::Privilege<Roles::ChildVASPRole>>($t31, $t27)
    call $MoveTo($Roles_Privilege_type_value($Roles_ChildVASPRole_type_value()), $t31, $t27);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 13917);
      goto Abort;
    }

    // return ()
    return;

    // L5:
L5:

    // $t32 := move($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t32 := $tmp;

    // destroy($t32)

    // $t33 := 0
    $tmp := $Integer(0);
    $t33 := $tmp;

    // abort($t33)
    if (true) { assume $DebugTrackAbort(13, 13779); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_new_designated_dealer_role (creating_account: $Value, new_account: $Value) returns ()
free requires is#$Address(creating_account);
free requires is#$Address(new_account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var calling_role: $Value; // $Roles_RoleId_type_value()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $Roles_RoleId_type_value()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $Roles_RoleId_type_value()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $BooleanType()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $Roles_RoleId_type_value()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $AddressType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $Roles_RoleId_type_value()
    var $t27: $Value; // $AddressType()
    var $t28: $Value; // $BooleanType()
    var $t29: $Value; // $Roles_DesignatedDealerRole_type_value()
    var $t30: $Value; // $BooleanType()
    var $t31: $Value; // $Roles_Privilege_type_value($Roles_DesignatedDealerRole_type_value())
    var $t32: $Value; // $AddressType()
    var $t33: $Value; // $IntegerType()
    var $t34: $Value; // $AddressType()
    var $t35: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 10145, 0, creating_account); }
    if (true) { assume $DebugTrackLocal(13, 10145, 1, new_account); }

    // bytecode translation starts here
    // $t34 := move(creating_account)
    call $tmp := $CopyOrMoveValue(creating_account);
    $t34 := $tmp;

    // $t35 := move(new_account)
    call $tmp := $CopyOrMoveValue(new_account);
    $t35 := $tmp;

    // $t7 := move($t34)
    call $tmp := $CopyOrMoveValue($t34);
    $t7 := $tmp;

    // $t8 := Signer::address_of($t7)
    call $t8 := $Signer_address_of($t7);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 10330);
      goto Abort;
    }
    assume is#$Address($t8);


    // $t9 := get_global<Roles::RoleId>($t8)
    call $tmp := $GetGlobal($t8, $Roles_RoleId_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 10300);
      goto Abort;
    }
    assume $Roles_RoleId_is_well_formed($tmp);
    $t9 := $tmp;

    // calling_role := $t9
    call $tmp := $CopyOrMoveValue($t9);
    calling_role := $tmp;
    if (true) { assume $DebugTrackLocal(13, 10285, 2, $tmp); }

    // $t10 := copy($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t10 := $tmp;

    // $t11 := Signer::address_of($t10)
    call $t11 := $Signer_address_of($t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 10473);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := exists<Roles::RoleId>($t11)
    call $tmp := $Exists($t11, $Roles_RoleId_type_value());
    $t12 := $tmp;

    // $t13 := !($t12)
    call $tmp := $Not($t12);
    $t13 := $tmp;

    // $t3 := $t13
    call $tmp := $CopyOrMoveValue($t13);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 10442, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t15 := move(calling_role)
    call $tmp := $CopyOrMoveValue(calling_role);
    $t15 := $tmp;

    // $t16 := get_field<Roles::RoleId>.role_id($t15)
    call $tmp := $GetFieldFromValue($t15, $Roles_RoleId_role_id);
    assume $IsValidU64($tmp);
    $t16 := $tmp;

    // $t17 := move($t16)
    call $tmp := $CopyOrMoveValue($t16);
    $t17 := $tmp;

    // $t18 := Roles::TREASURY_COMPLIANCE_ROLE_ID()
    call $t18 := $Roles_TREASURY_COMPLIANCE_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2037);
      goto Abort;
    }
    assume $IsValidU64($t18);


    // $t19 := ==($t17, $t18)
    $tmp := $Boolean($IsEqual($t17, $t18));
    $t19 := $tmp;

    // $t5 := $t19
    call $tmp := $CopyOrMoveValue($t19);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 10584, 5, $tmp); }

    // if ($t5) goto L3 else goto L4
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L5
    goto L5;

    // L2:
L2:

    // $t21 := move($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t21 := $tmp;

    // destroy($t21)

    // $t22 := move(calling_role)
    call $tmp := $CopyOrMoveValue(calling_role);
    $t22 := $tmp;

    // destroy($t22)

    // $t23 := 1
    $tmp := $Integer(1);
    $t23 := $tmp;

    // abort($t23)
    if (true) { assume $DebugTrackAbort(13, 10442); }
    goto Abort;

    // L3:
L3:

    // $t24 := copy($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t24 := $tmp;

    // $t25 := Roles::DESIGNATED_DEALER_ROLE_ID()
    call $t25 := $Roles_DESIGNATED_DEALER_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2086);
      goto Abort;
    }
    assume $IsValidU64($t25);


    // $t26 := pack Roles::RoleId($t25)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t25);
    $t26 := $tmp;

    // move_to<Roles::RoleId>($t26, $t24)
    call $MoveTo($Roles_RoleId_type_value(), $t26, $t24);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 10658);
      goto Abort;
    }

    // $t27 := move($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t27 := $tmp;

    // $t28 := false
    $tmp := $Boolean(false);
    $t28 := $tmp;

    // $t29 := pack Roles::DesignatedDealerRole($t28)
    call $tmp := $Roles_DesignatedDealerRole_pack(0, 0, 0, $t28);
    $t29 := $tmp;

    // $t30 := false
    $tmp := $Boolean(false);
    $t30 := $tmp;

    // $t31 := pack Roles::Privilege<Roles::DesignatedDealerRole>($t29, $t30)
    call $tmp := $Roles_Privilege_pack(0, 0, 0, $Roles_DesignatedDealerRole_type_value(), $t29, $t30);
    $t31 := $tmp;

    // move_to<Roles::Privilege<Roles::DesignatedDealerRole>>($t31, $t27)
    call $MoveTo($Roles_Privilege_type_value($Roles_DesignatedDealerRole_type_value()), $t31, $t27);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 10737);
      goto Abort;
    }

    // return ()
    return;

    // L5:
L5:

    // $t32 := move($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t32 := $tmp;

    // destroy($t32)

    // $t33 := 0
    $tmp := $Integer(0);
    $t33 := $tmp;

    // abort($t33)
    if (true) { assume $DebugTrackAbort(13, 10584); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_new_parent_vasp_role (creating_account: $Value, new_account: $Value) returns ()
free requires is#$Address(creating_account);
free requires is#$Address(new_account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var calling_role: $Value; // $Roles_RoleId_type_value()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $Roles_RoleId_type_value()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $Roles_RoleId_type_value()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $BooleanType()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $Roles_RoleId_type_value()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $Roles_RoleId_type_value()
    var $t25: $Value; // $BooleanType()
    var $t26: $Value; // $Roles_RoleId_type_value()
    var $t27: $Value; // $IntegerType()
    var $t28: $Value; // $IntegerType()
    var $t29: $Value; // $IntegerType()
    var $t30: $Value; // $BooleanType()
    var $t31: $Value; // $BooleanType()
    var $t32: $Value; // $BooleanType()
    var $t33: $Value; // $AddressType()
    var $t34: $Value; // $IntegerType()
    var $t35: $Value; // $Roles_RoleId_type_value()
    var $t36: $Value; // $AddressType()
    var $t37: $Value; // $BooleanType()
    var $t38: $Value; // $Roles_ParentVASPRole_type_value()
    var $t39: $Value; // $BooleanType()
    var $t40: $Value; // $Roles_Privilege_type_value($Roles_ParentVASPRole_type_value())
    var $t41: $Value; // $AddressType()
    var $t42: $Value; // $IntegerType()
    var $t43: $Value; // $AddressType()
    var $t44: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 12444, 0, creating_account); }
    if (true) { assume $DebugTrackLocal(13, 12444, 1, new_account); }

    // bytecode translation starts here
    // $t43 := move(creating_account)
    call $tmp := $CopyOrMoveValue(creating_account);
    $t43 := $tmp;

    // $t44 := move(new_account)
    call $tmp := $CopyOrMoveValue(new_account);
    $t44 := $tmp;

    // $t8 := move($t43)
    call $tmp := $CopyOrMoveValue($t43);
    $t8 := $tmp;

    // $t9 := Signer::address_of($t8)
    call $t9 := $Signer_address_of($t8);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 12623);
      goto Abort;
    }
    assume is#$Address($t9);


    // $t10 := get_global<Roles::RoleId>($t9)
    call $tmp := $GetGlobal($t9, $Roles_RoleId_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 12593);
      goto Abort;
    }
    assume $Roles_RoleId_is_well_formed($tmp);
    $t10 := $tmp;

    // calling_role := $t10
    call $tmp := $CopyOrMoveValue($t10);
    calling_role := $tmp;
    if (true) { assume $DebugTrackLocal(13, 12578, 2, $tmp); }

    // $t11 := copy($t44)
    call $tmp := $CopyOrMoveValue($t44);
    $t11 := $tmp;

    // $t12 := Signer::address_of($t11)
    call $t12 := $Signer_address_of($t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 12766);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := exists<Roles::RoleId>($t12)
    call $tmp := $Exists($t12, $Roles_RoleId_type_value());
    $t13 := $tmp;

    // $t14 := !($t13)
    call $tmp := $Not($t13);
    $t14 := $tmp;

    // $t3 := $t14
    call $tmp := $CopyOrMoveValue($t14);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 12735, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t16 := copy(calling_role)
    call $tmp := $CopyOrMoveValue(calling_role);
    $t16 := $tmp;

    // $t17 := get_field<Roles::RoleId>.role_id($t16)
    call $tmp := $GetFieldFromValue($t16, $Roles_RoleId_role_id);
    assume $IsValidU64($tmp);
    $t17 := $tmp;

    // $t18 := move($t17)
    call $tmp := $CopyOrMoveValue($t17);
    $t18 := $tmp;

    // $t19 := Roles::ASSOCIATION_ROOT_ROLE_ID()
    call $t19 := $Roles_ASSOCIATION_ROOT_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 1991);
      goto Abort;
    }
    assume $IsValidU64($t19);


    // $t20 := ==($t18, $t19)
    $tmp := $Boolean($IsEqual($t18, $t19));
    $t20 := $tmp;

    // if ($t20) goto L3 else goto L4
    $tmp := $t20;
    if (b#$Boolean($tmp)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L5
    goto L5;

    // L2:
L2:

    // $t21 := move($t44)
    call $tmp := $CopyOrMoveValue($t44);
    $t21 := $tmp;

    // destroy($t21)

    // $t22 := move(calling_role)
    call $tmp := $CopyOrMoveValue(calling_role);
    $t22 := $tmp;

    // destroy($t22)

    // $t23 := 1
    $tmp := $Integer(1);
    $t23 := $tmp;

    // abort($t23)
    if (true) { assume $DebugTrackAbort(13, 12735); }
    goto Abort;

    // L3:
L3:

    // $t24 := move(calling_role)
    call $tmp := $CopyOrMoveValue(calling_role);
    $t24 := $tmp;

    // destroy($t24)

    // $t25 := true
    $tmp := $Boolean(true);
    $t25 := $tmp;

    // $t7 := $t25
    call $tmp := $CopyOrMoveValue($t25);
    $t7 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 12828, 7, $tmp); }

    // goto L6
    goto L6;

    // L5:
L5:

    // $t26 := move(calling_role)
    call $tmp := $CopyOrMoveValue(calling_role);
    $t26 := $tmp;

    // $t27 := get_field<Roles::RoleId>.role_id($t26)
    call $tmp := $GetFieldFromValue($t26, $Roles_RoleId_role_id);
    assume $IsValidU64($tmp);
    $t27 := $tmp;

    // $t28 := move($t27)
    call $tmp := $CopyOrMoveValue($t27);
    $t28 := $tmp;

    // $t29 := Roles::TREASURY_COMPLIANCE_ROLE_ID()
    call $t29 := $Roles_TREASURY_COMPLIANCE_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2037);
      goto Abort;
    }
    assume $IsValidU64($t29);


    // $t30 := ==($t28, $t29)
    $tmp := $Boolean($IsEqual($t28, $t29));
    $t30 := $tmp;

    // $t7 := $t30
    call $tmp := $CopyOrMoveValue($t30);
    $t7 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 12828, 7, $tmp); }

    // goto L6
    goto L6;

    // L6:
L6:

    // $t5 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 12804, 5, $tmp); }

    // if ($t5) goto L7 else goto L8
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L7; } else { goto L8; }

    // L8:
L8:

    // goto L9
    goto L9;

    // L7:
L7:

    // $t33 := copy($t44)
    call $tmp := $CopyOrMoveValue($t44);
    $t33 := $tmp;

    // $t34 := Roles::PARENT_VASP_ROLE_ID()
    call $t34 := $Roles_PARENT_VASP_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2220);
      goto Abort;
    }
    assume $IsValidU64($t34);


    // $t35 := pack Roles::RoleId($t34)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t34);
    $t35 := $tmp;

    // move_to<Roles::RoleId>($t35, $t33)
    call $MoveTo($Roles_RoleId_type_value(), $t35, $t33);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 13124);
      goto Abort;
    }

    // $t36 := move($t44)
    call $tmp := $CopyOrMoveValue($t44);
    $t36 := $tmp;

    // $t37 := false
    $tmp := $Boolean(false);
    $t37 := $tmp;

    // $t38 := pack Roles::ParentVASPRole($t37)
    call $tmp := $Roles_ParentVASPRole_pack(0, 0, 0, $t37);
    $t38 := $tmp;

    // $t39 := false
    $tmp := $Boolean(false);
    $t39 := $tmp;

    // $t40 := pack Roles::Privilege<Roles::ParentVASPRole>($t38, $t39)
    call $tmp := $Roles_Privilege_pack(0, 0, 0, $Roles_ParentVASPRole_type_value(), $t38, $t39);
    $t40 := $tmp;

    // move_to<Roles::Privilege<Roles::ParentVASPRole>>($t40, $t36)
    call $MoveTo($Roles_Privilege_type_value($Roles_ParentVASPRole_type_value()), $t40, $t36);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 13201);
      goto Abort;
    }

    // return ()
    return;

    // L9:
L9:

    // $t41 := move($t44)
    call $tmp := $CopyOrMoveValue($t44);
    $t41 := $tmp;

    // destroy($t41)

    // $t42 := 0
    $tmp := $Integer(0);
    $t42 := $tmp;

    // abort($t42)
    if (true) { assume $DebugTrackAbort(13, 12804); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_new_unhosted_role (_creating_account: $Value, new_account: $Value) returns ()
free requires is#$Address(_creating_account);
free requires is#$Address(new_account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $Roles_RoleId_type_value()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $Roles_UnhostedRole_type_value()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $Roles_Privilege_type_value($Roles_UnhostedRole_type_value())
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 14178, 0, _creating_account); }
    if (true) { assume $DebugTrackLocal(13, 14178, 1, new_account); }

    // bytecode translation starts here
    // $t19 := move(_creating_account)
    call $tmp := $CopyOrMoveValue(_creating_account);
    $t19 := $tmp;

    // $t20 := move(new_account)
    call $tmp := $CopyOrMoveValue(new_account);
    $t20 := $tmp;

    // $t4 := copy($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t4 := $tmp;

    // $t5 := Signer::address_of($t4)
    call $t5 := $Signer_address_of($t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 14371);
      goto Abort;
    }
    assume is#$Address($t5);


    // $t6 := exists<Roles::RoleId>($t5)
    call $tmp := $Exists($t5, $Roles_RoleId_type_value());
    $t6 := $tmp;

    // $t7 := !($t6)
    call $tmp := $Not($t6);
    $t7 := $tmp;

    // $t2 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 14340, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t9 := copy($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t9 := $tmp;

    // $t10 := Roles::UNHOSTED_ROLE_ID()
    call $t10 := $Roles_UNHOSTED_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2301);
      goto Abort;
    }
    assume $IsValidU64($t10);


    // $t11 := pack Roles::RoleId($t10)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t10);
    $t11 := $tmp;

    // move_to<Roles::RoleId>($t11, $t9)
    call $MoveTo($Roles_RoleId_type_value(), $t11, $t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 14409);
      goto Abort;
    }

    // $t12 := move($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t12 := $tmp;

    // $t13 := false
    $tmp := $Boolean(false);
    $t13 := $tmp;

    // $t14 := pack Roles::UnhostedRole($t13)
    call $tmp := $Roles_UnhostedRole_pack(0, 0, 0, $t13);
    $t14 := $tmp;

    // $t15 := false
    $tmp := $Boolean(false);
    $t15 := $tmp;

    // $t16 := pack Roles::Privilege<Roles::UnhostedRole>($t14, $t15)
    call $tmp := $Roles_Privilege_pack(0, 0, 0, $Roles_UnhostedRole_type_value(), $t14, $t15);
    $t16 := $tmp;

    // move_to<Roles::Privilege<Roles::UnhostedRole>>($t16, $t12)
    call $MoveTo($Roles_Privilege_type_value($Roles_UnhostedRole_type_value()), $t16, $t12);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 14479);
      goto Abort;
    }

    // return ()
    return;

    // L2:
L2:

    // $t17 := move($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t17 := $tmp;

    // destroy($t17)

    // $t18 := 1
    $tmp := $Integer(1);
    $t18 := $tmp;

    // abort($t18)
    if (true) { assume $DebugTrackAbort(13, 14340); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_new_validator_operator_role (creating_account: $Value, new_account: $Value) returns ()
free requires is#$Address(creating_account);
free requires is#$Address(new_account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var calling_role: $Value; // $Roles_RoleId_type_value()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $Roles_RoleId_type_value()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $Roles_RoleId_type_value()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $BooleanType()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $Roles_RoleId_type_value()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $AddressType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $Roles_RoleId_type_value()
    var $t27: $Value; // $AddressType()
    var $t28: $Value; // $BooleanType()
    var $t29: $Value; // $Roles_ValidatorOperatorRole_type_value()
    var $t30: $Value; // $BooleanType()
    var $t31: $Value; // $Roles_Privilege_type_value($Roles_ValidatorOperatorRole_type_value())
    var $t32: $Value; // $AddressType()
    var $t33: $Value; // $IntegerType()
    var $t34: $Value; // $AddressType()
    var $t35: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 11686, 0, creating_account); }
    if (true) { assume $DebugTrackLocal(13, 11686, 1, new_account); }

    // bytecode translation starts here
    // $t34 := move(creating_account)
    call $tmp := $CopyOrMoveValue(creating_account);
    $t34 := $tmp;

    // $t35 := move(new_account)
    call $tmp := $CopyOrMoveValue(new_account);
    $t35 := $tmp;

    // $t7 := move($t34)
    call $tmp := $CopyOrMoveValue($t34);
    $t7 := $tmp;

    // $t8 := Signer::address_of($t7)
    call $t8 := $Signer_address_of($t7);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 11872);
      goto Abort;
    }
    assume is#$Address($t8);


    // $t9 := get_global<Roles::RoleId>($t8)
    call $tmp := $GetGlobal($t8, $Roles_RoleId_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 11842);
      goto Abort;
    }
    assume $Roles_RoleId_is_well_formed($tmp);
    $t9 := $tmp;

    // calling_role := $t9
    call $tmp := $CopyOrMoveValue($t9);
    calling_role := $tmp;
    if (true) { assume $DebugTrackLocal(13, 11827, 2, $tmp); }

    // $t10 := copy($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t10 := $tmp;

    // $t11 := Signer::address_of($t10)
    call $t11 := $Signer_address_of($t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 12015);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := exists<Roles::RoleId>($t11)
    call $tmp := $Exists($t11, $Roles_RoleId_type_value());
    $t12 := $tmp;

    // $t13 := !($t12)
    call $tmp := $Not($t12);
    $t13 := $tmp;

    // $t3 := $t13
    call $tmp := $CopyOrMoveValue($t13);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 11984, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t15 := move(calling_role)
    call $tmp := $CopyOrMoveValue(calling_role);
    $t15 := $tmp;

    // $t16 := get_field<Roles::RoleId>.role_id($t15)
    call $tmp := $GetFieldFromValue($t15, $Roles_RoleId_role_id);
    assume $IsValidU64($tmp);
    $t16 := $tmp;

    // $t17 := move($t16)
    call $tmp := $CopyOrMoveValue($t16);
    $t17 := $tmp;

    // $t18 := Roles::ASSOCIATION_ROOT_ROLE_ID()
    call $t18 := $Roles_ASSOCIATION_ROOT_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 1991);
      goto Abort;
    }
    assume $IsValidU64($t18);


    // $t19 := ==($t17, $t18)
    $tmp := $Boolean($IsEqual($t17, $t18));
    $t19 := $tmp;

    // $t5 := $t19
    call $tmp := $CopyOrMoveValue($t19);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 12053, 5, $tmp); }

    // if ($t5) goto L3 else goto L4
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L5
    goto L5;

    // L2:
L2:

    // $t21 := move($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t21 := $tmp;

    // destroy($t21)

    // $t22 := move(calling_role)
    call $tmp := $CopyOrMoveValue(calling_role);
    $t22 := $tmp;

    // destroy($t22)

    // $t23 := 1
    $tmp := $Integer(1);
    $t23 := $tmp;

    // abort($t23)
    if (true) { assume $DebugTrackAbort(13, 11984); }
    goto Abort;

    // L3:
L3:

    // $t24 := copy($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t24 := $tmp;

    // $t25 := Roles::VALIDATOR_OPERATOR_ROLE_ID()
    call $t25 := $Roles_VALIDATOR_OPERATOR_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2172);
      goto Abort;
    }
    assume $IsValidU64($t25);


    // $t26 := pack Roles::RoleId($t25)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t25);
    $t26 := $tmp;

    // move_to<Roles::RoleId>($t26, $t24)
    call $MoveTo($Roles_RoleId_type_value(), $t26, $t24);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 12124);
      goto Abort;
    }

    // $t27 := move($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t27 := $tmp;

    // $t28 := false
    $tmp := $Boolean(false);
    $t28 := $tmp;

    // $t29 := pack Roles::ValidatorOperatorRole($t28)
    call $tmp := $Roles_ValidatorOperatorRole_pack(0, 0, 0, $t28);
    $t29 := $tmp;

    // $t30 := false
    $tmp := $Boolean(false);
    $t30 := $tmp;

    // $t31 := pack Roles::Privilege<Roles::ValidatorOperatorRole>($t29, $t30)
    call $tmp := $Roles_Privilege_pack(0, 0, 0, $Roles_ValidatorOperatorRole_type_value(), $t29, $t30);
    $t31 := $tmp;

    // move_to<Roles::Privilege<Roles::ValidatorOperatorRole>>($t31, $t27)
    call $MoveTo($Roles_Privilege_type_value($Roles_ValidatorOperatorRole_type_value()), $t31, $t27);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 12204);
      goto Abort;
    }

    // return ()
    return;

    // L5:
L5:

    // $t32 := move($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t32 := $tmp;

    // destroy($t32)

    // $t33 := 0
    $tmp := $Integer(0);
    $t33 := $tmp;

    // abort($t33)
    if (true) { assume $DebugTrackAbort(13, 12053); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_new_validator_role (creating_account: $Value, new_account: $Value) returns ()
free requires is#$Address(creating_account);
free requires is#$Address(new_account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var calling_role: $Value; // $Roles_RoleId_type_value()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $Roles_RoleId_type_value()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $Roles_RoleId_type_value()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $BooleanType()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $Roles_RoleId_type_value()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $AddressType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $Roles_RoleId_type_value()
    var $t27: $Value; // $AddressType()
    var $t28: $Value; // $BooleanType()
    var $t29: $Value; // $Roles_ValidatorRole_type_value()
    var $t30: $Value; // $BooleanType()
    var $t31: $Value; // $Roles_Privilege_type_value($Roles_ValidatorRole_type_value())
    var $t32: $Value; // $AddressType()
    var $t33: $Value; // $IntegerType()
    var $t34: $Value; // $AddressType()
    var $t35: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 10965, 0, creating_account); }
    if (true) { assume $DebugTrackLocal(13, 10965, 1, new_account); }

    // bytecode translation starts here
    // $t34 := move(creating_account)
    call $tmp := $CopyOrMoveValue(creating_account);
    $t34 := $tmp;

    // $t35 := move(new_account)
    call $tmp := $CopyOrMoveValue(new_account);
    $t35 := $tmp;

    // $t7 := move($t34)
    call $tmp := $CopyOrMoveValue($t34);
    $t7 := $tmp;

    // $t8 := Signer::address_of($t7)
    call $t8 := $Signer_address_of($t7);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 11141);
      goto Abort;
    }
    assume is#$Address($t8);


    // $t9 := get_global<Roles::RoleId>($t8)
    call $tmp := $GetGlobal($t8, $Roles_RoleId_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 11111);
      goto Abort;
    }
    assume $Roles_RoleId_is_well_formed($tmp);
    $t9 := $tmp;

    // calling_role := $t9
    call $tmp := $CopyOrMoveValue($t9);
    calling_role := $tmp;
    if (true) { assume $DebugTrackLocal(13, 11096, 2, $tmp); }

    // $t10 := copy($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t10 := $tmp;

    // $t11 := Signer::address_of($t10)
    call $t11 := $Signer_address_of($t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 11284);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := exists<Roles::RoleId>($t11)
    call $tmp := $Exists($t11, $Roles_RoleId_type_value());
    $t12 := $tmp;

    // $t13 := !($t12)
    call $tmp := $Not($t12);
    $t13 := $tmp;

    // $t3 := $t13
    call $tmp := $CopyOrMoveValue($t13);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 11253, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t15 := move(calling_role)
    call $tmp := $CopyOrMoveValue(calling_role);
    $t15 := $tmp;

    // $t16 := get_field<Roles::RoleId>.role_id($t15)
    call $tmp := $GetFieldFromValue($t15, $Roles_RoleId_role_id);
    assume $IsValidU64($tmp);
    $t16 := $tmp;

    // $t17 := move($t16)
    call $tmp := $CopyOrMoveValue($t16);
    $t17 := $tmp;

    // $t18 := Roles::ASSOCIATION_ROOT_ROLE_ID()
    call $t18 := $Roles_ASSOCIATION_ROOT_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 1991);
      goto Abort;
    }
    assume $IsValidU64($t18);


    // $t19 := ==($t17, $t18)
    $tmp := $Boolean($IsEqual($t17, $t18));
    $t19 := $tmp;

    // $t5 := $t19
    call $tmp := $CopyOrMoveValue($t19);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 11322, 5, $tmp); }

    // if ($t5) goto L3 else goto L4
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L5
    goto L5;

    // L2:
L2:

    // $t21 := move($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t21 := $tmp;

    // destroy($t21)

    // $t22 := move(calling_role)
    call $tmp := $CopyOrMoveValue(calling_role);
    $t22 := $tmp;

    // destroy($t22)

    // $t23 := 1
    $tmp := $Integer(1);
    $t23 := $tmp;

    // abort($t23)
    if (true) { assume $DebugTrackAbort(13, 11253); }
    goto Abort;

    // L3:
L3:

    // $t24 := copy($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t24 := $tmp;

    // $t25 := Roles::VALIDATOR_ROLE_ID()
    call $t25 := $Roles_VALIDATOR_ROLE_ID();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2133);
      goto Abort;
    }
    assume $IsValidU64($t25);


    // $t26 := pack Roles::RoleId($t25)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t25);
    $t26 := $tmp;

    // move_to<Roles::RoleId>($t26, $t24)
    call $MoveTo($Roles_RoleId_type_value(), $t26, $t24);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 11393);
      goto Abort;
    }

    // $t27 := move($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t27 := $tmp;

    // $t28 := false
    $tmp := $Boolean(false);
    $t28 := $tmp;

    // $t29 := pack Roles::ValidatorRole($t28)
    call $tmp := $Roles_ValidatorRole_pack(0, 0, 0, $t28);
    $t29 := $tmp;

    // $t30 := false
    $tmp := $Boolean(false);
    $t30 := $tmp;

    // $t31 := pack Roles::Privilege<Roles::ValidatorRole>($t29, $t30)
    call $tmp := $Roles_Privilege_pack(0, 0, 0, $Roles_ValidatorRole_type_value(), $t29, $t30);
    $t31 := $tmp;

    // move_to<Roles::Privilege<Roles::ValidatorRole>>($t31, $t27)
    call $MoveTo($Roles_Privilege_type_value($Roles_ValidatorRole_type_value()), $t31, $t27);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 11464);
      goto Abort;
    }

    // return ()
    return;

    // L5:
L5:

    // $t32 := move($t35)
    call $tmp := $CopyOrMoveValue($t35);
    $t32 := $tmp;

    // destroy($t32)

    // $t33 := 0
    $tmp := $Integer(0);
    $t33 := $tmp;

    // abort($t33)
    if (true) { assume $DebugTrackAbort(13, 11322); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Roles_restore_capability_to_privilege ($tv0: $TypeValue, account: $Value, cap: $Value) returns ()
free requires is#$Address(account);
free requires $Roles_Capability_is_well_formed(cap);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Roles_spec_has_role_id($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual(old($Roles_spec_get_role_id($m, $txn, addr)), $Roles_spec_get_role_id($m, $txn, addr))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr))))))))));
{
    // declare local variables
    var account_address: $Value; // $AddressType()
    var owner_address: $Value; // $AddressType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $Roles_Capability_type_value($tv0)
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $AddressType()
    var $t16: $Reference; // ReferenceType($Roles_Privilege_type_value($tv0))
    var $t17: $Reference; // ReferenceType($BooleanType())
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $Roles_Capability_type_value($tv0)
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 17479, 0, account); }
    if (true) { assume $DebugTrackLocal(13, 17479, 1, cap); }

    // bytecode translation starts here
    // $t19 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t19 := $tmp;

    // $t20 := move(cap)
    call $tmp := $CopyOrMoveValue(cap);
    $t20 := $tmp;

    // $t6 := move($t19)
    call $tmp := $CopyOrMoveValue($t19);
    $t6 := $tmp;

    // $t7 := Signer::address_of($t6)
    call $t7 := $Signer_address_of($t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 17642);
      goto Abort;
    }
    assume is#$Address($t7);


    // account_address := $t7
    call $tmp := $CopyOrMoveValue($t7);
    account_address := $tmp;
    if (true) { assume $DebugTrackLocal(13, 17616, 2, $tmp); }

    // $t9 := unpack Roles::Capability<#0>($t20)
    call $t9 := $Roles_Capability_unpack($tv0, $t20);
    $t9 := $t9;

    // owner_address := $t9
    call $tmp := $CopyOrMoveValue($t9);
    owner_address := $tmp;
    if (true) { assume $DebugTrackLocal(13, 17693, 3, $tmp); }

    // $t12 := ==(owner_address, account_address)
    $tmp := $Boolean($IsEqual(owner_address, account_address));
    $t12 := $tmp;

    // $t4 := $t12
    call $tmp := $CopyOrMoveValue($t12);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 17860, 4, $tmp); }

    // if ($t4) goto L0 else goto L1
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t14 := false
    $tmp := $Boolean(false);
    $t14 := $tmp;

    // $t16 := borrow_global<Roles::Privilege<#0>>(owner_address)
    call $t16 := $BorrowGlobal(owner_address, $Roles_Privilege_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 17963);
      goto Abort;
    }
    assume $Roles_Privilege_is_well_formed($Dereference($t16));

    // UnpackRef($t16)

    // $t17 := borrow_field<Roles::Privilege<#0>>.is_extracted($t16)
    call $t17 := $BorrowField($t16, $Roles_Privilege_is_extracted);
    assume is#$Boolean($Dereference($t17));

    // Roles::Privilege <- $t16
    call $WritebackToGlobal($t16);

    // UnpackRef($t17)

    // write_ref($t17, $t14)
    call $t17 := $WriteRef($t17, $t14);

    // Roles::Privilege <- $t17
    call $WritebackToGlobal($t17);

    // Reference($t16) <- $t17
    call $t16 := $WritebackToReference($t17, $t16);

    // PackRef($t16)

    // PackRef($t17)

    // return ()
    return;

    // L2:
L2:

    // $t18 := 4
    $tmp := $Integer(4);
    $t18 := $tmp;

    // abort($t18)
    if (true) { assume $DebugTrackAbort(13, 17860); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}



// ** spec vars of module Vector



// ** spec funs of module Vector

function {:inline} $Vector_eq_push_back($tv0: $TypeValue, v1: $Value, v2: $Value, e: $Value): $Value {
    $Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($vlen_value(v1), $Integer(i#$Integer($vlen_value(v2)) + i#$Integer($Integer(1)))))) && b#$Boolean($Boolean($IsEqual($select_vector_by_value(v1, $Integer(i#$Integer($vlen_value(v1)) - i#$Integer($Integer(1)))), e))))) && b#$Boolean($Boolean($IsEqual($slice_vector(v1, $Range($Integer(0), $Integer(i#$Integer($vlen_value(v1)) - i#$Integer($Integer(1))))), $slice_vector(v2, $Range($Integer(0), $vlen_value(v2)))))))
}

function {:inline} $Vector_eq_append($tv0: $TypeValue, v: $Value, v1: $Value, v2: $Value): $Value {
    $Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($vlen_value(v), $Integer(i#$Integer($vlen_value(v1)) + i#$Integer($vlen_value(v2)))))) && b#$Boolean($Boolean($IsEqual($slice_vector(v, $Range($Integer(0), $vlen_value(v1))), v1))))) && b#$Boolean($Boolean($IsEqual($slice_vector(v, $Range($vlen_value(v1), $vlen_value(v))), v2))))
}



// ** structs of module Vector



// ** functions of module Vector

procedure {:inline 1} $Vector_singleton ($tv0: $TypeValue, e: $Value) returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var v: $Value; // $Vector_type_value($tv0)
    var $t2: $Value; // $Vector_type_value($tv0)
    var $t3: $Reference; // ReferenceType($Vector_type_value($tv0))
    var $t4: $Value; // $tv0
    var $t5: $Value; // $Vector_type_value($tv0)
    var $t6: $Value; // $tv0
    var $t7: $Value; // $Vector_type_value($tv0)
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(15, 1206, 0, e); }

    // bytecode translation starts here
    // $t6 := move(e)
    call $tmp := $CopyOrMoveValue(e);
    $t6 := $tmp;

    // $t2 := Vector::empty<#0>()
    call $t2 := $Vector_empty($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(15, 136);
      goto Abort;
    }
    assume $Vector_is_well_formed($t2);


    // v := $t2
    call $tmp := $CopyOrMoveValue($t2);
    v := $tmp;
    if (true) { assume $DebugTrackLocal(15, 1279, 1, $tmp); }

    // $t3 := borrow_local(v)
    call $t3 := $BorrowLoc(1, v);
    assume $Vector_is_well_formed($Dereference($t3));

    // UnpackRef($t3)

    // PackRef($t3)

    // $t7 := read_ref($t3)
    call $tmp := $ReadRef($t3);
    assume $Vector_is_well_formed($tmp);
    $t7 := $tmp;

    // $t7 := Vector::push_back<#0>($t7, $t6)
    call $t7 := $Vector_push_back($tv0, $t7, $t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(15, 499);
      goto Abort;
    }
    assume $Vector_is_well_formed($t7);


    // write_ref($t3, $t7)
    call $t3 := $WriteRef($t3, $t7);

    // LocalRoot(v) <- $t3
    call v := $WritebackToValue($t3, 1, v);

    // UnpackRef($t3)

    // PackRef($t3)

    // return v
    $ret0 := v;
    if (true) { assume $DebugTrackLocal(15, 1330, 8, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}



// ** spec vars of module Offer



// ** spec funs of module Offer

function {:inline} $Offer_is_allowed_recipient($m: $Memory, $txn: $Transaction, $tv0: $TypeValue, offer_addr: $Value, recipient: $Value): $Value {
    $Boolean(b#$Boolean($Boolean($IsEqual(recipient, $SelectField($ResourceValue($m, $Offer_Offer_type_value($tv0), offer_addr), $Offer_Offer_for)))) || b#$Boolean($Boolean($IsEqual(recipient, offer_addr))))
}



// ** structs of module Offer

const unique $Offer_Offer: $TypeName;
const $Offer_Offer_offered: $FieldName;
axiom $Offer_Offer_offered == 0;
const $Offer_Offer_for: $FieldName;
axiom $Offer_Offer_for == 1;
function $Offer_Offer_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Offer_Offer, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0][1 := $AddressType()], 2))
}
function {:inline} $Offer_Offer_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && is#$Address($SelectField($this, $Offer_Offer_for))
}
function {:inline} $Offer_Offer_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && is#$Address($SelectField($this, $Offer_Offer_for))
}

axiom (forall m: $Memory, a: $Value, $tv0: $TypeValue :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Offer_Offer_is_well_formed($ResourceValue(m, $Offer_Offer_type_value($tv0), a))
);

procedure {:inline 1} $Offer_Offer_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, offered: $Value, for: $Value) returns ($struct: $Value)
{
    assume is#$Address(for);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := offered][1 := for], 2));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Offer_Offer_unpack($tv0: $TypeValue, $struct: $Value) returns (offered: $Value, for: $Value)
{
    assume is#$Vector($struct);
    offered := $SelectField($struct, $Offer_Offer_offered);
    for := $SelectField($struct, $Offer_Offer_for);
    assume is#$Address(for);
}



// ** functions of module Offer

procedure {:inline 1} $Offer_address_of ($tv0: $TypeValue, offer_address: $Value) returns ($ret0: $Value)
free requires is#$Address(offer_address);
requires $ExistsTxnSenderAccount($m, $txn);
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($m, $Offer_Offer_type_value($tv0), offer_address))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(!b#$Boolean($ResourceExists($m, $Offer_Offer_type_value($tv0), offer_address)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall ty: $Value :: is#$Type(ty) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean(old($ResourceExists($m, $Offer_Offer_type_value(t#$Type(ty)), addr))))) ==> b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $Offer_Offer_type_value(t#$Type(ty)), addr)))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall ty: $Value :: is#$Type(ty) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($ResourceExists($m, $Offer_Offer_type_value(t#$Type(ty)), addr))) ==> b#$Boolean($Boolean(b#$Boolean($ResourceExists($m, $Offer_Offer_type_value(t#$Type(ty)), addr)) && b#$Boolean($Boolean($IsEqual($ResourceValue($m, $Offer_Offer_type_value(t#$Type(ty)), addr), old($ResourceValue($m, $Offer_Offer_type_value(t#$Type(ty)), addr))))))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $SelectField($ResourceValue($m, $Offer_Offer_type_value($tv0), offer_address), $Offer_Offer_for)))));
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $Offer_Offer_type_value($tv0)
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(11, 1473, 0, offer_address); }

    // bytecode translation starts here
    // $t5 := move(offer_address)
    call $tmp := $CopyOrMoveValue(offer_address);
    $t5 := $tmp;

    // $t2 := get_global<Offer::Offer<#0>>($t5)
    call $tmp := $GetGlobal($t5, $Offer_Offer_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 1558);
      goto Abort;
    }
    assume $Offer_Offer_is_well_formed($tmp);
    $t2 := $tmp;

    // $t3 := get_field<Offer::Offer<#0>>.for($t2)
    call $tmp := $GetFieldFromValue($t2, $Offer_Offer_for);
    assume is#$Address($tmp);
    $t3 := $tmp;

    // $t4 := move($t3)
    call $tmp := $CopyOrMoveValue($t3);
    $t4 := $tmp;

    // return $t4
    $ret0 := $t4;
    if (true) { assume $DebugTrackLocal(11, 1558, 6, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Offer_create ($tv0: $TypeValue, account: $Value, offered: $Value, for: $Value) returns ()
free requires is#$Address(account);
free requires is#$Address(for);
requires $ExistsTxnSenderAccount($m, $txn);
ensures b#$Boolean(old($ResourceExists($m, $Offer_Offer_type_value($tv0), $Signer_get_address($m, $txn, account)))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($ResourceExists($m, $Offer_Offer_type_value($tv0), $Signer_get_address($m, $txn, account))))));
ensures !$abort_flag ==> (b#$Boolean($ResourceExists($m, $Offer_Offer_type_value($tv0), $Signer_get_address($m, $txn, account))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ResourceValue($m, $Offer_Offer_type_value($tv0), $Signer_get_address($m, $txn, account)), $Vector($ExtendValueArray($ExtendValueArray($EmptyValueArray(), offered), for))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall ty: $Value :: is#$Type(ty) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($ResourceExists($m, $Offer_Offer_type_value(t#$Type(ty)), addr))) ==> b#$Boolean($Boolean(b#$Boolean($ResourceExists($m, $Offer_Offer_type_value(t#$Type(ty)), addr)) && b#$Boolean($Boolean($IsEqual($ResourceValue($m, $Offer_Offer_type_value(t#$Type(ty)), addr), old($ResourceValue($m, $Offer_Offer_type_value(t#$Type(ty)), addr))))))))))))))));
{
    // declare local variables
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $tv0
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $Offer_Offer_type_value($tv0)
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $tv0
    var $t9: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(11, 459, 0, account); }
    if (true) { assume $DebugTrackLocal(11, 459, 1, offered); }
    if (true) { assume $DebugTrackLocal(11, 459, 2, for); }

    // bytecode translation starts here
    // $t7 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t7 := $tmp;

    // $t8 := move(offered)
    call $tmp := $CopyOrMoveValue(offered);
    $t8 := $tmp;

    // $t9 := move(for)
    call $tmp := $CopyOrMoveValue(for);
    $t9 := $tmp;

    // $t3 := move($t7)
    call $tmp := $CopyOrMoveValue($t7);
    $t3 := $tmp;

    // $t6 := pack Offer::Offer<#0>($t8, $t9)
    call $tmp := $Offer_Offer_pack(0, 0, 0, $tv0, $t8, $t9);
    $t6 := $tmp;

    // move_to<Offer::Offer<#0>>($t6, $t3)
    call $MoveTo($Offer_Offer_type_value($tv0), $t6, $t3);
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 542);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Offer_exists_at ($tv0: $TypeValue, offer_address: $Value) returns ($ret0: $Value)
free requires is#$Address(offer_address);
requires $ExistsTxnSenderAccount($m, $txn);
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall ty: $Value :: is#$Type(ty) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean(old($ResourceExists($m, $Offer_Offer_type_value(t#$Type(ty)), addr))))) ==> b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $Offer_Offer_type_value(t#$Type(ty)), addr)))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall ty: $Value :: is#$Type(ty) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($ResourceExists($m, $Offer_Offer_type_value(t#$Type(ty)), addr))) ==> b#$Boolean($Boolean(b#$Boolean($ResourceExists($m, $Offer_Offer_type_value(t#$Type(ty)), addr)) && b#$Boolean($Boolean($IsEqual($ResourceValue($m, $Offer_Offer_type_value(t#$Type(ty)), addr), old($ResourceValue($m, $Offer_Offer_type_value(t#$Type(ty)), addr))))))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $ResourceExists($m, $Offer_Offer_type_value($tv0), offer_address)))));
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(11, 1251, 0, offer_address); }

    // bytecode translation starts here
    // $t3 := move(offer_address)
    call $tmp := $CopyOrMoveValue(offer_address);
    $t3 := $tmp;

    // $t2 := exists<Offer::Offer<#0>>($t3)
    call $tmp := $Exists($t3, $Offer_Offer_type_value($tv0));
    $t2 := $tmp;

    // return $t2
    $ret0 := $t2;
    if (true) { assume $DebugTrackLocal(11, 1317, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Offer_redeem ($tv0: $TypeValue, account: $Value, offer_address: $Value) returns ($ret0: $Value)
free requires is#$Address(account);
free requires is#$Address(offer_address);
requires $ExistsTxnSenderAccount($m, $txn);
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($m, $Offer_Offer_type_value($tv0), offer_address))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($Offer_is_allowed_recipient($m, $txn, $tv0, offer_address, $Signer_get_address($m, $txn, account)))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(!b#$Boolean($ResourceExists($m, $Offer_Offer_type_value($tv0), offer_address))))))
    || b#$Boolean(old(($Boolean(!b#$Boolean($Offer_is_allowed_recipient($m, $txn, $tv0, offer_address, $Signer_get_address($m, $txn, account))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall ty: $Value :: is#$Type(ty) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean(old($ResourceExists($m, $Offer_Offer_type_value(t#$Type(ty)), addr))))) ==> b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $Offer_Offer_type_value(t#$Type(ty)), addr)))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($ResourceExists($m, $Offer_Offer_type_value($tv0), offer_address))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $Offer_Offer_type_value($tv0), offer_address)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, old($SelectField($ResourceValue($m, $Offer_Offer_type_value($tv0), offer_address), $Offer_Offer_offered))))));
{
    // declare local variables
    var for: $Value; // $AddressType()
    var offered: $Value; // $tv0
    var sender: $Value; // $AddressType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $Offer_Offer_type_value($tv0)
    var $t10: $Value; // $tv0
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $BooleanType()
    var $t17: $Value; // $BooleanType()
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $BooleanType()
    var $t21: $Value; // $BooleanType()
    var $t22: $Value; // $BooleanType()
    var $t23: $Value; // $tv0
    var $t24: $Value; // $IntegerType()
    var $t25: $Value; // $AddressType()
    var $t26: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(11, 833, 0, account); }
    if (true) { assume $DebugTrackLocal(11, 833, 1, offer_address); }

    // bytecode translation starts here
    // $t25 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t25 := $tmp;

    // $t26 := move(offer_address)
    call $tmp := $CopyOrMoveValue(offer_address);
    $t26 := $tmp;

    // $t9 := move_from<Offer::Offer<#0>>($t26)
    call $tmp := $MoveFrom($t26, $Offer_Offer_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 970);
      goto Abort;
    }
    assume $Offer_Offer_is_well_formed($tmp);
    $t9 := $tmp;

    // ($t10, $t11) := unpack Offer::Offer<#0>($t9)
    call $t10, $t11 := $Offer_Offer_unpack($tv0, $t9);
    $t10 := $t10;
    $t11 := $t11;

    // for := $t11
    call $tmp := $CopyOrMoveValue($t11);
    for := $tmp;
    if (true) { assume $DebugTrackLocal(11, 962, 2, $tmp); }

    // offered := $t10
    call $tmp := $CopyOrMoveValue($t10);
    offered := $tmp;
    if (true) { assume $DebugTrackLocal(11, 953, 3, $tmp); }

    // $t12 := move($t25)
    call $tmp := $CopyOrMoveValue($t25);
    $t12 := $tmp;

    // $t13 := Signer::address_of($t12)
    call $t13 := $Signer_address_of($t12);
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 1037);
      goto Abort;
    }
    assume is#$Address($t13);


    // sender := $t13
    call $tmp := $CopyOrMoveValue($t13);
    sender := $tmp;
    if (true) { assume $DebugTrackLocal(11, 1020, 4, $tmp); }

    // $t16 := ==(sender, for)
    $tmp := $Boolean($IsEqual(sender, for));
    $t16 := $tmp;

    // if ($t16) goto L0 else goto L1
    $tmp := $t16;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t17 := true
    $tmp := $Boolean(true);
    $t17 := $tmp;

    // $t7 := $t17
    call $tmp := $CopyOrMoveValue($t17);
    $t7 := $tmp;
    if (true) { assume $DebugTrackLocal(11, 1110, 7, $tmp); }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t20 := ==(sender, $t26)
    $tmp := $Boolean($IsEqual(sender, $t26));
    $t20 := $tmp;

    // $t7 := $t20
    call $tmp := $CopyOrMoveValue($t20);
    $t7 := $tmp;
    if (true) { assume $DebugTrackLocal(11, 1110, 7, $tmp); }

    // goto L3
    goto L3;

    // L3:
L3:

    // $t5 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(11, 1103, 5, $tmp); }

    // if ($t5) goto L4 else goto L5
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L4; } else { goto L5; }

    // L5:
L5:

    // goto L6
    goto L6;

    // L4:
L4:

    // return offered
    $ret0 := offered;
    if (true) { assume $DebugTrackLocal(11, 1161, 27, $ret0); }
    return;

    // L6:
L6:

    // $t24 := 11
    $tmp := $Integer(11);
    $t24 := $tmp;

    // abort($t24)
    if (true) { assume $DebugTrackAbort(11, 1103); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}



// ** spec vars of module LCS



// ** spec funs of module LCS



// ** structs of module LCS



// ** functions of module LCS



// ** spec vars of module Event

var $Event_ehg_destroyed : $Value where is#$Boolean($Event_ehg_destroyed);
var $Event_total_num_of_event_handles : [$TypeValue]$Value where (forall $tv0: $TypeValue :: $IsValidNum($Event_total_num_of_event_handles[$tv0]));


// ** spec funs of module Event

function {:inline} $Event_ehg_exists($m: $Memory, $txn: $Transaction, addr: $Value): $Value {
    $ResourceExists($m, $Event_EventHandleGenerator_type_value(), addr)
}

function {:inline} $Event_get_ehg($m: $Memory, $txn: $Transaction, addr: $Value): $Value {
    $ResourceValue($m, $Event_EventHandleGenerator_type_value(), addr)
}

function {:inline} $Event_serialized_ehg_counter($m: $Memory, $txn: $Transaction, ehg: $Value): $Value {
    $LCS_serialize($m, $txn, $IntegerType(), $SelectField(ehg, $Event_EventHandleGenerator_counter))
}

function {:inline} $Event_serialized_ehg_addr($m: $Memory, $txn: $Transaction, ehg: $Value): $Value {
    $LCS_serialize($m, $txn, $AddressType(), $SelectField(ehg, $Event_EventHandleGenerator_addr))
}



// ** structs of module Event

const unique $Event_EventHandle: $TypeName;
const $Event_EventHandle_counter: $FieldName;
axiom $Event_EventHandle_counter == 0;
const $Event_EventHandle_guid: $FieldName;
axiom $Event_EventHandle_guid == 1;
function $Event_EventHandle_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Event_EventHandle, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $IntegerType()][1 := $Vector_type_value($IntegerType())], 2))
}
function {:inline} $Event_EventHandle_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && $IsValidU64($SelectField($this, $Event_EventHandle_counter))
      && $Vector_is_well_formed($SelectField($this, $Event_EventHandle_guid)) && (forall $$0: int :: {$select_vector($SelectField($this, $Event_EventHandle_guid),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Event_EventHandle_guid)) ==> $IsValidU8($select_vector($SelectField($this, $Event_EventHandle_guid),$$0)))
}
function {:inline} $Event_EventHandle_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && $IsValidU64($SelectField($this, $Event_EventHandle_counter))
      && $Vector_is_well_formed($SelectField($this, $Event_EventHandle_guid)) && (forall $$0: int :: {$select_vector($SelectField($this, $Event_EventHandle_guid),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Event_EventHandle_guid)) ==> $IsValidU8($select_vector($SelectField($this, $Event_EventHandle_guid),$$0)))
}

axiom (forall m: $Memory, a: $Value, $tv0: $TypeValue :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Event_EventHandle_is_well_formed($ResourceValue(m, $Event_EventHandle_type_value($tv0), a))
);

procedure {:inline 1} $Event_EventHandle_before_update_inv($tv0: $TypeValue, $before: $Value) {
    $Event_total_num_of_event_handles := $Event_total_num_of_event_handles[$tv0 := $Integer(i#$Integer($Event_total_num_of_event_handles[$tv0]) - i#$Integer($Integer(1)))];
}

procedure {:inline 1} $Event_EventHandle_after_update_inv($tv0: $TypeValue, $after: $Value) {
    $Event_total_num_of_event_handles := $Event_total_num_of_event_handles[$tv0 := $Integer(i#$Integer($Event_total_num_of_event_handles[$tv0]) + i#$Integer($Integer(1)))];
}

procedure {:inline 1} $Event_EventHandle_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, counter: $Value, guid: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(counter);
    assume $Vector_is_well_formed(guid) && (forall $$0: int :: {$select_vector(guid,$$0)} $$0 >= 0 && $$0 < $vlen(guid) ==> $IsValidU8($select_vector(guid,$$0)));
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := counter][1 := guid], 2));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
    $Event_total_num_of_event_handles := $Event_total_num_of_event_handles[$tv0 := $Integer(i#$Integer($Event_total_num_of_event_handles[$tv0]) + i#$Integer($Integer(1)))];
}

procedure {:inline 1} $Event_EventHandle_unpack($tv0: $TypeValue, $struct: $Value) returns (counter: $Value, guid: $Value)
{
    assume is#$Vector($struct);
    counter := $SelectField($struct, $Event_EventHandle_counter);
    assume $IsValidU64(counter);
    guid := $SelectField($struct, $Event_EventHandle_guid);
    assume $Vector_is_well_formed(guid) && (forall $$0: int :: {$select_vector(guid,$$0)} $$0 >= 0 && $$0 < $vlen(guid) ==> $IsValidU8($select_vector(guid,$$0)));
    $Event_total_num_of_event_handles := $Event_total_num_of_event_handles[$tv0 := $Integer(i#$Integer($Event_total_num_of_event_handles[$tv0]) - i#$Integer($Integer(1)))];
}

const unique $Event_EventHandleGenerator: $TypeName;
const $Event_EventHandleGenerator_counter: $FieldName;
axiom $Event_EventHandleGenerator_counter == 0;
const $Event_EventHandleGenerator_addr: $FieldName;
axiom $Event_EventHandleGenerator_addr == 1;
function $Event_EventHandleGenerator_type_value(): $TypeValue {
    $StructType($Event_EventHandleGenerator, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $IntegerType()][1 := $AddressType()], 2))
}
function {:inline} $Event_EventHandleGenerator_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && $IsValidU64($SelectField($this, $Event_EventHandleGenerator_counter))
      && is#$Address($SelectField($this, $Event_EventHandleGenerator_addr))
}
function {:inline} $Event_EventHandleGenerator_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && $IsValidU64($SelectField($this, $Event_EventHandleGenerator_counter))
      && is#$Address($SelectField($this, $Event_EventHandleGenerator_addr))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Event_EventHandleGenerator_is_well_formed($ResourceValue(m, $Event_EventHandleGenerator_type_value(), a))
);

procedure {:inline 1} $Event_EventHandleGenerator_before_update_inv($before: $Value) {
    $Event_ehg_destroyed := $Boolean(true);
}

procedure {:inline 1} $Event_EventHandleGenerator_after_update_inv($after: $Value) {
    $Event_ehg_destroyed := $Event_ehg_destroyed;
}

procedure {:inline 1} $Event_EventHandleGenerator_pack($file_id: int, $byte_index: int, $var_idx: int, counter: $Value, addr: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(counter);
    assume is#$Address(addr);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := counter][1 := addr], 2));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
    $Event_ehg_destroyed := $Event_ehg_destroyed;
}

procedure {:inline 1} $Event_EventHandleGenerator_unpack($struct: $Value) returns (counter: $Value, addr: $Value)
{
    assume is#$Vector($struct);
    counter := $SelectField($struct, $Event_EventHandleGenerator_counter);
    assume $IsValidU64(counter);
    addr := $SelectField($struct, $Event_EventHandleGenerator_addr);
    assume is#$Address(addr);
    $Event_ehg_destroyed := $Boolean(true);
}



// ** functions of module Event

procedure {:inline 1} $Event_destroy_handle ($tv0: $TypeValue, handle: $Value) returns ()
free requires $Event_EventHandle_is_well_formed(handle);
requires $ExistsTxnSenderAccount($m, $txn);
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(false)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Event_total_num_of_event_handles[$tv0], $Integer(i#$Integer(old($Event_total_num_of_event_handles[$tv0])) - i#$Integer($Integer(1)))))));
{
    // declare local variables
    var $t1: $Value; // $Event_EventHandle_type_value($tv0)
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $Vector_type_value($IntegerType())
    var $t4: $Value; // $Event_EventHandle_type_value($tv0)
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(5, 3326, 0, handle); }

    // bytecode translation starts here
    // $t4 := move(handle)
    call $tmp := $CopyOrMoveValue(handle);
    $t4 := $tmp;

    // ($t2, $t3) := unpack Event::EventHandle<#0>($t4)
    call $t2, $t3 := $Event_EventHandle_unpack($tv0, $t4);
    $t2 := $t2;
    $t3 := $t3;

    // destroy($t3)

    // destroy($t2)

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Event_emit_event ($tv0: $TypeValue, handle_ref: $Value, msg: $Value) returns ($ret0: $Value)
free requires $Event_EventHandle_is_well_formed(handle_ref);
requires $ExistsTxnSenderAccount($m, $txn);
ensures b#$Boolean(old($Boolean(i#$Integer($Integer(i#$Integer($SelectField(handle_ref, $Event_EventHandle_counter)) + i#$Integer($Integer(1)))) > i#$Integer($Integer($MAX_U64))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(i#$Integer($Integer(i#$Integer($SelectField(handle_ref, $Event_EventHandle_counter)) + i#$Integer($Integer(1)))) > i#$Integer($Integer($MAX_U64)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($SelectField($ret0, $Event_EventHandle_counter), $Integer(i#$Integer(old($SelectField(handle_ref, $Event_EventHandle_counter))) + i#$Integer($Integer(1)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($SelectField($ret0, $Event_EventHandle_guid), old($SelectField(handle_ref, $Event_EventHandle_guid))))));
{
    // declare local variables
    var guid: $Value; // $Vector_type_value($IntegerType())
    var $t3: $Reference; // ReferenceType($Event_EventHandle_type_value($tv0))
    var $t4: $Value; // $Vector_type_value($IntegerType())
    var $t5: $Value; // $Vector_type_value($IntegerType())
    var $t6: $Value; // $Vector_type_value($IntegerType())
    var $t7: $Reference; // ReferenceType($Event_EventHandle_type_value($tv0))
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $tv0
    var $t11: $Reference; // ReferenceType($Event_EventHandle_type_value($tv0))
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $IntegerType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Reference; // ReferenceType($Event_EventHandle_type_value($tv0))
    var $t17: $Reference; // ReferenceType($IntegerType())
    var $t18: $Value; // $Event_EventHandle_type_value($tv0)
    var $t19: $Reference; // ReferenceType($Event_EventHandle_type_value($tv0))
    var $t20: $Value; // $tv0
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(5, 2814, 0, handle_ref); }
    if (true) { assume $DebugTrackLocal(5, 2814, 1, msg); }

    // bytecode translation starts here
    // $t18 := move(handle_ref)
    call $tmp := $CopyOrMoveValue(handle_ref);
    $t18 := $tmp;

    // $t20 := move(msg)
    call $tmp := $CopyOrMoveValue(msg);
    $t20 := $tmp;

    // $t19 := borrow_local($t18)
    call $t19 := $BorrowLoc(18, $t18);
    assume $Event_EventHandle_is_well_formed($Dereference($t19));

    // UnpackRef($t19)
    call $Event_EventHandle_before_update_inv($tv0, $Dereference($t19));

    // $t3 := copy($t19)
    call $t3 := $CopyOrMoveRef($t19);

    // $t4 := get_field<Event::EventHandle<#0>>.guid($t3)
    call $tmp := $GetFieldFromReference($t3, $Event_EventHandle_guid);
    assume $Vector_is_well_formed($tmp) && (forall $$0: int :: {$select_vector($tmp,$$0)} $$0 >= 0 && $$0 < $vlen($tmp) ==> $IsValidU8($select_vector($tmp,$$0)));
    $t4 := $tmp;

    // Reference($t19) <- $t3
    call $t19 := $WritebackToReference($t3, $t19);

    // $t5 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t5 := $tmp;

    // guid := $t5
    call $tmp := $CopyOrMoveValue($t5);
    guid := $tmp;
    if (true) { assume $DebugTrackLocal(5, 2904, 2, $tmp); }

    // $t7 := copy($t19)
    call $t7 := $CopyOrMoveRef($t19);

    // $t8 := get_field<Event::EventHandle<#0>>.counter($t7)
    call $tmp := $GetFieldFromReference($t7, $Event_EventHandle_counter);
    assume $IsValidU64($tmp);
    $t8 := $tmp;

    // Reference($t19) <- $t7
    call $t19 := $WritebackToReference($t7, $t19);

    // $t9 := move($t8)
    call $tmp := $CopyOrMoveValue($t8);
    $t9 := $tmp;

    // Event::write_to_event_store<#0>(guid, $t9, $t20)
    call $Event_write_to_event_store($tv0, guid, $t9, $t20);
    if ($abort_flag) {
      assume $DebugTrackAbort(5, 3216);
      goto Abort;
    }

    // $t11 := copy($t19)
    call $t11 := $CopyOrMoveRef($t19);

    // $t12 := get_field<Event::EventHandle<#0>>.counter($t11)
    call $tmp := $GetFieldFromReference($t11, $Event_EventHandle_counter);
    assume $IsValidU64($tmp);
    $t12 := $tmp;

    // Reference($t19) <- $t11
    call $t19 := $WritebackToReference($t11, $t19);

    // $t13 := move($t12)
    call $tmp := $CopyOrMoveValue($t12);
    $t13 := $tmp;

    // $t14 := 1
    $tmp := $Integer(1);
    $t14 := $tmp;

    // $t15 := +($t13, $t14)
    call $tmp := $AddU64($t13, $t14);
    if ($abort_flag) {
      assume $DebugTrackAbort(5, 3043);
      goto Abort;
    }
    $t15 := $tmp;

    // $t16 := move($t19)
    call $t16 := $CopyOrMoveRef($t19);

    // $t17 := borrow_field<Event::EventHandle<#0>>.counter($t16)
    call $t17 := $BorrowField($t16, $Event_EventHandle_counter);
    assume $IsValidU64($Dereference($t17));

    // LocalRoot($t18) <- $t16
    call $t18 := $WritebackToValue($t16, 18, $t18);

    // UnpackRef($t17)

    // write_ref($t17, $t15)
    call $t17 := $WriteRef($t17, $t15);

    // LocalRoot($t18) <- $t17
    call $t18 := $WritebackToValue($t17, 18, $t18);

    // Reference($t16) <- $t17
    call $t16 := $WritebackToReference($t17, $t16);

    // PackRef($t16)
    call $Event_EventHandle_after_update_inv($tv0, $Dereference($t16));

    // PackRef($t17)

    // return $t18
    $ret0 := $t18;
    if (true) { assume $DebugTrackLocal(5, 3046, 21, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Event_fresh_guid (counter: $Value) returns ($ret0: $Value, $ret1: $Value)
free requires $Event_EventHandleGenerator_is_well_formed(counter);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(true))
    || b#$Boolean($Boolean(i#$Integer($Integer(i#$Integer($SelectField(counter, $Event_EventHandleGenerator_counter)) + i#$Integer($Integer(1)))) > i#$Integer($Integer($MAX_U64))));
ensures b#$Boolean(old($Boolean(i#$Integer($Integer(i#$Integer($SelectField(counter, $Event_EventHandleGenerator_counter)) + i#$Integer($Integer(1)))) > i#$Integer($Integer($MAX_U64))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(i#$Integer($Integer(i#$Integer($SelectField(counter, $Event_EventHandleGenerator_counter)) + i#$Integer($Integer(1)))) > i#$Integer($Integer($MAX_U64)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean(old($Event_ehg_exists($m, $txn, addr))))) && b#$Boolean($Event_ehg_exists($m, $txn, addr)))) ==> b#$Boolean($Boolean($IsEqual($SelectField($Event_get_ehg($m, $txn, addr), $Event_EventHandleGenerator_counter), $Integer(0))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Event_ehg_exists($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Event_ehg_exists($m, $txn, addr)) && b#$Boolean($Boolean(i#$Integer($SelectField($Event_get_ehg($m, $txn, addr), $Event_EventHandleGenerator_counter)) >= i#$Integer(old($SelectField($Event_get_ehg($m, $txn, addr), $Event_EventHandleGenerator_counter)))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($SelectField($ret1, $Event_EventHandleGenerator_counter), $Integer(i#$Integer($SelectField(old(counter), $Event_EventHandleGenerator_counter)) + i#$Integer($Integer(1)))))));
ensures !$abort_flag ==> (b#$Boolean($Vector_eq_append($IntegerType(), $ret0, old($Event_serialized_ehg_counter($m, $txn, counter)), old($Event_serialized_ehg_addr($m, $txn, counter)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(true)));
{
    // declare local variables
    var count_bytes: $Value; // $Vector_type_value($IntegerType())
    var sender_bytes: $Value; // $Vector_type_value($IntegerType())
    var $t3: $Reference; // ReferenceType($Event_EventHandleGenerator_type_value())
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $Vector_type_value($IntegerType())
    var $t6: $Reference; // ReferenceType($Event_EventHandleGenerator_type_value())
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $Vector_type_value($IntegerType())
    var $t9: $Reference; // ReferenceType($Event_EventHandleGenerator_type_value())
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Reference; // ReferenceType($Event_EventHandleGenerator_type_value())
    var $t15: $Reference; // ReferenceType($IntegerType())
    var $t16: $Reference; // ReferenceType($Vector_type_value($IntegerType()))
    var $t17: $Value; // $Vector_type_value($IntegerType())
    var $t18: $Value; // $Vector_type_value($IntegerType())
    var $t19: $Value; // $Event_EventHandleGenerator_type_value()
    var $t20: $Reference; // ReferenceType($Event_EventHandleGenerator_type_value())
    var $t21: $Value; // $Vector_type_value($IntegerType())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(5, 1871, 0, counter); }

    // bytecode translation starts here
    // $t19 := move(counter)
    call $tmp := $CopyOrMoveValue(counter);
    $t19 := $tmp;

    // $t20 := borrow_local($t19)
    call $t20 := $BorrowLoc(19, $t19);
    assume $Event_EventHandleGenerator_is_well_formed($Dereference($t20));

    // $t3 := copy($t20)
    call $t3 := $CopyOrMoveRef($t20);

    // $t4 := get_field<Event::EventHandleGenerator>.addr($t3)
    call $tmp := $GetFieldFromReference($t3, $Event_EventHandleGenerator_addr);
    assume is#$Address($tmp);
    $t4 := $tmp;

    // Reference($t20) <- $t3
    call $t20 := $WritebackToReference($t3, $t20);

    // $t5 := LCS::to_bytes<address>($t4)
    call $t5 := $LCS_to_bytes($AddressType(), $t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(5, 1968);
      goto Abort;
    }
    assume $Vector_is_well_formed($t5) && (forall $$0: int :: {$select_vector($t5,$$0)} $$0 >= 0 && $$0 < $vlen($t5) ==> $IsValidU8($select_vector($t5,$$0)));


    // sender_bytes := $t5
    call $tmp := $CopyOrMoveValue($t5);
    sender_bytes := $tmp;
    if (true) { assume $DebugTrackLocal(5, 1948, 2, $tmp); }

    // $t6 := copy($t20)
    call $t6 := $CopyOrMoveRef($t20);

    // $t7 := get_field<Event::EventHandleGenerator>.counter($t6)
    call $tmp := $GetFieldFromReference($t6, $Event_EventHandleGenerator_counter);
    assume $IsValidU64($tmp);
    $t7 := $tmp;

    // Reference($t20) <- $t6
    call $t20 := $WritebackToReference($t6, $t20);

    // $t8 := LCS::to_bytes<u64>($t7)
    call $t8 := $LCS_to_bytes($IntegerType(), $t7);
    if ($abort_flag) {
      assume $DebugTrackAbort(5, 2024);
      goto Abort;
    }
    assume $Vector_is_well_formed($t8) && (forall $$0: int :: {$select_vector($t8,$$0)} $$0 >= 0 && $$0 < $vlen($t8) ==> $IsValidU8($select_vector($t8,$$0)));


    // count_bytes := $t8
    call $tmp := $CopyOrMoveValue($t8);
    count_bytes := $tmp;
    if (true) { assume $DebugTrackLocal(5, 2005, 1, $tmp); }

    // $t9 := copy($t20)
    call $t9 := $CopyOrMoveRef($t20);

    // $t10 := get_field<Event::EventHandleGenerator>.counter($t9)
    call $tmp := $GetFieldFromReference($t9, $Event_EventHandleGenerator_counter);
    assume $IsValidU64($tmp);
    $t10 := $tmp;

    // Reference($t20) <- $t9
    call $t20 := $WritebackToReference($t9, $t20);

    // $t11 := move($t10)
    call $tmp := $CopyOrMoveValue($t10);
    $t11 := $tmp;

    // $t12 := 1
    $tmp := $Integer(1);
    $t12 := $tmp;

    // $t13 := +($t11, $t12)
    call $tmp := $AddU64($t11, $t12);
    if ($abort_flag) {
      assume $DebugTrackAbort(5, 2094);
      goto Abort;
    }
    $t13 := $tmp;

    // $t14 := move($t20)
    call $t14 := $CopyOrMoveRef($t20);

    // $t15 := borrow_field<Event::EventHandleGenerator>.counter($t14)
    call $t15 := $BorrowField($t14, $Event_EventHandleGenerator_counter);
    assume $IsValidU64($Dereference($t15));

    // LocalRoot($t19) <- $t14
    call $t19 := $WritebackToValue($t14, 19, $t19);

    // UnpackRef($t15)

    // write_ref($t15, $t13)
    call $t15 := $WriteRef($t15, $t13);

    // LocalRoot($t19) <- $t15
    call $t19 := $WritebackToValue($t15, 19, $t19);

    // Reference($t14) <- $t15
    call $t14 := $WritebackToReference($t15, $t14);

    // PackRef($t15)

    // $t16 := borrow_local(count_bytes)
    call $t16 := $BorrowLoc(1, count_bytes);
    assume $Vector_is_well_formed($Dereference($t16)) && (forall $$1: int :: {$select_vector($Dereference($t16),$$1)} $$1 >= 0 && $$1 < $vlen($Dereference($t16)) ==> $IsValidU8($select_vector($Dereference($t16),$$1)));

    // UnpackRef($t16)

    // PackRef($t16)

    // $t21 := read_ref($t16)
    call $tmp := $ReadRef($t16);
    assume $Vector_is_well_formed($tmp) && (forall $$0: int :: {$select_vector($tmp,$$0)} $$0 >= 0 && $$0 < $vlen($tmp) ==> $IsValidU8($select_vector($tmp,$$0)));
    $t21 := $tmp;

    // $t21 := Vector::append<u8>($t21, sender_bytes)
    call $t21 := $Vector_append($IntegerType(), $t21, sender_bytes);
    if ($abort_flag) {
      assume $DebugTrackAbort(5, 2213);
      goto Abort;
    }
    assume $Vector_is_well_formed($t21) && (forall $$0: int :: {$select_vector($t21,$$0)} $$0 >= 0 && $$0 < $vlen($t21) ==> $IsValidU8($select_vector($t21,$$0)));


    // write_ref($t16, $t21)
    call $t16 := $WriteRef($t16, $t21);

    // LocalRoot(count_bytes) <- $t16
    call count_bytes := $WritebackToValue($t16, 1, count_bytes);

    // UnpackRef($t16)

    // PackRef($t16)

    // return (count_bytes, $t19)
    $ret0 := count_bytes;
    if (true) { assume $DebugTrackLocal(5, 2262, 22, $ret0); }
    $ret1 := $t19;
    if (true) { assume $DebugTrackLocal(5, 2262, 23, $ret1); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}

procedure {:inline 1} $Event_new_event_handle ($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
ensures b#$Boolean(old($Boolean(!b#$Boolean($Event_ehg_exists($m, $txn, $Signer_get_address($m, $txn, account)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(i#$Integer($Integer(i#$Integer($SelectField($Event_get_ehg($m, $txn, $Signer_get_address($m, $txn, account)), $Event_EventHandleGenerator_counter)) + i#$Integer($Integer(1)))) > i#$Integer($Integer($MAX_U64))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(!b#$Boolean($Event_ehg_exists($m, $txn, $Signer_get_address($m, $txn, account)))))))
    || b#$Boolean(old(($Boolean(i#$Integer($Integer(i#$Integer($SelectField($Event_get_ehg($m, $txn, $Signer_get_address($m, $txn, account)), $Event_EventHandleGenerator_counter)) + i#$Integer($Integer(1)))) > i#$Integer($Integer($MAX_U64)))))));
ensures !$abort_flag ==> (b#$Boolean($Event_ehg_exists($m, $txn, $Signer_get_address($m, $txn, account))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($SelectField($Event_get_ehg($m, $txn, $Signer_get_address($m, $txn, account)), $Event_EventHandleGenerator_counter), $Integer(i#$Integer(old($SelectField($Event_get_ehg($m, $txn, $Signer_get_address($m, $txn, account)), $Event_EventHandleGenerator_counter))) + i#$Integer($Integer(1)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($SelectField($ret0, $Event_EventHandle_counter), $Integer(0)))));
ensures !$abort_flag ==> (b#$Boolean($Vector_eq_append($IntegerType(), $SelectField($ret0, $Event_EventHandle_guid), old($Event_serialized_ehg_counter($m, $txn, $Event_get_ehg($m, $txn, $Signer_get_address($m, $txn, account)))), old($Event_serialized_ehg_addr($m, $txn, $Event_get_ehg($m, $txn, $Signer_get_address($m, $txn, account)))))));
{
    // declare local variables
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $AddressType()
    var $t4: $Reference; // ReferenceType($Event_EventHandleGenerator_type_value())
    var $t5: $Value; // $Vector_type_value($IntegerType())
    var $t6: $Value; // $Event_EventHandle_type_value($tv0)
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $Event_EventHandleGenerator_type_value()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(5, 2361, 0, account); }

    // bytecode translation starts here
    // $t7 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t7 := $tmp;

    // $t1 := 0
    $tmp := $Integer(0);
    $t1 := $tmp;

    // $t2 := move($t7)
    call $tmp := $CopyOrMoveValue($t7);
    $t2 := $tmp;

    // $t3 := Signer::address_of($t2)
    call $t3 := $Signer_address_of($t2);
    if ($abort_flag) {
      assume $DebugTrackAbort(5, 2598);
      goto Abort;
    }
    assume is#$Address($t3);


    // $t4 := borrow_global<Event::EventHandleGenerator>($t3)
    call $t4 := $BorrowGlobal($t3, $Event_EventHandleGenerator_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(5, 2550);
      goto Abort;
    }
    assume $Event_EventHandleGenerator_is_well_formed($Dereference($t4));

    // UnpackRef($t4)
    call $Event_EventHandleGenerator_before_update_inv($Dereference($t4));

    // $t8 := read_ref($t4)
    call $tmp := $ReadRef($t4);
    assume $Event_EventHandleGenerator_is_well_formed($tmp);
    $t8 := $tmp;

    // ($t5, $t8) := Event::fresh_guid($t8)
    call $t5, $t8 := $Event_fresh_guid($t8);
    if ($abort_flag) {
      assume $DebugTrackAbort(5, 1875);
      goto Abort;
    }
    assume $Vector_is_well_formed($t5) && (forall $$0: int :: {$select_vector($t5,$$0)} $$0 >= 0 && $$0 < $vlen($t5) ==> $IsValidU8($select_vector($t5,$$0)));

    assume $Event_EventHandleGenerator_is_well_formed($t8);


    // write_ref($t4, $t8)
    call $t4 := $WriteRef($t4, $t8);

    // Event::EventHandleGenerator <- $t4
    call $WritebackToGlobal($t4);

    // PackRef($t4)
    call $Event_EventHandleGenerator_after_update_inv($Dereference($t4));

    // $t6 := pack Event::EventHandle<#0>($t1, $t5)
    call $tmp := $Event_EventHandle_pack(0, 0, 0, $tv0, $t1, $t5);
    $t6 := $tmp;

    // return $t6
    $ret0 := $t6;
    if (true) { assume $DebugTrackLocal(5, 2480, 9, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Event_publish_generator (account: $Value) returns ()
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(!b#$Boolean($Event_ehg_destroyed)))
    || b#$Boolean($ResourceExists($m, $Event_EventHandleGenerator_type_value(), $Signer_get_address($m, $txn, account)));
requires b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Event_ehg_exists($m, $txn, addr)) ==> b#$Boolean($Boolean($IsEqual($SelectField($Event_get_ehg($m, $txn, addr), $Event_EventHandleGenerator_addr), addr))))))))
    || b#$Boolean($ResourceExists($m, $Event_EventHandleGenerator_type_value(), $Signer_get_address($m, $txn, account)));
requires b#$Boolean($Boolean(true))
    || b#$Boolean($ResourceExists($m, $Event_EventHandleGenerator_type_value(), $Signer_get_address($m, $txn, account)));
ensures b#$Boolean(old($ResourceExists($m, $Event_EventHandleGenerator_type_value(), $Signer_get_address($m, $txn, account)))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($ResourceExists($m, $Event_EventHandleGenerator_type_value(), $Signer_get_address($m, $txn, account))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(!b#$Boolean($Event_ehg_destroyed))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Event_ehg_exists($m, $txn, addr)) ==> b#$Boolean($Boolean($IsEqual($SelectField($Event_get_ehg($m, $txn, addr), $Event_EventHandleGenerator_addr), addr)))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ResourceValue($m, $Event_EventHandleGenerator_type_value(), $Signer_get_address($m, $txn, account)), $Vector($ExtendValueArray($ExtendValueArray($EmptyValueArray(), $Integer(0)), $Signer_get_address($m, $txn, account)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Event_ehg_exists($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Event_ehg_exists($m, $txn, addr)) && b#$Boolean($Boolean($IsEqual($SelectField($Event_get_ehg($m, $txn, addr), $Event_EventHandleGenerator_counter), old($SelectField($Event_get_ehg($m, $txn, addr), $Event_EventHandleGenerator_counter)))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean(old($Event_ehg_exists($m, $txn, addr))))) && b#$Boolean($Event_ehg_exists($m, $txn, addr)))) ==> b#$Boolean($Boolean($IsEqual($SelectField($Event_get_ehg($m, $txn, addr), $Event_EventHandleGenerator_counter), $Integer(0))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean(old($Event_ehg_exists($m, $txn, addr))) ==> b#$Boolean($Boolean(b#$Boolean($Event_ehg_exists($m, $txn, addr)) && b#$Boolean($Boolean(i#$Integer($SelectField($Event_get_ehg($m, $txn, addr), $Event_EventHandleGenerator_counter)) >= i#$Integer(old($SelectField($Event_get_ehg($m, $txn, addr), $Event_EventHandleGenerator_counter)))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(true)));
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $Event_EventHandleGenerator_type_value()
    var $t6: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(5, 1144, 0, account); }

    // bytecode translation starts here
    // $t6 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t6 := $tmp;

    // $t1 := copy($t6)
    call $tmp := $CopyOrMoveValue($t6);
    $t1 := $tmp;

    // $t2 := 0
    $tmp := $Integer(0);
    $t2 := $tmp;

    // $t3 := move($t6)
    call $tmp := $CopyOrMoveValue($t6);
    $t3 := $tmp;

    // $t4 := Signer::address_of($t3)
    call $t4 := $Signer_address_of($t3);
    if ($abort_flag) {
      assume $DebugTrackAbort(5, 1266);
      goto Abort;
    }
    assume is#$Address($t4);


    // $t5 := pack Event::EventHandleGenerator($t2, $t4)
    call $tmp := $Event_EventHandleGenerator_pack(0, 0, 0, $t2, $t4);
    $t5 := $tmp;

    // move_to<Event::EventHandleGenerator>($t5, $t1)
    call $MoveTo($Event_EventHandleGenerator_type_value(), $t5, $t1);
    if ($abort_flag) {
      assume $DebugTrackAbort(5, 1201);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}



// ** spec vars of module LibraConfig



// ** spec funs of module LibraConfig

function {:inline} $LibraConfig_spec_get($m: $Memory, $txn: $Transaction, $tv0: $TypeValue): $Value {
    $SelectField($ResourceValue($m, $LibraConfig_LibraConfig_type_value($tv0), $Address(989845)), $LibraConfig_LibraConfig_payload)
}

function {:inline} $LibraConfig_spec_is_published($m: $Memory, $txn: $Transaction, $tv0: $TypeValue, addr: $Value): $Value {
    $ResourceExists($m, $LibraConfig_LibraConfig_type_value($tv0), addr)
}



// ** structs of module LibraConfig

const unique $LibraConfig_LibraConfig: $TypeName;
const $LibraConfig_LibraConfig_payload: $FieldName;
axiom $LibraConfig_LibraConfig_payload == 0;
function $LibraConfig_LibraConfig_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($LibraConfig_LibraConfig, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1))
}
function {:inline} $LibraConfig_LibraConfig_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
}
function {:inline} $LibraConfig_LibraConfig_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
}

axiom (forall m: $Memory, a: $Value, $tv0: $TypeValue :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $LibraConfig_LibraConfig_is_well_formed($ResourceValue(m, $LibraConfig_LibraConfig_type_value($tv0), a))
);

procedure {:inline 1} $LibraConfig_LibraConfig_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, payload: $Value) returns ($struct: $Value)
{
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := payload], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $LibraConfig_LibraConfig_unpack($tv0: $TypeValue, $struct: $Value) returns (payload: $Value)
{
    assume is#$Vector($struct);
    payload := $SelectField($struct, $LibraConfig_LibraConfig_payload);
}

const unique $LibraConfig_Configuration: $TypeName;
const $LibraConfig_Configuration_epoch: $FieldName;
axiom $LibraConfig_Configuration_epoch == 0;
const $LibraConfig_Configuration_last_reconfiguration_time: $FieldName;
axiom $LibraConfig_Configuration_last_reconfiguration_time == 1;
const $LibraConfig_Configuration_events: $FieldName;
axiom $LibraConfig_Configuration_events == 2;
function $LibraConfig_Configuration_type_value(): $TypeValue {
    $StructType($LibraConfig_Configuration, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $IntegerType()][1 := $IntegerType()][2 := $Event_EventHandle_type_value($LibraConfig_NewEpochEvent_type_value())], 3))
}
function {:inline} $LibraConfig_Configuration_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 3
      && $IsValidU64($SelectField($this, $LibraConfig_Configuration_epoch))
      && $IsValidU64($SelectField($this, $LibraConfig_Configuration_last_reconfiguration_time))
      && $Event_EventHandle_is_well_formed_types($SelectField($this, $LibraConfig_Configuration_events))
}
function {:inline} $LibraConfig_Configuration_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 3
      && $IsValidU64($SelectField($this, $LibraConfig_Configuration_epoch))
      && $IsValidU64($SelectField($this, $LibraConfig_Configuration_last_reconfiguration_time))
      && $Event_EventHandle_is_well_formed($SelectField($this, $LibraConfig_Configuration_events))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $LibraConfig_Configuration_is_well_formed($ResourceValue(m, $LibraConfig_Configuration_type_value(), a))
);

procedure {:inline 1} $LibraConfig_Configuration_before_update_inv($before: $Value) {
    call $Event_EventHandle_before_update_inv($LibraConfig_NewEpochEvent_type_value(), $SelectField($before, $LibraConfig_Configuration_events));
}

procedure {:inline 1} $LibraConfig_Configuration_after_update_inv($after: $Value) {
    call $Event_EventHandle_after_update_inv($LibraConfig_NewEpochEvent_type_value(), $SelectField($after, $LibraConfig_Configuration_events));
}

procedure {:inline 1} $LibraConfig_Configuration_pack($file_id: int, $byte_index: int, $var_idx: int, epoch: $Value, last_reconfiguration_time: $Value, events: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(epoch);
    assume $IsValidU64(last_reconfiguration_time);
    assume $Event_EventHandle_is_well_formed(events);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := epoch][1 := last_reconfiguration_time][2 := events], 3));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $LibraConfig_Configuration_unpack($struct: $Value) returns (epoch: $Value, last_reconfiguration_time: $Value, events: $Value)
{
    assume is#$Vector($struct);
    epoch := $SelectField($struct, $LibraConfig_Configuration_epoch);
    assume $IsValidU64(epoch);
    last_reconfiguration_time := $SelectField($struct, $LibraConfig_Configuration_last_reconfiguration_time);
    assume $IsValidU64(last_reconfiguration_time);
    events := $SelectField($struct, $LibraConfig_Configuration_events);
    assume $Event_EventHandle_is_well_formed(events);
}

const unique $LibraConfig_CreateOnChainConfig: $TypeName;
const $LibraConfig_CreateOnChainConfig_dummy_field: $FieldName;
axiom $LibraConfig_CreateOnChainConfig_dummy_field == 0;
function $LibraConfig_CreateOnChainConfig_type_value(): $TypeValue {
    $StructType($LibraConfig_CreateOnChainConfig, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $LibraConfig_CreateOnChainConfig_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $LibraConfig_CreateOnChainConfig_dummy_field))
}
function {:inline} $LibraConfig_CreateOnChainConfig_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $LibraConfig_CreateOnChainConfig_dummy_field))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $LibraConfig_CreateOnChainConfig_is_well_formed($ResourceValue(m, $LibraConfig_CreateOnChainConfig_type_value(), a))
);

procedure {:inline 1} $LibraConfig_CreateOnChainConfig_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $LibraConfig_CreateOnChainConfig_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $LibraConfig_CreateOnChainConfig_dummy_field);
    assume is#$Boolean(dummy_field);
}

const unique $LibraConfig_ModifyConfigCapability: $TypeName;
const $LibraConfig_ModifyConfigCapability_dummy_field: $FieldName;
axiom $LibraConfig_ModifyConfigCapability_dummy_field == 0;
function $LibraConfig_ModifyConfigCapability_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($LibraConfig_ModifyConfigCapability, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $LibraConfig_ModifyConfigCapability_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $LibraConfig_ModifyConfigCapability_dummy_field))
}
function {:inline} $LibraConfig_ModifyConfigCapability_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $LibraConfig_ModifyConfigCapability_dummy_field))
}

axiom (forall m: $Memory, a: $Value, $tv0: $TypeValue :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $LibraConfig_ModifyConfigCapability_is_well_formed($ResourceValue(m, $LibraConfig_ModifyConfigCapability_type_value($tv0), a))
);

procedure {:inline 1} $LibraConfig_ModifyConfigCapability_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $LibraConfig_ModifyConfigCapability_unpack($tv0: $TypeValue, $struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $LibraConfig_ModifyConfigCapability_dummy_field);
    assume is#$Boolean(dummy_field);
}

const unique $LibraConfig_NewEpochEvent: $TypeName;
const $LibraConfig_NewEpochEvent_epoch: $FieldName;
axiom $LibraConfig_NewEpochEvent_epoch == 0;
function $LibraConfig_NewEpochEvent_type_value(): $TypeValue {
    $StructType($LibraConfig_NewEpochEvent, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $IntegerType()], 1))
}
function {:inline} $LibraConfig_NewEpochEvent_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $IsValidU64($SelectField($this, $LibraConfig_NewEpochEvent_epoch))
}
function {:inline} $LibraConfig_NewEpochEvent_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $IsValidU64($SelectField($this, $LibraConfig_NewEpochEvent_epoch))
}

procedure {:inline 1} $LibraConfig_NewEpochEvent_pack($file_id: int, $byte_index: int, $var_idx: int, epoch: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(epoch);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := epoch], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $LibraConfig_NewEpochEvent_unpack($struct: $Value) returns (epoch: $Value)
{
    assume is#$Vector($struct);
    epoch := $SelectField($struct, $LibraConfig_NewEpochEvent_epoch);
    assume $IsValidU64(epoch);
}



// ** functions of module LibraConfig

procedure {:inline 1} $LibraConfig_initialize (config_account: $Value, _: $Value) returns ()
free requires is#$Address(config_account);
free requires $Roles_Capability_is_well_formed(_);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $Event_EventHandle_type_value($LibraConfig_NewEpochEvent_type_value())
    var $t14: $Value; // $LibraConfig_Configuration_type_value()
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $Roles_Capability_type_value($LibraConfig_CreateOnChainConfig_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 1346, 0, config_account); }
    if (true) { assume $DebugTrackLocal(10, 1346, 1, _); }

    // bytecode translation starts here
    // $t17 := move(config_account)
    call $tmp := $CopyOrMoveValue(config_account);
    $t17 := $tmp;

    // $t18 := move(_)
    call $tmp := $CopyOrMoveValue(_);
    $t18 := $tmp;

    // $t4 := copy($t17)
    call $tmp := $CopyOrMoveValue($t17);
    $t4 := $tmp;

    // $t5 := Signer::address_of($t4)
    call $t5 := $Signer_address_of($t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 1512);
      goto Abort;
    }
    assume is#$Address($t5);


    // $t6 := CoreAddresses::DEFAULT_CONFIG_ADDRESS()
    call $t6 := $CoreAddresses_DEFAULT_CONFIG_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 1557);
      goto Abort;
    }
    assume is#$Address($t6);


    // $t7 := ==($t5, $t6)
    $tmp := $Boolean($IsEqual($t5, $t6));
    $t7 := $tmp;

    // $t2 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 1497, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t9 := copy($t17)
    call $tmp := $CopyOrMoveValue($t17);
    $t9 := $tmp;

    // $t10 := 0
    $tmp := $Integer(0);
    $t10 := $tmp;

    // $t11 := 0
    $tmp := $Integer(0);
    $t11 := $tmp;

    // $t12 := move($t17)
    call $tmp := $CopyOrMoveValue($t17);
    $t12 := $tmp;

    // $t13 := Event::new_event_handle<LibraConfig::NewEpochEvent>($t12)
    call $t13 := $Event_new_event_handle($LibraConfig_NewEpochEvent_type_value(), $t12);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 1778);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t13);


    // $t14 := pack LibraConfig::Configuration($t10, $t11, $t13)
    call $tmp := $LibraConfig_Configuration_pack(0, 0, 0, $t10, $t11, $t13);
    $t14 := $tmp;

    // move_to<LibraConfig::Configuration>($t14, $t9)
    call $MoveTo($LibraConfig_Configuration_type_value(), $t14, $t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 1595);
      goto Abort;
    }

    // return ()
    return;

    // L2:
L2:

    // $t15 := move($t17)
    call $tmp := $CopyOrMoveValue($t17);
    $t15 := $tmp;

    // destroy($t15)

    // $t16 := 1
    $tmp := $Integer(1);
    $t16 := $tmp;

    // abort($t16)
    if (true) { assume $DebugTrackAbort(10, 1497); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $LibraConfig_claim_delegated_modify_config ($tv0: $TypeValue, account: $Value, offer_address: $Value) returns ()
free requires is#$Address(account);
free requires is#$Address(offer_address);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $LibraConfig_ModifyConfigCapability_type_value($tv0)
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 6218, 0, account); }
    if (true) { assume $DebugTrackLocal(10, 6218, 1, offer_address); }

    // bytecode translation starts here
    // $t6 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t6 := $tmp;

    // $t7 := move(offer_address)
    call $tmp := $CopyOrMoveValue(offer_address);
    $t7 := $tmp;

    // $t2 := copy($t6)
    call $tmp := $CopyOrMoveValue($t6);
    $t2 := $tmp;

    // $t3 := move($t6)
    call $tmp := $CopyOrMoveValue($t6);
    $t3 := $tmp;

    // $t5 := Offer::redeem<LibraConfig::ModifyConfigCapability<#0>>($t3, $t7)
    call $t5 := $Offer_redeem($LibraConfig_ModifyConfigCapability_type_value($tv0), $t3, $t7);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 6343);
      goto Abort;
    }
    assume $LibraConfig_ModifyConfigCapability_is_well_formed($t5);


    // move_to<LibraConfig::ModifyConfigCapability<#0>>($t5, $t2)
    call $MoveTo($LibraConfig_ModifyConfigCapability_type_value($tv0), $t5, $t2);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 6319);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $LibraConfig_emit_reconfiguration_event () returns ()
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var config_ref: $Reference; // ReferenceType($LibraConfig_Configuration_type_value())
    var $t1: $Value; // $AddressType()
    var $t2: $Reference; // ReferenceType($LibraConfig_Configuration_type_value())
    var $t3: $Reference; // ReferenceType($LibraConfig_Configuration_type_value())
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Reference; // ReferenceType($LibraConfig_Configuration_type_value())
    var $t9: $Reference; // ReferenceType($IntegerType())
    var $t10: $Reference; // ReferenceType($LibraConfig_Configuration_type_value())
    var $t11: $Reference; // ReferenceType($Event_EventHandle_type_value($LibraConfig_NewEpochEvent_type_value()))
    var $t12: $Reference; // ReferenceType($LibraConfig_Configuration_type_value())
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $IntegerType()
    var $t15: $Value; // $LibraConfig_NewEpochEvent_type_value()
    var $t16: $Value; // $Event_EventHandle_type_value($LibraConfig_NewEpochEvent_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t1 := CoreAddresses::DEFAULT_CONFIG_ADDRESS()
    call $t1 := $CoreAddresses_DEFAULT_CONFIG_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 7690);
      goto Abort;
    }
    assume is#$Address($t1);


    // $t2 := borrow_global<LibraConfig::Configuration>($t1)
    call $t2 := $BorrowGlobal($t1, $LibraConfig_Configuration_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 7642);
      goto Abort;
    }
    assume $LibraConfig_Configuration_is_well_formed($Dereference($t2));

    // UnpackRef($t2)
    call $LibraConfig_Configuration_before_update_inv($Dereference($t2));

    // config_ref := $t2
    call config_ref := $CopyOrMoveRef($t2);
    if (true) { assume $DebugTrackLocal(10, 7629, 0, $Dereference(config_ref)); }

    // $t3 := copy(config_ref)
    call $t3 := $CopyOrMoveRef(config_ref);

    // $t4 := get_field<LibraConfig::Configuration>.epoch($t3)
    call $tmp := $GetFieldFromReference($t3, $LibraConfig_Configuration_epoch);
    assume $IsValidU64($tmp);
    $t4 := $tmp;

    // Reference(config_ref) <- $t3
    call config_ref := $WritebackToReference($t3, config_ref);

    // $t5 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t5 := $tmp;

    // $t6 := 1
    $tmp := $Integer(1);
    $t6 := $tmp;

    // $t7 := +($t5, $t6)
    call $tmp := $AddU64($t5, $t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 7761);
      goto Abort;
    }
    $t7 := $tmp;

    // $t8 := copy(config_ref)
    call $t8 := $CopyOrMoveRef(config_ref);

    // $t9 := borrow_field<LibraConfig::Configuration>.epoch($t8)
    call $t9 := $BorrowField($t8, $LibraConfig_Configuration_epoch);
    assume $IsValidU64($Dereference($t9));

    // Reference(config_ref) <- $t8
    call config_ref := $WritebackToReference($t8, config_ref);

    // UnpackRef($t9)

    // write_ref($t9, $t7)
    call $t9 := $WriteRef($t9, $t7);
    if (true) { assume $DebugTrackLocal(10, 7725, 0, $Dereference(config_ref)); }

    // Reference(config_ref) <- $t9
    call config_ref := $WritebackToReference($t9, config_ref);

    // Reference($t8) <- $t9
    call $t8 := $WritebackToReference($t9, $t8);

    // PackRef($t9)

    // $t10 := copy(config_ref)
    call $t10 := $CopyOrMoveRef(config_ref);

    // $t11 := borrow_field<LibraConfig::Configuration>.events($t10)
    call $t11 := $BorrowField($t10, $LibraConfig_Configuration_events);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t11));

    // Reference(config_ref) <- $t10
    call config_ref := $WritebackToReference($t10, config_ref);

    // UnpackRef($t11)
    call $Event_EventHandle_before_update_inv($LibraConfig_NewEpochEvent_type_value(), $Dereference($t11));

    // $t12 := move(config_ref)
    call $t12 := $CopyOrMoveRef(config_ref);

    // $t13 := get_field<LibraConfig::Configuration>.epoch($t12)
    call $tmp := $GetFieldFromReference($t12, $LibraConfig_Configuration_epoch);
    assume $IsValidU64($tmp);
    $t13 := $tmp;

    // LibraConfig::Configuration <- $t12
    call $WritebackToGlobal($t12);

    // $t14 := move($t13)
    call $tmp := $CopyOrMoveValue($t13);
    $t14 := $tmp;

    // $t15 := pack LibraConfig::NewEpochEvent($t14)
    call $tmp := $LibraConfig_NewEpochEvent_pack(0, 0, 0, $t14);
    $t15 := $tmp;

    // PackRef($t11)
    call $Event_EventHandle_after_update_inv($LibraConfig_NewEpochEvent_type_value(), $Dereference($t11));

    // $t16 := read_ref($t11)
    call $tmp := $ReadRef($t11);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t16 := $tmp;

    // $t16 := Event::emit_event<LibraConfig::NewEpochEvent>($t16, $t15)
    call $t16 := $Event_emit_event($LibraConfig_NewEpochEvent_type_value(), $t16, $t15);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 7782);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t16);


    // write_ref($t11, $t16)
    call $t11 := $WriteRef($t11, $t16);
    if (true) { assume $DebugTrackLocal(10, 7559, 0, $Dereference(config_ref)); }

    // LibraConfig::Configuration <- $t11
    call $WritebackToGlobal($t11);

    // Reference($t10) <- $t11
    call $t10 := $WritebackToReference($t11, $t10);

    // Reference($t12) <- $t11
    call $t12 := $WritebackToReference($t11, $t12);

    // UnpackRef($t11)
    call $Event_EventHandle_before_update_inv($LibraConfig_NewEpochEvent_type_value(), $Dereference($t11));

    // PackRef($t11)
    call $Event_EventHandle_after_update_inv($LibraConfig_NewEpochEvent_type_value(), $Dereference($t11));

    // PackRef($t12)
    call $LibraConfig_Configuration_after_update_inv($Dereference($t12));

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $LibraConfig_get ($tv0: $TypeValue) returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var addr: $Value; // $AddressType()
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $LibraConfig_LibraConfig_type_value($tv0)
    var $t9: $Value; // $tv0
    var $t10: $Value; // $tv0
    var $t11: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t3 := CoreAddresses::DEFAULT_CONFIG_ADDRESS()
    call $t3 := $CoreAddresses_DEFAULT_CONFIG_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 2020);
      goto Abort;
    }
    assume is#$Address($t3);


    // addr := $t3
    call $tmp := $CopyOrMoveValue($t3);
    addr := $tmp;
    if (true) { assume $DebugTrackLocal(10, 1998, 0, $tmp); }

    // $t5 := exists<LibraConfig::LibraConfig<#0>>(addr)
    call $tmp := $Exists(addr, $LibraConfig_LibraConfig_type_value($tv0));
    $t5 := $tmp;

    // $t1 := $t5
    call $tmp := $CopyOrMoveValue($t5);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 2054, 1, $tmp); }

    // if ($t1) goto L0 else goto L1
    $tmp := $t1;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t8 := get_global<LibraConfig::LibraConfig<#0>>(addr)
    call $tmp := $GetGlobal(addr, $LibraConfig_LibraConfig_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 2111);
      goto Abort;
    }
    assume $LibraConfig_LibraConfig_is_well_formed($tmp);
    $t8 := $tmp;

    // $t9 := get_field<LibraConfig::LibraConfig<#0>>.payload($t8)
    call $tmp := $GetFieldFromValue($t8, $LibraConfig_LibraConfig_payload);
    $t9 := $tmp;

    // $t10 := move($t9)
    call $tmp := $CopyOrMoveValue($t9);
    $t10 := $tmp;

    // return $t10
    $ret0 := $t10;
    if (true) { assume $DebugTrackLocal(10, 2109, 12, $ret0); }
    return;

    // L2:
L2:

    // $t11 := 24
    $tmp := $Integer(24);
    $t11 := $tmp;

    // abort($t11)
    if (true) { assume $DebugTrackAbort(10, 2054); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $LibraConfig_grant_privileges (account: $Value) returns ()
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $LibraConfig_CreateOnChainConfig_type_value()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 824, 0, account); }

    // bytecode translation starts here
    // $t4 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := false
    $tmp := $Boolean(false);
    $t2 := $tmp;

    // $t3 := pack LibraConfig::CreateOnChainConfig($t2)
    call $tmp := $LibraConfig_CreateOnChainConfig_pack(0, 0, 0, $t2);
    $t3 := $tmp;

    // Roles::add_privilege_to_account_association_root_role<LibraConfig::CreateOnChainConfig>($t1, $t3)
    assume b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
    assume b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
    call $Roles_add_privilege_to_account_association_root_role($LibraConfig_CreateOnChainConfig_type_value(), $t1, $t3);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 887);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $LibraConfig_grant_privileges_for_config_TESTNET_HACK_REMOVE (account: $Value) returns ()
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $LibraConfig_CreateOnChainConfig_type_value()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 1028, 0, account); }

    // bytecode translation starts here
    // $t4 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := false
    $tmp := $Boolean(false);
    $t2 := $tmp;

    // $t3 := pack LibraConfig::CreateOnChainConfig($t2)
    call $tmp := $LibraConfig_CreateOnChainConfig_pack(0, 0, 0, $t2);
    $t3 := $tmp;

    // Roles::add_privilege_to_account_parent_vasp_role<LibraConfig::CreateOnChainConfig>($t1, $t3)
    assume b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
    assume b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
    call $Roles_add_privilege_to_account_parent_vasp_role($LibraConfig_CreateOnChainConfig_type_value(), $t1, $t3);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 1122);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $LibraConfig_publish_new_config ($tv0: $TypeValue, config_account: $Value, _: $Value, payload: $Value) returns ()
free requires is#$Address(config_account);
free requires $Roles_Capability_is_well_formed(_);
requires $ExistsTxnSenderAccount($m, $txn);
ensures !$abort_flag ==> (b#$Boolean(old($Boolean(!b#$Boolean($LibraConfig_spec_is_published($m, $txn, $tv0, $Signer_get_address($m, $txn, config_account)))))));
ensures !$abort_flag ==> (b#$Boolean($LibraConfig_spec_is_published($m, $txn, $tv0, $Signer_get_address($m, $txn, config_account))));
{
    // declare local variables
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $BooleanType()
    var $t12: $Value; // $LibraConfig_ModifyConfigCapability_type_value($tv0)
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $tv0
    var $t15: $Value; // $LibraConfig_LibraConfig_type_value($tv0)
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $Roles_Capability_type_value($LibraConfig_CreateOnChainConfig_type_value())
    var $t20: $Value; // $tv0
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 4627, 0, config_account); }
    if (true) { assume $DebugTrackLocal(10, 4627, 1, _); }
    if (true) { assume $DebugTrackLocal(10, 4627, 2, payload); }

    // bytecode translation starts here
    // $t18 := move(config_account)
    call $tmp := $CopyOrMoveValue(config_account);
    $t18 := $tmp;

    // $t19 := move(_)
    call $tmp := $CopyOrMoveValue(_);
    $t19 := $tmp;

    // $t20 := move(payload)
    call $tmp := $CopyOrMoveValue(payload);
    $t20 := $tmp;

    // $t5 := copy($t18)
    call $tmp := $CopyOrMoveValue($t18);
    $t5 := $tmp;

    // $t6 := Signer::address_of($t5)
    call $t6 := $Signer_address_of($t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 4809);
      goto Abort;
    }
    assume is#$Address($t6);


    // $t7 := CoreAddresses::DEFAULT_CONFIG_ADDRESS()
    call $t7 := $CoreAddresses_DEFAULT_CONFIG_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 4854);
      goto Abort;
    }
    assume is#$Address($t7);


    // $t8 := ==($t6, $t7)
    $tmp := $Boolean($IsEqual($t6, $t7));
    $t8 := $tmp;

    // $t3 := $t8
    call $tmp := $CopyOrMoveValue($t8);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 4794, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t10 := copy($t18)
    call $tmp := $CopyOrMoveValue($t18);
    $t10 := $tmp;

    // $t11 := false
    $tmp := $Boolean(false);
    $t11 := $tmp;

    // $t12 := pack LibraConfig::ModifyConfigCapability<#0>($t11)
    call $tmp := $LibraConfig_ModifyConfigCapability_pack(0, 0, 0, $tv0, $t11);
    $t12 := $tmp;

    // move_to<LibraConfig::ModifyConfigCapability<#0>>($t12, $t10)
    call $MoveTo($LibraConfig_ModifyConfigCapability_type_value($tv0), $t12, $t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 4892);
      goto Abort;
    }

    // $t13 := move($t18)
    call $tmp := $CopyOrMoveValue($t18);
    $t13 := $tmp;

    // $t15 := pack LibraConfig::LibraConfig<#0>($t20)
    call $tmp := $LibraConfig_LibraConfig_pack(0, 0, 0, $tv0, $t20);
    $t15 := $tmp;

    // move_to<LibraConfig::LibraConfig<#0>>($t15, $t13)
    call $MoveTo($LibraConfig_LibraConfig_type_value($tv0), $t15, $t13);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 4960);
      goto Abort;
    }

    // return ()
    return;

    // L2:
L2:

    // $t16 := move($t18)
    call $tmp := $CopyOrMoveValue($t18);
    $t16 := $tmp;

    // destroy($t16)

    // $t17 := 1
    $tmp := $Integer(1);
    $t17 := $tmp;

    // abort($t17)
    if (true) { assume $DebugTrackAbort(10, 4794); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $LibraConfig_publish_new_config_with_capability ($tv0: $TypeValue, config_account: $Value, _: $Value, payload: $Value) returns ($ret0: $Value)
free requires is#$Address(config_account);
free requires $Roles_Capability_is_well_formed(_);
requires $ExistsTxnSenderAccount($m, $txn);
ensures !$abort_flag ==> (b#$Boolean(old($Boolean(!b#$Boolean($LibraConfig_spec_is_published($m, $txn, $tv0, $Signer_get_address($m, $txn, config_account)))))));
ensures !$abort_flag ==> (b#$Boolean($LibraConfig_spec_is_published($m, $txn, $tv0, $Signer_get_address($m, $txn, config_account))));
{
    // declare local variables
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $tv0
    var $t12: $Value; // $LibraConfig_LibraConfig_type_value($tv0)
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $LibraConfig_ModifyConfigCapability_type_value($tv0)
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $Roles_Capability_type_value($LibraConfig_CreateOnChainConfig_type_value())
    var $t19: $Value; // $tv0
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 3450, 0, config_account); }
    if (true) { assume $DebugTrackLocal(10, 3450, 1, _); }
    if (true) { assume $DebugTrackLocal(10, 3450, 2, payload); }

    // bytecode translation starts here
    // $t17 := move(config_account)
    call $tmp := $CopyOrMoveValue(config_account);
    $t17 := $tmp;

    // $t18 := move(_)
    call $tmp := $CopyOrMoveValue(_);
    $t18 := $tmp;

    // $t19 := move(payload)
    call $tmp := $CopyOrMoveValue(payload);
    $t19 := $tmp;

    // $t5 := copy($t17)
    call $tmp := $CopyOrMoveValue($t17);
    $t5 := $tmp;

    // $t6 := Signer::address_of($t5)
    call $t6 := $Signer_address_of($t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 3681);
      goto Abort;
    }
    assume is#$Address($t6);


    // $t7 := CoreAddresses::DEFAULT_CONFIG_ADDRESS()
    call $t7 := $CoreAddresses_DEFAULT_CONFIG_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 3726);
      goto Abort;
    }
    assume is#$Address($t7);


    // $t8 := ==($t6, $t7)
    $tmp := $Boolean($IsEqual($t6, $t7));
    $t8 := $tmp;

    // $t3 := $t8
    call $tmp := $CopyOrMoveValue($t8);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 3666, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t10 := move($t17)
    call $tmp := $CopyOrMoveValue($t17);
    $t10 := $tmp;

    // $t12 := pack LibraConfig::LibraConfig<#0>($t19)
    call $tmp := $LibraConfig_LibraConfig_pack(0, 0, 0, $tv0, $t19);
    $t12 := $tmp;

    // move_to<LibraConfig::LibraConfig<#0>>($t12, $t10)
    call $MoveTo($LibraConfig_LibraConfig_type_value($tv0), $t12, $t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 3764);
      goto Abort;
    }

    // $t13 := false
    $tmp := $Boolean(false);
    $t13 := $tmp;

    // $t14 := pack LibraConfig::ModifyConfigCapability<#0>($t13)
    call $tmp := $LibraConfig_ModifyConfigCapability_pack(0, 0, 0, $tv0, $t13);
    $t14 := $tmp;

    // return $t14
    $ret0 := $t14;
    if (true) { assume $DebugTrackLocal(10, 4090, 20, $ret0); }
    return;

    // L2:
L2:

    // $t15 := move($t17)
    call $tmp := $CopyOrMoveValue($t17);
    $t15 := $tmp;

    // destroy($t15)

    // $t16 := 1
    $tmp := $Integer(1);
    $t16 := $tmp;

    // abort($t16)
    if (true) { assume $DebugTrackAbort(10, 3666); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $LibraConfig_publish_new_config_with_delegate ($tv0: $TypeValue, config_account: $Value, _: $Value, payload: $Value, delegate: $Value) returns ()
free requires is#$Address(config_account);
free requires $Roles_Capability_is_well_formed(_);
free requires is#$Address(delegate);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $BooleanType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $LibraConfig_ModifyConfigCapability_type_value($tv0)
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $tv0
    var $t17: $Value; // $LibraConfig_LibraConfig_type_value($tv0)
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $AddressType()
    var $t21: $Value; // $Roles_Capability_type_value($LibraConfig_CreateOnChainConfig_type_value())
    var $t22: $Value; // $tv0
    var $t23: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 5404, 0, config_account); }
    if (true) { assume $DebugTrackLocal(10, 5404, 1, _); }
    if (true) { assume $DebugTrackLocal(10, 5404, 2, payload); }
    if (true) { assume $DebugTrackLocal(10, 5404, 3, delegate); }

    // bytecode translation starts here
    // $t20 := move(config_account)
    call $tmp := $CopyOrMoveValue(config_account);
    $t20 := $tmp;

    // $t21 := move(_)
    call $tmp := $CopyOrMoveValue(_);
    $t21 := $tmp;

    // $t22 := move(payload)
    call $tmp := $CopyOrMoveValue(payload);
    $t22 := $tmp;

    // $t23 := move(delegate)
    call $tmp := $CopyOrMoveValue(delegate);
    $t23 := $tmp;

    // $t6 := copy($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t6 := $tmp;

    // $t7 := Signer::address_of($t6)
    call $t7 := $Signer_address_of($t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 5628);
      goto Abort;
    }
    assume is#$Address($t7);


    // $t8 := CoreAddresses::DEFAULT_CONFIG_ADDRESS()
    call $t8 := $CoreAddresses_DEFAULT_CONFIG_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 5673);
      goto Abort;
    }
    assume is#$Address($t8);


    // $t9 := ==($t7, $t8)
    $tmp := $Boolean($IsEqual($t7, $t8));
    $t9 := $tmp;

    // $t4 := $t9
    call $tmp := $CopyOrMoveValue($t9);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 5613, 4, $tmp); }

    // if ($t4) goto L0 else goto L1
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t11 := copy($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t11 := $tmp;

    // $t12 := false
    $tmp := $Boolean(false);
    $t12 := $tmp;

    // $t13 := pack LibraConfig::ModifyConfigCapability<#0>($t12)
    call $tmp := $LibraConfig_ModifyConfigCapability_pack(0, 0, 0, $tv0, $t12);
    $t13 := $tmp;

    // Offer::create<LibraConfig::ModifyConfigCapability<#0>>($t11, $t13, $t23)
    call $Offer_create($LibraConfig_ModifyConfigCapability_type_value($tv0), $t11, $t13, $t23);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 5718);
      goto Abort;
    }

    // $t15 := move($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t15 := $tmp;

    // $t17 := pack LibraConfig::LibraConfig<#0>($t22)
    call $tmp := $LibraConfig_LibraConfig_pack(0, 0, 0, $tv0, $t22);
    $t17 := $tmp;

    // move_to<LibraConfig::LibraConfig<#0>>($t17, $t15)
    call $MoveTo($LibraConfig_LibraConfig_type_value($tv0), $t17, $t15);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 5794);
      goto Abort;
    }

    // return ()
    return;

    // L2:
L2:

    // $t18 := move($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t18 := $tmp;

    // destroy($t18)

    // $t19 := 1
    $tmp := $Integer(1);
    $t19 := $tmp;

    // abort($t19)
    if (true) { assume $DebugTrackAbort(10, 5613); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $LibraConfig_publish_new_treasury_compliance_config ($tv0: $TypeValue, config_account: $Value, tc_account: $Value, _: $Value, payload: $Value) returns ()
free requires is#$Address(config_account);
free requires is#$Address(tc_account);
free requires $Roles_Capability_is_well_formed(_);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $tv0
    var $t6: $Value; // $LibraConfig_LibraConfig_type_value($tv0)
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $LibraConfig_ModifyConfigCapability_type_value($tv0)
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $Roles_Capability_type_value($LibraConfig_CreateOnChainConfig_type_value())
    var $t13: $Value; // $tv0
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 4203, 0, config_account); }
    if (true) { assume $DebugTrackLocal(10, 4203, 1, tc_account); }
    if (true) { assume $DebugTrackLocal(10, 4203, 2, _); }
    if (true) { assume $DebugTrackLocal(10, 4203, 3, payload); }

    // bytecode translation starts here
    // $t10 := move(config_account)
    call $tmp := $CopyOrMoveValue(config_account);
    $t10 := $tmp;

    // $t11 := move(tc_account)
    call $tmp := $CopyOrMoveValue(tc_account);
    $t11 := $tmp;

    // $t12 := move(_)
    call $tmp := $CopyOrMoveValue(_);
    $t12 := $tmp;

    // $t13 := move(payload)
    call $tmp := $CopyOrMoveValue(payload);
    $t13 := $tmp;

    // $t4 := move($t10)
    call $tmp := $CopyOrMoveValue($t10);
    $t4 := $tmp;

    // $t6 := pack LibraConfig::LibraConfig<#0>($t13)
    call $tmp := $LibraConfig_LibraConfig_pack(0, 0, 0, $tv0, $t13);
    $t6 := $tmp;

    // move_to<LibraConfig::LibraConfig<#0>>($t6, $t4)
    call $MoveTo($LibraConfig_LibraConfig_type_value($tv0), $t6, $t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 4420);
      goto Abort;
    }

    // $t7 := move($t11)
    call $tmp := $CopyOrMoveValue($t11);
    $t7 := $tmp;

    // $t8 := false
    $tmp := $Boolean(false);
    $t8 := $tmp;

    // $t9 := pack LibraConfig::ModifyConfigCapability<#0>($t8)
    call $tmp := $LibraConfig_ModifyConfigCapability_pack(0, 0, 0, $tv0, $t8);
    $t9 := $tmp;

    // move_to<LibraConfig::ModifyConfigCapability<#0>>($t9, $t7)
    call $MoveTo($LibraConfig_ModifyConfigCapability_type_value($tv0), $t9, $t7);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 4478);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $LibraConfig_reconfigure (_: $Value) returns ()
free requires $Roles_Capability_is_well_formed(_);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t1: $Value; // $Roles_Capability_type_value($Roles_AssociationRootRole_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 6418, 0, _); }

    // bytecode translation starts here
    // $t1 := move(_)
    call $tmp := $CopyOrMoveValue(_);
    $t1 := $tmp;

    // LibraConfig::reconfigure_()
    call $LibraConfig_reconfigure_();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 6630);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $LibraConfig_reconfigure_ () returns ()
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var config_ref: $Reference; // ReferenceType($LibraConfig_Configuration_type_value())
    var current_block_time: $Value; // $IntegerType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $AddressType()
    var $t6: $Reference; // ReferenceType($LibraConfig_Configuration_type_value())
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Reference; // ReferenceType($LibraConfig_Configuration_type_value())
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $IntegerType()
    var $t15: $Reference; // ReferenceType($LibraConfig_Configuration_type_value())
    var $t16: $Reference; // ReferenceType($IntegerType())
    var $t17: $Reference; // ReferenceType($LibraConfig_Configuration_type_value())
    var $t18: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t4 := LibraTimestamp::is_genesis()
    call $t4 := $LibraTimestamp_is_genesis();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 6800);
      goto Abort;
    }
    assume is#$Boolean($t4);


    // if ($t4) goto L0 else goto L1
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // return ()
    return;

    // L2:
L2:

    // $t5 := CoreAddresses::DEFAULT_CONFIG_ADDRESS()
    call $t5 := $CoreAddresses_DEFAULT_CONFIG_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 6920);
      goto Abort;
    }
    assume is#$Address($t5);


    // $t6 := borrow_global<LibraConfig::Configuration>($t5)
    call $t6 := $BorrowGlobal($t5, $LibraConfig_Configuration_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 6872);
      goto Abort;
    }
    assume $LibraConfig_Configuration_is_well_formed($Dereference($t6));

    // UnpackRef($t6)
    call $LibraConfig_Configuration_before_update_inv($Dereference($t6));

    // config_ref := $t6
    call config_ref := $CopyOrMoveRef($t6);
    if (true) { assume $DebugTrackLocal(10, 6859, 0, $Dereference(config_ref)); }

    // $t7 := LibraTimestamp::now_microseconds()
    call $t7 := $LibraTimestamp_now_microseconds();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 7198);
      goto Abort;
    }
    assume $IsValidU64($t7);


    // current_block_time := $t7
    call $tmp := $CopyOrMoveValue($t7);
    current_block_time := $tmp;
    if (true) { assume $DebugTrackLocal(10, 7161, 1, $tmp); }

    // $t9 := copy(config_ref)
    call $t9 := $CopyOrMoveRef(config_ref);

    // $t10 := get_field<LibraConfig::Configuration>.last_reconfiguration_time($t9)
    call $tmp := $GetFieldFromReference($t9, $LibraConfig_Configuration_last_reconfiguration_time);
    assume $IsValidU64($tmp);
    $t10 := $tmp;

    // Reference(config_ref) <- $t9
    call config_ref := $WritebackToReference($t9, config_ref);

    // $t11 := move($t10)
    call $tmp := $CopyOrMoveValue($t10);
    $t11 := $tmp;

    // $t12 := >(current_block_time, $t11)
    call $tmp := $Gt(current_block_time, $t11);
    $t12 := $tmp;

    // $t2 := $t12
    call $tmp := $CopyOrMoveValue($t12);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 7225, 2, $tmp); }

    // if ($t2) goto L3 else goto L4
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L5
    goto L5;

    // L3:
L3:

    // $t15 := move(config_ref)
    call $t15 := $CopyOrMoveRef(config_ref);

    // $t16 := borrow_field<LibraConfig::Configuration>.last_reconfiguration_time($t15)
    call $t16 := $BorrowField($t15, $LibraConfig_Configuration_last_reconfiguration_time);
    assume $IsValidU64($Dereference($t16));

    // LibraConfig::Configuration <- $t15
    call $WritebackToGlobal($t15);

    // UnpackRef($t16)

    // write_ref($t16, current_block_time)
    call $t16 := $WriteRef($t16, current_block_time);
    if (true) { assume $DebugTrackLocal(10, 7303, 0, $Dereference(config_ref)); }

    // LibraConfig::Configuration <- $t16
    call $WritebackToGlobal($t16);

    // Reference($t15) <- $t16
    call $t15 := $WritebackToReference($t16, $t15);

    // PackRef($t15)
    call $LibraConfig_Configuration_after_update_inv($Dereference($t15));

    // PackRef($t16)

    // LibraConfig::emit_reconfiguration_event()
    call $LibraConfig_emit_reconfiguration_event();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 7563);
      goto Abort;
    }

    // return ()
    return;

    // L5:
L5:

    // $t17 := move(config_ref)
    call $t17 := $CopyOrMoveRef(config_ref);

    // destroy($t17)

    // LibraConfig::Configuration <- $t17
    call $WritebackToGlobal($t17);

    // PackRef($t17)
    call $LibraConfig_Configuration_after_update_inv($Dereference($t17));

    // $t18 := 23
    $tmp := $Integer(23);
    $t18 := $tmp;

    // abort($t18)
    if (true) { assume $DebugTrackAbort(10, 7225); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $LibraConfig_set ($tv0: $TypeValue, account: $Value, payload: $Value) returns ()
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var addr: $Value; // $AddressType()
    var config: $Reference; // ReferenceType($LibraConfig_LibraConfig_type_value($tv0))
    var signer_address: $Value; // $AddressType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $BooleanType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $BooleanType()
    var $t17: $Value; // $BooleanType()
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $AddressType()
    var $t21: $Reference; // ReferenceType($LibraConfig_LibraConfig_type_value($tv0))
    var $t22: $Value; // $tv0
    var $t23: $Reference; // ReferenceType($LibraConfig_LibraConfig_type_value($tv0))
    var $t24: $Reference; // ReferenceType($tv0)
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $AddressType()
    var $t27: $Value; // $tv0
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 2305, 0, account); }
    if (true) { assume $DebugTrackLocal(10, 2305, 1, payload); }

    // bytecode translation starts here
    // $t26 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t26 := $tmp;

    // $t27 := move(payload)
    call $tmp := $CopyOrMoveValue(payload);
    $t27 := $tmp;

    // $t9 := CoreAddresses::DEFAULT_CONFIG_ADDRESS()
    call $t9 := $CoreAddresses_DEFAULT_CONFIG_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 2445);
      goto Abort;
    }
    assume is#$Address($t9);


    // addr := $t9
    call $tmp := $CopyOrMoveValue($t9);
    addr := $tmp;
    if (true) { assume $DebugTrackLocal(10, 2423, 2, $tmp); }

    // $t11 := exists<LibraConfig::LibraConfig<#0>>(addr)
    call $tmp := $Exists(addr, $LibraConfig_LibraConfig_type_value($tv0));
    $t11 := $tmp;

    // $t5 := $t11
    call $tmp := $CopyOrMoveValue($t11);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 2479, 5, $tmp); }

    // if ($t5) goto L0 else goto L1
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t13 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t13 := $tmp;

    // $t14 := Signer::address_of($t13)
    call $t14 := $Signer_address_of($t13);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 2563);
      goto Abort;
    }
    assume is#$Address($t14);


    // signer_address := $t14
    call $tmp := $CopyOrMoveValue($t14);
    signer_address := $tmp;
    if (true) { assume $DebugTrackLocal(10, 2538, 4, $tmp); }

    // $t16 := exists<LibraConfig::ModifyConfigCapability<#0>>(signer_address)
    call $tmp := $Exists(signer_address, $LibraConfig_ModifyConfigCapability_type_value($tv0));
    $t16 := $tmp;

    // $t7 := $t16
    call $tmp := $CopyOrMoveValue($t16);
    $t7 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 2592, 7, $tmp); }

    // if ($t7) goto L3 else goto L4
    $tmp := $t7;
    if (b#$Boolean($tmp)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L5
    goto L5;

    // L2:
L2:

    // $t18 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t18 := $tmp;

    // destroy($t18)

    // $t19 := 24
    $tmp := $Integer(24);
    $t19 := $tmp;

    // abort($t19)
    if (true) { assume $DebugTrackAbort(10, 2479); }
    goto Abort;

    // L3:
L3:

    // $t21 := borrow_global<LibraConfig::LibraConfig<#0>>(addr)
    call $t21 := $BorrowGlobal(addr, $LibraConfig_LibraConfig_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 2682);
      goto Abort;
    }
    assume $LibraConfig_LibraConfig_is_well_formed($Dereference($t21));

    // UnpackRef($t21)

    // config := $t21
    call config := $CopyOrMoveRef($t21);
    if (true) { assume $DebugTrackLocal(10, 2673, 3, $Dereference(config)); }

    // $t23 := move(config)
    call $t23 := $CopyOrMoveRef(config);

    // $t24 := borrow_field<LibraConfig::LibraConfig<#0>>.payload($t23)
    call $t24 := $BorrowField($t23, $LibraConfig_LibraConfig_payload);

    // LibraConfig::LibraConfig <- $t23
    call $WritebackToGlobal($t23);

    // UnpackRef($t24)

    // write_ref($t24, $t27)
    call $t24 := $WriteRef($t24, $t27);
    if (true) { assume $DebugTrackLocal(10, 2736, 3, $Dereference(config)); }

    // LibraConfig::LibraConfig <- $t24
    call $WritebackToGlobal($t24);

    // Reference($t23) <- $t24
    call $t23 := $WritebackToReference($t24, $t23);

    // PackRef($t23)

    // PackRef($t24)

    // LibraConfig::reconfigure_()
    call $LibraConfig_reconfigure_();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 6630);
      goto Abort;
    }

    // return ()
    return;

    // L5:
L5:

    // $t25 := 24
    $tmp := $Integer(24);
    $t25 := $tmp;

    // abort($t25)
    if (true) { assume $DebugTrackAbort(10, 2592); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $LibraConfig_set_with_capability ($tv0: $TypeValue, _cap: $Value, payload: $Value) returns ()
free requires $LibraConfig_ModifyConfigCapability_is_well_formed(_cap);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var addr: $Value; // $AddressType()
    var config: $Reference; // ReferenceType($LibraConfig_LibraConfig_type_value($tv0))
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $AddressType()
    var $t11: $Reference; // ReferenceType($LibraConfig_LibraConfig_type_value($tv0))
    var $t12: $Value; // $tv0
    var $t13: $Reference; // ReferenceType($LibraConfig_LibraConfig_type_value($tv0))
    var $t14: $Reference; // ReferenceType($tv0)
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $LibraConfig_ModifyConfigCapability_type_value($tv0)
    var $t17: $Value; // $tv0
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 2869, 0, _cap); }
    if (true) { assume $DebugTrackLocal(10, 2869, 1, payload); }

    // bytecode translation starts here
    // $t16 := move(_cap)
    call $tmp := $CopyOrMoveValue(_cap);
    $t16 := $tmp;

    // $t17 := move(payload)
    call $tmp := $CopyOrMoveValue(payload);
    $t17 := $tmp;

    // $t6 := CoreAddresses::DEFAULT_CONFIG_ADDRESS()
    call $t6 := $CoreAddresses_DEFAULT_CONFIG_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 3068);
      goto Abort;
    }
    assume is#$Address($t6);


    // addr := $t6
    call $tmp := $CopyOrMoveValue($t6);
    addr := $tmp;
    if (true) { assume $DebugTrackLocal(10, 3046, 2, $tmp); }

    // $t8 := exists<LibraConfig::LibraConfig<#0>>(addr)
    call $tmp := $Exists(addr, $LibraConfig_LibraConfig_type_value($tv0));
    $t8 := $tmp;

    // $t4 := $t8
    call $tmp := $CopyOrMoveValue($t8);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 3102, 4, $tmp); }

    // if ($t4) goto L0 else goto L1
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t11 := borrow_global<LibraConfig::LibraConfig<#0>>(addr)
    call $t11 := $BorrowGlobal(addr, $LibraConfig_LibraConfig_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 3170);
      goto Abort;
    }
    assume $LibraConfig_LibraConfig_is_well_formed($Dereference($t11));

    // UnpackRef($t11)

    // config := $t11
    call config := $CopyOrMoveRef($t11);
    if (true) { assume $DebugTrackLocal(10, 3161, 3, $Dereference(config)); }

    // $t13 := move(config)
    call $t13 := $CopyOrMoveRef(config);

    // $t14 := borrow_field<LibraConfig::LibraConfig<#0>>.payload($t13)
    call $t14 := $BorrowField($t13, $LibraConfig_LibraConfig_payload);

    // LibraConfig::LibraConfig <- $t13
    call $WritebackToGlobal($t13);

    // UnpackRef($t14)

    // write_ref($t14, $t17)
    call $t14 := $WriteRef($t14, $t17);
    if (true) { assume $DebugTrackLocal(10, 3224, 3, $Dereference(config)); }

    // LibraConfig::LibraConfig <- $t14
    call $WritebackToGlobal($t14);

    // Reference($t13) <- $t14
    call $t13 := $WritebackToReference($t14, $t13);

    // PackRef($t13)

    // PackRef($t14)

    // LibraConfig::reconfigure_()
    call $LibraConfig_reconfigure_();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 6630);
      goto Abort;
    }

    // return ()
    return;

    // L2:
L2:

    // $t15 := 24
    $tmp := $Integer(24);
    $t15 := $tmp;

    // abort($t15)
    if (true) { assume $DebugTrackAbort(10, 3102); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}



// ** spec vars of module RegisteredCurrencies



// ** spec funs of module RegisteredCurrencies

function {:inline} $RegisteredCurrencies_spec_is_initialized($m: $Memory, $txn: $Transaction): $Value {
    $LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())
}



// ** structs of module RegisteredCurrencies

const unique $RegisteredCurrencies_RegisteredCurrencies: $TypeName;
const $RegisteredCurrencies_RegisteredCurrencies_currency_codes: $FieldName;
axiom $RegisteredCurrencies_RegisteredCurrencies_currency_codes == 0;
function $RegisteredCurrencies_RegisteredCurrencies_type_value(): $TypeValue {
    $StructType($RegisteredCurrencies_RegisteredCurrencies, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $Vector_type_value($Vector_type_value($IntegerType()))], 1))
}
function {:inline} $RegisteredCurrencies_RegisteredCurrencies_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $Vector_is_well_formed($SelectField($this, $RegisteredCurrencies_RegisteredCurrencies_currency_codes)) && (forall $$0: int :: {$select_vector($SelectField($this, $RegisteredCurrencies_RegisteredCurrencies_currency_codes),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $RegisteredCurrencies_RegisteredCurrencies_currency_codes)) ==> $Vector_is_well_formed($select_vector($SelectField($this, $RegisteredCurrencies_RegisteredCurrencies_currency_codes),$$0)) && (forall $$1: int :: {$select_vector($select_vector($SelectField($this, $RegisteredCurrencies_RegisteredCurrencies_currency_codes),$$0),$$1)} $$1 >= 0 && $$1 < $vlen($select_vector($SelectField($this, $RegisteredCurrencies_RegisteredCurrencies_currency_codes),$$0)) ==> $IsValidU8($select_vector($select_vector($SelectField($this, $RegisteredCurrencies_RegisteredCurrencies_currency_codes),$$0),$$1))))
}
function {:inline} $RegisteredCurrencies_RegisteredCurrencies_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $Vector_is_well_formed($SelectField($this, $RegisteredCurrencies_RegisteredCurrencies_currency_codes)) && (forall $$0: int :: {$select_vector($SelectField($this, $RegisteredCurrencies_RegisteredCurrencies_currency_codes),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $RegisteredCurrencies_RegisteredCurrencies_currency_codes)) ==> $Vector_is_well_formed($select_vector($SelectField($this, $RegisteredCurrencies_RegisteredCurrencies_currency_codes),$$0)) && (forall $$1: int :: {$select_vector($select_vector($SelectField($this, $RegisteredCurrencies_RegisteredCurrencies_currency_codes),$$0),$$1)} $$1 >= 0 && $$1 < $vlen($select_vector($SelectField($this, $RegisteredCurrencies_RegisteredCurrencies_currency_codes),$$0)) ==> $IsValidU8($select_vector($select_vector($SelectField($this, $RegisteredCurrencies_RegisteredCurrencies_currency_codes),$$0),$$1))))
}

procedure {:inline 1} $RegisteredCurrencies_RegisteredCurrencies_pack($file_id: int, $byte_index: int, $var_idx: int, currency_codes: $Value) returns ($struct: $Value)
{
    assume $Vector_is_well_formed(currency_codes) && (forall $$0: int :: {$select_vector(currency_codes,$$0)} $$0 >= 0 && $$0 < $vlen(currency_codes) ==> $Vector_is_well_formed($select_vector(currency_codes,$$0)) && (forall $$1: int :: {$select_vector($select_vector(currency_codes,$$0),$$1)} $$1 >= 0 && $$1 < $vlen($select_vector(currency_codes,$$0)) ==> $IsValidU8($select_vector($select_vector(currency_codes,$$0),$$1))));
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := currency_codes], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $RegisteredCurrencies_RegisteredCurrencies_unpack($struct: $Value) returns (currency_codes: $Value)
{
    assume is#$Vector($struct);
    currency_codes := $SelectField($struct, $RegisteredCurrencies_RegisteredCurrencies_currency_codes);
    assume $Vector_is_well_formed(currency_codes) && (forall $$0: int :: {$select_vector(currency_codes,$$0)} $$0 >= 0 && $$0 < $vlen(currency_codes) ==> $Vector_is_well_formed($select_vector(currency_codes,$$0)) && (forall $$1: int :: {$select_vector($select_vector(currency_codes,$$0),$$1)} $$1 >= 0 && $$1 < $vlen($select_vector(currency_codes,$$0)) ==> $IsValidU8($select_vector($select_vector(currency_codes,$$0),$$1))));
}

const unique $RegisteredCurrencies_RegistrationCapability: $TypeName;
const $RegisteredCurrencies_RegistrationCapability_cap: $FieldName;
axiom $RegisteredCurrencies_RegistrationCapability_cap == 0;
function $RegisteredCurrencies_RegistrationCapability_type_value(): $TypeValue {
    $StructType($RegisteredCurrencies_RegistrationCapability, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $LibraConfig_ModifyConfigCapability_type_value($RegisteredCurrencies_RegisteredCurrencies_type_value())], 1))
}
function {:inline} $RegisteredCurrencies_RegistrationCapability_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $LibraConfig_ModifyConfigCapability_is_well_formed_types($SelectField($this, $RegisteredCurrencies_RegistrationCapability_cap))
}
function {:inline} $RegisteredCurrencies_RegistrationCapability_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $LibraConfig_ModifyConfigCapability_is_well_formed($SelectField($this, $RegisteredCurrencies_RegistrationCapability_cap))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $RegisteredCurrencies_RegistrationCapability_is_well_formed($ResourceValue(m, $RegisteredCurrencies_RegistrationCapability_type_value(), a))
);

procedure {:inline 1} $RegisteredCurrencies_RegistrationCapability_pack($file_id: int, $byte_index: int, $var_idx: int, cap: $Value) returns ($struct: $Value)
{
    assume $LibraConfig_ModifyConfigCapability_is_well_formed(cap);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := cap], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $RegisteredCurrencies_RegistrationCapability_unpack($struct: $Value) returns (cap: $Value)
{
    assume is#$Vector($struct);
    cap := $SelectField($struct, $RegisteredCurrencies_RegistrationCapability_cap);
    assume $LibraConfig_ModifyConfigCapability_is_well_formed(cap);
}



// ** functions of module RegisteredCurrencies

procedure {:inline 1} $RegisteredCurrencies_initialize (config_account: $Value, create_config_capability: $Value) returns ($ret0: $Value)
free requires is#$Address(config_account);
free requires $Roles_Capability_is_well_formed(create_config_capability);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)))) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(!b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)))))))))
    || b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn));
requires b#$Boolean($Boolean(b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())) && b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)) ==> b#$Boolean($Boolean($IsEqual(addr, $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS()))))))))))))
    || b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn));
ensures b#$Boolean(old($RegisteredCurrencies_spec_is_initialized($m, $txn))) ==> $abort_flag;
ensures !$abort_flag ==> (b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($RegisteredCurrencies_spec_is_initialized($m, $txn))) ==> b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)))) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(!b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())) && b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)) ==> b#$Boolean($Boolean($IsEqual(addr, $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($RegisteredCurrencies_spec_is_initialized($m, $txn))) ==> b#$Boolean($Boolean($IsEqual(old($SelectField($LibraConfig_spec_get($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value()), $RegisteredCurrencies_RegisteredCurrencies_currency_codes)), $SelectField($LibraConfig_spec_get($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value()), $RegisteredCurrencies_RegisteredCurrencies_currency_codes)))))));
{
    // declare local variables
    var cap: $Value; // $LibraConfig_ModifyConfigCapability_type_value($RegisteredCurrencies_RegisteredCurrencies_type_value())
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $Roles_Capability_type_value($LibraConfig_CreateOnChainConfig_type_value())
    var $t12: $Value; // $RegisteredCurrencies_RegisteredCurrencies_type_value()
    var $t13: $Value; // $LibraConfig_ModifyConfigCapability_type_value($RegisteredCurrencies_RegisteredCurrencies_type_value())
    var $t14: $Value; // $LibraConfig_ModifyConfigCapability_type_value($RegisteredCurrencies_RegisteredCurrencies_type_value())
    var $t15: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t16: $Value; // $Roles_Capability_type_value($LibraConfig_CreateOnChainConfig_type_value())
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $Roles_Capability_type_value($LibraConfig_CreateOnChainConfig_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(12, 665, 0, config_account); }
    if (true) { assume $DebugTrackLocal(12, 665, 1, create_config_capability); }

    // bytecode translation starts here
    // $t19 := move(config_account)
    call $tmp := $CopyOrMoveValue(config_account);
    $t19 := $tmp;

    // $t20 := move(create_config_capability)
    call $tmp := $CopyOrMoveValue(create_config_capability);
    $t20 := $tmp;

    // $t5 := copy($t19)
    call $tmp := $CopyOrMoveValue($t19);
    $t5 := $tmp;

    // $t6 := Signer::address_of($t5)
    call $t6 := $Signer_address_of($t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(12, 925);
      goto Abort;
    }
    assume is#$Address($t6);


    // $t7 := CoreAddresses::DEFAULT_CONFIG_ADDRESS()
    call $t7 := $CoreAddresses_DEFAULT_CONFIG_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(12, 970);
      goto Abort;
    }
    assume is#$Address($t7);


    // $t8 := ==($t6, $t7)
    $tmp := $Boolean($IsEqual($t6, $t7));
    $t8 := $tmp;

    // $t3 := $t8
    call $tmp := $CopyOrMoveValue($t8);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(12, 897, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t10 := move($t19)
    call $tmp := $CopyOrMoveValue($t19);
    $t10 := $tmp;

    // $t11 := move($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t11 := $tmp;

    // $t12 := RegisteredCurrencies::empty()
    call $t12 := $RegisteredCurrencies_empty();
    if ($abort_flag) {
      assume $DebugTrackAbort(12, 1240);
      goto Abort;
    }
    assume $RegisteredCurrencies_RegisteredCurrencies_is_well_formed($t12);


    // $t13 := LibraConfig::publish_new_config_with_capability<RegisteredCurrencies::RegisteredCurrencies>($t10, $t11, $t12)
    call $t13 := $LibraConfig_publish_new_config_with_capability($RegisteredCurrencies_RegisteredCurrencies_type_value(), $t10, $t11, $t12);
    if ($abort_flag) {
      assume $DebugTrackAbort(12, 1052);
      goto Abort;
    }
    assume $LibraConfig_ModifyConfigCapability_is_well_formed($t13);


    // cap := $t13
    call $tmp := $CopyOrMoveValue($t13);
    cap := $tmp;
    if (true) { assume $DebugTrackLocal(12, 1033, 2, $tmp); }

    // $t15 := pack RegisteredCurrencies::RegistrationCapability(cap)
    call $tmp := $RegisteredCurrencies_RegistrationCapability_pack(0, 0, 0, cap);
    $t15 := $tmp;

    // return $t15
    $ret0 := $t15;
    if (true) { assume $DebugTrackLocal(12, 1194, 21, $ret0); }
    return;

    // L2:
L2:

    // $t16 := move($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t16 := $tmp;

    // destroy($t16)

    // $t17 := move($t19)
    call $tmp := $CopyOrMoveValue($t19);
    $t17 := $tmp;

    // destroy($t17)

    // $t18 := 0
    $tmp := $Integer(0);
    $t18 := $tmp;

    // abort($t18)
    if (true) { assume $DebugTrackAbort(12, 897); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $RegisteredCurrencies_empty () returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)))) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(!b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)))))))));
requires b#$Boolean($Boolean(b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())) && b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)) ==> b#$Boolean($Boolean($IsEqual(addr, $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS()))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($RegisteredCurrencies_spec_is_initialized($m, $txn))) ==> b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)))) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(!b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())) && b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)) ==> b#$Boolean($Boolean($IsEqual(addr, $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($RegisteredCurrencies_spec_is_initialized($m, $txn))) ==> b#$Boolean($Boolean($IsEqual(old($SelectField($LibraConfig_spec_get($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value()), $RegisteredCurrencies_RegisteredCurrencies_currency_codes)), $SelectField($LibraConfig_spec_get($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value()), $RegisteredCurrencies_RegisteredCurrencies_currency_codes)))))));
{
    // declare local variables
    var $t0: $Value; // $Vector_type_value($Vector_type_value($IntegerType()))
    var $t1: $Value; // $RegisteredCurrencies_RegisteredCurrencies_type_value()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := Vector::empty<vector<u8>>()
    call $t0 := $Vector_empty($Vector_type_value($IntegerType()));
    if ($abort_flag) {
      assume $DebugTrackAbort(12, 1327);
      goto Abort;
    }
    assume $Vector_is_well_formed($t0) && (forall $$0: int :: {$select_vector($t0,$$0)} $$0 >= 0 && $$0 < $vlen($t0) ==> $Vector_is_well_formed($select_vector($t0,$$0)) && (forall $$1: int :: {$select_vector($select_vector($t0,$$0),$$1)} $$1 >= 0 && $$1 < $vlen($select_vector($t0,$$0)) ==> $IsValidU8($select_vector($select_vector($t0,$$0),$$1))));


    // $t1 := pack RegisteredCurrencies::RegisteredCurrencies($t0)
    call $tmp := $RegisteredCurrencies_RegisteredCurrencies_pack(0, 0, 0, $t0);
    $t1 := $tmp;

    // return $t1
    $ret0 := $t1;
    if (true) { assume $DebugTrackLocal(12, 1280, 2, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $RegisteredCurrencies_add_currency_code (currency_code: $Value, cap: $Value) returns ()
free requires $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));
free requires $RegisteredCurrencies_RegistrationCapability_is_well_formed(cap);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)))) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(!b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)))))))));
requires b#$Boolean($Boolean(b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())) && b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)) ==> b#$Boolean($Boolean($IsEqual(addr, $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS()))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean(old($RegisteredCurrencies_spec_is_initialized($m, $txn))) ==> b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)))) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(!b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())) && b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)) ==> b#$Boolean($Boolean($IsEqual(addr, $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())))))))))))));
{
    // declare local variables
    var config: $Value; // $RegisteredCurrencies_RegisteredCurrencies_type_value()
    var $t3: $Value; // $RegisteredCurrencies_RegisteredCurrencies_type_value()
    var $t4: $Reference; // ReferenceType($RegisteredCurrencies_RegisteredCurrencies_type_value())
    var $t5: $Reference; // ReferenceType($Vector_type_value($Vector_type_value($IntegerType())))
    var $t6: $Value; // $Vector_type_value($IntegerType())
    var $t7: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t8: $Value; // $LibraConfig_ModifyConfigCapability_type_value($RegisteredCurrencies_RegisteredCurrencies_type_value())
    var $t9: $Value; // $RegisteredCurrencies_RegisteredCurrencies_type_value()
    var $t10: $Value; // $Vector_type_value($IntegerType())
    var $t11: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t12: $Value; // $Vector_type_value($Vector_type_value($IntegerType()))
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(12, 1348, 0, currency_code); }
    if (true) { assume $DebugTrackLocal(12, 1348, 1, cap); }

    // bytecode translation starts here
    // $t10 := move(currency_code)
    call $tmp := $CopyOrMoveValue(currency_code);
    $t10 := $tmp;

    // $t11 := move(cap)
    call $tmp := $CopyOrMoveValue(cap);
    $t11 := $tmp;

    // $t3 := LibraConfig::get<RegisteredCurrencies::RegisteredCurrencies>()
    call $t3 := $LibraConfig_get($RegisteredCurrencies_RegisteredCurrencies_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(12, 1493);
      goto Abort;
    }
    assume $RegisteredCurrencies_RegisteredCurrencies_is_well_formed($t3);


    // config := $t3
    call $tmp := $CopyOrMoveValue($t3);
    config := $tmp;
    if (true) { assume $DebugTrackLocal(12, 1471, 2, $tmp); }

    // $t4 := borrow_local(config)
    call $t4 := $BorrowLoc(2, config);
    assume $RegisteredCurrencies_RegisteredCurrencies_is_well_formed($Dereference($t4));

    // UnpackRef($t4)

    // $t5 := borrow_field<RegisteredCurrencies::RegisteredCurrencies>.currency_codes($t4)
    call $t5 := $BorrowField($t4, $RegisteredCurrencies_RegisteredCurrencies_currency_codes);
    assume $Vector_is_well_formed($Dereference($t5)) && (forall $$1: int :: {$select_vector($Dereference($t5),$$1)} $$1 >= 0 && $$1 < $vlen($Dereference($t5)) ==> $Vector_is_well_formed($select_vector($Dereference($t5),$$1)) && (forall $$2: int :: {$select_vector($select_vector($Dereference($t5),$$1),$$2)} $$2 >= 0 && $$2 < $vlen($select_vector($Dereference($t5),$$1)) ==> $IsValidU8($select_vector($select_vector($Dereference($t5),$$1),$$2))));

    // LocalRoot(config) <- $t4
    call config := $WritebackToValue($t4, 2, config);

    // UnpackRef($t5)

    // PackRef($t5)

    // $t12 := read_ref($t5)
    call $tmp := $ReadRef($t5);
    assume $Vector_is_well_formed($tmp) && (forall $$0: int :: {$select_vector($tmp,$$0)} $$0 >= 0 && $$0 < $vlen($tmp) ==> $Vector_is_well_formed($select_vector($tmp,$$0)) && (forall $$1: int :: {$select_vector($select_vector($tmp,$$0),$$1)} $$1 >= 0 && $$1 < $vlen($select_vector($tmp,$$0)) ==> $IsValidU8($select_vector($select_vector($tmp,$$0),$$1))));
    $t12 := $tmp;

    // $t12 := Vector::push_back<vector<u8>>($t12, $t10)
    call $t12 := $Vector_push_back($Vector_type_value($IntegerType()), $t12, $t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(12, 1538);
      goto Abort;
    }
    assume $Vector_is_well_formed($t12) && (forall $$0: int :: {$select_vector($t12,$$0)} $$0 >= 0 && $$0 < $vlen($t12) ==> $Vector_is_well_formed($select_vector($t12,$$0)) && (forall $$1: int :: {$select_vector($select_vector($t12,$$0),$$1)} $$1 >= 0 && $$1 < $vlen($select_vector($t12,$$0)) ==> $IsValidU8($select_vector($select_vector($t12,$$0),$$1))));


    // write_ref($t5, $t12)
    call $t5 := $WriteRef($t5, $t12);

    // LocalRoot(config) <- $t5
    call config := $WritebackToValue($t5, 2, config);

    // Reference($t4) <- $t5
    call $t4 := $WritebackToReference($t5, $t4);

    // UnpackRef($t5)

    // PackRef($t4)

    // PackRef($t5)

    // $t7 := move($t11)
    call $tmp := $CopyOrMoveValue($t11);
    $t7 := $tmp;

    // $t8 := get_field<RegisteredCurrencies::RegistrationCapability>.cap($t7)
    call $tmp := $GetFieldFromValue($t7, $RegisteredCurrencies_RegistrationCapability_cap);
    assume $LibraConfig_ModifyConfigCapability_is_well_formed($tmp);
    $t8 := $tmp;

    // LibraConfig::set_with_capability<RegisteredCurrencies::RegisteredCurrencies>($t8, config)
    call $LibraConfig_set_with_capability($RegisteredCurrencies_RegisteredCurrencies_type_value(), $t8, config);
    if ($abort_flag) {
      assume $DebugTrackAbort(12, 1613);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}



// ** spec vars of module FixedPoint32



// ** spec funs of module FixedPoint32



// ** structs of module FixedPoint32

const unique $FixedPoint32_FixedPoint32: $TypeName;
const $FixedPoint32_FixedPoint32_value: $FieldName;
axiom $FixedPoint32_FixedPoint32_value == 0;
function $FixedPoint32_FixedPoint32_type_value(): $TypeValue {
    $StructType($FixedPoint32_FixedPoint32, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $IntegerType()], 1))
}
function {:inline} $FixedPoint32_FixedPoint32_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $IsValidU64($SelectField($this, $FixedPoint32_FixedPoint32_value))
}
function {:inline} $FixedPoint32_FixedPoint32_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $IsValidU64($SelectField($this, $FixedPoint32_FixedPoint32_value))
}

procedure {:inline 1} $FixedPoint32_FixedPoint32_pack($file_id: int, $byte_index: int, $var_idx: int, value: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(value);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := value], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $FixedPoint32_FixedPoint32_unpack($struct: $Value) returns (value: $Value)
{
    assume is#$Vector($struct);
    value := $SelectField($struct, $FixedPoint32_FixedPoint32_value);
    assume $IsValidU64(value);
}



// ** functions of module FixedPoint32

procedure {:inline 1} $FixedPoint32_create_from_rational (numerator: $Value, denominator: $Value) returns ($ret0: $Value)
free requires $IsValidU64(numerator);
free requires $IsValidU64(denominator);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var quotient: $Value; // $IntegerType()
    var scaled_denominator: $Value; // $IntegerType()
    var scaled_numerator: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $IntegerType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $BooleanType()
    var $t22: $Value; // $BooleanType()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $IntegerType()
    var $t25: $Value; // $BooleanType()
    var $t26: $Value; // $BooleanType()
    var $t27: $Value; // $BooleanType()
    var $t28: $Value; // $IntegerType()
    var $t29: $Value; // $IntegerType()
    var $t30: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t31: $Value; // $IntegerType()
    var $t32: $Value; // $IntegerType()
    var $t33: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(6, 2295, 0, numerator); }
    if (true) { assume $DebugTrackLocal(6, 2295, 1, denominator); }

    // bytecode translation starts here
    // $t32 := move(numerator)
    call $tmp := $CopyOrMoveValue(numerator);
    $t32 := $tmp;

    // $t33 := move(denominator)
    call $tmp := $CopyOrMoveValue(denominator);
    $t33 := $tmp;

    // $t9 := (u128)($t32)
    call $tmp := $CastU128($t32);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 2587);
      goto Abort;
    }
    $t9 := $tmp;

    // $t10 := 64
    $tmp := $Integer(64);
    $t10 := $tmp;

    // $t11 := <<($t9, $t10)
    call $tmp := $Shl($t9, $t10);
    $t11 := $tmp;

    // scaled_numerator := $t11
    call $tmp := $CopyOrMoveValue($t11);
    scaled_numerator := $tmp;
    if (true) { assume $DebugTrackLocal(6, 2568, 4, $tmp); }

    // $t13 := (u128)($t33)
    call $tmp := $CastU128($t33);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 2647);
      goto Abort;
    }
    $t13 := $tmp;

    // $t14 := 32
    $tmp := $Integer(32);
    $t14 := $tmp;

    // $t15 := <<($t13, $t14)
    call $tmp := $Shl($t13, $t14);
    $t15 := $tmp;

    // scaled_denominator := $t15
    call $tmp := $CopyOrMoveValue($t15);
    scaled_denominator := $tmp;
    if (true) { assume $DebugTrackLocal(6, 2626, 3, $tmp); }

    // $t18 := /(scaled_numerator, scaled_denominator)
    call $tmp := $Div(scaled_numerator, scaled_denominator);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 2810);
      goto Abort;
    }
    $t18 := $tmp;

    // quotient := $t18
    call $tmp := $CopyOrMoveValue($t18);
    quotient := $tmp;
    if (true) { assume $DebugTrackLocal(6, 2782, 2, $tmp); }

    // $t20 := 0
    $tmp := $Integer(0);
    $t20 := $tmp;

    // $t21 := !=(quotient, $t20)
    $tmp := $Boolean(!$IsEqual(quotient, $t20));
    $t21 := $tmp;

    // if ($t21) goto L0 else goto L1
    $tmp := $t21;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t22 := true
    $tmp := $Boolean(true);
    $t22 := $tmp;

    // $t7 := $t22
    call $tmp := $CopyOrMoveValue($t22);
    $t7 := $tmp;
    if (true) { assume $DebugTrackLocal(6, 3033, 7, $tmp); }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t24 := 0
    $tmp := $Integer(0);
    $t24 := $tmp;

    // $t25 := ==($t32, $t24)
    $tmp := $Boolean($IsEqual($t32, $t24));
    $t25 := $tmp;

    // $t7 := $t25
    call $tmp := $CopyOrMoveValue($t25);
    $t7 := $tmp;
    if (true) { assume $DebugTrackLocal(6, 3033, 7, $tmp); }

    // goto L3
    goto L3;

    // L3:
L3:

    // $t5 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(6, 3026, 5, $tmp); }

    // if ($t5) goto L4 else goto L5
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L4; } else { goto L5; }

    // L5:
L5:

    // goto L6
    goto L6;

    // L4:
L4:

    // $t29 := (u64)(quotient)
    call $tmp := $CastU64(quotient);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 3240);
      goto Abort;
    }
    $t29 := $tmp;

    // $t30 := pack FixedPoint32::FixedPoint32($t29)
    call $tmp := $FixedPoint32_FixedPoint32_pack(0, 0, 0, $t29);
    $t30 := $tmp;

    // return $t30
    $ret0 := $t30;
    if (true) { assume $DebugTrackLocal(6, 3218, 34, $ret0); }
    return;

    // L6:
L6:

    // $t31 := 16
    $tmp := $Integer(16);
    $t31 := $tmp;

    // abort($t31)
    if (true) { assume $DebugTrackAbort(6, 3026); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $FixedPoint32_create_from_raw_value (value: $Value) returns ($ret0: $Value)
free requires $IsValidU64(value);
requires $ExistsTxnSenderAccount($m, $txn);
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(false)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Vector($ExtendValueArray($EmptyValueArray(), value))))));
{
    // declare local variables
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t3: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(6, 3271, 0, value); }

    // bytecode translation starts here
    // $t3 := move(value)
    call $tmp := $CopyOrMoveValue(value);
    $t3 := $tmp;

    // $t2 := pack FixedPoint32::FixedPoint32($t3)
    call $tmp := $FixedPoint32_FixedPoint32_pack(0, 0, 0, $t3);
    $t2 := $tmp;

    // return $t2
    $ret0 := $t2;
    if (true) { assume $DebugTrackLocal(6, 3340, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $FixedPoint32_divide_u64 (num: $Value, divisor: $Value) returns ($ret0: $Value)
free requires $IsValidU64(num);
free requires $FixedPoint32_FixedPoint32_is_well_formed(divisor);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var quotient: $Value; // $IntegerType()
    var scaled_value: $Value; // $IntegerType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $IntegerType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(6, 1307, 0, num); }
    if (true) { assume $DebugTrackLocal(6, 1307, 1, divisor); }

    // bytecode translation starts here
    // $t16 := move(num)
    call $tmp := $CopyOrMoveValue(num);
    $t16 := $tmp;

    // $t17 := move(divisor)
    call $tmp := $CopyOrMoveValue(divisor);
    $t17 := $tmp;

    // $t5 := (u128)($t16)
    call $tmp := $CastU128($t16);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 1512);
      goto Abort;
    }
    $t5 := $tmp;

    // $t6 := 32
    $tmp := $Integer(32);
    $t6 := $tmp;

    // $t7 := <<($t5, $t6)
    call $tmp := $Shl($t5, $t6);
    $t7 := $tmp;

    // scaled_value := $t7
    call $tmp := $CopyOrMoveValue($t7);
    scaled_value := $tmp;
    if (true) { assume $DebugTrackLocal(6, 1497, 3, $tmp); }

    // $t9 := copy($t17)
    call $tmp := $CopyOrMoveValue($t17);
    $t9 := $tmp;

    // $t10 := get_field<FixedPoint32::FixedPoint32>.value($t9)
    call $tmp := $GetFieldFromValue($t9, $FixedPoint32_FixedPoint32_value);
    assume $IsValidU64($tmp);
    $t10 := $tmp;

    // $t11 := move($t10)
    call $tmp := $CopyOrMoveValue($t10);
    $t11 := $tmp;

    // $t12 := (u128)($t11)
    call $tmp := $CastU128($t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 1705);
      goto Abort;
    }
    $t12 := $tmp;

    // $t13 := /(scaled_value, $t12)
    call $tmp := $Div(scaled_value, $t12);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 1703);
      goto Abort;
    }
    $t13 := $tmp;

    // quotient := $t13
    call $tmp := $CopyOrMoveValue($t13);
    quotient := $tmp;
    if (true) { assume $DebugTrackLocal(6, 1679, 2, $tmp); }

    // $t15 := (u64)(quotient)
    call $tmp := $CastU64(quotient);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 1913);
      goto Abort;
    }
    $t15 := $tmp;

    // return $t15
    $ret0 := $t15;
    if (true) { assume $DebugTrackLocal(6, 1913, 18, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $FixedPoint32_get_raw_value (num: $Value) returns ($ret0: $Value)
free requires $FixedPoint32_FixedPoint32_is_well_formed(num);
requires $ExistsTxnSenderAccount($m, $txn);
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(false)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $SelectField(num, $FixedPoint32_FixedPoint32_value)))));
{
    // declare local variables
    var $t1: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(6, 3551, 0, num); }

    // bytecode translation starts here
    // $t4 := move(num)
    call $tmp := $CopyOrMoveValue(num);
    $t4 := $tmp;

    // $t1 := copy($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := get_field<FixedPoint32::FixedPoint32>.value($t1)
    call $tmp := $GetFieldFromValue($t1, $FixedPoint32_FixedPoint32_value);
    assume $IsValidU64($tmp);
    $t2 := $tmp;

    // $t3 := move($t2)
    call $tmp := $CopyOrMoveValue($t2);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(6, 3610, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $FixedPoint32_multiply_u64 (num: $Value, multiplier: $Value) returns ($ret0: $Value)
free requires $IsValidU64(num);
free requires $FixedPoint32_FixedPoint32_is_well_formed(multiplier);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var product: $Value; // $IntegerType()
    var unscaled_product: $Value; // $IntegerType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $IntegerType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(6, 400, 0, num); }
    if (true) { assume $DebugTrackLocal(6, 400, 1, multiplier); }

    // bytecode translation starts here
    // $t16 := move(num)
    call $tmp := $CopyOrMoveValue(num);
    $t16 := $tmp;

    // $t17 := move(multiplier)
    call $tmp := $CopyOrMoveValue(multiplier);
    $t17 := $tmp;

    // $t5 := (u128)($t16)
    call $tmp := $CastU128($t16);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 684);
      goto Abort;
    }
    $t5 := $tmp;

    // $t6 := copy($t17)
    call $tmp := $CopyOrMoveValue($t17);
    $t6 := $tmp;

    // $t7 := get_field<FixedPoint32::FixedPoint32>.value($t6)
    call $tmp := $GetFieldFromValue($t6, $FixedPoint32_FixedPoint32_value);
    assume $IsValidU64($tmp);
    $t7 := $tmp;

    // $t8 := move($t7)
    call $tmp := $CopyOrMoveValue($t7);
    $t8 := $tmp;

    // $t9 := (u128)($t8)
    call $tmp := $CastU128($t8);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 700);
      goto Abort;
    }
    $t9 := $tmp;

    // $t10 := *($t5, $t9)
    call $tmp := $MulU128($t5, $t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 698);
      goto Abort;
    }
    $t10 := $tmp;

    // unscaled_product := $t10
    call $tmp := $CopyOrMoveValue($t10);
    unscaled_product := $tmp;
    if (true) { assume $DebugTrackLocal(6, 665, 3, $tmp); }

    // $t12 := 32
    $tmp := $Integer(32);
    $t12 := $tmp;

    // $t13 := <<(unscaled_product, $t12)
    call $tmp := $Shr(unscaled_product, $t12);
    $t13 := $tmp;

    // product := $t13
    call $tmp := $CopyOrMoveValue($t13);
    product := $tmp;
    if (true) { assume $DebugTrackLocal(6, 873, 2, $tmp); }

    // $t15 := (u64)(product)
    call $tmp := $CastU64(product);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 1095);
      goto Abort;
    }
    $t15 := $tmp;

    // return $t15
    $ret0 := $t15;
    if (true) { assume $DebugTrackLocal(6, 1095, 18, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}



// ** spec vars of module Libra

var $Libra_sum_of_coin_values : [$TypeValue]$Value where (forall $tv0: $TypeValue :: $IsValidNum($Libra_sum_of_coin_values[$tv0]));


// ** spec funs of module Libra

function {:inline} $Libra_spec_currency_addr(): $Value {
    $Address(173345816)
}

function {:inline} $Libra_spec_is_currency($m: $Memory, $txn: $Transaction, $tv0: $TypeValue): $Value {
    $ResourceExists($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr())
}



// ** structs of module Libra

const unique $Libra_Libra: $TypeName;
const $Libra_Libra_value: $FieldName;
axiom $Libra_Libra_value == 0;
function $Libra_Libra_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Libra_Libra, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $IntegerType()], 1))
}
function {:inline} $Libra_Libra_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $IsValidU64($SelectField($this, $Libra_Libra_value))
}
function {:inline} $Libra_Libra_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $IsValidU64($SelectField($this, $Libra_Libra_value))
}

axiom (forall m: $Memory, a: $Value, $tv0: $TypeValue :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Libra_Libra_is_well_formed($ResourceValue(m, $Libra_Libra_type_value($tv0), a))
);

procedure {:inline 1} $Libra_Libra_before_update_inv($tv0: $TypeValue, $before: $Value) {
    $Libra_sum_of_coin_values := $Libra_sum_of_coin_values[$tv0 := $Integer(i#$Integer($Libra_sum_of_coin_values[$tv0]) - i#$Integer($SelectField($before, $Libra_Libra_value)))];
}

procedure {:inline 1} $Libra_Libra_after_update_inv($tv0: $TypeValue, $after: $Value) {
    $Libra_sum_of_coin_values := $Libra_sum_of_coin_values[$tv0 := $Integer(i#$Integer($Libra_sum_of_coin_values[$tv0]) + i#$Integer($SelectField($after, $Libra_Libra_value)))];
}

procedure {:inline 1} $Libra_Libra_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, value: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(value);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := value], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
    $Libra_sum_of_coin_values := $Libra_sum_of_coin_values[$tv0 := $Integer(i#$Integer($Libra_sum_of_coin_values[$tv0]) + i#$Integer($SelectField($struct, $Libra_Libra_value)))];
}

procedure {:inline 1} $Libra_Libra_unpack($tv0: $TypeValue, $struct: $Value) returns (value: $Value)
{
    assume is#$Vector($struct);
    value := $SelectField($struct, $Libra_Libra_value);
    assume $IsValidU64(value);
    $Libra_sum_of_coin_values := $Libra_sum_of_coin_values[$tv0 := $Integer(i#$Integer($Libra_sum_of_coin_values[$tv0]) - i#$Integer($SelectField($struct, $Libra_Libra_value)))];
}

const unique $Libra_BurnCapability: $TypeName;
const $Libra_BurnCapability_dummy_field: $FieldName;
axiom $Libra_BurnCapability_dummy_field == 0;
function $Libra_BurnCapability_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Libra_BurnCapability, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $Libra_BurnCapability_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Libra_BurnCapability_dummy_field))
}
function {:inline} $Libra_BurnCapability_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Libra_BurnCapability_dummy_field))
}

axiom (forall m: $Memory, a: $Value, $tv0: $TypeValue :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Libra_BurnCapability_is_well_formed($ResourceValue(m, $Libra_BurnCapability_type_value($tv0), a))
);

procedure {:inline 1} $Libra_BurnCapability_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Libra_BurnCapability_unpack($tv0: $TypeValue, $struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $Libra_BurnCapability_dummy_field);
    assume is#$Boolean(dummy_field);
}

const unique $Libra_BurnEvent: $TypeName;
const $Libra_BurnEvent_amount: $FieldName;
axiom $Libra_BurnEvent_amount == 0;
const $Libra_BurnEvent_currency_code: $FieldName;
axiom $Libra_BurnEvent_currency_code == 1;
const $Libra_BurnEvent_preburn_address: $FieldName;
axiom $Libra_BurnEvent_preburn_address == 2;
function $Libra_BurnEvent_type_value(): $TypeValue {
    $StructType($Libra_BurnEvent, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $IntegerType()][1 := $Vector_type_value($IntegerType())][2 := $AddressType()], 3))
}
function {:inline} $Libra_BurnEvent_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 3
      && $IsValidU64($SelectField($this, $Libra_BurnEvent_amount))
      && $Vector_is_well_formed($SelectField($this, $Libra_BurnEvent_currency_code)) && (forall $$0: int :: {$select_vector($SelectField($this, $Libra_BurnEvent_currency_code),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Libra_BurnEvent_currency_code)) ==> $IsValidU8($select_vector($SelectField($this, $Libra_BurnEvent_currency_code),$$0)))
      && is#$Address($SelectField($this, $Libra_BurnEvent_preburn_address))
}
function {:inline} $Libra_BurnEvent_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 3
      && $IsValidU64($SelectField($this, $Libra_BurnEvent_amount))
      && $Vector_is_well_formed($SelectField($this, $Libra_BurnEvent_currency_code)) && (forall $$0: int :: {$select_vector($SelectField($this, $Libra_BurnEvent_currency_code),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Libra_BurnEvent_currency_code)) ==> $IsValidU8($select_vector($SelectField($this, $Libra_BurnEvent_currency_code),$$0)))
      && is#$Address($SelectField($this, $Libra_BurnEvent_preburn_address))
}

procedure {:inline 1} $Libra_BurnEvent_pack($file_id: int, $byte_index: int, $var_idx: int, amount: $Value, currency_code: $Value, preburn_address: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(amount);
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));
    assume is#$Address(preburn_address);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := amount][1 := currency_code][2 := preburn_address], 3));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Libra_BurnEvent_unpack($struct: $Value) returns (amount: $Value, currency_code: $Value, preburn_address: $Value)
{
    assume is#$Vector($struct);
    amount := $SelectField($struct, $Libra_BurnEvent_amount);
    assume $IsValidU64(amount);
    currency_code := $SelectField($struct, $Libra_BurnEvent_currency_code);
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));
    preburn_address := $SelectField($struct, $Libra_BurnEvent_preburn_address);
    assume is#$Address(preburn_address);
}

const unique $Libra_CancelBurnEvent: $TypeName;
const $Libra_CancelBurnEvent_amount: $FieldName;
axiom $Libra_CancelBurnEvent_amount == 0;
const $Libra_CancelBurnEvent_currency_code: $FieldName;
axiom $Libra_CancelBurnEvent_currency_code == 1;
const $Libra_CancelBurnEvent_preburn_address: $FieldName;
axiom $Libra_CancelBurnEvent_preburn_address == 2;
function $Libra_CancelBurnEvent_type_value(): $TypeValue {
    $StructType($Libra_CancelBurnEvent, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $IntegerType()][1 := $Vector_type_value($IntegerType())][2 := $AddressType()], 3))
}
function {:inline} $Libra_CancelBurnEvent_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 3
      && $IsValidU64($SelectField($this, $Libra_CancelBurnEvent_amount))
      && $Vector_is_well_formed($SelectField($this, $Libra_CancelBurnEvent_currency_code)) && (forall $$0: int :: {$select_vector($SelectField($this, $Libra_CancelBurnEvent_currency_code),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Libra_CancelBurnEvent_currency_code)) ==> $IsValidU8($select_vector($SelectField($this, $Libra_CancelBurnEvent_currency_code),$$0)))
      && is#$Address($SelectField($this, $Libra_CancelBurnEvent_preburn_address))
}
function {:inline} $Libra_CancelBurnEvent_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 3
      && $IsValidU64($SelectField($this, $Libra_CancelBurnEvent_amount))
      && $Vector_is_well_formed($SelectField($this, $Libra_CancelBurnEvent_currency_code)) && (forall $$0: int :: {$select_vector($SelectField($this, $Libra_CancelBurnEvent_currency_code),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Libra_CancelBurnEvent_currency_code)) ==> $IsValidU8($select_vector($SelectField($this, $Libra_CancelBurnEvent_currency_code),$$0)))
      && is#$Address($SelectField($this, $Libra_CancelBurnEvent_preburn_address))
}

procedure {:inline 1} $Libra_CancelBurnEvent_pack($file_id: int, $byte_index: int, $var_idx: int, amount: $Value, currency_code: $Value, preburn_address: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(amount);
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));
    assume is#$Address(preburn_address);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := amount][1 := currency_code][2 := preburn_address], 3));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Libra_CancelBurnEvent_unpack($struct: $Value) returns (amount: $Value, currency_code: $Value, preburn_address: $Value)
{
    assume is#$Vector($struct);
    amount := $SelectField($struct, $Libra_CancelBurnEvent_amount);
    assume $IsValidU64(amount);
    currency_code := $SelectField($struct, $Libra_CancelBurnEvent_currency_code);
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));
    preburn_address := $SelectField($struct, $Libra_CancelBurnEvent_preburn_address);
    assume is#$Address(preburn_address);
}

const unique $Libra_CurrencyInfo: $TypeName;
const $Libra_CurrencyInfo_total_value: $FieldName;
axiom $Libra_CurrencyInfo_total_value == 0;
const $Libra_CurrencyInfo_preburn_value: $FieldName;
axiom $Libra_CurrencyInfo_preburn_value == 1;
const $Libra_CurrencyInfo_to_lbr_exchange_rate: $FieldName;
axiom $Libra_CurrencyInfo_to_lbr_exchange_rate == 2;
const $Libra_CurrencyInfo_is_synthetic: $FieldName;
axiom $Libra_CurrencyInfo_is_synthetic == 3;
const $Libra_CurrencyInfo_scaling_factor: $FieldName;
axiom $Libra_CurrencyInfo_scaling_factor == 4;
const $Libra_CurrencyInfo_fractional_part: $FieldName;
axiom $Libra_CurrencyInfo_fractional_part == 5;
const $Libra_CurrencyInfo_currency_code: $FieldName;
axiom $Libra_CurrencyInfo_currency_code == 6;
const $Libra_CurrencyInfo_can_mint: $FieldName;
axiom $Libra_CurrencyInfo_can_mint == 7;
const $Libra_CurrencyInfo_mint_events: $FieldName;
axiom $Libra_CurrencyInfo_mint_events == 8;
const $Libra_CurrencyInfo_burn_events: $FieldName;
axiom $Libra_CurrencyInfo_burn_events == 9;
const $Libra_CurrencyInfo_preburn_events: $FieldName;
axiom $Libra_CurrencyInfo_preburn_events == 10;
const $Libra_CurrencyInfo_cancel_burn_events: $FieldName;
axiom $Libra_CurrencyInfo_cancel_burn_events == 11;
const $Libra_CurrencyInfo_exchange_rate_update_events: $FieldName;
axiom $Libra_CurrencyInfo_exchange_rate_update_events == 12;
function $Libra_CurrencyInfo_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Libra_CurrencyInfo, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $IntegerType()][1 := $IntegerType()][2 := $FixedPoint32_FixedPoint32_type_value()][3 := $BooleanType()][4 := $IntegerType()][5 := $IntegerType()][6 := $Vector_type_value($IntegerType())][7 := $BooleanType()][8 := $Event_EventHandle_type_value($Libra_MintEvent_type_value())][9 := $Event_EventHandle_type_value($Libra_BurnEvent_type_value())][10 := $Event_EventHandle_type_value($Libra_PreburnEvent_type_value())][11 := $Event_EventHandle_type_value($Libra_CancelBurnEvent_type_value())][12 := $Event_EventHandle_type_value($Libra_ToLBRExchangeRateUpdateEvent_type_value())], 13))
}
function {:inline} $Libra_CurrencyInfo_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 13
      && $IsValidU128($SelectField($this, $Libra_CurrencyInfo_total_value))
      && $IsValidU64($SelectField($this, $Libra_CurrencyInfo_preburn_value))
      && $FixedPoint32_FixedPoint32_is_well_formed_types($SelectField($this, $Libra_CurrencyInfo_to_lbr_exchange_rate))
      && is#$Boolean($SelectField($this, $Libra_CurrencyInfo_is_synthetic))
      && $IsValidU64($SelectField($this, $Libra_CurrencyInfo_scaling_factor))
      && $IsValidU64($SelectField($this, $Libra_CurrencyInfo_fractional_part))
      && $Vector_is_well_formed($SelectField($this, $Libra_CurrencyInfo_currency_code)) && (forall $$0: int :: {$select_vector($SelectField($this, $Libra_CurrencyInfo_currency_code),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Libra_CurrencyInfo_currency_code)) ==> $IsValidU8($select_vector($SelectField($this, $Libra_CurrencyInfo_currency_code),$$0)))
      && is#$Boolean($SelectField($this, $Libra_CurrencyInfo_can_mint))
      && $Event_EventHandle_is_well_formed_types($SelectField($this, $Libra_CurrencyInfo_mint_events))
      && $Event_EventHandle_is_well_formed_types($SelectField($this, $Libra_CurrencyInfo_burn_events))
      && $Event_EventHandle_is_well_formed_types($SelectField($this, $Libra_CurrencyInfo_preburn_events))
      && $Event_EventHandle_is_well_formed_types($SelectField($this, $Libra_CurrencyInfo_cancel_burn_events))
      && $Event_EventHandle_is_well_formed_types($SelectField($this, $Libra_CurrencyInfo_exchange_rate_update_events))
}
function {:inline} $Libra_CurrencyInfo_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 13
      && $IsValidU128($SelectField($this, $Libra_CurrencyInfo_total_value))
      && $IsValidU64($SelectField($this, $Libra_CurrencyInfo_preburn_value))
      && $FixedPoint32_FixedPoint32_is_well_formed($SelectField($this, $Libra_CurrencyInfo_to_lbr_exchange_rate))
      && is#$Boolean($SelectField($this, $Libra_CurrencyInfo_is_synthetic))
      && $IsValidU64($SelectField($this, $Libra_CurrencyInfo_scaling_factor))
      && $IsValidU64($SelectField($this, $Libra_CurrencyInfo_fractional_part))
      && $Vector_is_well_formed($SelectField($this, $Libra_CurrencyInfo_currency_code)) && (forall $$0: int :: {$select_vector($SelectField($this, $Libra_CurrencyInfo_currency_code),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Libra_CurrencyInfo_currency_code)) ==> $IsValidU8($select_vector($SelectField($this, $Libra_CurrencyInfo_currency_code),$$0)))
      && is#$Boolean($SelectField($this, $Libra_CurrencyInfo_can_mint))
      && $Event_EventHandle_is_well_formed($SelectField($this, $Libra_CurrencyInfo_mint_events))
      && $Event_EventHandle_is_well_formed($SelectField($this, $Libra_CurrencyInfo_burn_events))
      && $Event_EventHandle_is_well_formed($SelectField($this, $Libra_CurrencyInfo_preburn_events))
      && $Event_EventHandle_is_well_formed($SelectField($this, $Libra_CurrencyInfo_cancel_burn_events))
      && $Event_EventHandle_is_well_formed($SelectField($this, $Libra_CurrencyInfo_exchange_rate_update_events))
}

axiom (forall m: $Memory, a: $Value, $tv0: $TypeValue :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Libra_CurrencyInfo_is_well_formed($ResourceValue(m, $Libra_CurrencyInfo_type_value($tv0), a))
);

procedure {:inline 1} $Libra_CurrencyInfo_before_update_inv($tv0: $TypeValue, $before: $Value) {
    call $Event_EventHandle_before_update_inv($Libra_MintEvent_type_value(), $SelectField($before, $Libra_CurrencyInfo_mint_events));
    call $Event_EventHandle_before_update_inv($Libra_BurnEvent_type_value(), $SelectField($before, $Libra_CurrencyInfo_burn_events));
    call $Event_EventHandle_before_update_inv($Libra_PreburnEvent_type_value(), $SelectField($before, $Libra_CurrencyInfo_preburn_events));
    call $Event_EventHandle_before_update_inv($Libra_CancelBurnEvent_type_value(), $SelectField($before, $Libra_CurrencyInfo_cancel_burn_events));
    call $Event_EventHandle_before_update_inv($Libra_ToLBRExchangeRateUpdateEvent_type_value(), $SelectField($before, $Libra_CurrencyInfo_exchange_rate_update_events));
}

procedure {:inline 1} $Libra_CurrencyInfo_after_update_inv($tv0: $TypeValue, $after: $Value) {
    call $Event_EventHandle_after_update_inv($Libra_MintEvent_type_value(), $SelectField($after, $Libra_CurrencyInfo_mint_events));
    call $Event_EventHandle_after_update_inv($Libra_BurnEvent_type_value(), $SelectField($after, $Libra_CurrencyInfo_burn_events));
    call $Event_EventHandle_after_update_inv($Libra_PreburnEvent_type_value(), $SelectField($after, $Libra_CurrencyInfo_preburn_events));
    call $Event_EventHandle_after_update_inv($Libra_CancelBurnEvent_type_value(), $SelectField($after, $Libra_CurrencyInfo_cancel_burn_events));
    call $Event_EventHandle_after_update_inv($Libra_ToLBRExchangeRateUpdateEvent_type_value(), $SelectField($after, $Libra_CurrencyInfo_exchange_rate_update_events));
}

procedure {:inline 1} $Libra_CurrencyInfo_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, total_value: $Value, preburn_value: $Value, to_lbr_exchange_rate: $Value, is_synthetic: $Value, scaling_factor: $Value, fractional_part: $Value, currency_code: $Value, can_mint: $Value, mint_events: $Value, burn_events: $Value, preburn_events: $Value, cancel_burn_events: $Value, exchange_rate_update_events: $Value) returns ($struct: $Value)
{
    assume $IsValidU128(total_value);
    assume $IsValidU64(preburn_value);
    assume $FixedPoint32_FixedPoint32_is_well_formed(to_lbr_exchange_rate);
    assume is#$Boolean(is_synthetic);
    assume $IsValidU64(scaling_factor);
    assume $IsValidU64(fractional_part);
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));
    assume is#$Boolean(can_mint);
    assume $Event_EventHandle_is_well_formed(mint_events);
    assume $Event_EventHandle_is_well_formed(burn_events);
    assume $Event_EventHandle_is_well_formed(preburn_events);
    assume $Event_EventHandle_is_well_formed(cancel_burn_events);
    assume $Event_EventHandle_is_well_formed(exchange_rate_update_events);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := total_value][1 := preburn_value][2 := to_lbr_exchange_rate][3 := is_synthetic][4 := scaling_factor][5 := fractional_part][6 := currency_code][7 := can_mint][8 := mint_events][9 := burn_events][10 := preburn_events][11 := cancel_burn_events][12 := exchange_rate_update_events], 13));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Libra_CurrencyInfo_unpack($tv0: $TypeValue, $struct: $Value) returns (total_value: $Value, preburn_value: $Value, to_lbr_exchange_rate: $Value, is_synthetic: $Value, scaling_factor: $Value, fractional_part: $Value, currency_code: $Value, can_mint: $Value, mint_events: $Value, burn_events: $Value, preburn_events: $Value, cancel_burn_events: $Value, exchange_rate_update_events: $Value)
{
    assume is#$Vector($struct);
    total_value := $SelectField($struct, $Libra_CurrencyInfo_total_value);
    assume $IsValidU128(total_value);
    preburn_value := $SelectField($struct, $Libra_CurrencyInfo_preburn_value);
    assume $IsValidU64(preburn_value);
    to_lbr_exchange_rate := $SelectField($struct, $Libra_CurrencyInfo_to_lbr_exchange_rate);
    assume $FixedPoint32_FixedPoint32_is_well_formed(to_lbr_exchange_rate);
    is_synthetic := $SelectField($struct, $Libra_CurrencyInfo_is_synthetic);
    assume is#$Boolean(is_synthetic);
    scaling_factor := $SelectField($struct, $Libra_CurrencyInfo_scaling_factor);
    assume $IsValidU64(scaling_factor);
    fractional_part := $SelectField($struct, $Libra_CurrencyInfo_fractional_part);
    assume $IsValidU64(fractional_part);
    currency_code := $SelectField($struct, $Libra_CurrencyInfo_currency_code);
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));
    can_mint := $SelectField($struct, $Libra_CurrencyInfo_can_mint);
    assume is#$Boolean(can_mint);
    mint_events := $SelectField($struct, $Libra_CurrencyInfo_mint_events);
    assume $Event_EventHandle_is_well_formed(mint_events);
    burn_events := $SelectField($struct, $Libra_CurrencyInfo_burn_events);
    assume $Event_EventHandle_is_well_formed(burn_events);
    preburn_events := $SelectField($struct, $Libra_CurrencyInfo_preburn_events);
    assume $Event_EventHandle_is_well_formed(preburn_events);
    cancel_burn_events := $SelectField($struct, $Libra_CurrencyInfo_cancel_burn_events);
    assume $Event_EventHandle_is_well_formed(cancel_burn_events);
    exchange_rate_update_events := $SelectField($struct, $Libra_CurrencyInfo_exchange_rate_update_events);
    assume $Event_EventHandle_is_well_formed(exchange_rate_update_events);
}

const unique $Libra_CurrencyRegistrationCapability: $TypeName;
const $Libra_CurrencyRegistrationCapability_cap: $FieldName;
axiom $Libra_CurrencyRegistrationCapability_cap == 0;
function $Libra_CurrencyRegistrationCapability_type_value(): $TypeValue {
    $StructType($Libra_CurrencyRegistrationCapability, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $RegisteredCurrencies_RegistrationCapability_type_value()], 1))
}
function {:inline} $Libra_CurrencyRegistrationCapability_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $RegisteredCurrencies_RegistrationCapability_is_well_formed_types($SelectField($this, $Libra_CurrencyRegistrationCapability_cap))
}
function {:inline} $Libra_CurrencyRegistrationCapability_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $RegisteredCurrencies_RegistrationCapability_is_well_formed($SelectField($this, $Libra_CurrencyRegistrationCapability_cap))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Libra_CurrencyRegistrationCapability_is_well_formed($ResourceValue(m, $Libra_CurrencyRegistrationCapability_type_value(), a))
);

procedure {:inline 1} $Libra_CurrencyRegistrationCapability_pack($file_id: int, $byte_index: int, $var_idx: int, cap: $Value) returns ($struct: $Value)
{
    assume $RegisteredCurrencies_RegistrationCapability_is_well_formed(cap);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := cap], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Libra_CurrencyRegistrationCapability_unpack($struct: $Value) returns (cap: $Value)
{
    assume is#$Vector($struct);
    cap := $SelectField($struct, $Libra_CurrencyRegistrationCapability_cap);
    assume $RegisteredCurrencies_RegistrationCapability_is_well_formed(cap);
}

const unique $Libra_MintCapability: $TypeName;
const $Libra_MintCapability_dummy_field: $FieldName;
axiom $Libra_MintCapability_dummy_field == 0;
function $Libra_MintCapability_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Libra_MintCapability, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $Libra_MintCapability_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Libra_MintCapability_dummy_field))
}
function {:inline} $Libra_MintCapability_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Libra_MintCapability_dummy_field))
}

axiom (forall m: $Memory, a: $Value, $tv0: $TypeValue :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Libra_MintCapability_is_well_formed($ResourceValue(m, $Libra_MintCapability_type_value($tv0), a))
);

procedure {:inline 1} $Libra_MintCapability_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Libra_MintCapability_unpack($tv0: $TypeValue, $struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $Libra_MintCapability_dummy_field);
    assume is#$Boolean(dummy_field);
}

const unique $Libra_MintEvent: $TypeName;
const $Libra_MintEvent_amount: $FieldName;
axiom $Libra_MintEvent_amount == 0;
const $Libra_MintEvent_currency_code: $FieldName;
axiom $Libra_MintEvent_currency_code == 1;
function $Libra_MintEvent_type_value(): $TypeValue {
    $StructType($Libra_MintEvent, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $IntegerType()][1 := $Vector_type_value($IntegerType())], 2))
}
function {:inline} $Libra_MintEvent_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && $IsValidU64($SelectField($this, $Libra_MintEvent_amount))
      && $Vector_is_well_formed($SelectField($this, $Libra_MintEvent_currency_code)) && (forall $$0: int :: {$select_vector($SelectField($this, $Libra_MintEvent_currency_code),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Libra_MintEvent_currency_code)) ==> $IsValidU8($select_vector($SelectField($this, $Libra_MintEvent_currency_code),$$0)))
}
function {:inline} $Libra_MintEvent_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && $IsValidU64($SelectField($this, $Libra_MintEvent_amount))
      && $Vector_is_well_formed($SelectField($this, $Libra_MintEvent_currency_code)) && (forall $$0: int :: {$select_vector($SelectField($this, $Libra_MintEvent_currency_code),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Libra_MintEvent_currency_code)) ==> $IsValidU8($select_vector($SelectField($this, $Libra_MintEvent_currency_code),$$0)))
}

procedure {:inline 1} $Libra_MintEvent_pack($file_id: int, $byte_index: int, $var_idx: int, amount: $Value, currency_code: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(amount);
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := amount][1 := currency_code], 2));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Libra_MintEvent_unpack($struct: $Value) returns (amount: $Value, currency_code: $Value)
{
    assume is#$Vector($struct);
    amount := $SelectField($struct, $Libra_MintEvent_amount);
    assume $IsValidU64(amount);
    currency_code := $SelectField($struct, $Libra_MintEvent_currency_code);
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));
}

const unique $Libra_Preburn: $TypeName;
const $Libra_Preburn_requests: $FieldName;
axiom $Libra_Preburn_requests == 0;
function $Libra_Preburn_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Libra_Preburn, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $Vector_type_value($Libra_Libra_type_value($tv0))], 1))
}
function {:inline} $Libra_Preburn_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $Vector_is_well_formed($SelectField($this, $Libra_Preburn_requests)) && (forall $$0: int :: {$select_vector($SelectField($this, $Libra_Preburn_requests),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Libra_Preburn_requests)) ==> $Libra_Libra_is_well_formed_types($select_vector($SelectField($this, $Libra_Preburn_requests),$$0)))
}
function {:inline} $Libra_Preburn_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $Vector_is_well_formed($SelectField($this, $Libra_Preburn_requests)) && (forall $$0: int :: {$select_vector($SelectField($this, $Libra_Preburn_requests),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Libra_Preburn_requests)) ==> $Libra_Libra_is_well_formed($select_vector($SelectField($this, $Libra_Preburn_requests),$$0)))
}

axiom (forall m: $Memory, a: $Value, $tv0: $TypeValue :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Libra_Preburn_is_well_formed($ResourceValue(m, $Libra_Preburn_type_value($tv0), a))
);

procedure {:inline 1} $Libra_Preburn_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, requests: $Value) returns ($struct: $Value)
{
    assume $Vector_is_well_formed(requests) && (forall $$0: int :: {$select_vector(requests,$$0)} $$0 >= 0 && $$0 < $vlen(requests) ==> $Libra_Libra_is_well_formed($select_vector(requests,$$0)));
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := requests], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Libra_Preburn_unpack($tv0: $TypeValue, $struct: $Value) returns (requests: $Value)
{
    assume is#$Vector($struct);
    requests := $SelectField($struct, $Libra_Preburn_requests);
    assume $Vector_is_well_formed(requests) && (forall $$0: int :: {$select_vector(requests,$$0)} $$0 >= 0 && $$0 < $vlen(requests) ==> $Libra_Libra_is_well_formed($select_vector(requests,$$0)));
}

const unique $Libra_PreburnEvent: $TypeName;
const $Libra_PreburnEvent_amount: $FieldName;
axiom $Libra_PreburnEvent_amount == 0;
const $Libra_PreburnEvent_currency_code: $FieldName;
axiom $Libra_PreburnEvent_currency_code == 1;
const $Libra_PreburnEvent_preburn_address: $FieldName;
axiom $Libra_PreburnEvent_preburn_address == 2;
function $Libra_PreburnEvent_type_value(): $TypeValue {
    $StructType($Libra_PreburnEvent, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $IntegerType()][1 := $Vector_type_value($IntegerType())][2 := $AddressType()], 3))
}
function {:inline} $Libra_PreburnEvent_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 3
      && $IsValidU64($SelectField($this, $Libra_PreburnEvent_amount))
      && $Vector_is_well_formed($SelectField($this, $Libra_PreburnEvent_currency_code)) && (forall $$0: int :: {$select_vector($SelectField($this, $Libra_PreburnEvent_currency_code),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Libra_PreburnEvent_currency_code)) ==> $IsValidU8($select_vector($SelectField($this, $Libra_PreburnEvent_currency_code),$$0)))
      && is#$Address($SelectField($this, $Libra_PreburnEvent_preburn_address))
}
function {:inline} $Libra_PreburnEvent_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 3
      && $IsValidU64($SelectField($this, $Libra_PreburnEvent_amount))
      && $Vector_is_well_formed($SelectField($this, $Libra_PreburnEvent_currency_code)) && (forall $$0: int :: {$select_vector($SelectField($this, $Libra_PreburnEvent_currency_code),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Libra_PreburnEvent_currency_code)) ==> $IsValidU8($select_vector($SelectField($this, $Libra_PreburnEvent_currency_code),$$0)))
      && is#$Address($SelectField($this, $Libra_PreburnEvent_preburn_address))
}

procedure {:inline 1} $Libra_PreburnEvent_pack($file_id: int, $byte_index: int, $var_idx: int, amount: $Value, currency_code: $Value, preburn_address: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(amount);
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));
    assume is#$Address(preburn_address);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := amount][1 := currency_code][2 := preburn_address], 3));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Libra_PreburnEvent_unpack($struct: $Value) returns (amount: $Value, currency_code: $Value, preburn_address: $Value)
{
    assume is#$Vector($struct);
    amount := $SelectField($struct, $Libra_PreburnEvent_amount);
    assume $IsValidU64(amount);
    currency_code := $SelectField($struct, $Libra_PreburnEvent_currency_code);
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));
    preburn_address := $SelectField($struct, $Libra_PreburnEvent_preburn_address);
    assume is#$Address(preburn_address);
}

const unique $Libra_RegisterNewCurrency: $TypeName;
const $Libra_RegisterNewCurrency_dummy_field: $FieldName;
axiom $Libra_RegisterNewCurrency_dummy_field == 0;
function $Libra_RegisterNewCurrency_type_value(): $TypeValue {
    $StructType($Libra_RegisterNewCurrency, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $Libra_RegisterNewCurrency_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Libra_RegisterNewCurrency_dummy_field))
}
function {:inline} $Libra_RegisterNewCurrency_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Libra_RegisterNewCurrency_dummy_field))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Libra_RegisterNewCurrency_is_well_formed($ResourceValue(m, $Libra_RegisterNewCurrency_type_value(), a))
);

procedure {:inline 1} $Libra_RegisterNewCurrency_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Libra_RegisterNewCurrency_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $Libra_RegisterNewCurrency_dummy_field);
    assume is#$Boolean(dummy_field);
}

const unique $Libra_ToLBRExchangeRateUpdateEvent: $TypeName;
const $Libra_ToLBRExchangeRateUpdateEvent_currency_code: $FieldName;
axiom $Libra_ToLBRExchangeRateUpdateEvent_currency_code == 0;
const $Libra_ToLBRExchangeRateUpdateEvent_new_to_lbr_exchange_rate: $FieldName;
axiom $Libra_ToLBRExchangeRateUpdateEvent_new_to_lbr_exchange_rate == 1;
function $Libra_ToLBRExchangeRateUpdateEvent_type_value(): $TypeValue {
    $StructType($Libra_ToLBRExchangeRateUpdateEvent, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $Vector_type_value($IntegerType())][1 := $IntegerType()], 2))
}
function {:inline} $Libra_ToLBRExchangeRateUpdateEvent_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && $Vector_is_well_formed($SelectField($this, $Libra_ToLBRExchangeRateUpdateEvent_currency_code)) && (forall $$0: int :: {$select_vector($SelectField($this, $Libra_ToLBRExchangeRateUpdateEvent_currency_code),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Libra_ToLBRExchangeRateUpdateEvent_currency_code)) ==> $IsValidU8($select_vector($SelectField($this, $Libra_ToLBRExchangeRateUpdateEvent_currency_code),$$0)))
      && $IsValidU64($SelectField($this, $Libra_ToLBRExchangeRateUpdateEvent_new_to_lbr_exchange_rate))
}
function {:inline} $Libra_ToLBRExchangeRateUpdateEvent_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && $Vector_is_well_formed($SelectField($this, $Libra_ToLBRExchangeRateUpdateEvent_currency_code)) && (forall $$0: int :: {$select_vector($SelectField($this, $Libra_ToLBRExchangeRateUpdateEvent_currency_code),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Libra_ToLBRExchangeRateUpdateEvent_currency_code)) ==> $IsValidU8($select_vector($SelectField($this, $Libra_ToLBRExchangeRateUpdateEvent_currency_code),$$0)))
      && $IsValidU64($SelectField($this, $Libra_ToLBRExchangeRateUpdateEvent_new_to_lbr_exchange_rate))
}

procedure {:inline 1} $Libra_ToLBRExchangeRateUpdateEvent_pack($file_id: int, $byte_index: int, $var_idx: int, currency_code: $Value, new_to_lbr_exchange_rate: $Value) returns ($struct: $Value)
{
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));
    assume $IsValidU64(new_to_lbr_exchange_rate);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := currency_code][1 := new_to_lbr_exchange_rate], 2));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Libra_ToLBRExchangeRateUpdateEvent_unpack($struct: $Value) returns (currency_code: $Value, new_to_lbr_exchange_rate: $Value)
{
    assume is#$Vector($struct);
    currency_code := $SelectField($struct, $Libra_ToLBRExchangeRateUpdateEvent_currency_code);
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));
    new_to_lbr_exchange_rate := $SelectField($struct, $Libra_ToLBRExchangeRateUpdateEvent_new_to_lbr_exchange_rate);
    assume $IsValidU64(new_to_lbr_exchange_rate);
}



// ** functions of module Libra

procedure {:inline 1} $Libra_initialize (config_account: $Value, create_config_capability: $Value) returns ()
free requires is#$Address(config_account);
free requires $Roles_Capability_is_well_formed(create_config_capability);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)))) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(!b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)))))))));
requires b#$Boolean($Boolean(b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())) && b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)) ==> b#$Boolean($Boolean($IsEqual(addr, $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS()))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)))) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(!b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())) && b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)) ==> b#$Boolean($Boolean($IsEqual(addr, $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())))))))))))));
{
    // declare local variables
    var cap: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $Roles_Capability_type_value($LibraConfig_CreateOnChainConfig_type_value())
    var $t12: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t15: $Value; // $Libra_CurrencyRegistrationCapability_type_value()
    var $t16: $Value; // $Roles_Capability_type_value($LibraConfig_CreateOnChainConfig_type_value())
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $Roles_Capability_type_value($LibraConfig_CreateOnChainConfig_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 10383, 0, config_account); }
    if (true) { assume $DebugTrackLocal(9, 10383, 1, create_config_capability); }

    // bytecode translation starts here
    // $t19 := move(config_account)
    call $tmp := $CopyOrMoveValue(config_account);
    $t19 := $tmp;

    // $t20 := move(create_config_capability)
    call $tmp := $CopyOrMoveValue(create_config_capability);
    $t20 := $tmp;

    // $t5 := copy($t19)
    call $tmp := $CopyOrMoveValue($t19);
    $t5 := $tmp;

    // $t6 := Signer::address_of($t5)
    call $t6 := $Signer_address_of($t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 10585);
      goto Abort;
    }
    assume is#$Address($t6);


    // $t7 := CoreAddresses::DEFAULT_CONFIG_ADDRESS()
    call $t7 := $CoreAddresses_DEFAULT_CONFIG_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 10630);
      goto Abort;
    }
    assume is#$Address($t7);


    // $t8 := ==($t6, $t7)
    $tmp := $Boolean($IsEqual($t6, $t7));
    $t8 := $tmp;

    // $t3 := $t8
    call $tmp := $CopyOrMoveValue($t8);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 10557, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t10 := copy($t19)
    call $tmp := $CopyOrMoveValue($t19);
    $t10 := $tmp;

    // $t11 := move($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t11 := $tmp;

    // $t12 := RegisteredCurrencies::initialize($t10, $t11)
    call $t12 := $RegisteredCurrencies_initialize($t10, $t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 10721);
      goto Abort;
    }
    assume $RegisteredCurrencies_RegistrationCapability_is_well_formed($t12);


    // cap := $t12
    call $tmp := $CopyOrMoveValue($t12);
    cap := $tmp;
    if (true) { assume $DebugTrackLocal(9, 10693, 2, $tmp); }

    // $t13 := move($t19)
    call $tmp := $CopyOrMoveValue($t19);
    $t13 := $tmp;

    // $t15 := pack Libra::CurrencyRegistrationCapability(cap)
    call $tmp := $Libra_CurrencyRegistrationCapability_pack(0, 0, 0, cap);
    $t15 := $tmp;

    // move_to<Libra::CurrencyRegistrationCapability>($t15, $t13)
    call $MoveTo($Libra_CurrencyRegistrationCapability_type_value(), $t15, $t13);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 10783);
      goto Abort;
    }

    // return ()
    return;

    // L2:
L2:

    // $t16 := move($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t16 := $tmp;

    // destroy($t16)

    // $t17 := move($t19)
    call $tmp := $CopyOrMoveValue($t19);
    $t17 := $tmp;

    // destroy($t17)

    // $t18 := 0
    $tmp := $Integer(0);
    $t18 := $tmp;

    // abort($t18)
    if (true) { assume $DebugTrackAbort(9, 10557); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Libra_grant_privileges (account: $Value) returns ()
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $Libra_RegisterNewCurrency_type_value()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 9955, 0, account); }

    // bytecode translation starts here
    // $t4 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := false
    $tmp := $Boolean(false);
    $t2 := $tmp;

    // $t3 := pack Libra::RegisterNewCurrency($t2)
    call $tmp := $Libra_RegisterNewCurrency_pack(0, 0, 0, $t2);
    $t3 := $tmp;

    // Roles::add_privilege_to_account_treasury_compliance_role<Libra::RegisterNewCurrency>($t1, $t3)
    assume b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_ASSOCIATION_ROOT_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_AssociationRootRole_type_value()), addr)))))))));
    assume b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($Roles_spec_has_role_id($m, $txn, addr)) ==> b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($Roles_spec_get_role_id($m, $txn, addr), $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID()))) ==> b#$Boolean($ResourceExists($m, $Roles_Privilege_type_value($Roles_TreasuryComplianceRole_type_value()), addr)))))))));
    call $Roles_add_privilege_to_account_treasury_compliance_role($Libra_RegisterNewCurrency_type_value(), $t1, $t3);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 10018);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Libra_currency_code ($tv0: $TypeValue) returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t2: $Value; // $Vector_type_value($IntegerType())
    var $t3: $Value; // $Vector_type_value($IntegerType())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 31527);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := get_global<Libra::CurrencyInfo<#0>>($t0)
    call $tmp := $GetGlobal($t0, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 31474);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($tmp);
    $t1 := $tmp;

    // $t2 := get_field<Libra::CurrencyInfo<#0>>.currency_code($t1)
    call $tmp := $GetFieldFromValue($t1, $Libra_CurrencyInfo_currency_code);
    assume $Vector_is_well_formed($tmp) && (forall $$0: int :: {$select_vector($tmp,$$0)} $$0 >= 0 && $$0 < $vlen($tmp) ==> $IsValidU8($select_vector($tmp,$$0)));
    $t2 := $tmp;

    // $t3 := move($t2)
    call $tmp := $CopyOrMoveValue($t2);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(9, 31472, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_value ($tv0: $TypeValue, coin: $Value) returns ($ret0: $Value)
free requires $Libra_Libra_is_well_formed(coin);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t1: $Value; // $Libra_Libra_type_value($tv0)
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 24128, 0, coin); }

    // bytecode translation starts here
    // $t4 := move(coin)
    call $tmp := $CopyOrMoveValue(coin);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := get_field<Libra::Libra<#0>>.value($t1)
    call $tmp := $GetFieldFromValue($t1, $Libra_Libra_value);
    assume $IsValidU64($tmp);
    $t2 := $tmp;

    // $t3 := move($t2)
    call $tmp := $CopyOrMoveValue($t2);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(9, 24194, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_approx_lbr_for_coin ($tv0: $TypeValue, coin: $Value) returns ($ret0: $Value)
free requires $Libra_Libra_is_well_formed(coin);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var from_value: $Value; // $IntegerType()
    var $t2: $Value; // $Libra_Libra_type_value($tv0)
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 29817, 0, coin); }

    // bytecode translation starts here
    // $t6 := move(coin)
    call $tmp := $CopyOrMoveValue(coin);
    $t6 := $tmp;

    // $t2 := move($t6)
    call $tmp := $CopyOrMoveValue($t6);
    $t2 := $tmp;

    // $t3 := Libra::value<#0>($t2)
    call $t3 := $Libra_value($tv0, $t2);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 24139);
      goto Abort;
    }
    assume $IsValidU64($t3);


    // from_value := $t3
    call $tmp := $CopyOrMoveValue($t3);
    from_value := $tmp;
    if (true) { assume $DebugTrackLocal(9, 29935, 1, $tmp); }

    // $t5 := Libra::approx_lbr_for_value<#0>(from_value)
    call $t5 := $Libra_approx_lbr_for_value($tv0, from_value);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 29410);
      goto Abort;
    }
    assume $IsValidU64($t5);


    // return $t5
    $ret0 := $t5;
    if (true) { assume $DebugTrackLocal(9, 29969, 7, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_approx_lbr_for_value ($tv0: $TypeValue, from_value: $Value) returns ($ret0: $Value)
free requires $IsValidU64(from_value);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var lbr_exchange_rate: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t2: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 29399, 0, from_value); }

    // bytecode translation starts here
    // $t6 := move(from_value)
    call $tmp := $CopyOrMoveValue(from_value);
    $t6 := $tmp;

    // $t2 := Libra::lbr_exchange_rate<#0>()
    call $t2 := $Libra_lbr_exchange_rate($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 32524);
      goto Abort;
    }
    assume $FixedPoint32_FixedPoint32_is_well_formed($t2);


    // lbr_exchange_rate := $t2
    call $tmp := $CopyOrMoveValue($t2);
    lbr_exchange_rate := $tmp;
    if (true) { assume $DebugTrackLocal(9, 29507, 1, $tmp); }

    // $t5 := FixedPoint32::multiply_u64($t6, lbr_exchange_rate)
    call $t5 := $FixedPoint32_multiply_u64($t6, lbr_exchange_rate);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 29584);
      goto Abort;
    }
    assume $IsValidU64($t5);


    // return $t5
    $ret0 := $t5;
    if (true) { assume $DebugTrackLocal(9, 29570, 7, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_assert_is_coin ($tv0: $TypeValue) returns ()
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t0: $Value; // $BooleanType()
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t2 := Libra::is_currency<#0>()
    call $t2 := $Libra_is_currency($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 30145);
      goto Abort;
    }
    assume is#$Boolean($t2);


    // $t0 := $t2
    call $tmp := $CopyOrMoveValue($t2);
    $t0 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 33842, 0, $tmp); }

    // if ($t0) goto L0 else goto L1
    $tmp := $t0;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // return ()
    return;

    // L2:
L2:

    // $t4 := 1
    $tmp := $Integer(1);
    $t4 := $tmp;

    // abort($t4)
    if (true) { assume $DebugTrackAbort(9, 33842); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Libra_burn ($tv0: $TypeValue, account: $Value, preburn_address: $Value) returns ()
free requires is#$Address(account);
free requires is#$Address(preburn_address);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 12468, 0, account); }
    if (true) { assume $DebugTrackLocal(9, 12468, 1, preburn_address); }

    // bytecode translation starts here
    // $t6 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t6 := $tmp;

    // $t7 := move(preburn_address)
    call $tmp := $CopyOrMoveValue(preburn_address);
    $t7 := $tmp;

    // $t3 := move($t6)
    call $tmp := $CopyOrMoveValue($t6);
    $t3 := $tmp;

    // $t4 := Signer::address_of($t3)
    call $t4 := $Signer_address_of($t3);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 12728);
      goto Abort;
    }
    assume is#$Address($t4);


    // $t5 := get_global<Libra::BurnCapability<#0>>($t4)
    call $tmp := $GetGlobal($t4, $Libra_BurnCapability_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 12680);
      goto Abort;
    }
    assume $Libra_BurnCapability_is_well_formed($tmp);
    $t5 := $tmp;

    // Libra::burn_with_capability<#0>($t7, $t5)
    call $Libra_burn_with_capability($tv0, $t7, $t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 18861);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Libra_burn_with_capability ($tv0: $TypeValue, preburn_address: $Value, capability: $Value) returns ()
free requires is#$Address(preburn_address);
free requires $Libra_BurnCapability_is_well_formed(capability);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t8: $Value; // $Libra_Preburn_type_value($tv0)
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 18850, 0, preburn_address); }
    if (true) { assume $DebugTrackLocal(9, 18850, 1, capability); }

    // bytecode translation starts here
    // $t6 := move(preburn_address)
    call $tmp := $CopyOrMoveValue(preburn_address);
    $t6 := $tmp;

    // $t7 := move(capability)
    call $tmp := $CopyOrMoveValue(capability);
    $t7 := $tmp;

    // $t3 := borrow_global<Libra::Preburn<#0>>($t6)
    call $t3 := $BorrowGlobal($t6, $Libra_Preburn_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 19117);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($Dereference($t3));

    // UnpackRef($t3)

    // $t5 := move($t7)
    call $tmp := $CopyOrMoveValue($t7);
    $t5 := $tmp;

    // PackRef($t3)

    // $t8 := read_ref($t3)
    call $tmp := $ReadRef($t3);
    assume $Libra_Preburn_is_well_formed($tmp);
    $t8 := $tmp;

    // $t8 := Libra::burn_with_resource_cap<#0>($t8, $t6, $t5)
    call $t8 := $Libra_burn_with_resource_cap($tv0, $t8, $t6, $t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 19820);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($t8);


    // write_ref($t3, $t8)
    call $t3 := $WriteRef($t3, $t8);

    // Libra::Preburn <- $t3
    call $WritebackToGlobal($t3);

    // UnpackRef($t3)

    // PackRef($t3)

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Libra_burn_with_resource_cap ($tv0: $TypeValue, preburn: $Value, preburn_address: $Value, _capability: $Value) returns ($ret0: $Value)
free requires $Libra_Preburn_is_well_formed(preburn);
free requires is#$Address(preburn_address);
free requires $Libra_BurnCapability_is_well_formed(_capability);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var currency_code: $Value; // $Vector_type_value($IntegerType())
    var info: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var value: $Value; // $IntegerType()
    var $t6: $Value; // $Vector_type_value($IntegerType())
    var $t7: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t8: $Reference; // ReferenceType($Vector_type_value($Libra_Libra_type_value($tv0)))
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $Libra_Libra_type_value($tv0)
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $AddressType()
    var $t13: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t14: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t21: $Reference; // ReferenceType($IntegerType())
    var $t22: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $IntegerType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $IntegerType()
    var $t27: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t28: $Reference; // ReferenceType($IntegerType())
    var $t29: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t30: $Value; // $BooleanType()
    var $t31: $Value; // $BooleanType()
    var $t32: $Value; // $BooleanType()
    var $t33: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t34: $Reference; // ReferenceType($Event_EventHandle_type_value($Libra_BurnEvent_type_value()))
    var $t35: $Value; // $IntegerType()
    var $t36: $Value; // $Vector_type_value($IntegerType())
    var $t37: $Value; // $AddressType()
    var $t38: $Value; // $Libra_BurnEvent_type_value()
    var $t39: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t40: $Value; // $Libra_Preburn_type_value($tv0)
    var $t41: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t42: $Value; // $AddressType()
    var $t43: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t44: $Value; // $Vector_type_value($Libra_Libra_type_value($tv0))
    var $t45: $Value; // $Event_EventHandle_type_value($Libra_BurnEvent_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 19809, 0, preburn); }
    if (true) { assume $DebugTrackLocal(9, 19809, 1, preburn_address); }
    if (true) { assume $DebugTrackLocal(9, 19809, 2, _capability); }

    // bytecode translation starts here
    // $t40 := move(preburn)
    call $tmp := $CopyOrMoveValue(preburn);
    $t40 := $tmp;

    // $t42 := move(preburn_address)
    call $tmp := $CopyOrMoveValue(preburn_address);
    $t42 := $tmp;

    // $t43 := move(_capability)
    call $tmp := $CopyOrMoveValue(_capability);
    $t43 := $tmp;

    // $t41 := borrow_local($t40)
    call $t41 := $BorrowLoc(40, $t40);
    assume $Libra_Preburn_is_well_formed($Dereference($t41));

    // UnpackRef($t41)

    // $t6 := Libra::currency_code<#0>()
    call $t6 := $Libra_currency_code($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 31398);
      goto Abort;
    }
    assume $Vector_is_well_formed($t6) && (forall $$0: int :: {$select_vector($t6,$$0)} $$0 >= 0 && $$0 < $vlen($t6) ==> $IsValidU8($select_vector($t6,$$0)));


    // currency_code := $t6
    call $tmp := $CopyOrMoveValue($t6);
    currency_code := $tmp;
    if (true) { assume $DebugTrackLocal(9, 20018, 3, $tmp); }

    // $t7 := move($t41)
    call $t7 := $CopyOrMoveRef($t41);

    // $t8 := borrow_field<Libra::Preburn<#0>>.requests($t7)
    call $t8 := $BorrowField($t7, $Libra_Preburn_requests);
    assume $Vector_is_well_formed($Dereference($t8)) && (forall $$1: int :: {$select_vector($Dereference($t8),$$1)} $$1 >= 0 && $$1 < $vlen($Dereference($t8)) ==> $Libra_Libra_is_well_formed_types($select_vector($Dereference($t8),$$1)));

    // LocalRoot($t40) <- $t7
    call $t40 := $WritebackToValue($t7, 40, $t40);

    // UnpackRef($t8)

    // $t9 := 0
    $tmp := $Integer(0);
    $t9 := $tmp;

    // PackRef($t8)

    // $t44 := read_ref($t8)
    call $tmp := $ReadRef($t8);
    assume $Vector_is_well_formed($tmp) && (forall $$0: int :: {$select_vector($tmp,$$0)} $$0 >= 0 && $$0 < $vlen($tmp) ==> $Libra_Libra_is_well_formed($select_vector($tmp,$$0)));
    $t44 := $tmp;

    // ($t10, $t44) := Vector::remove<Libra::Libra<#0>>($t44, $t9)
    call $t10, $t44 := $Vector_remove($Libra_Libra_type_value($tv0), $t44, $t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 20160);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t10);

    assume $Vector_is_well_formed($t44) && (forall $$0: int :: {$select_vector($t44,$$0)} $$0 >= 0 && $$0 < $vlen($t44) ==> $Libra_Libra_is_well_formed($select_vector($t44,$$0)));


    // write_ref($t8, $t44)
    call $t8 := $WriteRef($t8, $t44);
    if (true) { assume $DebugTrackLocal(9, 19809, 4, $Dereference(info)); }

    // LocalRoot($t40) <- $t8
    call $t40 := $WritebackToValue($t8, 40, $t40);

    // Reference($t7) <- $t8
    call $t7 := $WritebackToReference($t8, $t7);

    // UnpackRef($t8)

    // PackRef($t7)

    // PackRef($t8)

    // $t11 := unpack Libra::Libra<#0>($t10)
    call $t11 := $Libra_Libra_unpack($tv0, $t10);
    $t11 := $t11;

    // value := $t11
    call $tmp := $CopyOrMoveValue($t11);
    value := $tmp;
    if (true) { assume $DebugTrackLocal(9, 20142, 5, $tmp); }

    // $t12 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t12 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 20303);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := borrow_global<Libra::CurrencyInfo<#0>>($t12)
    call $t13 := $BorrowGlobal($t12, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 20246);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($Dereference($t13));

    // UnpackRef($t13)
    call $Libra_CurrencyInfo_before_update_inv($tv0, $Dereference($t13));

    // info := $t13
    call info := $CopyOrMoveRef($t13);
    if (true) { assume $DebugTrackLocal(9, 20239, 4, $Dereference(info)); }

    // $t14 := copy(info)
    call $t14 := $CopyOrMoveRef(info);

    // $t15 := get_field<Libra::CurrencyInfo<#0>>.total_value($t14)
    call $tmp := $GetFieldFromReference($t14, $Libra_CurrencyInfo_total_value);
    assume $IsValidU128($tmp);
    $t15 := $tmp;

    // Reference(info) <- $t14
    call info := $WritebackToReference($t14, info);

    // $t16 := move($t15)
    call $tmp := $CopyOrMoveValue($t15);
    $t16 := $tmp;

    // $t18 := (u128)(value)
    call $tmp := $CastU128(value);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 20375);
      goto Abort;
    }
    $t18 := $tmp;

    // $t19 := -($t16, $t18)
    call $tmp := $Sub($t16, $t18);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 20373);
      goto Abort;
    }
    $t19 := $tmp;

    // $t20 := copy(info)
    call $t20 := $CopyOrMoveRef(info);

    // $t21 := borrow_field<Libra::CurrencyInfo<#0>>.total_value($t20)
    call $t21 := $BorrowField($t20, $Libra_CurrencyInfo_total_value);
    assume $IsValidU128($Dereference($t21));

    // Reference(info) <- $t20
    call info := $WritebackToReference($t20, info);

    // UnpackRef($t21)

    // write_ref($t21, $t19)
    call $t21 := $WriteRef($t21, $t19);
    if (true) { assume $DebugTrackLocal(9, 20337, 4, $Dereference(info)); }

    // Reference(info) <- $t21
    call info := $WritebackToReference($t21, info);

    // Reference($t20) <- $t21
    call $t20 := $WritebackToReference($t21, $t20);

    // PackRef($t21)

    // $t22 := copy(info)
    call $t22 := $CopyOrMoveRef(info);

    // $t23 := get_field<Libra::CurrencyInfo<#0>>.preburn_value($t22)
    call $tmp := $GetFieldFromReference($t22, $Libra_CurrencyInfo_preburn_value);
    assume $IsValidU64($tmp);
    $t23 := $tmp;

    // Reference(info) <- $t22
    call info := $WritebackToReference($t22, info);

    // $t24 := move($t23)
    call $tmp := $CopyOrMoveValue($t23);
    $t24 := $tmp;

    // $t26 := -($t24, value)
    call $tmp := $Sub($t24, value);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 20440);
      goto Abort;
    }
    $t26 := $tmp;

    // $t27 := copy(info)
    call $t27 := $CopyOrMoveRef(info);

    // $t28 := borrow_field<Libra::CurrencyInfo<#0>>.preburn_value($t27)
    call $t28 := $BorrowField($t27, $Libra_CurrencyInfo_preburn_value);
    assume $IsValidU64($Dereference($t28));

    // Reference(info) <- $t27
    call info := $WritebackToReference($t27, info);

    // UnpackRef($t28)

    // write_ref($t28, $t26)
    call $t28 := $WriteRef($t28, $t26);
    if (true) { assume $DebugTrackLocal(9, 20400, 4, $Dereference(info)); }

    // Reference(info) <- $t28
    call info := $WritebackToReference($t28, info);

    // Reference($t27) <- $t28
    call $t27 := $WritebackToReference($t28, $t27);

    // PackRef($t28)

    // $t29 := copy(info)
    call $t29 := $CopyOrMoveRef(info);

    // $t30 := get_field<Libra::CurrencyInfo<#0>>.is_synthetic($t29)
    call $tmp := $GetFieldFromReference($t29, $Libra_CurrencyInfo_is_synthetic);
    assume is#$Boolean($tmp);
    $t30 := $tmp;

    // Reference(info) <- $t29
    call info := $WritebackToReference($t29, info);

    // $t31 := move($t30)
    call $tmp := $CopyOrMoveValue($t30);
    $t31 := $tmp;

    // $t32 := !($t31)
    call $tmp := $Not($t31);
    $t32 := $tmp;

    // if ($t32) goto L0 else goto L1
    $tmp := $t32;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t33 := move(info)
    call $t33 := $CopyOrMoveRef(info);

    // $t34 := borrow_field<Libra::CurrencyInfo<#0>>.burn_events($t33)
    call $t34 := $BorrowField($t33, $Libra_CurrencyInfo_burn_events);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t34));

    // Libra::CurrencyInfo <- $t33
    call $WritebackToGlobal($t33);

    // UnpackRef($t34)
    call $Event_EventHandle_before_update_inv($Libra_BurnEvent_type_value(), $Dereference($t34));

    // $t38 := pack Libra::BurnEvent(value, currency_code, $t42)
    call $tmp := $Libra_BurnEvent_pack(0, 0, 0, value, currency_code, $t42);
    $t38 := $tmp;

    // PackRef($t34)
    call $Event_EventHandle_after_update_inv($Libra_BurnEvent_type_value(), $Dereference($t34));

    // $t45 := read_ref($t34)
    call $tmp := $ReadRef($t34);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t45 := $tmp;

    // $t45 := Event::emit_event<Libra::BurnEvent>($t45, $t38)
    call $t45 := $Event_emit_event($Libra_BurnEvent_type_value(), $t45, $t38);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 20561);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t45);


    // write_ref($t34, $t45)
    call $t34 := $WriteRef($t34, $t45);
    if (true) { assume $DebugTrackLocal(9, 19809, 4, $Dereference(info)); }

    // Libra::CurrencyInfo <- $t34
    call $WritebackToGlobal($t34);

    // Reference($t33) <- $t34
    call $t33 := $WritebackToReference($t34, $t33);

    // UnpackRef($t34)
    call $Event_EventHandle_before_update_inv($Libra_BurnEvent_type_value(), $Dereference($t34));

    // PackRef($t33)
    call $Libra_CurrencyInfo_after_update_inv($tv0, $Dereference($t33));

    // PackRef($t34)
    call $Event_EventHandle_after_update_inv($Libra_BurnEvent_type_value(), $Dereference($t34));

    // goto L3
    goto L3;

    // L2:
L2:

    // $t39 := move(info)
    call $t39 := $CopyOrMoveRef(info);

    // destroy($t39)

    // Libra::CurrencyInfo <- $t39
    call $WritebackToGlobal($t39);

    // PackRef($t39)
    call $Libra_CurrencyInfo_after_update_inv($tv0, $Dereference($t39));

    // goto L3
    goto L3;

    // L3:
L3:

    // return $t40
    $ret0 := $t40;
    if (true) { assume $DebugTrackLocal(9, 20789, 46, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_cancel_burn ($tv0: $TypeValue, account: $Value, preburn_address: $Value) returns ($ret0: $Value)
free requires is#$Address(account);
free requires is#$Address(preburn_address);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t6: $Value; // $Libra_Libra_type_value($tv0)
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 13116, 0, account); }
    if (true) { assume $DebugTrackLocal(9, 13116, 1, preburn_address); }

    // bytecode translation starts here
    // $t7 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t7 := $tmp;

    // $t8 := move(preburn_address)
    call $tmp := $CopyOrMoveValue(preburn_address);
    $t8 := $tmp;

    // $t3 := move($t7)
    call $tmp := $CopyOrMoveValue($t7);
    $t3 := $tmp;

    // $t4 := Signer::address_of($t3)
    call $t4 := $Signer_address_of($t3);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 13407);
      goto Abort;
    }
    assume is#$Address($t4);


    // $t5 := get_global<Libra::BurnCapability<#0>>($t4)
    call $tmp := $GetGlobal($t4, $Libra_BurnCapability_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 13359);
      goto Abort;
    }
    assume $Libra_BurnCapability_is_well_formed($tmp);
    $t5 := $tmp;

    // $t6 := Libra::cancel_burn_with_capability<#0>($t8, $t5)
    call $t6 := $Libra_cancel_burn_with_capability($tv0, $t8, $t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 21307);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t6);


    // return $t6
    $ret0 := $t6;
    if (true) { assume $DebugTrackLocal(9, 13289, 9, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_cancel_burn_with_capability ($tv0: $TypeValue, preburn_address: $Value, _capability: $Value) returns ($ret0: $Value)
free requires is#$Address(preburn_address);
free requires $Libra_BurnCapability_is_well_formed(_capability);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var amount: $Value; // $IntegerType()
    var coin: $Value; // $Libra_Libra_type_value($tv0)
    var currency_code: $Value; // $Vector_type_value($IntegerType())
    var info: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var preburn: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t7: $Value; // $AddressType()
    var $t8: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t9: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t10: $Reference; // ReferenceType($Vector_type_value($Libra_Libra_type_value($tv0)))
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $Libra_Libra_type_value($tv0)
    var $t13: $Value; // $Vector_type_value($IntegerType())
    var $t14: $Value; // $AddressType()
    var $t15: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t16: $Value; // $Libra_Libra_type_value($tv0)
    var $t17: $Value; // $IntegerType()
    var $t18: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $IntegerType()
    var $t22: $Value; // $IntegerType()
    var $t23: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t24: $Reference; // ReferenceType($IntegerType())
    var $t25: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t26: $Value; // $BooleanType()
    var $t27: $Value; // $BooleanType()
    var $t28: $Value; // $BooleanType()
    var $t29: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t30: $Reference; // ReferenceType($Event_EventHandle_type_value($Libra_CancelBurnEvent_type_value()))
    var $t31: $Value; // $IntegerType()
    var $t32: $Value; // $Vector_type_value($IntegerType())
    var $t33: $Value; // $AddressType()
    var $t34: $Value; // $Libra_CancelBurnEvent_type_value()
    var $t35: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t36: $Value; // $Libra_Libra_type_value($tv0)
    var $t37: $Value; // $AddressType()
    var $t38: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t39: $Value; // $Vector_type_value($Libra_Libra_type_value($tv0))
    var $t40: $Value; // $Event_EventHandle_type_value($Libra_CancelBurnEvent_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 21296, 0, preburn_address); }
    if (true) { assume $DebugTrackLocal(9, 21296, 1, _capability); }

    // bytecode translation starts here
    // $t37 := move(preburn_address)
    call $tmp := $CopyOrMoveValue(preburn_address);
    $t37 := $tmp;

    // $t38 := move(_capability)
    call $tmp := $CopyOrMoveValue(_capability);
    $t38 := $tmp;

    // $t8 := borrow_global<Libra::Preburn<#0>>($t37)
    call $t8 := $BorrowGlobal($t37, $Libra_Preburn_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 21566);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($Dereference($t8));

    // UnpackRef($t8)

    // preburn := $t8
    call preburn := $CopyOrMoveRef($t8);
    if (true) { assume $DebugTrackLocal(9, 21556, 6, $Dereference(preburn)); }

    // $t9 := move(preburn)
    call $t9 := $CopyOrMoveRef(preburn);

    // $t10 := borrow_field<Libra::Preburn<#0>>.requests($t9)
    call $t10 := $BorrowField($t9, $Libra_Preburn_requests);
    assume $Vector_is_well_formed($Dereference($t10)) && (forall $$1: int :: {$select_vector($Dereference($t10),$$1)} $$1 >= 0 && $$1 < $vlen($Dereference($t10)) ==> $Libra_Libra_is_well_formed_types($select_vector($Dereference($t10),$$1)));

    // Libra::Preburn <- $t9
    call $WritebackToGlobal($t9);

    // UnpackRef($t10)

    // $t11 := 0
    $tmp := $Integer(0);
    $t11 := $tmp;

    // PackRef($t10)

    // $t39 := read_ref($t10)
    call $tmp := $ReadRef($t10);
    assume $Vector_is_well_formed($tmp) && (forall $$0: int :: {$select_vector($tmp,$$0)} $$0 >= 0 && $$0 < $vlen($tmp) ==> $Libra_Libra_is_well_formed($select_vector($tmp,$$0)));
    $t39 := $tmp;

    // ($t12, $t39) := Vector::remove<Libra::Libra<#0>>($t39, $t11)
    call $t12, $t39 := $Vector_remove($Libra_Libra_type_value($tv0), $t39, $t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 21648);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t12);

    assume $Vector_is_well_formed($t39) && (forall $$0: int :: {$select_vector($t39,$$0)} $$0 >= 0 && $$0 < $vlen($t39) ==> $Libra_Libra_is_well_formed($select_vector($t39,$$0)));


    // write_ref($t10, $t39)
    call $t10 := $WriteRef($t10, $t39);
    if (true) { assume $DebugTrackLocal(9, 22408, 5, $Dereference(info)); }
    if (true) { assume $DebugTrackLocal(9, 22408, 6, $Dereference(preburn)); }

    // Libra::Preburn <- $t10
    call $WritebackToGlobal($t10);

    // Reference($t9) <- $t10
    call $t9 := $WritebackToReference($t10, $t9);

    // UnpackRef($t10)

    // PackRef($t9)

    // PackRef($t10)

    // coin := $t12
    call $tmp := $CopyOrMoveValue($t12);
    coin := $tmp;
    if (true) { assume $DebugTrackLocal(9, 21633, 3, $tmp); }

    // $t13 := Libra::currency_code<#0>()
    call $t13 := $Libra_currency_code($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 31398);
      goto Abort;
    }
    assume $Vector_is_well_formed($t13) && (forall $$0: int :: {$select_vector($t13,$$0)} $$0 >= 0 && $$0 < $vlen($t13) ==> $IsValidU8($select_vector($t13,$$0)));


    // currency_code := $t13
    call $tmp := $CopyOrMoveValue($t13);
    currency_code := $tmp;
    if (true) { assume $DebugTrackLocal(9, 21727, 4, $tmp); }

    // $t14 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t14 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 21846);
      goto Abort;
    }
    assume is#$Address($t14);


    // $t15 := borrow_global<Libra::CurrencyInfo<#0>>($t14)
    call $t15 := $BorrowGlobal($t14, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 21789);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($Dereference($t15));

    // UnpackRef($t15)
    call $Libra_CurrencyInfo_before_update_inv($tv0, $Dereference($t15));

    // info := $t15
    call info := $CopyOrMoveRef($t15);
    if (true) { assume $DebugTrackLocal(9, 21782, 5, $Dereference(info)); }

    // $t16 := copy(coin)
    call $tmp := $CopyOrMoveValue(coin);
    $t16 := $tmp;

    // $t17 := Libra::value<#0>($t16)
    call $t17 := $Libra_value($tv0, $t16);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 24139);
      goto Abort;
    }
    assume $IsValidU64($t17);


    // amount := $t17
    call $tmp := $CopyOrMoveValue($t17);
    amount := $tmp;
    if (true) { assume $DebugTrackLocal(9, 21884, 2, $tmp); }

    // $t18 := copy(info)
    call $t18 := $CopyOrMoveRef(info);

    // $t19 := get_field<Libra::CurrencyInfo<#0>>.preburn_value($t18)
    call $tmp := $GetFieldFromReference($t18, $Libra_CurrencyInfo_preburn_value);
    assume $IsValidU64($tmp);
    $t19 := $tmp;

    // Reference(info) <- $t18
    call info := $WritebackToReference($t18, info);

    // $t20 := move($t19)
    call $tmp := $CopyOrMoveValue($t19);
    $t20 := $tmp;

    // $t22 := -($t20, amount)
    call $tmp := $Sub($t20, amount);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 21955);
      goto Abort;
    }
    $t22 := $tmp;

    // $t23 := copy(info)
    call $t23 := $CopyOrMoveRef(info);

    // $t24 := borrow_field<Libra::CurrencyInfo<#0>>.preburn_value($t23)
    call $t24 := $BorrowField($t23, $Libra_CurrencyInfo_preburn_value);
    assume $IsValidU64($Dereference($t24));

    // Reference(info) <- $t23
    call info := $WritebackToReference($t23, info);

    // UnpackRef($t24)

    // write_ref($t24, $t22)
    call $t24 := $WriteRef($t24, $t22);
    if (true) { assume $DebugTrackLocal(9, 21915, 5, $Dereference(info)); }
    if (true) { assume $DebugTrackLocal(9, 21915, 6, $Dereference(preburn)); }

    // Reference(info) <- $t24
    call info := $WritebackToReference($t24, info);

    // Reference($t23) <- $t24
    call $t23 := $WritebackToReference($t24, $t23);

    // PackRef($t24)

    // $t25 := copy(info)
    call $t25 := $CopyOrMoveRef(info);

    // $t26 := get_field<Libra::CurrencyInfo<#0>>.is_synthetic($t25)
    call $tmp := $GetFieldFromReference($t25, $Libra_CurrencyInfo_is_synthetic);
    assume is#$Boolean($tmp);
    $t26 := $tmp;

    // Reference(info) <- $t25
    call info := $WritebackToReference($t25, info);

    // $t27 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t27 := $tmp;

    // $t28 := !($t27)
    call $tmp := $Not($t27);
    $t28 := $tmp;

    // if ($t28) goto L0 else goto L1
    $tmp := $t28;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t29 := move(info)
    call $t29 := $CopyOrMoveRef(info);

    // $t30 := borrow_field<Libra::CurrencyInfo<#0>>.cancel_burn_events($t29)
    call $t30 := $BorrowField($t29, $Libra_CurrencyInfo_cancel_burn_events);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t30));

    // Libra::CurrencyInfo <- $t29
    call $WritebackToGlobal($t29);

    // UnpackRef($t30)
    call $Event_EventHandle_before_update_inv($Libra_CancelBurnEvent_type_value(), $Dereference($t30));

    // $t34 := pack Libra::CancelBurnEvent(amount, currency_code, $t37)
    call $tmp := $Libra_CancelBurnEvent_pack(0, 0, 0, amount, currency_code, $t37);
    $t34 := $tmp;

    // PackRef($t30)
    call $Event_EventHandle_after_update_inv($Libra_CancelBurnEvent_type_value(), $Dereference($t30));

    // $t40 := read_ref($t30)
    call $tmp := $ReadRef($t30);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t40 := $tmp;

    // $t40 := Event::emit_event<Libra::CancelBurnEvent>($t40, $t34)
    call $t40 := $Event_emit_event($Libra_CancelBurnEvent_type_value(), $t40, $t34);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 22163);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t40);


    // write_ref($t30, $t40)
    call $t30 := $WriteRef($t30, $t40);
    if (true) { assume $DebugTrackLocal(9, 21296, 5, $Dereference(info)); }
    if (true) { assume $DebugTrackLocal(9, 21296, 6, $Dereference(preburn)); }

    // Libra::CurrencyInfo <- $t30
    call $WritebackToGlobal($t30);

    // Reference($t29) <- $t30
    call $t29 := $WritebackToReference($t30, $t29);

    // UnpackRef($t30)
    call $Event_EventHandle_before_update_inv($Libra_CancelBurnEvent_type_value(), $Dereference($t30));

    // PackRef($t29)
    call $Libra_CurrencyInfo_after_update_inv($tv0, $Dereference($t29));

    // PackRef($t30)
    call $Event_EventHandle_after_update_inv($Libra_CancelBurnEvent_type_value(), $Dereference($t30));

    // goto L3
    goto L3;

    // L2:
L2:

    // $t35 := move(info)
    call $t35 := $CopyOrMoveRef(info);

    // destroy($t35)

    // Libra::CurrencyInfo <- $t35
    call $WritebackToGlobal($t35);

    // PackRef($t35)
    call $Libra_CurrencyInfo_after_update_inv($tv0, $Dereference($t35));

    // goto L3
    goto L3;

    // L3:
L3:

    // return coin
    $ret0 := coin;
    if (true) { assume $DebugTrackLocal(9, 22408, 41, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_create_preburn ($tv0: $TypeValue, _: $Value) returns ($ret0: $Value)
free requires $Roles_Capability_is_well_formed(_);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $Vector_type_value($Libra_Libra_type_value($tv0))
    var $t6: $Value; // $Libra_Preburn_type_value($tv0)
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $Roles_Capability_type_value($Roles_TreasuryComplianceRole_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 16839, 0, _); }

    // bytecode translation starts here
    // $t8 := move(_)
    call $tmp := $CopyOrMoveValue(_);
    $t8 := $tmp;

    // $t3 := Libra::is_currency<#0>()
    call $t3 := $Libra_is_currency($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 30145);
      goto Abort;
    }
    assume is#$Boolean($t3);


    // $t1 := $t3
    call $tmp := $CopyOrMoveValue($t3);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 16958, 1, $tmp); }

    // if ($t1) goto L0 else goto L1
    $tmp := $t1;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t5 := Vector::empty<Libra::Libra<#0>>()
    call $t5 := $Vector_empty($Libra_Libra_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 17042);
      goto Abort;
    }
    assume $Vector_is_well_formed($t5) && (forall $$0: int :: {$select_vector($t5,$$0)} $$0 >= 0 && $$0 < $vlen($t5) ==> $Libra_Libra_is_well_formed($select_vector($t5,$$0)));


    // $t6 := pack Libra::Preburn<#0>($t5)
    call $tmp := $Libra_Preburn_pack(0, 0, 0, $tv0, $t5);
    $t6 := $tmp;

    // return $t6
    $ret0 := $t6;
    if (true) { assume $DebugTrackLocal(9, 17004, 9, $ret0); }
    return;

    // L2:
L2:

    // $t7 := 201
    $tmp := $Integer(201);
    $t7 := $tmp;

    // abort($t7)
    if (true) { assume $DebugTrackAbort(9, 16958); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_deposit ($tv0: $TypeValue, coin: $Value, check: $Value) returns ($ret0: $Value)
free requires $Libra_Libra_is_well_formed(coin);
free requires $Libra_Libra_is_well_formed(check);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var value: $Value; // $IntegerType()
    var $t3: $Value; // $Libra_Libra_type_value($tv0)
    var $t4: $Value; // $IntegerType()
    var $t5: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t11: $Reference; // ReferenceType($IntegerType())
    var $t12: $Value; // $Libra_Libra_type_value($tv0)
    var $t13: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t14: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 25771, 0, coin); }
    if (true) { assume $DebugTrackLocal(9, 25771, 1, check); }

    // bytecode translation starts here
    // $t12 := move(coin)
    call $tmp := $CopyOrMoveValue(coin);
    $t12 := $tmp;

    // $t14 := move(check)
    call $tmp := $CopyOrMoveValue(check);
    $t14 := $tmp;

    // $t13 := borrow_local($t12)
    call $t13 := $BorrowLoc(12, $t12);
    assume $Libra_Libra_is_well_formed($Dereference($t13));

    // UnpackRef($t13)
    call $Libra_Libra_before_update_inv($tv0, $Dereference($t13));

    // $t4 := unpack Libra::Libra<#0>($t14)
    call $t4 := $Libra_Libra_unpack($tv0, $t14);
    $t4 := $t4;

    // value := $t4
    call $tmp := $CopyOrMoveValue($t4);
    value := $tmp;
    if (true) { assume $DebugTrackLocal(9, 25874, 2, $tmp); }

    // $t5 := copy($t13)
    call $t5 := $CopyOrMoveRef($t13);

    // $t6 := get_field<Libra::Libra<#0>>.value($t5)
    call $tmp := $GetFieldFromReference($t5, $Libra_Libra_value);
    assume $IsValidU64($tmp);
    $t6 := $tmp;

    // Reference($t13) <- $t5
    call $t13 := $WritebackToReference($t5, $t13);

    // $t7 := move($t6)
    call $tmp := $CopyOrMoveValue($t6);
    $t7 := $tmp;

    // $t9 := +($t7, value)
    call $tmp := $AddU64($t7, value);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 25923);
      goto Abort;
    }
    $t9 := $tmp;

    // $t10 := move($t13)
    call $t10 := $CopyOrMoveRef($t13);

    // $t11 := borrow_field<Libra::Libra<#0>>.value($t10)
    call $t11 := $BorrowField($t10, $Libra_Libra_value);
    assume $IsValidU64($Dereference($t11));

    // LocalRoot($t12) <- $t10
    call $t12 := $WritebackToValue($t10, 12, $t12);

    // UnpackRef($t11)

    // write_ref($t11, $t9)
    call $t11 := $WriteRef($t11, $t9);

    // LocalRoot($t12) <- $t11
    call $t12 := $WritebackToValue($t11, 12, $t12);

    // Reference($t10) <- $t11
    call $t10 := $WritebackToReference($t11, $t10);

    // PackRef($t10)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t10));

    // PackRef($t11)

    // return $t12
    $ret0 := $t12;
    if (true) { assume $DebugTrackLocal(9, 25930, 15, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_destroy_zero ($tv0: $TypeValue, coin: $Value) returns ()
free requires $Libra_Libra_is_well_formed(coin);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $IntegerType()
    var value: $Value; // $IntegerType()
    var $t4: $Value; // $Libra_Libra_type_value($tv0)
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 26254, 0, coin); }

    // bytecode translation starts here
    // $t11 := move(coin)
    call $tmp := $CopyOrMoveValue(coin);
    $t11 := $tmp;

    // $t5 := unpack Libra::Libra<#0>($t11)
    call $t5 := $Libra_Libra_unpack($tv0, $t11);
    $t5 := $t5;

    // value := $t5
    call $tmp := $CopyOrMoveValue($t5);
    value := $tmp;
    if (true) { assume $DebugTrackLocal(9, 26333, 3, $tmp); }

    // $t7 := 0
    $tmp := $Integer(0);
    $t7 := $tmp;

    // $t8 := ==(value, $t7)
    $tmp := $Boolean($IsEqual(value, $t7));
    $t8 := $tmp;

    // $t1 := $t8
    call $tmp := $CopyOrMoveValue($t8);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 26357, 1, $tmp); }

    // if ($t1) goto L0 else goto L1
    $tmp := $t1;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // return ()
    return;

    // L2:
L2:

    // $t10 := 5
    $tmp := $Integer(5);
    $t10 := $tmp;

    // abort($t10)
    if (true) { assume $DebugTrackAbort(9, 26357); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Libra_fractional_part ($tv0: $TypeValue) returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 31222);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := get_global<Libra::CurrencyInfo<#0>>($t0)
    call $tmp := $GetGlobal($t0, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 31169);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($tmp);
    $t1 := $tmp;

    // $t2 := get_field<Libra::CurrencyInfo<#0>>.fractional_part($t1)
    call $tmp := $GetFieldFromValue($t1, $Libra_CurrencyInfo_fractional_part);
    assume $IsValidU64($tmp);
    $t2 := $tmp;

    // $t3 := move($t2)
    call $tmp := $CopyOrMoveValue($t2);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(9, 31169, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_is_currency ($tv0: $TypeValue) returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $BooleanType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 30231);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := exists<Libra::CurrencyInfo<#0>>($t0)
    call $tmp := $Exists($t0, $Libra_CurrencyInfo_type_value($tv0));
    $t1 := $tmp;

    // return $t1
    $ret0 := $t1;
    if (true) { assume $DebugTrackLocal(9, 30185, 2, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_is_synthetic_currency ($tv0: $TypeValue) returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var addr: $Value; // $AddressType()
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $BooleanType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t2 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t2 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 30510);
      goto Abort;
    }
    assume is#$Address($t2);


    // addr := $t2
    call $tmp := $CopyOrMoveValue($t2);
    addr := $tmp;
    if (true) { assume $DebugTrackLocal(9, 30488, 0, $tmp); }

    // $t4 := exists<Libra::CurrencyInfo<#0>>(addr)
    call $tmp := $Exists(addr, $Libra_CurrencyInfo_type_value($tv0));
    $t4 := $tmp;

    // if ($t4) goto L0 else goto L1
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t6 := get_global<Libra::CurrencyInfo<#0>>(addr)
    call $tmp := $GetGlobal(addr, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 30595);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($tmp);
    $t6 := $tmp;

    // $t7 := get_field<Libra::CurrencyInfo<#0>>.is_synthetic($t6)
    call $tmp := $GetFieldFromValue($t6, $Libra_CurrencyInfo_is_synthetic);
    assume is#$Boolean($tmp);
    $t7 := $tmp;

    // $t8 := move($t7)
    call $tmp := $CopyOrMoveValue($t7);
    $t8 := $tmp;

    // $t1 := $t8
    call $tmp := $CopyOrMoveValue($t8);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 30543, 1, $tmp); }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t9 := false
    $tmp := $Boolean(false);
    $t9 := $tmp;

    // $t1 := $t9
    call $tmp := $CopyOrMoveValue($t9);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 30543, 1, $tmp); }

    // goto L3
    goto L3;

    // L3:
L3:

    // return $t1
    $ret0 := $t1;
    if (true) { assume $DebugTrackLocal(9, 30543, 11, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_join ($tv0: $TypeValue, coin1: $Value, coin2: $Value) returns ($ret0: $Value)
free requires $Libra_Libra_is_well_formed(coin1);
free requires $Libra_Libra_is_well_formed(coin2);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t2: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t3: $Value; // $Libra_Libra_type_value($tv0)
    var $t4: $Value; // $Libra_Libra_type_value($tv0)
    var $t5: $Value; // $Libra_Libra_type_value($tv0)
    var $t6: $Value; // $Libra_Libra_type_value($tv0)
    var $t7: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 25440, 0, coin1); }
    if (true) { assume $DebugTrackLocal(9, 25440, 1, coin2); }

    // bytecode translation starts here
    // $t5 := move(coin1)
    call $tmp := $CopyOrMoveValue(coin1);
    $t5 := $tmp;

    // $t6 := move(coin2)
    call $tmp := $CopyOrMoveValue(coin2);
    $t6 := $tmp;

    // $t2 := borrow_local($t5)
    call $t2 := $BorrowLoc(5, $t5);
    assume $Libra_Libra_is_well_formed($Dereference($t2));

    // UnpackRef($t2)
    call $Libra_Libra_before_update_inv($tv0, $Dereference($t2));

    // PackRef($t2)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t2));

    // $t7 := read_ref($t2)
    call $tmp := $ReadRef($t2);
    assume $Libra_Libra_is_well_formed($tmp);
    $t7 := $tmp;

    // $t7 := Libra::deposit<#0>($t7, $t6)
    call $t7 := $Libra_deposit($tv0, $t7, $t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 25782);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t7);


    // write_ref($t2, $t7)
    call $t2 := $WriteRef($t2, $t7);

    // LocalRoot($t5) <- $t2
    call $t5 := $WritebackToValue($t2, 5, $t5);

    // UnpackRef($t2)
    call $Libra_Libra_before_update_inv($tv0, $Dereference($t2));

    // PackRef($t2)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t2));

    // return $t5
    $ret0 := $t5;
    if (true) { assume $DebugTrackLocal(9, 25578, 8, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_lbr_exchange_rate ($tv0: $TypeValue) returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t2: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t3: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 32659);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := get_global<Libra::CurrencyInfo<#0>>($t0)
    call $tmp := $GetGlobal($t0, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 32606);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($tmp);
    $t1 := $tmp;

    // $t2 := get_field<Libra::CurrencyInfo<#0>>.to_lbr_exchange_rate($t1)
    call $tmp := $GetFieldFromValue($t1, $Libra_CurrencyInfo_to_lbr_exchange_rate);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t2 := $tmp;

    // $t3 := move($t2)
    call $tmp := $CopyOrMoveValue($t2);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(9, 32604, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_market_cap ($tv0: $TypeValue) returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 29171);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := get_global<Libra::CurrencyInfo<#0>>($t0)
    call $tmp := $GetGlobal($t0, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 29118);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($tmp);
    $t1 := $tmp;

    // $t2 := get_field<Libra::CurrencyInfo<#0>>.total_value($t1)
    call $tmp := $GetFieldFromValue($t1, $Libra_CurrencyInfo_total_value);
    assume $IsValidU128($tmp);
    $t2 := $tmp;

    // $t3 := move($t2)
    call $tmp := $CopyOrMoveValue($t2);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(9, 29118, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_mint ($tv0: $TypeValue, account: $Value, amount: $Value) returns ($ret0: $Value)
free requires is#$Address(account);
free requires $IsValidU64(amount);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $Libra_MintCapability_type_value($tv0)
    var $t6: $Value; // $Libra_Libra_type_value($tv0)
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 11954, 0, account); }
    if (true) { assume $DebugTrackLocal(9, 11954, 1, amount); }

    // bytecode translation starts here
    // $t7 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t7 := $tmp;

    // $t8 := move(amount)
    call $tmp := $CopyOrMoveValue(amount);
    $t8 := $tmp;

    // $t3 := move($t7)
    call $tmp := $CopyOrMoveValue($t7);
    $t3 := $tmp;

    // $t4 := Signer::address_of($t3)
    call $t4 := $Signer_address_of($t3);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 12182);
      goto Abort;
    }
    assume is#$Address($t4);


    // $t5 := get_global<Libra::MintCapability<#0>>($t4)
    call $tmp := $GetGlobal($t4, $Libra_MintCapability_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 12134);
      goto Abort;
    }
    assume $Libra_MintCapability_is_well_formed($tmp);
    $t5 := $tmp;

    // $t6 := Libra::mint_with_capability<#0>($t8, $t5)
    call $t6 := $Libra_mint_with_capability($tv0, $t8, $t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 14001);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t6);


    // return $t6
    $ret0 := $t6;
    if (true) { assume $DebugTrackLocal(9, 12080, 9, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_mint_with_capability ($tv0: $TypeValue, value: $Value, _capability: $Value) returns ($ret0: $Value)
free requires $IsValidU64(value);
free requires $Libra_MintCapability_is_well_formed(_capability);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var currency_code: $Value; // $Vector_type_value($IntegerType())
    var info: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $BooleanType()
    var $t11: $Value; // $BooleanType()
    var $t12: $Value; // $Vector_type_value($IntegerType())
    var $t13: $Value; // $AddressType()
    var $t14: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t15: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t16: $Value; // $BooleanType()
    var $t17: $Value; // $BooleanType()
    var $t18: $Value; // $BooleanType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t21: $Value; // $IntegerType()
    var $t22: $Value; // $IntegerType()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $IntegerType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t27: $Reference; // ReferenceType($IntegerType())
    var $t28: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t29: $Value; // $BooleanType()
    var $t30: $Value; // $BooleanType()
    var $t31: $Value; // $BooleanType()
    var $t32: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t33: $Value; // $IntegerType()
    var $t34: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t35: $Reference; // ReferenceType($Event_EventHandle_type_value($Libra_MintEvent_type_value()))
    var $t36: $Value; // $IntegerType()
    var $t37: $Value; // $Vector_type_value($IntegerType())
    var $t38: $Value; // $Libra_MintEvent_type_value()
    var $t39: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t40: $Value; // $IntegerType()
    var $t41: $Value; // $Libra_Libra_type_value($tv0)
    var $t42: $Value; // $IntegerType()
    var $t43: $Value; // $Libra_MintCapability_type_value($tv0)
    var $t44: $Value; // $Event_EventHandle_type_value($Libra_MintEvent_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 13990, 0, value); }
    if (true) { assume $DebugTrackLocal(9, 13990, 1, _capability); }

    // bytecode translation starts here
    // $t42 := move(value)
    call $tmp := $CopyOrMoveValue(value);
    $t42 := $tmp;

    // $t43 := move(_capability)
    call $tmp := $CopyOrMoveValue(_capability);
    $t43 := $tmp;

    // Libra::assert_is_coin<#0>()
    call $Libra_assert_is_coin($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 33805);
      goto Abort;
    }

    // $t9 := 1000000000000000
    $tmp := $Integer(1000000000000000);
    $t9 := $tmp;

    // $t10 := <=($t42, $t9)
    call $tmp := $Le($t42, $t9);
    $t10 := $tmp;

    // $t4 := $t10
    call $tmp := $CopyOrMoveValue($t10);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 14625, 4, $tmp); }

    // if ($t4) goto L0 else goto L1
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t12 := Libra::currency_code<#0>()
    call $t12 := $Libra_currency_code($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 31398);
      goto Abort;
    }
    assume $Vector_is_well_formed($t12) && (forall $$0: int :: {$select_vector($t12,$$0)} $$0 >= 0 && $$0 < $vlen($t12) ==> $IsValidU8($select_vector($t12,$$0)));


    // currency_code := $t12
    call $tmp := $CopyOrMoveValue($t12);
    currency_code := $tmp;
    if (true) { assume $DebugTrackLocal(9, 14680, 2, $tmp); }

    // $t13 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t13 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 14856);
      goto Abort;
    }
    assume is#$Address($t13);


    // $t14 := borrow_global<Libra::CurrencyInfo<#0>>($t13)
    call $t14 := $BorrowGlobal($t13, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 14799);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($Dereference($t14));

    // UnpackRef($t14)
    call $Libra_CurrencyInfo_before_update_inv($tv0, $Dereference($t14));

    // info := $t14
    call info := $CopyOrMoveRef($t14);
    if (true) { assume $DebugTrackLocal(9, 14792, 3, $Dereference(info)); }

    // $t15 := copy(info)
    call $t15 := $CopyOrMoveRef(info);

    // $t16 := get_field<Libra::CurrencyInfo<#0>>.can_mint($t15)
    call $tmp := $GetFieldFromReference($t15, $Libra_CurrencyInfo_can_mint);
    assume is#$Boolean($tmp);
    $t16 := $tmp;

    // Reference(info) <- $t15
    call info := $WritebackToReference($t15, info);

    // $t17 := move($t16)
    call $tmp := $CopyOrMoveValue($t16);
    $t17 := $tmp;

    // $t6 := $t17
    call $tmp := $CopyOrMoveValue($t17);
    $t6 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 14890, 6, $tmp); }

    // if ($t6) goto L3 else goto L4
    $tmp := $t6;
    if (b#$Boolean($tmp)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L5
    goto L5;

    // L2:
L2:

    // $t19 := 11
    $tmp := $Integer(11);
    $t19 := $tmp;

    // abort($t19)
    if (true) { assume $DebugTrackAbort(9, 14625); }
    goto Abort;

    // L3:
L3:

    // $t20 := copy(info)
    call $t20 := $CopyOrMoveRef(info);

    // $t21 := get_field<Libra::CurrencyInfo<#0>>.total_value($t20)
    call $tmp := $GetFieldFromReference($t20, $Libra_CurrencyInfo_total_value);
    assume $IsValidU128($tmp);
    $t21 := $tmp;

    // Reference(info) <- $t20
    call info := $WritebackToReference($t20, info);

    // $t22 := move($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t22 := $tmp;

    // $t24 := (u128)($t42)
    call $tmp := $CastU128($t42);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 14962);
      goto Abort;
    }
    $t24 := $tmp;

    // $t25 := +($t22, $t24)
    call $tmp := $AddU128($t22, $t24);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 14960);
      goto Abort;
    }
    $t25 := $tmp;

    // $t26 := copy(info)
    call $t26 := $CopyOrMoveRef(info);

    // $t27 := borrow_field<Libra::CurrencyInfo<#0>>.total_value($t26)
    call $t27 := $BorrowField($t26, $Libra_CurrencyInfo_total_value);
    assume $IsValidU128($Dereference($t27));

    // Reference(info) <- $t26
    call info := $WritebackToReference($t26, info);

    // UnpackRef($t27)

    // write_ref($t27, $t25)
    call $t27 := $WriteRef($t27, $t25);
    if (true) { assume $DebugTrackLocal(9, 14924, 3, $Dereference(info)); }

    // Reference(info) <- $t27
    call info := $WritebackToReference($t27, info);

    // Reference($t26) <- $t27
    call $t26 := $WritebackToReference($t27, $t26);

    // PackRef($t27)

    // $t28 := copy(info)
    call $t28 := $CopyOrMoveRef(info);

    // $t29 := get_field<Libra::CurrencyInfo<#0>>.is_synthetic($t28)
    call $tmp := $GetFieldFromReference($t28, $Libra_CurrencyInfo_is_synthetic);
    assume is#$Boolean($tmp);
    $t29 := $tmp;

    // Reference(info) <- $t28
    call info := $WritebackToReference($t28, info);

    // $t30 := move($t29)
    call $tmp := $CopyOrMoveValue($t29);
    $t30 := $tmp;

    // $t31 := !($t30)
    call $tmp := $Not($t30);
    $t31 := $tmp;

    // if ($t31) goto L6 else goto L7
    $tmp := $t31;
    if (b#$Boolean($tmp)) { goto L6; } else { goto L7; }

    // L7:
L7:

    // goto L8
    goto L8;

    // L5:
L5:

    // $t32 := move(info)
    call $t32 := $CopyOrMoveRef(info);

    // destroy($t32)

    // Libra::CurrencyInfo <- $t32
    call $WritebackToGlobal($t32);

    // PackRef($t32)
    call $Libra_CurrencyInfo_after_update_inv($tv0, $Dereference($t32));

    // $t33 := 4
    $tmp := $Integer(4);
    $t33 := $tmp;

    // abort($t33)
    if (true) { assume $DebugTrackAbort(9, 14890); }
    goto Abort;

    // L6:
L6:

    // $t34 := move(info)
    call $t34 := $CopyOrMoveRef(info);

    // $t35 := borrow_field<Libra::CurrencyInfo<#0>>.mint_events($t34)
    call $t35 := $BorrowField($t34, $Libra_CurrencyInfo_mint_events);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t35));

    // Libra::CurrencyInfo <- $t34
    call $WritebackToGlobal($t34);

    // UnpackRef($t35)
    call $Event_EventHandle_before_update_inv($Libra_MintEvent_type_value(), $Dereference($t35));

    // $t38 := pack Libra::MintEvent($t42, currency_code)
    call $tmp := $Libra_MintEvent_pack(0, 0, 0, $t42, currency_code);
    $t38 := $tmp;

    // PackRef($t35)
    call $Event_EventHandle_after_update_inv($Libra_MintEvent_type_value(), $Dereference($t35));

    // $t44 := read_ref($t35)
    call $tmp := $ReadRef($t35);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t44 := $tmp;

    // $t44 := Event::emit_event<Libra::MintEvent>($t44, $t38)
    call $t44 := $Event_emit_event($Libra_MintEvent_type_value(), $t44, $t38);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 15091);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t44);


    // write_ref($t35, $t44)
    call $t35 := $WriteRef($t35, $t44);
    if (true) { assume $DebugTrackLocal(9, 15310, 3, $Dereference(info)); }

    // Libra::CurrencyInfo <- $t35
    call $WritebackToGlobal($t35);

    // Reference($t34) <- $t35
    call $t34 := $WritebackToReference($t35, $t34);

    // UnpackRef($t35)
    call $Event_EventHandle_before_update_inv($Libra_MintEvent_type_value(), $Dereference($t35));

    // PackRef($t34)
    call $Libra_CurrencyInfo_after_update_inv($tv0, $Dereference($t34));

    // PackRef($t35)
    call $Event_EventHandle_after_update_inv($Libra_MintEvent_type_value(), $Dereference($t35));

    // goto L9
    goto L9;

    // L8:
L8:

    // $t39 := move(info)
    call $t39 := $CopyOrMoveRef(info);

    // destroy($t39)

    // Libra::CurrencyInfo <- $t39
    call $WritebackToGlobal($t39);

    // PackRef($t39)
    call $Libra_CurrencyInfo_after_update_inv($tv0, $Dereference($t39));

    // goto L9
    goto L9;

    // L9:
L9:

    // $t41 := pack Libra::Libra<#0>($t42)
    call $tmp := $Libra_Libra_pack(0, 0, 0, $tv0, $t42);
    $t41 := $tmp;

    // return $t41
    $ret0 := $t41;
    if (true) { assume $DebugTrackLocal(9, 15292, 45, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_new_preburn ($tv0: $TypeValue) returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t0: $Value; // $Vector_type_value($Libra_Libra_type_value($tv0))
    var $t1: $Value; // $Libra_Preburn_type_value($tv0)
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // Libra::assert_is_coin<#0>()
    call $Libra_assert_is_coin($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 33805);
      goto Abort;
    }

    // $t0 := Vector::empty<Libra::Libra<#0>>()
    call $t0 := $Vector_empty($Libra_Libra_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 13725);
      goto Abort;
    }
    assume $Vector_is_well_formed($t0) && (forall $$0: int :: {$select_vector($t0,$$0)} $$0 >= 0 && $$0 < $vlen($t0) ==> $Libra_Libra_is_well_formed($select_vector($t0,$$0)));


    // $t1 := pack Libra::Preburn<#0>($t0)
    call $tmp := $Libra_Preburn_pack(0, 0, 0, $tv0, $t0);
    $t1 := $tmp;

    // return $t1
    $ret0 := $t1;
    if (true) { assume $DebugTrackLocal(9, 13687, 2, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_preburn_to ($tv0: $TypeValue, account: $Value, coin: $Value) returns ()
free requires is#$Address(account);
free requires $Libra_Libra_is_well_formed(coin);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var sender: $Value; // $AddressType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $Libra_Libra_type_value($tv0)
    var $t6: $Value; // $AddressType()
    var $t7: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $Libra_Libra_type_value($tv0)
    var $t11: $Value; // $Libra_Preburn_type_value($tv0)
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 18029, 0, account); }
    if (true) { assume $DebugTrackLocal(9, 18029, 1, coin); }

    // bytecode translation starts here
    // $t9 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t9 := $tmp;

    // $t10 := move(coin)
    call $tmp := $CopyOrMoveValue(coin);
    $t10 := $tmp;

    // $t3 := move($t9)
    call $tmp := $CopyOrMoveValue($t9);
    $t3 := $tmp;

    // $t4 := Signer::address_of($t3)
    call $t4 := $Signer_address_of($t3);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 18173);
      goto Abort;
    }
    assume is#$Address($t4);


    // sender := $t4
    call $tmp := $CopyOrMoveValue($t4);
    sender := $tmp;
    if (true) { assume $DebugTrackLocal(9, 18156, 2, $tmp); }

    // $t7 := borrow_global<Libra::Preburn<#0>>(sender)
    call $t7 := $BorrowGlobal(sender, $Libra_Preburn_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 18230);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($Dereference($t7));

    // UnpackRef($t7)

    // PackRef($t7)

    // $t11 := read_ref($t7)
    call $tmp := $ReadRef($t7);
    assume $Libra_Preburn_is_well_formed($tmp);
    $t11 := $tmp;

    // $t11 := Libra::preburn_with_resource<#0>($t10, $t11, sender)
    call $t11 := $Libra_preburn_with_resource($tv0, $t10, $t11, sender);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 15697);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($t11);


    // write_ref($t7, $t11)
    call $t7 := $WriteRef($t7, $t11);

    // Libra::Preburn <- $t7
    call $WritebackToGlobal($t7);

    // UnpackRef($t7)

    // PackRef($t7)

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Libra_preburn_value ($tv0: $TypeValue) returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 23628);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := get_global<Libra::CurrencyInfo<#0>>($t0)
    call $tmp := $GetGlobal($t0, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 23575);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($tmp);
    $t1 := $tmp;

    // $t2 := get_field<Libra::CurrencyInfo<#0>>.preburn_value($t1)
    call $tmp := $GetFieldFromValue($t1, $Libra_CurrencyInfo_preburn_value);
    assume $IsValidU64($tmp);
    $t2 := $tmp;

    // $t3 := move($t2)
    call $tmp := $CopyOrMoveValue($t2);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(9, 23575, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_preburn_with_resource ($tv0: $TypeValue, coin: $Value, preburn: $Value, preburn_address: $Value) returns ($ret0: $Value)
free requires $Libra_Libra_is_well_formed(coin);
free requires $Libra_Preburn_is_well_formed(preburn);
free requires is#$Address(preburn_address);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var coin_value: $Value; // $IntegerType()
    var currency_code: $Value; // $Vector_type_value($IntegerType())
    var info: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t6: $Value; // $Libra_Libra_type_value($tv0)
    var $t7: $Value; // $IntegerType()
    var $t8: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t9: $Reference; // ReferenceType($Vector_type_value($Libra_Libra_type_value($tv0)))
    var $t10: $Value; // $Libra_Libra_type_value($tv0)
    var $t11: $Value; // $Vector_type_value($IntegerType())
    var $t12: $Value; // $AddressType()
    var $t13: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t14: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t20: $Reference; // ReferenceType($IntegerType())
    var $t21: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t22: $Value; // $BooleanType()
    var $t23: $Value; // $BooleanType()
    var $t24: $Value; // $BooleanType()
    var $t25: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t26: $Reference; // ReferenceType($Event_EventHandle_type_value($Libra_PreburnEvent_type_value()))
    var $t27: $Value; // $IntegerType()
    var $t28: $Value; // $Vector_type_value($IntegerType())
    var $t29: $Value; // $AddressType()
    var $t30: $Value; // $Libra_PreburnEvent_type_value()
    var $t31: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t32: $Value; // $Libra_Libra_type_value($tv0)
    var $t33: $Value; // $Libra_Preburn_type_value($tv0)
    var $t34: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t35: $Value; // $AddressType()
    var $t36: $Value; // $Vector_type_value($Libra_Libra_type_value($tv0))
    var $t37: $Value; // $Event_EventHandle_type_value($Libra_PreburnEvent_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 15686, 0, coin); }
    if (true) { assume $DebugTrackLocal(9, 15686, 1, preburn); }
    if (true) { assume $DebugTrackLocal(9, 15686, 2, preburn_address); }

    // bytecode translation starts here
    // $t32 := move(coin)
    call $tmp := $CopyOrMoveValue(coin);
    $t32 := $tmp;

    // $t33 := move(preburn)
    call $tmp := $CopyOrMoveValue(preburn);
    $t33 := $tmp;

    // $t35 := move(preburn_address)
    call $tmp := $CopyOrMoveValue(preburn_address);
    $t35 := $tmp;

    // $t34 := borrow_local($t33)
    call $t34 := $BorrowLoc(33, $t33);
    assume $Libra_Preburn_is_well_formed($Dereference($t34));

    // UnpackRef($t34)

    // $t6 := copy($t32)
    call $tmp := $CopyOrMoveValue($t32);
    $t6 := $tmp;

    // $t7 := Libra::value<#0>($t6)
    call $t7 := $Libra_value($tv0, $t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 24139);
      goto Abort;
    }
    assume $IsValidU64($t7);


    // coin_value := $t7
    call $tmp := $CopyOrMoveValue($t7);
    coin_value := $tmp;
    if (true) { assume $DebugTrackLocal(9, 15878, 3, $tmp); }

    // $t8 := move($t34)
    call $t8 := $CopyOrMoveRef($t34);

    // $t9 := borrow_field<Libra::Preburn<#0>>.requests($t8)
    call $t9 := $BorrowField($t8, $Libra_Preburn_requests);
    assume $Vector_is_well_formed($Dereference($t9)) && (forall $$1: int :: {$select_vector($Dereference($t9),$$1)} $$1 >= 0 && $$1 < $vlen($Dereference($t9)) ==> $Libra_Libra_is_well_formed_types($select_vector($Dereference($t9),$$1)));

    // LocalRoot($t33) <- $t8
    call $t33 := $WritebackToValue($t8, 33, $t33);

    // UnpackRef($t9)

    // PackRef($t9)

    // $t36 := read_ref($t9)
    call $tmp := $ReadRef($t9);
    assume $Vector_is_well_formed($tmp) && (forall $$0: int :: {$select_vector($tmp,$$0)} $$0 >= 0 && $$0 < $vlen($tmp) ==> $Libra_Libra_is_well_formed($select_vector($tmp,$$0)));
    $t36 := $tmp;

    // $t36 := Vector::push_back<Libra::Libra<#0>>($t36, $t32)
    call $t36 := $Vector_push_back($Libra_Libra_type_value($tv0), $t36, $t32);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 15921);
      goto Abort;
    }
    assume $Vector_is_well_formed($t36) && (forall $$0: int :: {$select_vector($t36,$$0)} $$0 >= 0 && $$0 < $vlen($t36) ==> $Libra_Libra_is_well_formed($select_vector($t36,$$0)));


    // write_ref($t9, $t36)
    call $t9 := $WriteRef($t9, $t36);
    if (true) { assume $DebugTrackLocal(9, 15686, 5, $Dereference(info)); }

    // LocalRoot($t33) <- $t9
    call $t33 := $WritebackToValue($t9, 33, $t33);

    // Reference($t8) <- $t9
    call $t8 := $WritebackToReference($t9, $t8);

    // UnpackRef($t9)

    // PackRef($t8)

    // PackRef($t9)

    // $t11 := Libra::currency_code<#0>()
    call $t11 := $Libra_currency_code($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 31398);
      goto Abort;
    }
    assume $Vector_is_well_formed($t11) && (forall $$0: int :: {$select_vector($t11,$$0)} $$0 >= 0 && $$0 < $vlen($t11) ==> $IsValidU8($select_vector($t11,$$0)));


    // currency_code := $t11
    call $tmp := $CopyOrMoveValue($t11);
    currency_code := $tmp;
    if (true) { assume $DebugTrackLocal(9, 16007, 4, $tmp); }

    // $t12 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t12 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 16126);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := borrow_global<Libra::CurrencyInfo<#0>>($t12)
    call $t13 := $BorrowGlobal($t12, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 16069);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($Dereference($t13));

    // UnpackRef($t13)
    call $Libra_CurrencyInfo_before_update_inv($tv0, $Dereference($t13));

    // info := $t13
    call info := $CopyOrMoveRef($t13);
    if (true) { assume $DebugTrackLocal(9, 16062, 5, $Dereference(info)); }

    // $t14 := copy(info)
    call $t14 := $CopyOrMoveRef(info);

    // $t15 := get_field<Libra::CurrencyInfo<#0>>.preburn_value($t14)
    call $tmp := $GetFieldFromReference($t14, $Libra_CurrencyInfo_preburn_value);
    assume $IsValidU64($tmp);
    $t15 := $tmp;

    // Reference(info) <- $t14
    call info := $WritebackToReference($t14, info);

    // $t16 := move($t15)
    call $tmp := $CopyOrMoveValue($t15);
    $t16 := $tmp;

    // $t18 := +($t16, coin_value)
    call $tmp := $AddU64($t16, coin_value);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 16200);
      goto Abort;
    }
    $t18 := $tmp;

    // $t19 := copy(info)
    call $t19 := $CopyOrMoveRef(info);

    // $t20 := borrow_field<Libra::CurrencyInfo<#0>>.preburn_value($t19)
    call $t20 := $BorrowField($t19, $Libra_CurrencyInfo_preburn_value);
    assume $IsValidU64($Dereference($t20));

    // Reference(info) <- $t19
    call info := $WritebackToReference($t19, info);

    // UnpackRef($t20)

    // write_ref($t20, $t18)
    call $t20 := $WriteRef($t20, $t18);
    if (true) { assume $DebugTrackLocal(9, 16160, 5, $Dereference(info)); }

    // Reference(info) <- $t20
    call info := $WritebackToReference($t20, info);

    // Reference($t19) <- $t20
    call $t19 := $WritebackToReference($t20, $t19);

    // PackRef($t20)

    // $t21 := copy(info)
    call $t21 := $CopyOrMoveRef(info);

    // $t22 := get_field<Libra::CurrencyInfo<#0>>.is_synthetic($t21)
    call $tmp := $GetFieldFromReference($t21, $Libra_CurrencyInfo_is_synthetic);
    assume is#$Boolean($tmp);
    $t22 := $tmp;

    // Reference(info) <- $t21
    call info := $WritebackToReference($t21, info);

    // $t23 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t23 := $tmp;

    // $t24 := !($t23)
    call $tmp := $Not($t23);
    $t24 := $tmp;

    // if ($t24) goto L0 else goto L1
    $tmp := $t24;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t25 := move(info)
    call $t25 := $CopyOrMoveRef(info);

    // $t26 := borrow_field<Libra::CurrencyInfo<#0>>.preburn_events($t25)
    call $t26 := $BorrowField($t25, $Libra_CurrencyInfo_preburn_events);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t26));

    // Libra::CurrencyInfo <- $t25
    call $WritebackToGlobal($t25);

    // UnpackRef($t26)
    call $Event_EventHandle_before_update_inv($Libra_PreburnEvent_type_value(), $Dereference($t26));

    // $t30 := pack Libra::PreburnEvent(coin_value, currency_code, $t35)
    call $tmp := $Libra_PreburnEvent_pack(0, 0, 0, coin_value, currency_code, $t35);
    $t30 := $tmp;

    // PackRef($t26)
    call $Event_EventHandle_after_update_inv($Libra_PreburnEvent_type_value(), $Dereference($t26));

    // $t37 := read_ref($t26)
    call $tmp := $ReadRef($t26);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t37 := $tmp;

    // $t37 := Event::emit_event<Libra::PreburnEvent>($t37, $t30)
    call $t37 := $Event_emit_event($Libra_PreburnEvent_type_value(), $t37, $t30);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 16329);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t37);


    // write_ref($t26, $t37)
    call $t26 := $WriteRef($t26, $t37);
    if (true) { assume $DebugTrackLocal(9, 15686, 5, $Dereference(info)); }

    // Libra::CurrencyInfo <- $t26
    call $WritebackToGlobal($t26);

    // Reference($t25) <- $t26
    call $t25 := $WritebackToReference($t26, $t25);

    // UnpackRef($t26)
    call $Event_EventHandle_before_update_inv($Libra_PreburnEvent_type_value(), $Dereference($t26));

    // PackRef($t25)
    call $Libra_CurrencyInfo_after_update_inv($tv0, $Dereference($t25));

    // PackRef($t26)
    call $Event_EventHandle_after_update_inv($Libra_PreburnEvent_type_value(), $Dereference($t26));

    // goto L3
    goto L3;

    // L2:
L2:

    // $t31 := move(info)
    call $t31 := $CopyOrMoveRef(info);

    // destroy($t31)

    // Libra::CurrencyInfo <- $t31
    call $WritebackToGlobal($t31);

    // PackRef($t31)
    call $Libra_CurrencyInfo_after_update_inv($tv0, $Dereference($t31));

    // goto L3
    goto L3;

    // L3:
L3:

    // return $t33
    $ret0 := $t33;
    if (true) { assume $DebugTrackLocal(9, 16567, 38, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_publish_burn_capability ($tv0: $TypeValue, account: $Value, cap: $Value, _: $Value) returns ()
free requires is#$Address(account);
free requires $Libra_BurnCapability_is_well_formed(cap);
free requires $Roles_Capability_is_well_formed(_);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t7: $Value; // $Roles_Capability_type_value($Roles_TreasuryComplianceRole_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 11508, 0, account); }
    if (true) { assume $DebugTrackLocal(9, 11508, 1, cap); }
    if (true) { assume $DebugTrackLocal(9, 11508, 2, _); }

    // bytecode translation starts here
    // $t5 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t5 := $tmp;

    // $t6 := move(cap)
    call $tmp := $CopyOrMoveValue(cap);
    $t6 := $tmp;

    // $t7 := move(_)
    call $tmp := $CopyOrMoveValue(_);
    $t7 := $tmp;

    // Libra::assert_is_coin<#0>()
    call $Libra_assert_is_coin($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 33805);
      goto Abort;
    }

    // $t3 := move($t5)
    call $tmp := $CopyOrMoveValue($t5);
    $t3 := $tmp;

    // move_to<Libra::BurnCapability<#0>>($t6, $t3)
    call $MoveTo($Libra_BurnCapability_type_value($tv0), $t6, $t3);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 11719);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Libra_publish_mint_capability ($tv0: $TypeValue, account: $Value, cap: $Value, _: $Value) returns ()
free requires is#$Address(account);
free requires $Libra_MintCapability_is_well_formed(cap);
free requires $Roles_Capability_is_well_formed(_);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $Libra_MintCapability_type_value($tv0)
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $Libra_MintCapability_type_value($tv0)
    var $t7: $Value; // $Roles_Capability_type_value($Roles_TreasuryComplianceRole_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 11061, 0, account); }
    if (true) { assume $DebugTrackLocal(9, 11061, 1, cap); }
    if (true) { assume $DebugTrackLocal(9, 11061, 2, _); }

    // bytecode translation starts here
    // $t5 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t5 := $tmp;

    // $t6 := move(cap)
    call $tmp := $CopyOrMoveValue(cap);
    $t6 := $tmp;

    // $t7 := move(_)
    call $tmp := $CopyOrMoveValue(_);
    $t7 := $tmp;

    // Libra::assert_is_coin<#0>()
    call $Libra_assert_is_coin($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 33805);
      goto Abort;
    }

    // $t3 := move($t5)
    call $tmp := $CopyOrMoveValue($t5);
    $t3 := $tmp;

    // move_to<Libra::MintCapability<#0>>($t6, $t3)
    call $MoveTo($Libra_MintCapability_type_value($tv0), $t6, $t3);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 11272);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Libra_publish_preburn_to_account ($tv0: $TypeValue, account: $Value, tc_capability: $Value) returns ()
free requires is#$Address(account);
free requires $Roles_Capability_is_well_formed(tc_capability);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $Roles_Capability_type_value($Roles_TreasuryComplianceRole_type_value())
    var $t9: $Value; // $Libra_Preburn_type_value($tv0)
    var $t10: $Value; // $Roles_Capability_type_value($Roles_TreasuryComplianceRole_type_value())
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $Roles_Capability_type_value($Roles_TreasuryComplianceRole_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 17377, 0, account); }
    if (true) { assume $DebugTrackLocal(9, 17377, 1, tc_capability); }

    // bytecode translation starts here
    // $t13 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t13 := $tmp;

    // $t14 := move(tc_capability)
    call $tmp := $CopyOrMoveValue(tc_capability);
    $t14 := $tmp;

    // $t4 := Libra::is_synthetic_currency<#0>()
    call $t4 := $Libra_is_synthetic_currency($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 30408);
      goto Abort;
    }
    assume is#$Boolean($t4);


    // $t5 := !($t4)
    call $tmp := $Not($t4);
    $t5 := $tmp;

    // $t2 := $t5
    call $tmp := $CopyOrMoveValue($t5);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 17550, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t7 := move($t13)
    call $tmp := $CopyOrMoveValue($t13);
    $t7 := $tmp;

    // $t8 := move($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t8 := $tmp;

    // $t9 := Libra::create_preburn<#0>($t8)
    call $t9 := $Libra_create_preburn($tv0, $t8);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 16850);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($t9);


    // move_to<Libra::Preburn<#0>>($t9, $t7)
    call $MoveTo($Libra_Preburn_type_value($tv0), $t9, $t7);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 17607);
      goto Abort;
    }

    // return ()
    return;

    // L2:
L2:

    // $t10 := move($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t10 := $tmp;

    // destroy($t10)

    // $t11 := move($t13)
    call $tmp := $CopyOrMoveValue($t13);
    $t11 := $tmp;

    // destroy($t11)

    // $t12 := 202
    $tmp := $Integer(202);
    $t12 := $tmp;

    // abort($t12)
    if (true) { assume $DebugTrackAbort(9, 17550); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Libra_register_currency ($tv0: $TypeValue, account: $Value, _: $Value, to_lbr_exchange_rate: $Value, is_synthetic: $Value, scaling_factor: $Value, fractional_part: $Value, currency_code: $Value) returns ($ret0: $Value, $ret1: $Value)
free requires is#$Address(account);
free requires $Roles_Capability_is_well_formed(_);
free requires $FixedPoint32_FixedPoint32_is_well_formed(to_lbr_exchange_rate);
free requires is#$Boolean(is_synthetic);
free requires $IsValidU64(scaling_factor);
free requires $IsValidU64(fractional_part);
free requires $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)))) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(!b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)))))))));
requires b#$Boolean($Boolean(b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())) && b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)) ==> b#$Boolean($Boolean($IsEqual(addr, $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS()))))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)))) ==> b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(!b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr))))))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($RegisteredCurrencies_spec_is_initialized($m, $txn)) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())) && b#$Boolean($Boolean((forall addr: $Value :: is#$Address(addr) ==> b#$Boolean($Boolean(b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value(), addr)) ==> b#$Boolean($Boolean($IsEqual(addr, $CoreAddresses_SPEC_DEFAULT_CONFIG_ADDRESS())))))))))))));
{
    // declare local variables
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t18: $Value; // $BooleanType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $Vector_type_value($IntegerType())
    var $t22: $Value; // $BooleanType()
    var $t23: $Value; // $AddressType()
    var $t24: $Value; // $Event_EventHandle_type_value($Libra_MintEvent_type_value())
    var $t25: $Value; // $AddressType()
    var $t26: $Value; // $Event_EventHandle_type_value($Libra_BurnEvent_type_value())
    var $t27: $Value; // $AddressType()
    var $t28: $Value; // $Event_EventHandle_type_value($Libra_PreburnEvent_type_value())
    var $t29: $Value; // $AddressType()
    var $t30: $Value; // $Event_EventHandle_type_value($Libra_CancelBurnEvent_type_value())
    var $t31: $Value; // $AddressType()
    var $t32: $Value; // $Event_EventHandle_type_value($Libra_ToLBRExchangeRateUpdateEvent_type_value())
    var $t33: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t34: $Value; // $Vector_type_value($IntegerType())
    var $t35: $Value; // $AddressType()
    var $t36: $Value; // $Libra_CurrencyRegistrationCapability_type_value()
    var $t37: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t38: $Value; // $BooleanType()
    var $t39: $Value; // $Libra_MintCapability_type_value($tv0)
    var $t40: $Value; // $BooleanType()
    var $t41: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t42: $Value; // $AddressType()
    var $t43: $Value; // $IntegerType()
    var $t44: $Value; // $AddressType()
    var $t45: $Value; // $Roles_Capability_type_value($Libra_RegisterNewCurrency_type_value())
    var $t46: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t47: $Value; // $BooleanType()
    var $t48: $Value; // $IntegerType()
    var $t49: $Value; // $IntegerType()
    var $t50: $Value; // $Vector_type_value($IntegerType())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 27386, 0, account); }
    if (true) { assume $DebugTrackLocal(9, 27386, 1, _); }
    if (true) { assume $DebugTrackLocal(9, 27386, 2, to_lbr_exchange_rate); }
    if (true) { assume $DebugTrackLocal(9, 27386, 3, is_synthetic); }
    if (true) { assume $DebugTrackLocal(9, 27386, 4, scaling_factor); }
    if (true) { assume $DebugTrackLocal(9, 27386, 5, fractional_part); }
    if (true) { assume $DebugTrackLocal(9, 27386, 6, currency_code); }

    // bytecode translation starts here
    // $t44 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t44 := $tmp;

    // $t45 := move(_)
    call $tmp := $CopyOrMoveValue(_);
    $t45 := $tmp;

    // $t46 := move(to_lbr_exchange_rate)
    call $tmp := $CopyOrMoveValue(to_lbr_exchange_rate);
    $t46 := $tmp;

    // $t47 := move(is_synthetic)
    call $tmp := $CopyOrMoveValue(is_synthetic);
    $t47 := $tmp;

    // $t48 := move(scaling_factor)
    call $tmp := $CopyOrMoveValue(scaling_factor);
    $t48 := $tmp;

    // $t49 := move(fractional_part)
    call $tmp := $CopyOrMoveValue(fractional_part);
    $t49 := $tmp;

    // $t50 := move(currency_code)
    call $tmp := $CopyOrMoveValue(currency_code);
    $t50 := $tmp;

    // $t9 := copy($t44)
    call $tmp := $CopyOrMoveValue($t44);
    $t9 := $tmp;

    // $t10 := Signer::address_of($t9)
    call $t10 := $Signer_address_of($t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 27899);
      goto Abort;
    }
    assume is#$Address($t10);


    // $t11 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t11 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 27937);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := ==($t10, $t11)
    $tmp := $Boolean($IsEqual($t10, $t11));
    $t12 := $tmp;

    // $t7 := $t12
    call $tmp := $CopyOrMoveValue($t12);
    $t7 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 27871, 7, $tmp); }

    // if ($t7) goto L0 else goto L1
    $tmp := $t7;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t14 := copy($t44)
    call $tmp := $CopyOrMoveValue($t44);
    $t14 := $tmp;

    // $t15 := 0
    $tmp := $Integer(0);
    $t15 := $tmp;

    // $t16 := 0
    $tmp := $Integer(0);
    $t16 := $tmp;

    // $t22 := true
    $tmp := $Boolean(true);
    $t22 := $tmp;

    // $t23 := copy($t44)
    call $tmp := $CopyOrMoveValue($t44);
    $t23 := $tmp;

    // $t24 := Event::new_event_handle<Libra::MintEvent>($t23)
    call $t24 := $Event_new_event_handle($Libra_MintEvent_type_value(), $t23);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 28320);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t24);


    // $t25 := copy($t44)
    call $tmp := $CopyOrMoveValue($t44);
    $t25 := $tmp;

    // $t26 := Event::new_event_handle<Libra::BurnEvent>($t25)
    call $t26 := $Event_new_event_handle($Libra_BurnEvent_type_value(), $t25);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 28390);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t26);


    // $t27 := copy($t44)
    call $tmp := $CopyOrMoveValue($t44);
    $t27 := $tmp;

    // $t28 := Event::new_event_handle<Libra::PreburnEvent>($t27)
    call $t28 := $Event_new_event_handle($Libra_PreburnEvent_type_value(), $t27);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 28463);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t28);


    // $t29 := copy($t44)
    call $tmp := $CopyOrMoveValue($t44);
    $t29 := $tmp;

    // $t30 := Event::new_event_handle<Libra::CancelBurnEvent>($t29)
    call $t30 := $Event_new_event_handle($Libra_CancelBurnEvent_type_value(), $t29);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 28543);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t30);


    // $t31 := move($t44)
    call $tmp := $CopyOrMoveValue($t44);
    $t31 := $tmp;

    // $t32 := Event::new_event_handle<Libra::ToLBRExchangeRateUpdateEvent>($t31)
    call $t32 := $Event_new_event_handle($Libra_ToLBRExchangeRateUpdateEvent_type_value(), $t31);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 28635);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t32);


    // $t33 := pack Libra::CurrencyInfo<#0>($t15, $t16, $t46, $t47, $t48, $t49, $t50, $t22, $t24, $t26, $t28, $t30, $t32)
    call $tmp := $Libra_CurrencyInfo_pack(0, 0, 0, $tv0, $t15, $t16, $t46, $t47, $t48, $t49, $t50, $t22, $t24, $t26, $t28, $t30, $t32);
    $t33 := $tmp;

    // move_to<Libra::CurrencyInfo<#0>>($t33, $t14)
    call $MoveTo($Libra_CurrencyInfo_type_value($tv0), $t33, $t14);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 27996);
      goto Abort;
    }

    // $t35 := CoreAddresses::DEFAULT_CONFIG_ADDRESS()
    call $t35 := $CoreAddresses_DEFAULT_CONFIG_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 28853);
      goto Abort;
    }
    assume is#$Address($t35);


    // $t36 := get_global<Libra::CurrencyRegistrationCapability>($t35)
    call $tmp := $GetGlobal($t35, $Libra_CurrencyRegistrationCapability_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 28792);
      goto Abort;
    }
    assume $Libra_CurrencyRegistrationCapability_is_well_formed($tmp);
    $t36 := $tmp;

    // $t37 := get_field<Libra::CurrencyRegistrationCapability>.cap($t36)
    call $tmp := $GetFieldFromValue($t36, $Libra_CurrencyRegistrationCapability_cap);
    assume $RegisteredCurrencies_RegistrationCapability_is_well_formed($tmp);
    $t37 := $tmp;

    // RegisteredCurrencies::add_currency_code($t50, $t37)
    call $RegisteredCurrencies_add_currency_code($t50, $t37);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 28733);
      goto Abort;
    }

    // $t38 := false
    $tmp := $Boolean(false);
    $t38 := $tmp;

    // $t39 := pack Libra::MintCapability<#0>($t38)
    call $tmp := $Libra_MintCapability_pack(0, 0, 0, $tv0, $t38);
    $t39 := $tmp;

    // $t40 := false
    $tmp := $Boolean(false);
    $t40 := $tmp;

    // $t41 := pack Libra::BurnCapability<#0>($t40)
    call $tmp := $Libra_BurnCapability_pack(0, 0, 0, $tv0, $t40);
    $t41 := $tmp;

    // return ($t39, $t41)
    $ret0 := $t39;
    if (true) { assume $DebugTrackLocal(9, 28902, 51, $ret0); }
    $ret1 := $t41;
    if (true) { assume $DebugTrackLocal(9, 28902, 52, $ret1); }
    return;

    // L2:
L2:

    // $t42 := move($t44)
    call $tmp := $CopyOrMoveValue($t44);
    $t42 := $tmp;

    // destroy($t42)

    // $t43 := 8
    $tmp := $Integer(8);
    $t43 := $tmp;

    // abort($t43)
    if (true) { assume $DebugTrackAbort(9, 27871); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}

procedure {:inline 1} $Libra_scaling_factor ($tv0: $TypeValue) returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 30900);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := get_global<Libra::CurrencyInfo<#0>>($t0)
    call $tmp := $GetGlobal($t0, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 30847);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($tmp);
    $t1 := $tmp;

    // $t2 := get_field<Libra::CurrencyInfo<#0>>.scaling_factor($t1)
    call $tmp := $GetFieldFromValue($t1, $Libra_CurrencyInfo_scaling_factor);
    assume $IsValidU64($tmp);
    $t2 := $tmp;

    // $t3 := move($t2)
    call $tmp := $CopyOrMoveValue($t2);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(9, 30847, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_remove_burn_capability ($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 23062, 0, account); }

    // bytecode translation starts here
    // $t4 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := Signer::address_of($t1)
    call $t2 := $Signer_address_of($t1);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 23232);
      goto Abort;
    }
    assume is#$Address($t2);


    // $t3 := move_from<Libra::BurnCapability<#0>>($t2)
    call $tmp := $MoveFrom($t2, $Libra_BurnCapability_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 23188);
      goto Abort;
    }
    assume $Libra_BurnCapability_is_well_formed($tmp);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(9, 23188, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_remove_mint_capability ($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
free requires is#$Address(account);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $Libra_MintCapability_type_value($tv0)
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 22642, 0, account); }

    // bytecode translation starts here
    // $t4 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := Signer::address_of($t1)
    call $t2 := $Signer_address_of($t1);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 22812);
      goto Abort;
    }
    assume is#$Address($t2);


    // $t3 := move_from<Libra::MintCapability<#0>>($t2)
    call $tmp := $MoveFrom($t2, $Libra_MintCapability_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 22768);
      goto Abort;
    }
    assume $Libra_MintCapability_is_well_formed($tmp);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(9, 22768, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_split ($tv0: $TypeValue, coin: $Value, amount: $Value) returns ($ret0: $Value, $ret1: $Value)
free requires $Libra_Libra_is_well_formed(coin);
free requires $IsValidU64(amount);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var other: $Value; // $Libra_Libra_type_value($tv0)
    var $t3: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $Libra_Libra_type_value($tv0)
    var $t6: $Value; // $Libra_Libra_type_value($tv0)
    var $t7: $Value; // $Libra_Libra_type_value($tv0)
    var $t8: $Value; // $Libra_Libra_type_value($tv0)
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 24453, 0, coin); }
    if (true) { assume $DebugTrackLocal(9, 24453, 1, amount); }

    // bytecode translation starts here
    // $t8 := move(coin)
    call $tmp := $CopyOrMoveValue(coin);
    $t8 := $tmp;

    // $t9 := move(amount)
    call $tmp := $CopyOrMoveValue(amount);
    $t9 := $tmp;

    // $t3 := borrow_local($t8)
    call $t3 := $BorrowLoc(8, $t8);
    assume $Libra_Libra_is_well_formed($Dereference($t3));

    // UnpackRef($t3)
    call $Libra_Libra_before_update_inv($tv0, $Dereference($t3));

    // PackRef($t3)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t3));

    // $t10 := read_ref($t3)
    call $tmp := $ReadRef($t3);
    assume $Libra_Libra_is_well_formed($tmp);
    $t10 := $tmp;

    // ($t5, $t10) := Libra::withdraw<#0>($t10, $t9)
    call $t5, $t10 := $Libra_withdraw($tv0, $t10, $t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 25017);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t5);

    assume $Libra_Libra_is_well_formed($t10);


    // write_ref($t3, $t10)
    call $t3 := $WriteRef($t3, $t10);

    // LocalRoot($t8) <- $t3
    call $t8 := $WritebackToValue($t3, 8, $t8);

    // UnpackRef($t3)
    call $Libra_Libra_before_update_inv($tv0, $Dereference($t3));

    // PackRef($t3)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t3));

    // other := $t5
    call $tmp := $CopyOrMoveValue($t5);
    other := $tmp;
    if (true) { assume $DebugTrackLocal(9, 24566, 2, $tmp); }

    // return ($t8, other)
    $ret0 := $t8;
    if (true) { assume $DebugTrackLocal(9, 24611, 11, $ret0); }
    $ret1 := other;
    if (true) { assume $DebugTrackLocal(9, 24611, 12, $ret1); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}

procedure {:inline 1} $Libra_update_lbr_exchange_rate ($tv0: $TypeValue, _: $Value, lbr_exchange_rate: $Value) returns ()
free requires $Roles_Capability_is_well_formed(_);
free requires $FixedPoint32_FixedPoint32_is_well_formed(lbr_exchange_rate);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var currency_info: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t3: $Value; // $AddressType()
    var $t4: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t5: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t6: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t7: $Reference; // ReferenceType($FixedPoint32_FixedPoint32_type_value())
    var $t8: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t9: $Reference; // ReferenceType($Event_EventHandle_type_value($Libra_ToLBRExchangeRateUpdateEvent_type_value()))
    var $t10: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t11: $Value; // $Vector_type_value($IntegerType())
    var $t12: $Value; // $Vector_type_value($IntegerType())
    var $t13: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t14: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t15: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $Libra_ToLBRExchangeRateUpdateEvent_type_value()
    var $t18: $Value; // $Roles_Capability_type_value($Roles_TreasuryComplianceRole_type_value())
    var $t19: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t20: $Value; // $Event_EventHandle_type_value($Libra_ToLBRExchangeRateUpdateEvent_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 31716, 0, _); }
    if (true) { assume $DebugTrackLocal(9, 31716, 1, lbr_exchange_rate); }

    // bytecode translation starts here
    // $t18 := move(_)
    call $tmp := $CopyOrMoveValue(_);
    $t18 := $tmp;

    // $t19 := move(lbr_exchange_rate)
    call $tmp := $CopyOrMoveValue(lbr_exchange_rate);
    $t19 := $tmp;

    // Libra::assert_is_coin<#0>()
    call $Libra_assert_is_coin($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 33805);
      goto Abort;
    }

    // $t3 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t3 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 32014);
      goto Abort;
    }
    assume is#$Address($t3);


    // $t4 := borrow_global<Libra::CurrencyInfo<#0>>($t3)
    call $t4 := $BorrowGlobal($t3, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 31953);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($Dereference($t4));

    // UnpackRef($t4)
    call $Libra_CurrencyInfo_before_update_inv($tv0, $Dereference($t4));

    // currency_info := $t4
    call currency_info := $CopyOrMoveRef($t4);
    if (true) { assume $DebugTrackLocal(9, 31937, 2, $Dereference(currency_info)); }

    // $t6 := copy(currency_info)
    call $t6 := $CopyOrMoveRef(currency_info);

    // $t7 := borrow_field<Libra::CurrencyInfo<#0>>.to_lbr_exchange_rate($t6)
    call $t7 := $BorrowField($t6, $Libra_CurrencyInfo_to_lbr_exchange_rate);
    assume $FixedPoint32_FixedPoint32_is_well_formed_types($Dereference($t7));

    // Reference(currency_info) <- $t6
    call currency_info := $WritebackToReference($t6, currency_info);

    // UnpackRef($t7)

    // write_ref($t7, $t19)
    call $t7 := $WriteRef($t7, $t19);
    if (true) { assume $DebugTrackLocal(9, 32048, 2, $Dereference(currency_info)); }

    // Reference(currency_info) <- $t7
    call currency_info := $WritebackToReference($t7, currency_info);

    // Reference($t6) <- $t7
    call $t6 := $WritebackToReference($t7, $t6);

    // PackRef($t7)

    // $t8 := copy(currency_info)
    call $t8 := $CopyOrMoveRef(currency_info);

    // $t9 := borrow_field<Libra::CurrencyInfo<#0>>.exchange_rate_update_events($t8)
    call $t9 := $BorrowField($t8, $Libra_CurrencyInfo_exchange_rate_update_events);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t9));

    // Reference(currency_info) <- $t8
    call currency_info := $WritebackToReference($t8, currency_info);

    // UnpackRef($t9)
    call $Event_EventHandle_before_update_inv($Libra_ToLBRExchangeRateUpdateEvent_type_value(), $Dereference($t9));

    // $t10 := copy(currency_info)
    call $t10 := $CopyOrMoveRef(currency_info);

    // $t11 := get_field<Libra::CurrencyInfo<#0>>.currency_code($t10)
    call $tmp := $GetFieldFromReference($t10, $Libra_CurrencyInfo_currency_code);
    assume $Vector_is_well_formed($tmp) && (forall $$0: int :: {$select_vector($tmp,$$0)} $$0 >= 0 && $$0 < $vlen($tmp) ==> $IsValidU8($select_vector($tmp,$$0)));
    $t11 := $tmp;

    // Reference(currency_info) <- $t10
    call currency_info := $WritebackToReference($t10, currency_info);

    // $t12 := move($t11)
    call $tmp := $CopyOrMoveValue($t11);
    $t12 := $tmp;

    // $t13 := move(currency_info)
    call $t13 := $CopyOrMoveRef(currency_info);

    // $t14 := get_field<Libra::CurrencyInfo<#0>>.to_lbr_exchange_rate($t13)
    call $tmp := $GetFieldFromReference($t13, $Libra_CurrencyInfo_to_lbr_exchange_rate);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t14 := $tmp;

    // Libra::CurrencyInfo <- $t13
    call $WritebackToGlobal($t13);

    // $t15 := move($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t15 := $tmp;

    // $t16 := FixedPoint32::get_raw_value($t15)
    call $t16 := $FixedPoint32_get_raw_value($t15);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 32352);
      goto Abort;
    }
    assume $IsValidU64($t16);


    // $t17 := pack Libra::ToLBRExchangeRateUpdateEvent($t12, $t16)
    call $tmp := $Libra_ToLBRExchangeRateUpdateEvent_pack(0, 0, 0, $t12, $t16);
    $t17 := $tmp;

    // PackRef($t9)
    call $Event_EventHandle_after_update_inv($Libra_ToLBRExchangeRateUpdateEvent_type_value(), $Dereference($t9));

    // $t20 := read_ref($t9)
    call $tmp := $ReadRef($t9);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t20 := $tmp;

    // $t20 := Event::emit_event<Libra::ToLBRExchangeRateUpdateEvent>($t20, $t17)
    call $t20 := $Event_emit_event($Libra_ToLBRExchangeRateUpdateEvent_type_value(), $t20, $t17);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 32119);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t20);


    // write_ref($t9, $t20)
    call $t9 := $WriteRef($t9, $t20);
    if (true) { assume $DebugTrackLocal(9, 31716, 2, $Dereference(currency_info)); }

    // Libra::CurrencyInfo <- $t9
    call $WritebackToGlobal($t9);

    // Reference($t8) <- $t9
    call $t8 := $WritebackToReference($t9, $t8);

    // Reference($t10) <- $t9
    call $t10 := $WritebackToReference($t9, $t10);

    // Reference($t13) <- $t9
    call $t13 := $WritebackToReference($t9, $t13);

    // UnpackRef($t9)
    call $Event_EventHandle_before_update_inv($Libra_ToLBRExchangeRateUpdateEvent_type_value(), $Dereference($t9));

    // PackRef($t9)
    call $Event_EventHandle_after_update_inv($Libra_ToLBRExchangeRateUpdateEvent_type_value(), $Dereference($t9));

    // PackRef($t13)
    call $Libra_CurrencyInfo_after_update_inv($tv0, $Dereference($t13));

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Libra_update_minting_ability ($tv0: $TypeValue, _: $Value, can_mint: $Value) returns ()
free requires $Roles_Capability_is_well_formed(_);
free requires is#$Boolean(can_mint);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var currency_info: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t3: $Value; // $AddressType()
    var $t4: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t5: $Value; // $BooleanType()
    var $t6: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t7: $Reference; // ReferenceType($BooleanType())
    var $t8: $Value; // $Roles_Capability_type_value($Roles_TreasuryComplianceRole_type_value())
    var $t9: $Value; // $BooleanType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 33229, 0, _); }
    if (true) { assume $DebugTrackLocal(9, 33229, 1, can_mint); }

    // bytecode translation starts here
    // $t8 := move(_)
    call $tmp := $CopyOrMoveValue(_);
    $t8 := $tmp;

    // $t9 := move(can_mint)
    call $tmp := $CopyOrMoveValue(can_mint);
    $t9 := $tmp;

    // Libra::assert_is_coin<#0>()
    call $Libra_assert_is_coin($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 33805);
      goto Abort;
    }

    // $t3 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t3 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 33478);
      goto Abort;
    }
    assume is#$Address($t3);


    // $t4 := borrow_global<Libra::CurrencyInfo<#0>>($t3)
    call $t4 := $BorrowGlobal($t3, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 33421);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($Dereference($t4));

    // UnpackRef($t4)
    call $Libra_CurrencyInfo_before_update_inv($tv0, $Dereference($t4));

    // currency_info := $t4
    call currency_info := $CopyOrMoveRef($t4);
    if (true) { assume $DebugTrackLocal(9, 33405, 2, $Dereference(currency_info)); }

    // $t6 := move(currency_info)
    call $t6 := $CopyOrMoveRef(currency_info);

    // $t7 := borrow_field<Libra::CurrencyInfo<#0>>.can_mint($t6)
    call $t7 := $BorrowField($t6, $Libra_CurrencyInfo_can_mint);
    assume is#$Boolean($Dereference($t7));

    // Libra::CurrencyInfo <- $t6
    call $WritebackToGlobal($t6);

    // UnpackRef($t7)

    // write_ref($t7, $t9)
    call $t7 := $WriteRef($t7, $t9);
    if (true) { assume $DebugTrackLocal(9, 33512, 2, $Dereference(currency_info)); }

    // Libra::CurrencyInfo <- $t7
    call $WritebackToGlobal($t7);

    // Reference($t6) <- $t7
    call $t6 := $WritebackToReference($t7, $t6);

    // PackRef($t6)
    call $Libra_CurrencyInfo_after_update_inv($tv0, $Dereference($t6));

    // PackRef($t7)

    // return ()
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $Libra_withdraw ($tv0: $TypeValue, coin: $Value, amount: $Value) returns ($ret0: $Value, $ret1: $Value)
free requires $Libra_Libra_is_well_formed(coin);
free requires $IsValidU64(amount);
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $IntegerType()
    var $t15: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t16: $Reference; // ReferenceType($IntegerType())
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $Libra_Libra_type_value($tv0)
    var $t19: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $Libra_Libra_type_value($tv0)
    var $t22: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t23: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 25006, 0, coin); }
    if (true) { assume $DebugTrackLocal(9, 25006, 1, amount); }

    // bytecode translation starts here
    // $t21 := move(coin)
    call $tmp := $CopyOrMoveValue(coin);
    $t21 := $tmp;

    // $t23 := move(amount)
    call $tmp := $CopyOrMoveValue(amount);
    $t23 := $tmp;

    // $t22 := borrow_local($t21)
    call $t22 := $BorrowLoc(21, $t21);
    assume $Libra_Libra_is_well_formed($Dereference($t22));

    // UnpackRef($t22)
    call $Libra_Libra_before_update_inv($tv0, $Dereference($t22));

    // $t4 := copy($t22)
    call $t4 := $CopyOrMoveRef($t22);

    // $t5 := get_field<Libra::Libra<#0>>.value($t4)
    call $tmp := $GetFieldFromReference($t4, $Libra_Libra_value);
    assume $IsValidU64($tmp);
    $t5 := $tmp;

    // Reference($t22) <- $t4
    call $t22 := $WritebackToReference($t4, $t22);

    // $t6 := move($t5)
    call $tmp := $CopyOrMoveValue($t5);
    $t6 := $tmp;

    // $t8 := >=($t6, $t23)
    call $tmp := $Ge($t6, $t23);
    $t8 := $tmp;

    // $t2 := $t8
    call $tmp := $CopyOrMoveValue($t8);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 25165, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t10 := copy($t22)
    call $t10 := $CopyOrMoveRef($t22);

    // $t11 := get_field<Libra::Libra<#0>>.value($t10)
    call $tmp := $GetFieldFromReference($t10, $Libra_Libra_value);
    assume $IsValidU64($tmp);
    $t11 := $tmp;

    // Reference($t22) <- $t10
    call $t22 := $WritebackToReference($t10, $t22);

    // $t12 := move($t11)
    call $tmp := $CopyOrMoveValue($t11);
    $t12 := $tmp;

    // $t14 := -($t12, $t23)
    call $tmp := $Sub($t12, $t23);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 25231);
      goto Abort;
    }
    $t14 := $tmp;

    // $t15 := move($t22)
    call $t15 := $CopyOrMoveRef($t22);

    // $t16 := borrow_field<Libra::Libra<#0>>.value($t15)
    call $t16 := $BorrowField($t15, $Libra_Libra_value);
    assume $IsValidU64($Dereference($t16));

    // LocalRoot($t21) <- $t15
    call $t21 := $WritebackToValue($t15, 21, $t21);

    // UnpackRef($t16)

    // write_ref($t16, $t14)
    call $t16 := $WriteRef($t16, $t14);

    // LocalRoot($t21) <- $t16
    call $t21 := $WritebackToValue($t16, 21, $t21);

    // Reference($t15) <- $t16
    call $t15 := $WritebackToReference($t16, $t15);

    // PackRef($t15)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t15));

    // PackRef($t16)

    // $t18 := pack Libra::Libra<#0>($t23)
    call $tmp := $Libra_Libra_pack(0, 0, 0, $tv0, $t23);
    $t18 := $tmp;

    // return ($t18, $t21)
    $ret0 := $t18;
    if (true) { assume $DebugTrackLocal(9, 25249, 24, $ret0); }
    $ret1 := $t21;
    if (true) { assume $DebugTrackLocal(9, 25249, 25, $ret1); }
    return;

    // L2:
L2:

    // $t19 := move($t22)
    call $t19 := $CopyOrMoveRef($t22);

    // destroy($t19)

    // LocalRoot($t21) <- $t19
    call $t21 := $WritebackToValue($t19, 21, $t21);

    // PackRef($t19)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t19));

    // $t20 := 10
    $tmp := $Integer(10);
    $t20 := $tmp;

    // abort($t20)
    if (true) { assume $DebugTrackAbort(9, 25165); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}

procedure {:inline 1} $Libra_zero ($tv0: $TypeValue) returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], old($Libra_sum_of_coin_values[$tv0])))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0)))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value)))))));
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $t1: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // Libra::assert_is_coin<#0>()
    call $Libra_assert_is_coin($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 33805);
      goto Abort;
    }

    // $t0 := 0
    $tmp := $Integer(0);
    $t0 := $tmp;

    // $t1 := pack Libra::Libra<#0>($t0)
    call $tmp := $Libra_Libra_pack(0, 0, 0, $tv0, $t0);
    $t1 := $tmp;

    // return $t1
    $ret0 := $t1;
    if (true) { assume $DebugTrackLocal(9, 23932, 2, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}



// ** spec vars of module Coin1



// ** spec funs of module Coin1



// ** structs of module Coin1

const unique $Coin1_Coin1: $TypeName;
const $Coin1_Coin1_dummy_field: $FieldName;
axiom $Coin1_Coin1_dummy_field == 0;
function $Coin1_Coin1_type_value(): $TypeValue {
    $StructType($Coin1_Coin1, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $Coin1_Coin1_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Coin1_Coin1_dummy_field))
}
function {:inline} $Coin1_Coin1_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Coin1_Coin1_dummy_field))
}

procedure {:inline 1} $Coin1_Coin1_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Coin1_Coin1_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $Coin1_Coin1_dummy_field);
    assume is#$Boolean(dummy_field);
}



// ** functions of module Coin1

procedure {:inline 1} $Coin1_initialize (account: $Value, register_currency_capability: $Value) returns ($ret0: $Value, $ret1: $Value)
free requires is#$Address(account);
free requires $Roles_Capability_is_well_formed(register_currency_capability);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $Roles_Capability_type_value($Libra_RegisterNewCurrency_type_value())
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $Vector_type_value($IntegerType())
    var $t11: $Value; // $Libra_MintCapability_type_value($Coin1_Coin1_type_value())
    var $t12: $Value; // $Libra_BurnCapability_type_value($Coin1_Coin1_type_value())
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $Roles_Capability_type_value($Libra_RegisterNewCurrency_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(2, 165, 0, account); }
    if (true) { assume $DebugTrackLocal(2, 165, 1, register_currency_capability); }

    // bytecode translation starts here
    // $t13 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t13 := $tmp;

    // $t14 := move(register_currency_capability)
    call $tmp := $CopyOrMoveValue(register_currency_capability);
    $t14 := $tmp;

    // $t2 := move($t13)
    call $tmp := $CopyOrMoveValue($t13);
    $t2 := $tmp;

    // $t3 := move($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t3 := $tmp;

    // $t4 := 1
    $tmp := $Integer(1);
    $t4 := $tmp;

    // $t5 := 2
    $tmp := $Integer(2);
    $t5 := $tmp;

    // $t6 := FixedPoint32::create_from_rational($t4, $t5)
    call $t6 := $FixedPoint32_create_from_rational($t4, $t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(2, 526);
      goto Abort;
    }
    assume $FixedPoint32_FixedPoint32_is_well_formed($t6);


    // $t7 := false
    $tmp := $Boolean(false);
    $t7 := $tmp;

    // $t8 := 1000000
    $tmp := $Integer(1000000);
    $t8 := $tmp;

    // $t9 := 100
    $tmp := $Integer(100);
    $t9 := $tmp;

    // $t10 := [67, 111, 105, 110, 49]
    $tmp := $push_back_vector($push_back_vector($push_back_vector($push_back_vector($push_back_vector($mk_vector(), $Integer(67)), $Integer(111)), $Integer(105)), $Integer(110)), $Integer(49));
    $t10 := $tmp;

    // ($t11, $t12) := Libra::register_currency<Coin1::Coin1>($t2, $t3, $t6, $t7, $t8, $t9, $t10)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin1_Coin1_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t11, $t12 := $Libra_register_currency($Coin1_Coin1_type_value(), $t2, $t3, $t6, $t7, $t8, $t9, $t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(2, 411);
      goto Abort;
    }
    assume $Libra_MintCapability_is_well_formed($t11);

    assume $Libra_BurnCapability_is_well_formed($t12);


    // return ($t11, $t12)
    $ret0 := $t11;
    if (true) { assume $DebugTrackLocal(2, 404, 15, $ret0); }
    $ret1 := $t12;
    if (true) { assume $DebugTrackLocal(2, 404, 16, $ret1); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}



// ** spec vars of module Coin2



// ** spec funs of module Coin2



// ** structs of module Coin2

const unique $Coin2_Coin2: $TypeName;
const $Coin2_Coin2_dummy_field: $FieldName;
axiom $Coin2_Coin2_dummy_field == 0;
function $Coin2_Coin2_type_value(): $TypeValue {
    $StructType($Coin2_Coin2, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $Coin2_Coin2_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Coin2_Coin2_dummy_field))
}
function {:inline} $Coin2_Coin2_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Coin2_Coin2_dummy_field))
}

procedure {:inline 1} $Coin2_Coin2_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Coin2_Coin2_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $Coin2_Coin2_dummy_field);
    assume is#$Boolean(dummy_field);
}



// ** functions of module Coin2

procedure {:inline 1} $Coin2_initialize (account: $Value, cap: $Value) returns ($ret0: $Value, $ret1: $Value)
free requires is#$Address(account);
free requires $Roles_Capability_is_well_formed(cap);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $Roles_Capability_type_value($Libra_RegisterNewCurrency_type_value())
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $Vector_type_value($IntegerType())
    var $t11: $Value; // $Libra_MintCapability_type_value($Coin2_Coin2_type_value())
    var $t12: $Value; // $Libra_BurnCapability_type_value($Coin2_Coin2_type_value())
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $Roles_Capability_type_value($Libra_RegisterNewCurrency_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(3, 165, 0, account); }
    if (true) { assume $DebugTrackLocal(3, 165, 1, cap); }

    // bytecode translation starts here
    // $t13 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t13 := $tmp;

    // $t14 := move(cap)
    call $tmp := $CopyOrMoveValue(cap);
    $t14 := $tmp;

    // $t2 := move($t13)
    call $tmp := $CopyOrMoveValue($t13);
    $t2 := $tmp;

    // $t3 := move($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t3 := $tmp;

    // $t4 := 1
    $tmp := $Integer(1);
    $t4 := $tmp;

    // $t5 := 2
    $tmp := $Integer(2);
    $t5 := $tmp;

    // $t6 := FixedPoint32::create_from_rational($t4, $t5)
    call $t6 := $FixedPoint32_create_from_rational($t4, $t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(3, 476);
      goto Abort;
    }
    assume $FixedPoint32_FixedPoint32_is_well_formed($t6);


    // $t7 := false
    $tmp := $Boolean(false);
    $t7 := $tmp;

    // $t8 := 1000000
    $tmp := $Integer(1000000);
    $t8 := $tmp;

    // $t9 := 100
    $tmp := $Integer(100);
    $t9 := $tmp;

    // $t10 := [67, 111, 105, 110, 50]
    $tmp := $push_back_vector($push_back_vector($push_back_vector($push_back_vector($push_back_vector($mk_vector(), $Integer(67)), $Integer(111)), $Integer(105)), $Integer(110)), $Integer(50));
    $t10 := $tmp;

    // ($t11, $t12) := Libra::register_currency<Coin2::Coin2>($t2, $t3, $t6, $t7, $t8, $t9, $t10)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin2_Coin2_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t11, $t12 := $Libra_register_currency($Coin2_Coin2_type_value(), $t2, $t3, $t6, $t7, $t8, $t9, $t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(3, 386);
      goto Abort;
    }
    assume $Libra_MintCapability_is_well_formed($t11);

    assume $Libra_BurnCapability_is_well_formed($t12);


    // return ($t11, $t12)
    $ret0 := $t11;
    if (true) { assume $DebugTrackLocal(3, 379, 15, $ret0); }
    $ret1 := $t12;
    if (true) { assume $DebugTrackLocal(3, 379, 16, $ret1); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}



// ** spec vars of module LBR



// ** spec funs of module LBR



// ** structs of module LBR

const unique $LBR_LBR: $TypeName;
const $LBR_LBR_dummy_field: $FieldName;
axiom $LBR_LBR_dummy_field == 0;
function $LBR_LBR_type_value(): $TypeValue {
    $StructType($LBR_LBR, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $LBR_LBR_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $LBR_LBR_dummy_field))
}
function {:inline} $LBR_LBR_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $LBR_LBR_dummy_field))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $LBR_LBR_is_well_formed($ResourceValue(m, $LBR_LBR_type_value(), a))
);

procedure {:inline 1} $LBR_LBR_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $LBR_LBR_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $LBR_LBR_dummy_field);
    assume is#$Boolean(dummy_field);
}

const unique $LBR_Reserve: $TypeName;
const $LBR_Reserve_mint_cap: $FieldName;
axiom $LBR_Reserve_mint_cap == 0;
const $LBR_Reserve_burn_cap: $FieldName;
axiom $LBR_Reserve_burn_cap == 1;
const $LBR_Reserve_preburn_cap: $FieldName;
axiom $LBR_Reserve_preburn_cap == 2;
const $LBR_Reserve_coin1: $FieldName;
axiom $LBR_Reserve_coin1 == 3;
const $LBR_Reserve_coin2: $FieldName;
axiom $LBR_Reserve_coin2 == 4;
function $LBR_Reserve_type_value(): $TypeValue {
    $StructType($LBR_Reserve, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $Libra_MintCapability_type_value($LBR_LBR_type_value())][1 := $Libra_BurnCapability_type_value($LBR_LBR_type_value())][2 := $Libra_Preburn_type_value($LBR_LBR_type_value())][3 := $LBR_ReserveComponent_type_value($Coin1_Coin1_type_value())][4 := $LBR_ReserveComponent_type_value($Coin2_Coin2_type_value())], 5))
}
function {:inline} $LBR_Reserve_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 5
      && $Libra_MintCapability_is_well_formed_types($SelectField($this, $LBR_Reserve_mint_cap))
      && $Libra_BurnCapability_is_well_formed_types($SelectField($this, $LBR_Reserve_burn_cap))
      && $Libra_Preburn_is_well_formed_types($SelectField($this, $LBR_Reserve_preburn_cap))
      && $LBR_ReserveComponent_is_well_formed_types($SelectField($this, $LBR_Reserve_coin1))
      && $LBR_ReserveComponent_is_well_formed_types($SelectField($this, $LBR_Reserve_coin2))
}
function {:inline} $LBR_Reserve_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 5
      && $Libra_MintCapability_is_well_formed($SelectField($this, $LBR_Reserve_mint_cap))
      && $Libra_BurnCapability_is_well_formed($SelectField($this, $LBR_Reserve_burn_cap))
      && $Libra_Preburn_is_well_formed($SelectField($this, $LBR_Reserve_preburn_cap))
      && $LBR_ReserveComponent_is_well_formed($SelectField($this, $LBR_Reserve_coin1))
      && $LBR_ReserveComponent_is_well_formed($SelectField($this, $LBR_Reserve_coin2))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $LBR_Reserve_is_well_formed($ResourceValue(m, $LBR_Reserve_type_value(), a))
);

procedure {:inline 1} $LBR_Reserve_before_update_inv($before: $Value) {
    call $LBR_ReserveComponent_before_update_inv($Coin1_Coin1_type_value(), $SelectField($before, $LBR_Reserve_coin1));
    call $LBR_ReserveComponent_before_update_inv($Coin2_Coin2_type_value(), $SelectField($before, $LBR_Reserve_coin2));
}

procedure {:inline 1} $LBR_Reserve_after_update_inv($after: $Value) {
    call $LBR_ReserveComponent_after_update_inv($Coin1_Coin1_type_value(), $SelectField($after, $LBR_Reserve_coin1));
    call $LBR_ReserveComponent_after_update_inv($Coin2_Coin2_type_value(), $SelectField($after, $LBR_Reserve_coin2));
}

procedure {:inline 1} $LBR_Reserve_pack($file_id: int, $byte_index: int, $var_idx: int, mint_cap: $Value, burn_cap: $Value, preburn_cap: $Value, coin1: $Value, coin2: $Value) returns ($struct: $Value)
{
    assume $Libra_MintCapability_is_well_formed(mint_cap);
    assume $Libra_BurnCapability_is_well_formed(burn_cap);
    assume $Libra_Preburn_is_well_formed(preburn_cap);
    assume $LBR_ReserveComponent_is_well_formed(coin1);
    assume $LBR_ReserveComponent_is_well_formed(coin2);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := mint_cap][1 := burn_cap][2 := preburn_cap][3 := coin1][4 := coin2], 5));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $LBR_Reserve_unpack($struct: $Value) returns (mint_cap: $Value, burn_cap: $Value, preburn_cap: $Value, coin1: $Value, coin2: $Value)
{
    assume is#$Vector($struct);
    mint_cap := $SelectField($struct, $LBR_Reserve_mint_cap);
    assume $Libra_MintCapability_is_well_formed(mint_cap);
    burn_cap := $SelectField($struct, $LBR_Reserve_burn_cap);
    assume $Libra_BurnCapability_is_well_formed(burn_cap);
    preburn_cap := $SelectField($struct, $LBR_Reserve_preburn_cap);
    assume $Libra_Preburn_is_well_formed(preburn_cap);
    coin1 := $SelectField($struct, $LBR_Reserve_coin1);
    assume $LBR_ReserveComponent_is_well_formed(coin1);
    coin2 := $SelectField($struct, $LBR_Reserve_coin2);
    assume $LBR_ReserveComponent_is_well_formed(coin2);
}

const unique $LBR_ReserveComponent: $TypeName;
const $LBR_ReserveComponent_ratio: $FieldName;
axiom $LBR_ReserveComponent_ratio == 0;
const $LBR_ReserveComponent_backing: $FieldName;
axiom $LBR_ReserveComponent_backing == 1;
function $LBR_ReserveComponent_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($LBR_ReserveComponent, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $FixedPoint32_FixedPoint32_type_value()][1 := $Libra_Libra_type_value($tv0)], 2))
}
function {:inline} $LBR_ReserveComponent_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && $FixedPoint32_FixedPoint32_is_well_formed_types($SelectField($this, $LBR_ReserveComponent_ratio))
      && $Libra_Libra_is_well_formed_types($SelectField($this, $LBR_ReserveComponent_backing))
}
function {:inline} $LBR_ReserveComponent_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && $FixedPoint32_FixedPoint32_is_well_formed($SelectField($this, $LBR_ReserveComponent_ratio))
      && $Libra_Libra_is_well_formed($SelectField($this, $LBR_ReserveComponent_backing))
}

axiom (forall m: $Memory, a: $Value, $tv0: $TypeValue :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $LBR_ReserveComponent_is_well_formed($ResourceValue(m, $LBR_ReserveComponent_type_value($tv0), a))
);

procedure {:inline 1} $LBR_ReserveComponent_before_update_inv($tv0: $TypeValue, $before: $Value) {
    call $Libra_Libra_before_update_inv($tv0, $SelectField($before, $LBR_ReserveComponent_backing));
}

procedure {:inline 1} $LBR_ReserveComponent_after_update_inv($tv0: $TypeValue, $after: $Value) {
    call $Libra_Libra_after_update_inv($tv0, $SelectField($after, $LBR_ReserveComponent_backing));
}

procedure {:inline 1} $LBR_ReserveComponent_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, ratio: $Value, backing: $Value) returns ($struct: $Value)
{
    assume $FixedPoint32_FixedPoint32_is_well_formed(ratio);
    assume $Libra_Libra_is_well_formed(backing);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := ratio][1 := backing], 2));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $LBR_ReserveComponent_unpack($tv0: $TypeValue, $struct: $Value) returns (ratio: $Value, backing: $Value)
{
    assume is#$Vector($struct);
    ratio := $SelectField($struct, $LBR_ReserveComponent_ratio);
    assume $FixedPoint32_FixedPoint32_is_well_formed(ratio);
    backing := $SelectField($struct, $LBR_ReserveComponent_backing);
    assume $Libra_Libra_is_well_formed(backing);
}



// ** functions of module LBR

procedure {:inline 1} $LBR_initialize (association: $Value, register_currency_capability: $Value, tc_capability: $Value) returns ()
free requires is#$Address(association);
free requires $Roles_Capability_is_well_formed(register_currency_capability);
free requires $Roles_Capability_is_well_formed(tc_capability);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var burn_cap: $Value; // $Libra_BurnCapability_type_value($LBR_LBR_type_value())
    var coin1: $Value; // $LBR_ReserveComponent_type_value($Coin1_Coin1_type_value())
    var coin2: $Value; // $LBR_ReserveComponent_type_value($Coin2_Coin2_type_value())
    var mint_cap: $Value; // $Libra_MintCapability_type_value($LBR_LBR_type_value())
    var preburn_cap: $Value; // $Libra_Preburn_type_value($LBR_LBR_type_value())
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $Roles_Capability_type_value($Libra_RegisterNewCurrency_type_value())
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t20: $Value; // $BooleanType()
    var $t21: $Value; // $IntegerType()
    var $t22: $Value; // $IntegerType()
    var $t23: $Value; // $Vector_type_value($IntegerType())
    var $t24: $Value; // $Libra_MintCapability_type_value($LBR_LBR_type_value())
    var $t25: $Value; // $Libra_BurnCapability_type_value($LBR_LBR_type_value())
    var $t26: $Value; // $Roles_Capability_type_value($Roles_TreasuryComplianceRole_type_value())
    var $t27: $Value; // $Libra_Preburn_type_value($LBR_LBR_type_value())
    var $t28: $Value; // $IntegerType()
    var $t29: $Value; // $IntegerType()
    var $t30: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t31: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t32: $Value; // $LBR_ReserveComponent_type_value($Coin1_Coin1_type_value())
    var $t33: $Value; // $IntegerType()
    var $t34: $Value; // $IntegerType()
    var $t35: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t36: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t37: $Value; // $LBR_ReserveComponent_type_value($Coin2_Coin2_type_value())
    var $t38: $Value; // $AddressType()
    var $t39: $Value; // $Libra_MintCapability_type_value($LBR_LBR_type_value())
    var $t40: $Value; // $Libra_BurnCapability_type_value($LBR_LBR_type_value())
    var $t41: $Value; // $Libra_Preburn_type_value($LBR_LBR_type_value())
    var $t42: $Value; // $LBR_ReserveComponent_type_value($Coin1_Coin1_type_value())
    var $t43: $Value; // $LBR_ReserveComponent_type_value($Coin2_Coin2_type_value())
    var $t44: $Value; // $LBR_Reserve_type_value()
    var $t45: $Value; // $Roles_Capability_type_value($Roles_TreasuryComplianceRole_type_value())
    var $t46: $Value; // $Roles_Capability_type_value($Libra_RegisterNewCurrency_type_value())
    var $t47: $Value; // $AddressType()
    var $t48: $Value; // $IntegerType()
    var $t49: $Value; // $AddressType()
    var $t50: $Value; // $Roles_Capability_type_value($Libra_RegisterNewCurrency_type_value())
    var $t51: $Value; // $Roles_Capability_type_value($Roles_TreasuryComplianceRole_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(7, 1590, 0, association); }
    if (true) { assume $DebugTrackLocal(7, 1590, 1, register_currency_capability); }
    if (true) { assume $DebugTrackLocal(7, 1590, 2, tc_capability); }

    // bytecode translation starts here
    // $t49 := move(association)
    call $tmp := $CopyOrMoveValue(association);
    $t49 := $tmp;

    // $t50 := move(register_currency_capability)
    call $tmp := $CopyOrMoveValue(register_currency_capability);
    $t50 := $tmp;

    // $t51 := move(tc_capability)
    call $tmp := $CopyOrMoveValue(tc_capability);
    $t51 := $tmp;

    // $t10 := copy($t49)
    call $tmp := $CopyOrMoveValue($t49);
    $t10 := $tmp;

    // $t11 := Signer::address_of($t10)
    call $t11 := $Signer_address_of($t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 1841);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t12 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 1883);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := ==($t11, $t12)
    $tmp := $Boolean($IsEqual($t11, $t12));
    $t13 := $tmp;

    // $t8 := $t13
    call $tmp := $CopyOrMoveValue($t13);
    $t8 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 1826, 8, $tmp); }

    // if ($t8) goto L0 else goto L1
    $tmp := $t8;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t15 := copy($t49)
    call $tmp := $CopyOrMoveValue($t49);
    $t15 := $tmp;

    // $t16 := move($t50)
    call $tmp := $CopyOrMoveValue($t50);
    $t16 := $tmp;

    // $t17 := 1
    $tmp := $Integer(1);
    $t17 := $tmp;

    // $t18 := 1
    $tmp := $Integer(1);
    $t18 := $tmp;

    // $t19 := FixedPoint32::create_from_rational($t17, $t18)
    call $t19 := $FixedPoint32_create_from_rational($t17, $t18);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 2109);
      goto Abort;
    }
    assume $FixedPoint32_FixedPoint32_is_well_formed($t19);


    // $t20 := true
    $tmp := $Boolean(true);
    $t20 := $tmp;

    // $t21 := 1000000
    $tmp := $Integer(1000000);
    $t21 := $tmp;

    // $t22 := 1000
    $tmp := $Integer(1000);
    $t22 := $tmp;

    // $t23 := [76, 66, 82]
    $tmp := $push_back_vector($push_back_vector($push_back_vector($mk_vector(), $Integer(76)), $Integer(66)), $Integer(82));
    $t23 := $tmp;

    // ($t24, $t25) := Libra::register_currency<LBR::LBR>($t15, $t16, $t19, $t20, $t21, $t22, $t23)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($LBR_LBR_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t24, $t25 := $Libra_register_currency($LBR_LBR_type_value(), $t15, $t16, $t19, $t20, $t21, $t22, $t23);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 1992);
      goto Abort;
    }
    assume $Libra_MintCapability_is_well_formed($t24);

    assume $Libra_BurnCapability_is_well_formed($t25);


    // burn_cap := $t25
    call $tmp := $CopyOrMoveValue($t25);
    burn_cap := $tmp;
    if (true) { assume $DebugTrackLocal(7, 1973, 3, $tmp); }

    // mint_cap := $t24
    call $tmp := $CopyOrMoveValue($t24);
    mint_cap := $tmp;
    if (true) { assume $DebugTrackLocal(7, 1963, 6, $tmp); }

    // $t26 := move($t51)
    call $tmp := $CopyOrMoveValue($t51);
    $t26 := $tmp;

    // $t27 := Libra::create_preburn<LBR::LBR>($t26)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($LBR_LBR_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t27 := $Libra_create_preburn($LBR_LBR_type_value(), $t26);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 2354);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($t27);


    // preburn_cap := $t27
    call $tmp := $CopyOrMoveValue($t27);
    preburn_cap := $tmp;
    if (true) { assume $DebugTrackLocal(7, 2333, 7, $tmp); }

    // $t28 := 1
    $tmp := $Integer(1);
    $t28 := $tmp;

    // $t29 := 2
    $tmp := $Integer(2);
    $t29 := $tmp;

    // $t30 := FixedPoint32::create_from_rational($t28, $t29)
    call $t30 := $FixedPoint32_create_from_rational($t28, $t29);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 2469);
      goto Abort;
    }
    assume $FixedPoint32_FixedPoint32_is_well_formed($t30);


    // $t31 := Libra::zero<Coin1::Coin1>()
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin1_Coin1_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t31 := $Libra_zero($Coin1_Coin1_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 2525);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t31);


    // $t32 := pack LBR::ReserveComponent<Coin1::Coin1>($t30, $t31)
    call $tmp := $LBR_ReserveComponent_pack(0, 0, 0, $Coin1_Coin1_type_value(), $t30, $t31);
    $t32 := $tmp;

    // coin1 := $t32
    call $tmp := $CopyOrMoveValue($t32);
    coin1 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 2402, 4, $tmp); }

    // $t33 := 1
    $tmp := $Integer(1);
    $t33 := $tmp;

    // $t34 := 2
    $tmp := $Integer(2);
    $t34 := $tmp;

    // $t35 := FixedPoint32::create_from_rational($t33, $t34)
    call $t35 := $FixedPoint32_create_from_rational($t33, $t34);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 2630);
      goto Abort;
    }
    assume $FixedPoint32_FixedPoint32_is_well_formed($t35);


    // $t36 := Libra::zero<Coin2::Coin2>()
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin2_Coin2_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t36 := $Libra_zero($Coin2_Coin2_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 2686);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t36);


    // $t37 := pack LBR::ReserveComponent<Coin2::Coin2>($t35, $t36)
    call $tmp := $LBR_ReserveComponent_pack(0, 0, 0, $Coin2_Coin2_type_value(), $t35, $t36);
    $t37 := $tmp;

    // coin2 := $t37
    call $tmp := $CopyOrMoveValue($t37);
    coin2 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 2563, 5, $tmp); }

    // $t38 := move($t49)
    call $tmp := $CopyOrMoveValue($t49);
    $t38 := $tmp;

    // $t44 := pack LBR::Reserve(mint_cap, burn_cap, preburn_cap, coin1, coin2)
    call $tmp := $LBR_Reserve_pack(0, 0, 0, mint_cap, burn_cap, preburn_cap, coin1, coin2);
    $t44 := $tmp;

    // move_to<LBR::Reserve>($t44, $t38)
    call $MoveTo($LBR_Reserve_type_value(), $t44, $t38);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 2720);
      goto Abort;
    }

    // return ()
    return;

    // L2:
L2:

    // $t45 := move($t51)
    call $tmp := $CopyOrMoveValue($t51);
    $t45 := $tmp;

    // destroy($t45)

    // $t46 := move($t50)
    call $tmp := $CopyOrMoveValue($t50);
    $t46 := $tmp;

    // destroy($t46)

    // $t47 := move($t49)
    call $tmp := $CopyOrMoveValue($t49);
    $t47 := $tmp;

    // destroy($t47)

    // $t48 := 0
    $tmp := $Integer(0);
    $t48 := $tmp;

    // abort($t48)
    if (true) { assume $DebugTrackAbort(7, 1826); }
    goto Abort;

Abort:
    $abort_flag := true;
    $m := $saved_m;
}

procedure {:inline 1} $LBR_create (amount_lbr: $Value, coin1: $Value, coin2: $Value) returns ($ret0: $Value, $ret1: $Value, $ret2: $Value)
free requires $IsValidU64(amount_lbr);
free requires $Libra_Libra_is_well_formed(coin1);
free requires $Libra_Libra_is_well_formed(coin2);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var coin1_exact: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var coin2_exact: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var num_coin1: $Value; // $IntegerType()
    var num_coin2: $Value; // $IntegerType()
    var reserve: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $BooleanType()
    var $t11: $Value; // $Libra_Libra_type_value($LBR_LBR_type_value())
    var $t12: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t13: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t14: $Value; // $AddressType()
    var $t15: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t19: $Value; // $LBR_ReserveComponent_type_value($Coin1_Coin1_type_value())
    var $t20: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t21: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t22: $Value; // $IntegerType()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $IntegerType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t27: $Value; // $LBR_ReserveComponent_type_value($Coin2_Coin2_type_value())
    var $t28: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t29: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t30: $Value; // $IntegerType()
    var $t31: $Value; // $IntegerType()
    var $t32: $Reference; // ReferenceType($Libra_Libra_type_value($Coin1_Coin1_type_value()))
    var $t33: $Value; // $IntegerType()
    var $t34: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t35: $Reference; // ReferenceType($Libra_Libra_type_value($Coin2_Coin2_type_value()))
    var $t36: $Value; // $IntegerType()
    var $t37: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t38: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t39: $Reference; // ReferenceType($LBR_ReserveComponent_type_value($Coin1_Coin1_type_value()))
    var $t40: $Reference; // ReferenceType($Libra_Libra_type_value($Coin1_Coin1_type_value()))
    var $t41: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t42: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t43: $Reference; // ReferenceType($LBR_ReserveComponent_type_value($Coin2_Coin2_type_value()))
    var $t44: $Reference; // ReferenceType($Libra_Libra_type_value($Coin2_Coin2_type_value()))
    var $t45: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t46: $Value; // $IntegerType()
    var $t47: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t48: $Value; // $Libra_MintCapability_type_value($LBR_LBR_type_value())
    var $t49: $Value; // $Libra_Libra_type_value($LBR_LBR_type_value())
    var $t50: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t51: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t52: $Value; // $IntegerType()
    var $t53: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t54: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t55: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t56: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(7, 4170, 0, amount_lbr); }
    if (true) { assume $DebugTrackLocal(7, 4170, 1, coin1); }
    if (true) { assume $DebugTrackLocal(7, 4170, 2, coin2); }

    // bytecode translation starts here
    // $t52 := move(amount_lbr)
    call $tmp := $CopyOrMoveValue(amount_lbr);
    $t52 := $tmp;

    // $t53 := move(coin1)
    call $tmp := $CopyOrMoveValue(coin1);
    $t53 := $tmp;

    // $t54 := move(coin2)
    call $tmp := $CopyOrMoveValue(coin2);
    $t54 := $tmp;

    // $t9 := 0
    $tmp := $Integer(0);
    $t9 := $tmp;

    // $t10 := ==($t52, $t9)
    $tmp := $Boolean($IsEqual($t52, $t9));
    $t10 := $tmp;

    // if ($t10) goto L0 else goto L1
    $tmp := $t10;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t11 := Libra::zero<LBR::LBR>()
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($LBR_LBR_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t11 := $Libra_zero($LBR_LBR_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4386);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t11);


    // return ($t11, $t53, $t54)
    $ret0 := $t11;
    if (true) { assume $DebugTrackLocal(7, 4371, 57, $ret0); }
    $ret1 := $t53;
    if (true) { assume $DebugTrackLocal(7, 4371, 58, $ret1); }
    $ret2 := $t54;
    if (true) { assume $DebugTrackLocal(7, 4371, 59, $ret2); }
    return;

    // L2:
L2:

    // $t14 := CoreAddresses::ASSOCIATION_ROOT_ADDRESS()
    call $t14 := $CoreAddresses_ASSOCIATION_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4478);
      goto Abort;
    }
    assume is#$Address($t14);


    // $t15 := borrow_global<LBR::Reserve>($t14)
    call $t15 := $BorrowGlobal($t14, $LBR_Reserve_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4436);
      goto Abort;
    }
    assume $LBR_Reserve_is_well_formed($Dereference($t15));

    // UnpackRef($t15)
    call $LBR_Reserve_before_update_inv($Dereference($t15));

    // reserve := $t15
    call reserve := $CopyOrMoveRef($t15);
    if (true) { assume $DebugTrackLocal(7, 4426, 7, $Dereference(reserve)); }

    // $t16 := 1
    $tmp := $Integer(1);
    $t16 := $tmp;

    // $t18 := copy(reserve)
    call $t18 := $CopyOrMoveRef(reserve);

    // $t19 := get_field<LBR::Reserve>.coin1($t18)
    call $tmp := $GetFieldFromReference($t18, $LBR_Reserve_coin1);
    assume $LBR_ReserveComponent_is_well_formed($tmp);
    $t19 := $tmp;

    // Reference(reserve) <- $t18
    call reserve := $WritebackToReference($t18, reserve);

    // $t20 := get_field<LBR::ReserveComponent<Coin1::Coin1>>.ratio($t19)
    call $tmp := $GetFieldFromValue($t19, $LBR_ReserveComponent_ratio);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t20 := $tmp;

    // $t21 := move($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t21 := $tmp;

    // $t22 := FixedPoint32::multiply_u64($t52, $t21)
    call $t22 := $FixedPoint32_multiply_u64($t52, $t21);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4549);
      goto Abort;
    }
    assume $IsValidU64($t22);


    // $t23 := +($t16, $t22)
    call $tmp := $AddU64($t16, $t22);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4533);
      goto Abort;
    }
    $t23 := $tmp;

    // num_coin1 := $t23
    call $tmp := $CopyOrMoveValue($t23);
    num_coin1 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 4519, 5, $tmp); }

    // $t24 := 1
    $tmp := $Integer(1);
    $t24 := $tmp;

    // $t26 := copy(reserve)
    call $t26 := $CopyOrMoveRef(reserve);

    // $t27 := get_field<LBR::Reserve>.coin2($t26)
    call $tmp := $GetFieldFromReference($t26, $LBR_Reserve_coin2);
    assume $LBR_ReserveComponent_is_well_formed($tmp);
    $t27 := $tmp;

    // Reference(reserve) <- $t26
    call reserve := $WritebackToReference($t26, reserve);

    // $t28 := get_field<LBR::ReserveComponent<Coin2::Coin2>>.ratio($t27)
    call $tmp := $GetFieldFromValue($t27, $LBR_ReserveComponent_ratio);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t28 := $tmp;

    // $t29 := move($t28)
    call $tmp := $CopyOrMoveValue($t28);
    $t29 := $tmp;

    // $t30 := FixedPoint32::multiply_u64($t52, $t29)
    call $t30 := $FixedPoint32_multiply_u64($t52, $t29);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4640);
      goto Abort;
    }
    assume $IsValidU64($t30);


    // $t31 := +($t24, $t30)
    call $tmp := $AddU64($t24, $t30);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4624);
      goto Abort;
    }
    $t31 := $tmp;

    // num_coin2 := $t31
    call $tmp := $CopyOrMoveValue($t31);
    num_coin2 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 4610, 6, $tmp); }

    // $t32 := borrow_local($t53)
    call $t32 := $BorrowLoc(53, $t53);
    assume $Libra_Libra_is_well_formed($Dereference($t32));

    // UnpackRef($t32)
    call $Libra_Libra_before_update_inv($Coin1_Coin1_type_value(), $Dereference($t32));

    // PackRef($t32)
    call $Libra_Libra_after_update_inv($Coin1_Coin1_type_value(), $Dereference($t32));

    // $t55 := read_ref($t32)
    call $tmp := $ReadRef($t32);
    assume $Libra_Libra_is_well_formed($tmp);
    $t55 := $tmp;

    // ($t34, $t55) := Libra::withdraw<Coin1::Coin1>($t55, num_coin1)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin1_Coin1_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t34, $t55 := $Libra_withdraw($Coin1_Coin1_type_value(), $t55, num_coin1);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4722);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t34);

    assume $Libra_Libra_is_well_formed($t55);


    // write_ref($t32, $t55)
    call $t32 := $WriteRef($t32, $t55);
    if (true) { assume $DebugTrackLocal(7, 4901, 7, $Dereference(reserve)); }

    // LocalRoot($t53) <- $t32
    call $t53 := $WritebackToValue($t32, 53, $t53);

    // UnpackRef($t32)
    call $Libra_Libra_before_update_inv($Coin1_Coin1_type_value(), $Dereference($t32));

    // PackRef($t32)
    call $Libra_Libra_after_update_inv($Coin1_Coin1_type_value(), $Dereference($t32));

    // coin1_exact := $t34
    call $tmp := $CopyOrMoveValue($t34);
    coin1_exact := $tmp;
    if (true) { assume $DebugTrackLocal(7, 4701, 3, $tmp); }

    // $t35 := borrow_local($t54)
    call $t35 := $BorrowLoc(54, $t54);
    assume $Libra_Libra_is_well_formed($Dereference($t35));

    // UnpackRef($t35)
    call $Libra_Libra_before_update_inv($Coin2_Coin2_type_value(), $Dereference($t35));

    // PackRef($t35)
    call $Libra_Libra_after_update_inv($Coin2_Coin2_type_value(), $Dereference($t35));

    // $t56 := read_ref($t35)
    call $tmp := $ReadRef($t35);
    assume $Libra_Libra_is_well_formed($tmp);
    $t56 := $tmp;

    // ($t37, $t56) := Libra::withdraw<Coin2::Coin2>($t56, num_coin2)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin2_Coin2_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t37, $t56 := $Libra_withdraw($Coin2_Coin2_type_value(), $t56, num_coin2);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4788);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t37);

    assume $Libra_Libra_is_well_formed($t56);


    // write_ref($t35, $t56)
    call $t35 := $WriteRef($t35, $t56);
    if (true) { assume $DebugTrackLocal(7, 5006, 7, $Dereference(reserve)); }

    // LocalRoot($t54) <- $t35
    call $t54 := $WritebackToValue($t35, 54, $t54);

    // UnpackRef($t35)
    call $Libra_Libra_before_update_inv($Coin2_Coin2_type_value(), $Dereference($t35));

    // PackRef($t35)
    call $Libra_Libra_after_update_inv($Coin2_Coin2_type_value(), $Dereference($t35));

    // coin2_exact := $t37
    call $tmp := $CopyOrMoveValue($t37);
    coin2_exact := $tmp;
    if (true) { assume $DebugTrackLocal(7, 4767, 4, $tmp); }

    // $t38 := copy(reserve)
    call $t38 := $CopyOrMoveRef(reserve);

    // $t39 := borrow_field<LBR::Reserve>.coin1($t38)
    call $t39 := $BorrowField($t38, $LBR_Reserve_coin1);
    assume $LBR_ReserveComponent_is_well_formed_types($Dereference($t39));

    // Reference(reserve) <- $t38
    call reserve := $WritebackToReference($t38, reserve);

    // UnpackRef($t39)
    call $LBR_ReserveComponent_before_update_inv($Coin1_Coin1_type_value(), $Dereference($t39));

    // $t40 := borrow_field<LBR::ReserveComponent<Coin1::Coin1>>.backing($t39)
    call $t40 := $BorrowField($t39, $LBR_ReserveComponent_backing);
    assume $Libra_Libra_is_well_formed_types($Dereference($t40));

    // Reference(reserve) <- $t39
    call reserve := $WritebackToReference($t39, reserve);

    // Reference($t38) <- $t39
    call $t38 := $WritebackToReference($t39, $t38);

    // UnpackRef($t40)
    call $Libra_Libra_before_update_inv($Coin1_Coin1_type_value(), $Dereference($t40));

    // PackRef($t40)
    call $Libra_Libra_after_update_inv($Coin1_Coin1_type_value(), $Dereference($t40));

    // $t55 := read_ref($t40)
    call $tmp := $ReadRef($t40);
    assume $Libra_Libra_is_well_formed($tmp);
    $t55 := $tmp;

    // $t55 := Libra::deposit<Coin1::Coin1>($t55, coin1_exact)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin1_Coin1_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t55 := $Libra_deposit($Coin1_Coin1_type_value(), $t55, coin1_exact);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4836);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t55);


    // write_ref($t40, $t55)
    call $t40 := $WriteRef($t40, $t55);
    if (true) { assume $DebugTrackLocal(7, 4967, 7, $Dereference(reserve)); }

    // Reference(reserve) <- $t40
    call reserve := $WritebackToReference($t40, reserve);

    // Reference($t39) <- $t40
    call $t39 := $WritebackToReference($t40, $t39);

    // Reference($t38) <- $t39
    call $t38 := $WritebackToReference($t39, $t38);

    // UnpackRef($t40)
    call $Libra_Libra_before_update_inv($Coin1_Coin1_type_value(), $Dereference($t40));

    // PackRef($t39)
    call $LBR_ReserveComponent_after_update_inv($Coin1_Coin1_type_value(), $Dereference($t39));

    // PackRef($t40)
    call $Libra_Libra_after_update_inv($Coin1_Coin1_type_value(), $Dereference($t40));

    // $t42 := copy(reserve)
    call $t42 := $CopyOrMoveRef(reserve);

    // $t43 := borrow_field<LBR::Reserve>.coin2($t42)
    call $t43 := $BorrowField($t42, $LBR_Reserve_coin2);
    assume $LBR_ReserveComponent_is_well_formed_types($Dereference($t43));

    // Reference(reserve) <- $t42
    call reserve := $WritebackToReference($t42, reserve);

    // UnpackRef($t43)
    call $LBR_ReserveComponent_before_update_inv($Coin2_Coin2_type_value(), $Dereference($t43));

    // $t44 := borrow_field<LBR::ReserveComponent<Coin2::Coin2>>.backing($t43)
    call $t44 := $BorrowField($t43, $LBR_ReserveComponent_backing);
    assume $Libra_Libra_is_well_formed_types($Dereference($t44));

    // Reference(reserve) <- $t43
    call reserve := $WritebackToReference($t43, reserve);

    // Reference($t42) <- $t43
    call $t42 := $WritebackToReference($t43, $t42);

    // UnpackRef($t44)
    call $Libra_Libra_before_update_inv($Coin2_Coin2_type_value(), $Dereference($t44));

    // PackRef($t44)
    call $Libra_Libra_after_update_inv($Coin2_Coin2_type_value(), $Dereference($t44));

    // $t56 := read_ref($t44)
    call $tmp := $ReadRef($t44);
    assume $Libra_Libra_is_well_formed($tmp);
    $t56 := $tmp;

    // $t56 := Libra::deposit<Coin2::Coin2>($t56, coin2_exact)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin2_Coin2_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t56 := $Libra_deposit($Coin2_Coin2_type_value(), $t56, coin2_exact);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4901);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t56);


    // write_ref($t44, $t56)
    call $t44 := $WriteRef($t44, $t56);
    if (true) { assume $DebugTrackLocal(7, 5032, 7, $Dereference(reserve)); }

    // Reference(reserve) <- $t44
    call reserve := $WritebackToReference($t44, reserve);

    // Reference($t43) <- $t44
    call $t43 := $WritebackToReference($t44, $t43);

    // Reference($t42) <- $t43
    call $t42 := $WritebackToReference($t43, $t42);

    // UnpackRef($t44)
    call $Libra_Libra_before_update_inv($Coin2_Coin2_type_value(), $Dereference($t44));

    // PackRef($t43)
    call $LBR_ReserveComponent_after_update_inv($Coin2_Coin2_type_value(), $Dereference($t43));

    // PackRef($t44)
    call $Libra_Libra_after_update_inv($Coin2_Coin2_type_value(), $Dereference($t44));

    // $t47 := move(reserve)
    call $t47 := $CopyOrMoveRef(reserve);

    // $t48 := get_field<LBR::Reserve>.mint_cap($t47)
    call $tmp := $GetFieldFromReference($t47, $LBR_Reserve_mint_cap);
    assume $Libra_MintCapability_is_well_formed($tmp);
    $t48 := $tmp;

    // LBR::Reserve <- $t47
    call $WritebackToGlobal($t47);

    // PackRef($t47)
    call $LBR_Reserve_after_update_inv($Dereference($t47));

    // $t49 := Libra::mint_with_capability<LBR::LBR>($t52, $t48)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($LBR_LBR_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t49 := $Libra_mint_with_capability($LBR_LBR_type_value(), $t52, $t48);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4967);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t49);


    // return ($t49, $t53, $t54)
    $ret0 := $t49;
    if (true) { assume $DebugTrackLocal(7, 4959, 57, $ret0); }
    $ret1 := $t53;
    if (true) { assume $DebugTrackLocal(7, 4959, 58, $ret1); }
    $ret2 := $t54;
    if (true) { assume $DebugTrackLocal(7, 4959, 59, $ret2); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
    $ret2 := $DefaultValue();
}

procedure {:inline 1} $LBR_mint (account: $Value, amount_lbr: $Value) returns ($ret0: $Value)
free requires is#$Address(account);
free requires $IsValidU64(amount_lbr);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var coin1: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var coin2: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var lbr: $Value; // $Libra_Libra_type_value($LBR_LBR_type_value())
    var leftover1: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var leftover2: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var num_coin1: $Value; // $IntegerType()
    var num_coin2: $Value; // $IntegerType()
    var reserve: $Value; // $LBR_Reserve_type_value()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $LBR_Reserve_type_value()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $LBR_Reserve_type_value()
    var $t15: $Value; // $LBR_ReserveComponent_type_value($Coin1_Coin1_type_value())
    var $t16: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t17: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $IntegerType()
    var $t22: $Value; // $LBR_Reserve_type_value()
    var $t23: $Value; // $LBR_ReserveComponent_type_value($Coin2_Coin2_type_value())
    var $t24: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t25: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t26: $Value; // $IntegerType()
    var $t27: $Value; // $IntegerType()
    var $t28: $Value; // $AddressType()
    var $t29: $Value; // $IntegerType()
    var $t30: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t31: $Value; // $AddressType()
    var $t32: $Value; // $IntegerType()
    var $t33: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t34: $Value; // $IntegerType()
    var $t35: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t36: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t37: $Value; // $Libra_Libra_type_value($LBR_LBR_type_value())
    var $t38: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t39: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t40: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t41: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t42: $Value; // $Libra_Libra_type_value($LBR_LBR_type_value())
    var $t43: $Value; // $AddressType()
    var $t44: $Value; // $IntegerType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(7, 6168, 0, account); }
    if (true) { assume $DebugTrackLocal(7, 6168, 1, amount_lbr); }

    // bytecode translation starts here
    // $t43 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t43 := $tmp;

    // $t44 := move(amount_lbr)
    call $tmp := $CopyOrMoveValue(amount_lbr);
    $t44 := $tmp;

    // $t10 := CoreAddresses::ASSOCIATION_ROOT_ADDRESS()
    call $t10 := $CoreAddresses_ASSOCIATION_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 6310);
      goto Abort;
    }
    assume is#$Address($t10);


    // $t11 := get_global<LBR::Reserve>($t10)
    call $tmp := $GetGlobal($t10, $LBR_Reserve_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 6272);
      goto Abort;
    }
    assume $LBR_Reserve_is_well_formed($tmp);
    $t11 := $tmp;

    // reserve := $t11
    call $tmp := $CopyOrMoveValue($t11);
    reserve := $tmp;
    if (true) { assume $DebugTrackLocal(7, 6262, 9, $tmp); }

    // $t12 := 1
    $tmp := $Integer(1);
    $t12 := $tmp;

    // $t14 := copy(reserve)
    call $tmp := $CopyOrMoveValue(reserve);
    $t14 := $tmp;

    // $t15 := get_field<LBR::Reserve>.coin1($t14)
    call $tmp := $GetFieldFromValue($t14, $LBR_Reserve_coin1);
    assume $LBR_ReserveComponent_is_well_formed($tmp);
    $t15 := $tmp;

    // $t16 := get_field<LBR::ReserveComponent<Coin1::Coin1>>.ratio($t15)
    call $tmp := $GetFieldFromValue($t15, $LBR_ReserveComponent_ratio);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t16 := $tmp;

    // $t17 := move($t16)
    call $tmp := $CopyOrMoveValue($t16);
    $t17 := $tmp;

    // $t18 := FixedPoint32::multiply_u64($t44, $t17)
    call $t18 := $FixedPoint32_multiply_u64($t44, $t17);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 6381);
      goto Abort;
    }
    assume $IsValidU64($t18);


    // $t19 := +($t12, $t18)
    call $tmp := $AddU64($t12, $t18);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 6365);
      goto Abort;
    }
    $t19 := $tmp;

    // num_coin1 := $t19
    call $tmp := $CopyOrMoveValue($t19);
    num_coin1 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 6351, 7, $tmp); }

    // $t20 := 1
    $tmp := $Integer(1);
    $t20 := $tmp;

    // $t22 := move(reserve)
    call $tmp := $CopyOrMoveValue(reserve);
    $t22 := $tmp;

    // $t23 := get_field<LBR::Reserve>.coin2($t22)
    call $tmp := $GetFieldFromValue($t22, $LBR_Reserve_coin2);
    assume $LBR_ReserveComponent_is_well_formed($tmp);
    $t23 := $tmp;

    // $t24 := get_field<LBR::ReserveComponent<Coin2::Coin2>>.ratio($t23)
    call $tmp := $GetFieldFromValue($t23, $LBR_ReserveComponent_ratio);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t24 := $tmp;

    // $t25 := move($t24)
    call $tmp := $CopyOrMoveValue($t24);
    $t25 := $tmp;

    // $t26 := FixedPoint32::multiply_u64($t44, $t25)
    call $t26 := $FixedPoint32_multiply_u64($t44, $t25);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 6472);
      goto Abort;
    }
    assume $IsValidU64($t26);


    // $t27 := +($t20, $t26)
    call $tmp := $AddU64($t20, $t26);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 6456);
      goto Abort;
    }
    $t27 := $tmp;

    // num_coin2 := $t27
    call $tmp := $CopyOrMoveValue($t27);
    num_coin2 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 6442, 8, $tmp); }

    // $t28 := copy($t43)
    call $tmp := $CopyOrMoveValue($t43);
    $t28 := $tmp;

    // $t30 := Libra::mint<Coin1::Coin1>($t28, num_coin1)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin1_Coin1_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t30 := $Libra_mint($Coin1_Coin1_type_value(), $t28, num_coin1);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 6548);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t30);


    // coin1 := $t30
    call $tmp := $CopyOrMoveValue($t30);
    coin1 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 6533, 2, $tmp); }

    // $t31 := move($t43)
    call $tmp := $CopyOrMoveValue($t43);
    $t31 := $tmp;

    // $t33 := Libra::mint<Coin2::Coin2>($t31, num_coin2)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin2_Coin2_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t33 := $Libra_mint($Coin2_Coin2_type_value(), $t31, num_coin2);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 6608);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t33);


    // coin2 := $t33
    call $tmp := $CopyOrMoveValue($t33);
    coin2 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 6593, 3, $tmp); }

    // ($t37, $t38, $t39) := LBR::create($t44, coin1, coin2)
    call $t37, $t38, $t39 := $LBR_create($t44, coin1, coin2);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4181);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t37);

    assume $Libra_Libra_is_well_formed($t38);

    assume $Libra_Libra_is_well_formed($t39);


    // leftover2 := $t39
    call $tmp := $CopyOrMoveValue($t39);
    leftover2 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 6670, 6, $tmp); }

    // leftover1 := $t38
    call $tmp := $CopyOrMoveValue($t38);
    leftover1 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 6659, 5, $tmp); }

    // lbr := $t37
    call $tmp := $CopyOrMoveValue($t37);
    lbr := $tmp;
    if (true) { assume $DebugTrackLocal(7, 6654, 4, $tmp); }

    // Libra::destroy_zero<Coin1::Coin1>(leftover1)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin1_Coin1_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $Libra_destroy_zero($Coin1_Coin1_type_value(), leftover1);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 6732);
      goto Abort;
    }

    // Libra::destroy_zero<Coin2::Coin2>(leftover2)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin2_Coin2_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $Libra_destroy_zero($Coin2_Coin2_type_value(), leftover2);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 6772);
      goto Abort;
    }

    // return lbr
    $ret0 := lbr;
    if (true) { assume $DebugTrackLocal(7, 6805, 45, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $LBR_is_lbr ($tv0: $TypeValue) returns ($ret0: $Value)
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var $t0: $Value; // $BooleanType()
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $Vector_type_value($IntegerType())
    var $t3: $Value; // $Vector_type_value($IntegerType())
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $BooleanType()
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t1 := Libra::is_currency<#0>()
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t1 := $Libra_is_currency($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 2909);
      goto Abort;
    }
    assume is#$Boolean($t1);


    // if ($t1) goto L0 else goto L1
    $tmp := $t1;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t2 := Libra::currency_code<#0>()
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t2 := $Libra_currency_code($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 2955);
      goto Abort;
    }
    assume $Vector_is_well_formed($t2) && (forall $$0: int :: {$select_vector($t2,$$0)} $$0 >= 0 && $$0 < $vlen($t2) ==> $IsValidU8($select_vector($t2,$$0)));


    // $t3 := Libra::currency_code<LBR::LBR>()
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($LBR_LBR_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t3 := $Libra_currency_code($LBR_LBR_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 2991);
      goto Abort;
    }
    assume $Vector_is_well_formed($t3) && (forall $$0: int :: {$select_vector($t3,$$0)} $$0 >= 0 && $$0 < $vlen($t3) ==> $IsValidU8($select_vector($t3,$$0)));


    // $t4 := ==($t2, $t3)
    $tmp := $Boolean($IsEqual($t2, $t3));
    $t4 := $tmp;

    // $t0 := $t4
    call $tmp := $CopyOrMoveValue($t4);
    $t0 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 2902, 0, $tmp); }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t5 := false
    $tmp := $Boolean(false);
    $t5 := $tmp;

    // $t0 := $t5
    call $tmp := $CopyOrMoveValue($t5);
    $t0 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 2902, 0, $tmp); }

    // goto L3
    goto L3;

    // L3:
L3:

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(7, 2902, 7, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $LBR_swap_into (coin1: $Value, coin2: $Value) returns ($ret0: $Value, $ret1: $Value, $ret2: $Value)
free requires $Libra_Libra_is_well_formed(coin1);
free requires $Libra_Libra_is_well_formed(coin2);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var coin1_value: $Value; // $IntegerType()
    var coin2_value: $Value; // $IntegerType()
    var lbr_num_coin1: $Value; // $IntegerType()
    var lbr_num_coin2: $Value; // $IntegerType()
    var num_lbr: $Value; // $IntegerType()
    var reserve: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $AddressType()
    var $t11: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t12: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $BooleanType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $IntegerType()
    var $t22: $Value; // $BooleanType()
    var $t23: $Value; // $BooleanType()
    var $t24: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t25: $Value; // $Libra_Libra_type_value($LBR_LBR_type_value())
    var $t26: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t27: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t28: $Value; // $IntegerType()
    var $t29: $Value; // $IntegerType()
    var $t30: $Value; // $IntegerType()
    var $t31: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t32: $Value; // $LBR_ReserveComponent_type_value($Coin1_Coin1_type_value())
    var $t33: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t34: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t35: $Value; // $IntegerType()
    var $t36: $Value; // $IntegerType()
    var $t37: $Value; // $IntegerType()
    var $t38: $Value; // $IntegerType()
    var $t39: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t40: $Value; // $LBR_ReserveComponent_type_value($Coin2_Coin2_type_value())
    var $t41: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t42: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t43: $Value; // $IntegerType()
    var $t44: $Value; // $IntegerType()
    var $t45: $Value; // $IntegerType()
    var $t46: $Value; // $BooleanType()
    var $t47: $Value; // $IntegerType()
    var $t48: $Value; // $IntegerType()
    var $t49: $Value; // $IntegerType()
    var $t50: $Value; // $IntegerType()
    var $t51: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t52: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t53: $Value; // $Libra_Libra_type_value($LBR_LBR_type_value())
    var $t54: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t55: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t56: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t57: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(7, 3178, 0, coin1); }
    if (true) { assume $DebugTrackLocal(7, 3178, 1, coin2); }

    // bytecode translation starts here
    // $t56 := move(coin1)
    call $tmp := $CopyOrMoveValue(coin1);
    $t56 := $tmp;

    // $t57 := move(coin2)
    call $tmp := $CopyOrMoveValue(coin2);
    $t57 := $tmp;

    // $t10 := CoreAddresses::ASSOCIATION_ROOT_ADDRESS()
    call $t10 := $CoreAddresses_ASSOCIATION_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 3392);
      goto Abort;
    }
    assume is#$Address($t10);


    // $t11 := borrow_global<LBR::Reserve>($t10)
    call $t11 := $BorrowGlobal($t10, $LBR_Reserve_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 3350);
      goto Abort;
    }
    assume $LBR_Reserve_is_well_formed($Dereference($t11));

    // UnpackRef($t11)
    call $LBR_Reserve_before_update_inv($Dereference($t11));

    // reserve := $t11
    call reserve := $CopyOrMoveRef($t11);
    if (true) { assume $DebugTrackLocal(7, 3340, 7, $Dereference(reserve)); }

    // $t12 := copy($t56)
    call $tmp := $CopyOrMoveValue($t56);
    $t12 := $tmp;

    // $t13 := Libra::value<Coin1::Coin1>($t12)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin1_Coin1_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t13 := $Libra_value($Coin1_Coin1_type_value(), $t12);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 3454);
      goto Abort;
    }
    assume $IsValidU64($t13);


    // coin1_value := $t13
    call $tmp := $CopyOrMoveValue($t13);
    coin1_value := $tmp;
    if (true) { assume $DebugTrackLocal(7, 3433, 2, $tmp); }

    // $t14 := copy($t57)
    call $tmp := $CopyOrMoveValue($t57);
    $t14 := $tmp;

    // $t15 := Libra::value<Coin2::Coin2>($t14)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin2_Coin2_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t15 := $Libra_value($Coin2_Coin2_type_value(), $t14);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 3502);
      goto Abort;
    }
    assume $IsValidU64($t15);


    // coin2_value := $t15
    call $tmp := $CopyOrMoveValue($t15);
    coin2_value := $tmp;
    if (true) { assume $DebugTrackLocal(7, 3481, 3, $tmp); }

    // $t17 := 1
    $tmp := $Integer(1);
    $t17 := $tmp;

    // $t18 := <=(coin1_value, $t17)
    call $tmp := $Le(coin1_value, $t17);
    $t18 := $tmp;

    // if ($t18) goto L0 else goto L1
    $tmp := $t18;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t19 := true
    $tmp := $Boolean(true);
    $t19 := $tmp;

    // $t8 := $t19
    call $tmp := $CopyOrMoveValue($t19);
    $t8 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 3529, 8, $tmp); }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t21 := 1
    $tmp := $Integer(1);
    $t21 := $tmp;

    // $t22 := <=(coin2_value, $t21)
    call $tmp := $Le(coin2_value, $t21);
    $t22 := $tmp;

    // $t8 := $t22
    call $tmp := $CopyOrMoveValue($t22);
    $t8 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 3529, 8, $tmp); }

    // goto L3
    goto L3;

    // L3:
L3:

    // if ($t8) goto L4 else goto L5
    $tmp := $t8;
    if (b#$Boolean($tmp)) { goto L4; } else { goto L5; }

    // L5:
L5:

    // goto L6
    goto L6;

    // L4:
L4:

    // $t24 := move(reserve)
    call $t24 := $CopyOrMoveRef(reserve);

    // destroy($t24)

    // LBR::Reserve <- $t24
    call $WritebackToGlobal($t24);

    // PackRef($t24)
    call $LBR_Reserve_after_update_inv($Dereference($t24));

    // $t25 := Libra::zero<LBR::LBR>()
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($LBR_LBR_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t25 := $Libra_zero($LBR_LBR_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 3582);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t25);


    // return ($t25, $t56, $t57)
    $ret0 := $t25;
    if (true) { assume $DebugTrackLocal(7, 3567, 58, $ret0); }
    $ret1 := $t56;
    if (true) { assume $DebugTrackLocal(7, 3567, 59, $ret1); }
    $ret2 := $t57;
    if (true) { assume $DebugTrackLocal(7, 3567, 60, $ret2); }
    return;

    // L6:
L6:

    // $t29 := 1
    $tmp := $Integer(1);
    $t29 := $tmp;

    // $t30 := -(coin1_value, $t29)
    call $tmp := $Sub(coin1_value, $t29);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 3675);
      goto Abort;
    }
    $t30 := $tmp;

    // $t31 := copy(reserve)
    call $t31 := $CopyOrMoveRef(reserve);

    // $t32 := get_field<LBR::Reserve>.coin1($t31)
    call $tmp := $GetFieldFromReference($t31, $LBR_Reserve_coin1);
    assume $LBR_ReserveComponent_is_well_formed($tmp);
    $t32 := $tmp;

    // Reference(reserve) <- $t31
    call reserve := $WritebackToReference($t31, reserve);

    // $t33 := get_field<LBR::ReserveComponent<Coin1::Coin1>>.ratio($t32)
    call $tmp := $GetFieldFromValue($t32, $LBR_ReserveComponent_ratio);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t33 := $tmp;

    // $t34 := move($t33)
    call $tmp := $CopyOrMoveValue($t33);
    $t34 := $tmp;

    // $t35 := FixedPoint32::divide_u64($t30, $t34)
    call $t35 := $FixedPoint32_divide_u64($t30, $t34);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 3652);
      goto Abort;
    }
    assume $IsValidU64($t35);


    // lbr_num_coin1 := $t35
    call $tmp := $CopyOrMoveValue($t35);
    lbr_num_coin1 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 3622, 4, $tmp); }

    // $t37 := 1
    $tmp := $Integer(1);
    $t37 := $tmp;

    // $t38 := -(coin2_value, $t37)
    call $tmp := $Sub(coin2_value, $t37);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 3769);
      goto Abort;
    }
    $t38 := $tmp;

    // $t39 := move(reserve)
    call $t39 := $CopyOrMoveRef(reserve);

    // $t40 := get_field<LBR::Reserve>.coin2($t39)
    call $tmp := $GetFieldFromReference($t39, $LBR_Reserve_coin2);
    assume $LBR_ReserveComponent_is_well_formed($tmp);
    $t40 := $tmp;

    // LBR::Reserve <- $t39
    call $WritebackToGlobal($t39);

    // PackRef($t39)
    call $LBR_Reserve_after_update_inv($Dereference($t39));

    // $t41 := get_field<LBR::ReserveComponent<Coin2::Coin2>>.ratio($t40)
    call $tmp := $GetFieldFromValue($t40, $LBR_ReserveComponent_ratio);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t41 := $tmp;

    // $t42 := move($t41)
    call $tmp := $CopyOrMoveValue($t41);
    $t42 := $tmp;

    // $t43 := FixedPoint32::divide_u64($t38, $t42)
    call $t43 := $FixedPoint32_divide_u64($t38, $t42);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 3746);
      goto Abort;
    }
    assume $IsValidU64($t43);


    // lbr_num_coin2 := $t43
    call $tmp := $CopyOrMoveValue($t43);
    lbr_num_coin2 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 3716, 5, $tmp); }

    // $t46 := <(lbr_num_coin2, lbr_num_coin1)
    call $tmp := $Lt(lbr_num_coin2, lbr_num_coin1);
    $t46 := $tmp;

    // if ($t46) goto L7 else goto L8
    $tmp := $t46;
    if (b#$Boolean($tmp)) { goto L7; } else { goto L8; }

    // L8:
L8:

    // goto L9
    goto L9;

    // L7:
L7:

    // $t9 := lbr_num_coin2
    call $tmp := $CopyOrMoveValue(lbr_num_coin2);
    $t9 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 3820, 9, $tmp); }

    // goto L10
    goto L10;

    // L9:
L9:

    // $t9 := lbr_num_coin1
    call $tmp := $CopyOrMoveValue(lbr_num_coin1);
    $t9 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 3820, 9, $tmp); }

    // goto L10
    goto L10;

    // L10:
L10:

    // num_lbr := $t9
    call $tmp := $CopyOrMoveValue($t9);
    num_lbr := $tmp;
    if (true) { assume $DebugTrackLocal(7, 3810, 6, $tmp); }

    // ($t53, $t54, $t55) := LBR::create(num_lbr, $t56, $t57)
    call $t53, $t54, $t55 := $LBR_create(num_lbr, $t56, $t57);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4181);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t53);

    assume $Libra_Libra_is_well_formed($t54);

    assume $Libra_Libra_is_well_formed($t55);


    // return ($t53, $t54, $t55)
    $ret0 := $t53;
    if (true) { assume $DebugTrackLocal(7, 3945, 58, $ret0); }
    $ret1 := $t54;
    if (true) { assume $DebugTrackLocal(7, 3945, 59, $ret1); }
    $ret2 := $t55;
    if (true) { assume $DebugTrackLocal(7, 3945, 60, $ret2); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
    $ret2 := $DefaultValue();
}

procedure {:inline 1} $LBR_unpack (account: $Value, coin: $Value) returns ($ret0: $Value, $ret1: $Value)
free requires is#$Address(account);
free requires $Libra_Libra_is_well_formed(coin);
requires $ExistsTxnSenderAccount($m, $txn);
{
    // declare local variables
    var coin1: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var coin1_amount: $Value; // $IntegerType()
    var coin2: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var coin2_amount: $Value; // $IntegerType()
    var ratio_multiplier: $Value; // $IntegerType()
    var reserve: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var sender: $Value; // $AddressType()
    var $t9: $Value; // $AddressType()
    var $t10: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t11: $Value; // $Libra_Libra_type_value($LBR_LBR_type_value())
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $Libra_Libra_type_value($LBR_LBR_type_value())
    var $t16: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t17: $Reference; // ReferenceType($Libra_Preburn_type_value($LBR_LBR_type_value()))
    var $t18: $Value; // $AddressType()
    var $t19: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t20: $Reference; // ReferenceType($Libra_Preburn_type_value($LBR_LBR_type_value()))
    var $t21: $Value; // $AddressType()
    var $t22: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t23: $Value; // $Libra_BurnCapability_type_value($LBR_LBR_type_value())
    var $t24: $Value; // $IntegerType()
    var $t25: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t26: $Value; // $LBR_ReserveComponent_type_value($Coin1_Coin1_type_value())
    var $t27: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t28: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t29: $Value; // $IntegerType()
    var $t30: $Value; // $IntegerType()
    var $t31: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t32: $Value; // $LBR_ReserveComponent_type_value($Coin2_Coin2_type_value())
    var $t33: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t34: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t35: $Value; // $IntegerType()
    var $t36: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t37: $Reference; // ReferenceType($LBR_ReserveComponent_type_value($Coin1_Coin1_type_value()))
    var $t38: $Reference; // ReferenceType($Libra_Libra_type_value($Coin1_Coin1_type_value()))
    var $t39: $Value; // $IntegerType()
    var $t40: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t41: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t42: $Reference; // ReferenceType($LBR_ReserveComponent_type_value($Coin2_Coin2_type_value()))
    var $t43: $Reference; // ReferenceType($Libra_Libra_type_value($Coin2_Coin2_type_value()))
    var $t44: $Value; // $IntegerType()
    var $t45: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t46: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t47: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t48: $Value; // $AddressType()
    var $t49: $Value; // $Libra_Libra_type_value($LBR_LBR_type_value())
    var $t50: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t51: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t52: $Value; // $Libra_Preburn_type_value($LBR_LBR_type_value())
    var $tmp: $Value;
    var $saved_m: $Memory;

    // initialize function execution
    assume !$abort_flag;
    $saved_m := $m;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(7, 5142, 0, account); }
    if (true) { assume $DebugTrackLocal(7, 5142, 1, coin); }

    // bytecode translation starts here
    // $t48 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t48 := $tmp;

    // $t49 := move(coin)
    call $tmp := $CopyOrMoveValue(coin);
    $t49 := $tmp;

    // $t9 := CoreAddresses::ASSOCIATION_ROOT_ADDRESS()
    call $t9 := $CoreAddresses_ASSOCIATION_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5313);
      goto Abort;
    }
    assume is#$Address($t9);


    // $t10 := borrow_global<LBR::Reserve>($t9)
    call $t10 := $BorrowGlobal($t9, $LBR_Reserve_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5271);
      goto Abort;
    }
    assume $LBR_Reserve_is_well_formed($Dereference($t10));

    // UnpackRef($t10)
    call $LBR_Reserve_before_update_inv($Dereference($t10));

    // reserve := $t10
    call reserve := $CopyOrMoveRef($t10);
    if (true) { assume $DebugTrackLocal(7, 5261, 7, $Dereference(reserve)); }

    // $t11 := copy($t49)
    call $tmp := $CopyOrMoveValue($t49);
    $t11 := $tmp;

    // $t12 := Libra::value<LBR::LBR>($t11)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($LBR_LBR_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t12 := $Libra_value($LBR_LBR_type_value(), $t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5380);
      goto Abort;
    }
    assume $IsValidU64($t12);


    // ratio_multiplier := $t12
    call $tmp := $CopyOrMoveValue($t12);
    ratio_multiplier := $tmp;
    if (true) { assume $DebugTrackLocal(7, 5354, 6, $tmp); }

    // $t13 := move($t48)
    call $tmp := $CopyOrMoveValue($t48);
    $t13 := $tmp;

    // $t14 := Signer::address_of($t13)
    call $t14 := $Signer_address_of($t13);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5423);
      goto Abort;
    }
    assume is#$Address($t14);


    // sender := $t14
    call $tmp := $CopyOrMoveValue($t14);
    sender := $tmp;
    if (true) { assume $DebugTrackLocal(7, 5406, 8, $tmp); }

    // $t16 := copy(reserve)
    call $t16 := $CopyOrMoveRef(reserve);

    // $t17 := borrow_field<LBR::Reserve>.preburn_cap($t16)
    call $t17 := $BorrowField($t16, $LBR_Reserve_preburn_cap);
    assume $Libra_Preburn_is_well_formed_types($Dereference($t17));

    // Reference(reserve) <- $t16
    call reserve := $WritebackToReference($t16, reserve);

    // UnpackRef($t17)

    // PackRef($t17)

    // $t52 := read_ref($t17)
    call $tmp := $ReadRef($t17);
    assume $Libra_Preburn_is_well_formed($tmp);
    $t52 := $tmp;

    // $t52 := Libra::preburn_with_resource<LBR::LBR>($t49, $t52, sender)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($LBR_LBR_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t52 := $Libra_preburn_with_resource($LBR_LBR_type_value(), $t49, $t52, sender);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5459);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($t52);


    // write_ref($t17, $t52)
    call $t17 := $WriteRef($t17, $t52);
    if (true) { assume $DebugTrackLocal(7, 5949, 7, $Dereference(reserve)); }

    // Reference(reserve) <- $t17
    call reserve := $WritebackToReference($t17, reserve);

    // Reference($t16) <- $t17
    call $t16 := $WritebackToReference($t17, $t16);

    // UnpackRef($t17)

    // PackRef($t17)

    // $t19 := copy(reserve)
    call $t19 := $CopyOrMoveRef(reserve);

    // $t20 := borrow_field<LBR::Reserve>.preburn_cap($t19)
    call $t20 := $BorrowField($t19, $LBR_Reserve_preburn_cap);
    assume $Libra_Preburn_is_well_formed_types($Dereference($t20));

    // Reference(reserve) <- $t19
    call reserve := $WritebackToReference($t19, reserve);

    // UnpackRef($t20)

    // $t22 := copy(reserve)
    call $t22 := $CopyOrMoveRef(reserve);

    // $t23 := get_field<LBR::Reserve>.burn_cap($t22)
    call $tmp := $GetFieldFromReference($t22, $LBR_Reserve_burn_cap);
    assume $Libra_BurnCapability_is_well_formed($tmp);
    $t23 := $tmp;

    // Reference(reserve) <- $t22
    call reserve := $WritebackToReference($t22, reserve);

    // PackRef($t20)

    // $t52 := read_ref($t20)
    call $tmp := $ReadRef($t20);
    assume $Libra_Preburn_is_well_formed($tmp);
    $t52 := $tmp;

    // $t52 := Libra::burn_with_resource_cap<LBR::LBR>($t52, sender, $t23)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $LBR_LBR_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$LBR_LBR_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($LBR_LBR_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t52 := $Libra_burn_with_resource_cap($LBR_LBR_type_value(), $t52, sender, $t23);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5537);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($t52);


    // write_ref($t20, $t52)
    call $t20 := $WriteRef($t20, $t52);
    if (true) { assume $DebugTrackLocal(7, 5897, 7, $Dereference(reserve)); }

    // Reference(reserve) <- $t20
    call reserve := $WritebackToReference($t20, reserve);

    // Reference($t19) <- $t20
    call $t19 := $WritebackToReference($t20, $t19);

    // Reference($t22) <- $t20
    call $t22 := $WritebackToReference($t20, $t22);

    // UnpackRef($t20)

    // PackRef($t20)

    // $t25 := copy(reserve)
    call $t25 := $CopyOrMoveRef(reserve);

    // $t26 := get_field<LBR::Reserve>.coin1($t25)
    call $tmp := $GetFieldFromReference($t25, $LBR_Reserve_coin1);
    assume $LBR_ReserveComponent_is_well_formed($tmp);
    $t26 := $tmp;

    // Reference(reserve) <- $t25
    call reserve := $WritebackToReference($t25, reserve);

    // $t27 := get_field<LBR::ReserveComponent<Coin1::Coin1>>.ratio($t26)
    call $tmp := $GetFieldFromValue($t26, $LBR_ReserveComponent_ratio);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t27 := $tmp;

    // $t28 := move($t27)
    call $tmp := $CopyOrMoveValue($t27);
    $t28 := $tmp;

    // $t29 := FixedPoint32::multiply_u64(ratio_multiplier, $t28)
    call $t29 := $FixedPoint32_multiply_u64(ratio_multiplier, $t28);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5655);
      goto Abort;
    }
    assume $IsValidU64($t29);


    // coin1_amount := $t29
    call $tmp := $CopyOrMoveValue($t29);
    coin1_amount := $tmp;
    if (true) { assume $DebugTrackLocal(7, 5626, 3, $tmp); }

    // $t31 := copy(reserve)
    call $t31 := $CopyOrMoveRef(reserve);

    // $t32 := get_field<LBR::Reserve>.coin2($t31)
    call $tmp := $GetFieldFromReference($t31, $LBR_Reserve_coin2);
    assume $LBR_ReserveComponent_is_well_formed($tmp);
    $t32 := $tmp;

    // Reference(reserve) <- $t31
    call reserve := $WritebackToReference($t31, reserve);

    // $t33 := get_field<LBR::ReserveComponent<Coin2::Coin2>>.ratio($t32)
    call $tmp := $GetFieldFromValue($t32, $LBR_ReserveComponent_ratio);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t33 := $tmp;

    // $t34 := move($t33)
    call $tmp := $CopyOrMoveValue($t33);
    $t34 := $tmp;

    // $t35 := FixedPoint32::multiply_u64(ratio_multiplier, $t34)
    call $t35 := $FixedPoint32_multiply_u64(ratio_multiplier, $t34);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5751);
      goto Abort;
    }
    assume $IsValidU64($t35);


    // coin2_amount := $t35
    call $tmp := $CopyOrMoveValue($t35);
    coin2_amount := $tmp;
    if (true) { assume $DebugTrackLocal(7, 5722, 5, $tmp); }

    // $t36 := copy(reserve)
    call $t36 := $CopyOrMoveRef(reserve);

    // $t37 := borrow_field<LBR::Reserve>.coin1($t36)
    call $t37 := $BorrowField($t36, $LBR_Reserve_coin1);
    assume $LBR_ReserveComponent_is_well_formed_types($Dereference($t37));

    // Reference(reserve) <- $t36
    call reserve := $WritebackToReference($t36, reserve);

    // UnpackRef($t37)
    call $LBR_ReserveComponent_before_update_inv($Coin1_Coin1_type_value(), $Dereference($t37));

    // $t38 := borrow_field<LBR::ReserveComponent<Coin1::Coin1>>.backing($t37)
    call $t38 := $BorrowField($t37, $LBR_ReserveComponent_backing);
    assume $Libra_Libra_is_well_formed_types($Dereference($t38));

    // Reference(reserve) <- $t37
    call reserve := $WritebackToReference($t37, reserve);

    // Reference($t36) <- $t37
    call $t36 := $WritebackToReference($t37, $t36);

    // UnpackRef($t38)
    call $Libra_Libra_before_update_inv($Coin1_Coin1_type_value(), $Dereference($t38));

    // PackRef($t38)
    call $Libra_Libra_after_update_inv($Coin1_Coin1_type_value(), $Dereference($t38));

    // $t50 := read_ref($t38)
    call $tmp := $ReadRef($t38);
    assume $Libra_Libra_is_well_formed($tmp);
    $t50 := $tmp;

    // ($t40, $t50) := Libra::withdraw<Coin1::Coin1>($t50, coin1_amount)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin1_Coin1_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin1_Coin1_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin1_Coin1_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t40, $t50 := $Libra_withdraw($Coin1_Coin1_type_value(), $t50, coin1_amount);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5833);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t40);

    assume $Libra_Libra_is_well_formed($t50);


    // write_ref($t38, $t50)
    call $t38 := $WriteRef($t38, $t50);
    if (true) { assume $DebugTrackLocal(7, 5980, 7, $Dereference(reserve)); }

    // Reference(reserve) <- $t38
    call reserve := $WritebackToReference($t38, reserve);

    // Reference($t37) <- $t38
    call $t37 := $WritebackToReference($t38, $t37);

    // Reference($t36) <- $t37
    call $t36 := $WritebackToReference($t37, $t36);

    // UnpackRef($t38)
    call $Libra_Libra_before_update_inv($Coin1_Coin1_type_value(), $Dereference($t38));

    // PackRef($t37)
    call $LBR_ReserveComponent_after_update_inv($Coin1_Coin1_type_value(), $Dereference($t37));

    // PackRef($t38)
    call $Libra_Libra_after_update_inv($Coin1_Coin1_type_value(), $Dereference($t38));

    // coin1 := $t40
    call $tmp := $CopyOrMoveValue($t40);
    coin1 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 5818, 2, $tmp); }

    // $t41 := move(reserve)
    call $t41 := $CopyOrMoveRef(reserve);

    // $t42 := borrow_field<LBR::Reserve>.coin2($t41)
    call $t42 := $BorrowField($t41, $LBR_Reserve_coin2);
    assume $LBR_ReserveComponent_is_well_formed_types($Dereference($t42));

    // LBR::Reserve <- $t41
    call $WritebackToGlobal($t41);

    // UnpackRef($t42)
    call $LBR_ReserveComponent_before_update_inv($Coin2_Coin2_type_value(), $Dereference($t42));

    // $t43 := borrow_field<LBR::ReserveComponent<Coin2::Coin2>>.backing($t42)
    call $t43 := $BorrowField($t42, $LBR_ReserveComponent_backing);
    assume $Libra_Libra_is_well_formed_types($Dereference($t43));

    // LBR::Reserve <- $t42
    call $WritebackToGlobal($t42);

    // Reference($t41) <- $t42
    call $t41 := $WritebackToReference($t42, $t41);

    // UnpackRef($t43)
    call $Libra_Libra_before_update_inv($Coin2_Coin2_type_value(), $Dereference($t43));

    // PackRef($t43)
    call $Libra_Libra_after_update_inv($Coin2_Coin2_type_value(), $Dereference($t43));

    // $t51 := read_ref($t43)
    call $tmp := $ReadRef($t43);
    assume $Libra_Libra_is_well_formed($tmp);
    $t51 := $tmp;

    // ($t45, $t51) := Libra::withdraw<Coin2::Coin2>($t51, coin2_amount)
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $Integer(0))))));
    assume b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $Coin2_Coin2_type_value())) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$Coin2_Coin2_type_value()], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($Coin2_Coin2_type_value()), $Libra_spec_currency_addr()), $Libra_CurrencyInfo_total_value))))));
    call $t45, $t51 := $Libra_withdraw($Coin2_Coin2_type_value(), $t51, coin2_amount);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5912);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t45);

    assume $Libra_Libra_is_well_formed($t51);


    // write_ref($t43, $t51)
    call $t43 := $WriteRef($t43, $t51);
    if (true) { assume $DebugTrackLocal(7, 5142, 7, $Dereference(reserve)); }

    // LBR::Reserve <- $t43
    call $WritebackToGlobal($t43);

    // Reference($t42) <- $t43
    call $t42 := $WritebackToReference($t43, $t42);

    // Reference($t41) <- $t42
    call $t41 := $WritebackToReference($t42, $t41);

    // UnpackRef($t43)
    call $Libra_Libra_before_update_inv($Coin2_Coin2_type_value(), $Dereference($t43));

    // PackRef($t41)
    call $LBR_Reserve_after_update_inv($Dereference($t41));

    // PackRef($t42)
    call $LBR_ReserveComponent_after_update_inv($Coin2_Coin2_type_value(), $Dereference($t42));

    // PackRef($t43)
    call $Libra_Libra_after_update_inv($Coin2_Coin2_type_value(), $Dereference($t43));

    // coin2 := $t45
    call $tmp := $CopyOrMoveValue($t45);
    coin2 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 5897, 4, $tmp); }

    // return (coin1, coin2)
    $ret0 := coin1;
    if (true) { assume $DebugTrackLocal(7, 5972, 53, $ret0); }
    $ret1 := coin2;
    if (true) { assume $DebugTrackLocal(7, 5972, 54, $ret1); }
    return;

Abort:
    $abort_flag := true;
    $m := $saved_m;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}
