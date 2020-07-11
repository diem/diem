
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
        var l1, m1, l2, m2 := l#$ValueArray(a1), v#$ValueArray(a1), l#$ValueArray(a2), v#$ValueArray(a2);
        $ValueArray(
            (lambda i: int ::
                if i >= 0 && i < l1 + l2 then
                    if i < l1 then m1[i] else m2[i - l1]
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
    dst' := $Reference(l#$Reference(src1), $ConcatPath(p#$Reference(src1), p#$Reference(dst)), v#$Reference(dst));
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

procedure {:inline 1} $AddU64_unchecked(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU64(src1) && $IsValidU64(src2);
{
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

procedure {:inline 1} $AddU128_unchecked(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU128(src1) && $IsValidU128(src2);
{
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
// Native Signer

procedure {:inline 1} $Signer_borrow_address(signer: $Value) returns (res: $Value)
    free requires is#$Address(signer);
{
    res := signer;
}

// ==================================================================================
// Native signature

// Signature related functionality is handled via uninterpreted functions. This is sound
// currently because we verify every code path based on signature verification with
// an arbitrary interpretation.

function $Signature_spec_ed25519_validate_pubkey($m: $Memory, $txn: $Transaction, public_key: $Value): $Value;
function $Signature_spec_ed25519_verify($m: $Memory, $txn: $Transaction,
                                        signature: $Value, public_key: $Value, message: $Value): $Value;

axiom (forall $m: $Memory, $txn: $Transaction, public_key: $Value ::
        is#$Boolean($Signature_spec_ed25519_validate_pubkey($m, $txn, public_key)));

axiom (forall $m: $Memory, $txn: $Transaction, signature, public_key, message: $Value ::
        is#$Boolean($Signature_spec_ed25519_verify($m, $txn, signature, public_key, message)));


procedure {:inline 1} $Signature_ed25519_validate_pubkey(public_key: $Value) returns (res: $Value) {
    res := $Signature_spec_ed25519_validate_pubkey($m, $txn, public_key);
}

procedure {:inline 1} $Signature_ed25519_verify(
        signature: $Value, public_key: $Value, message: $Value) returns (res: $Value) {
    res := $Signature_spec_ed25519_verify($m, $txn, signature, public_key, message);
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
// Native Signer::spec_address_of

function {:inline} $Signer_spec_address_of($m: $Memory, $txn: $Transaction, signer: $Value): $Value
{
    // A signer is currently identical to an address.
    signer
}

// ==================================================================================
// Mocked out Event module

procedure {:inline 1} $Event_new_event_handle(t: $TypeValue, signer: $Value) returns (res: $Value) {
}

procedure {:inline 1} $Event_publish_generator(account: $Value) {
}

procedure {:inline 1} $Event_emit_event(t: $TypeValue, handler: $Value, msg: $Value) returns (res: $Value) {
    res := handler;
}



// ** spec vars of module Signer



// ** spec funs of module Signer



// ** structs of module Signer



// ** functions of module Signer

procedure {:inline 1} $Signer_address_of_$def(s: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(15, 407, 0, s); }

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
      assume $DebugTrackAbort(15, 324);
      goto Abort;
    }
    assume is#$Address($t2);


    // $t3 := move($t2)
    call $tmp := $CopyOrMoveValue($t2);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(15, 460, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Signer_address_of_$direct_inter(s: $Value) returns ($ret0: $Value)
;ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(false)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Signer_spec_address_of($m, $txn, s)))));

procedure {:inline 1} $Signer_address_of_$direct_intra(s: $Value) returns ($ret0: $Value)
;ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(false)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Signer_spec_address_of($m, $txn, s)))));

procedure {:inline 1} $Signer_address_of(s: $Value) returns ($ret0: $Value)
;ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(false)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Signer_spec_address_of($m, $txn, s)))));



// ** spec vars of module CoreAddresses



// ** spec funs of module CoreAddresses

function {:inline} $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS(): $Value {
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



// ** structs of module CoreAddresses



// ** functions of module CoreAddresses

procedure {:inline 1} $CoreAddresses_CURRENCY_INFO_ADDRESS_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0xa550c18
    $tmp := $Address(173345816);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(4, 884, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $CoreAddresses_CURRENCY_INFO_ADDRESS_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_CURRENCY_INFO_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_CURRENCY_INFO_ADDRESS_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_CURRENCY_INFO_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_CURRENCY_INFO_ADDRESS() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_CURRENCY_INFO_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_LIBRA_ROOT_ADDRESS_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0xa550c18
    $tmp := $Address(173345816);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(4, 324, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $CoreAddresses_LIBRA_ROOT_ADDRESS_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_LIBRA_ROOT_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_LIBRA_ROOT_ADDRESS_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_LIBRA_ROOT_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_LIBRA_ROOT_ADDRESS() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_LIBRA_ROOT_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_TREASURY_COMPLIANCE_ADDRESS_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0xb1e55ed
    $tmp := $Address(186537453);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(4, 1352, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $CoreAddresses_TREASURY_COMPLIANCE_ADDRESS_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_TREASURY_COMPLIANCE_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_TREASURY_COMPLIANCE_ADDRESS_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_TREASURY_COMPLIANCE_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_TREASURY_COMPLIANCE_ADDRESS() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_TREASURY_COMPLIANCE_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_VM_RESERVED_ADDRESS_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0x0
    $tmp := $Address(0);
    $t0 := $tmp;

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(4, 1888, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $CoreAddresses_VM_RESERVED_ADDRESS_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_VM_RESERVED_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_VM_RESERVED_ADDRESS_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_VM_RESERVED_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_VM_RESERVED_ADDRESS() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_VM_RESERVED_ADDRESS_$def();
}




// ** spec vars of module LibraTimestamp



// ** spec funs of module LibraTimestamp

function {:inline} $LibraTimestamp_spec_is_genesis($m: $Memory, $txn: $Transaction): $Value {
    $Boolean(!b#$Boolean($ResourceExists($m, $LibraTimestamp_TimeHasStarted_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))
}

function {:inline} $LibraTimestamp_spec_is_not_initialized($m: $Memory, $txn: $Transaction): $Value {
    $Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_root_ctm_initialized($m, $txn)))) || b#$Boolean($Boolean($IsEqual($LibraTimestamp_spec_now_microseconds($m, $txn), $Integer(0)))))
}

function {:inline} $LibraTimestamp_root_ctm_initialized($m: $Memory, $txn: $Transaction): $Value {
    $ResourceExists($m, $LibraTimestamp_CurrentTimeMicroseconds_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())
}

function {:inline} $LibraTimestamp_spec_now_microseconds($m: $Memory, $txn: $Transaction): $Value {
    $SelectField($ResourceValue($m, $LibraTimestamp_CurrentTimeMicroseconds_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()), $LibraTimestamp_CurrentTimeMicroseconds_microseconds)
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

procedure {:inline 1} $LibraTimestamp_initialize_$def(lr_account: $Value) returns ()
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
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $LibraTimestamp_CurrentTimeMicroseconds_type_value()
    var $t14: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(11, 1223, 0, lr_account); }

    // bytecode translation starts here
    // $t14 := move(lr_account)
    call $tmp := $CopyOrMoveValue(lr_account);
    $t14 := $tmp;

    // $t4 := copy($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t4 := $tmp;

    // $t5 := Signer::address_of($t4)
    call $t5 := $Signer_address_of($t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 1366);
      goto Abort;
    }
    assume is#$Address($t5);


    // $t6 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t6 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 1407);
      goto Abort;
    }
    assume is#$Address($t6);


    // $t7 := ==($t5, $t6)
    $tmp := $Boolean($IsEqual($t5, $t6));
    $t7 := $tmp;

    // $t2 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(11, 1351, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t9 := move($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t9 := $tmp;

    // destroy($t9)

    // $t10 := 0
    $tmp := $Integer(0);
    $t10 := $tmp;

    // abort($t10)
    if (true) { assume $DebugTrackAbort(11, 1351); }
    goto Abort;

    // L0:
L0:

    // $t11 := move($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t11 := $tmp;

    // $t12 := 0
    $tmp := $Integer(0);
    $t12 := $tmp;

    // $t13 := pack LibraTimestamp::CurrentTimeMicroseconds($t12)
    call $tmp := $LibraTimestamp_CurrentTimeMicroseconds_pack(0, 0, 0, $t12);
    $t13 := $tmp;

    // move_to<LibraTimestamp::CurrentTimeMicroseconds>($t13, $t11)
    call $MoveTo($LibraTimestamp_CurrentTimeMicroseconds_type_value(), $t13, $t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 1603);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $LibraTimestamp_initialize_$direct_inter(lr_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    call $LibraTimestamp_initialize_$def(lr_account);
}


procedure {:inline 1} $LibraTimestamp_initialize_$direct_intra(lr_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    call $LibraTimestamp_initialize_$def(lr_account);
}


procedure {:inline 1} $LibraTimestamp_initialize(lr_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    call $LibraTimestamp_initialize_$def(lr_account);
}


procedure {:inline 1} $LibraTimestamp_is_genesis_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $BooleanType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t0 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 3971);
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
    if (true) { assume $DebugTrackLocal(11, 3932, 3, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $LibraTimestamp_is_genesis_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $LibraTimestamp_is_genesis_$def();
}


procedure {:inline 1} $LibraTimestamp_is_genesis_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $LibraTimestamp_is_genesis_$def();
}


procedure {:inline 1} $LibraTimestamp_is_genesis() returns ($ret0: $Value)
{
    call $ret0 := $LibraTimestamp_is_genesis_$def();
}


procedure {:inline 1} $LibraTimestamp_is_not_initialized_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $BooleanType()
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $BooleanType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t1 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t1 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 4215);
      goto Abort;
    }
    assume is#$Address($t1);


    // $t2 := exists<LibraTimestamp::CurrentTimeMicroseconds>($t1)
    call $tmp := $Exists($t1, $LibraTimestamp_CurrentTimeMicroseconds_type_value());
    $t2 := $tmp;

    // $t3 := !($t2)
    call $tmp := $Not($t2);
    $t3 := $tmp;

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t4 := true
    $tmp := $Boolean(true);
    $t4 := $tmp;

    // $t0 := $t4
    call $tmp := $CopyOrMoveValue($t4);
    $t0 := $tmp;
    if (true) { assume $DebugTrackLocal(11, 4167, 0, $tmp); }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t5 := LibraTimestamp::now_microseconds()
    call $t5 := $LibraTimestamp_now_microseconds();
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 3649);
      goto Abort;
    }
    assume $IsValidU64($t5);


    // $t6 := 0
    $tmp := $Integer(0);
    $t6 := $tmp;

    // $t7 := ==($t5, $t6)
    $tmp := $Boolean($IsEqual($t5, $t6));
    $t7 := $tmp;

    // $t0 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t0 := $tmp;
    if (true) { assume $DebugTrackLocal(11, 4167, 0, $tmp); }

    // goto L3
    goto L3;

    // L3:
L3:

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(11, 4167, 9, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $LibraTimestamp_is_not_initialized_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $LibraTimestamp_is_not_initialized_$def();
}


procedure {:inline 1} $LibraTimestamp_is_not_initialized_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $LibraTimestamp_is_not_initialized_$def();
}


procedure {:inline 1} $LibraTimestamp_is_not_initialized() returns ($ret0: $Value)
{
    call $ret0 := $LibraTimestamp_is_not_initialized_$def();
}


procedure {:inline 1} $LibraTimestamp_now_microseconds_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $LibraTimestamp_CurrentTimeMicroseconds_type_value()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t0 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 3770);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := get_global<LibraTimestamp::CurrentTimeMicroseconds>($t0)
    call $tmp := $GetGlobal($t0, $LibraTimestamp_CurrentTimeMicroseconds_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 3716);
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
    if (true) { assume $DebugTrackLocal(11, 3716, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $LibraTimestamp_now_microseconds_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $LibraTimestamp_now_microseconds_$def();
}


procedure {:inline 1} $LibraTimestamp_now_microseconds_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $LibraTimestamp_now_microseconds_$def();
}


procedure {:inline 1} $LibraTimestamp_now_microseconds() returns ($ret0: $Value)
{
    call $ret0 := $LibraTimestamp_now_microseconds_$def();
}


procedure {:inline 1} $LibraTimestamp_reset_time_has_started_for_test_$def() returns ()
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $LibraTimestamp_TimeHasStarted_type_value()
    var $t2: $Value; // $BooleanType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t0 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 2585);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := move_from<LibraTimestamp::TimeHasStarted>($t0)
    call $tmp := $MoveFrom($t0, $LibraTimestamp_TimeHasStarted_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 2544);
      goto Abort;
    }
    assume $LibraTimestamp_TimeHasStarted_is_well_formed($tmp);
    $t1 := $tmp;

    // $t2 := unpack LibraTimestamp::TimeHasStarted($t1)
    call $t2 := $LibraTimestamp_TimeHasStarted_unpack($t1);
    $t2 := $t2;

    // destroy($t2)

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $LibraTimestamp_reset_time_has_started_for_test_$direct_inter() returns ()
{
    call $LibraTimestamp_reset_time_has_started_for_test_$def();
}


procedure {:inline 1} $LibraTimestamp_reset_time_has_started_for_test_$direct_intra() returns ()
{
    call $LibraTimestamp_reset_time_has_started_for_test_$def();
}


procedure {:inline 1} $LibraTimestamp_reset_time_has_started_for_test() returns ()
{
    call $LibraTimestamp_reset_time_has_started_for_test_$def();
}


procedure {:inline 1} $LibraTimestamp_set_time_has_started_$def(lr_account: $Value) returns ()
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
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $BooleanType()
    var $t18: $Value; // $BooleanType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $BooleanType()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $IntegerType()
    var $t23: $Value; // $AddressType()
    var $t24: $Value; // $BooleanType()
    var $t25: $Value; // $LibraTimestamp_TimeHasStarted_type_value()
    var $t26: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(11, 1742, 0, lr_account); }

    // bytecode translation starts here
    // $t26 := move(lr_account)
    call $tmp := $CopyOrMoveValue(lr_account);
    $t26 := $tmp;

    // $t6 := copy($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t6 := $tmp;

    // $t7 := Signer::address_of($t6)
    call $t7 := $Signer_address_of($t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 1853);
      goto Abort;
    }
    assume is#$Address($t7);


    // $t8 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t8 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 1894);
      goto Abort;
    }
    assume is#$Address($t8);


    // $t9 := ==($t7, $t8)
    $tmp := $Boolean($IsEqual($t7, $t8));
    $t9 := $tmp;

    // $t1 := $t9
    call $tmp := $CopyOrMoveValue($t9);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(11, 1838, 1, $tmp); }

    // if ($t1) goto L0 else goto L1
    $tmp := $t1;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t11 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t11 := $tmp;

    // destroy($t11)

    // $t12 := 0
    $tmp := $Integer(0);
    $t12 := $tmp;

    // abort($t12)
    if (true) { assume $DebugTrackAbort(11, 1838); }
    goto Abort;

    // L0:
L0:

    // $t13 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t13 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 2073);
      goto Abort;
    }
    assume is#$Address($t13);


    // $t14 := exists<LibraTimestamp::CurrentTimeMicroseconds>($t13)
    call $tmp := $Exists($t13, $LibraTimestamp_CurrentTimeMicroseconds_type_value());
    $t14 := $tmp;

    // if ($t14) goto L2 else goto L3
    $tmp := $t14;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // goto L4
    goto L4;

    // L2:
L2:

    // $t15 := LibraTimestamp::now_microseconds()
    call $t15 := $LibraTimestamp_now_microseconds();
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 3649);
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
    if (true) { assume $DebugTrackLocal(11, 2026, 5, $tmp); }

    // goto L5
    goto L5;

    // L4:
L4:

    // $t18 := false
    $tmp := $Boolean(false);
    $t18 := $tmp;

    // $t5 := $t18
    call $tmp := $CopyOrMoveValue($t18);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(11, 2026, 5, $tmp); }

    // goto L5
    goto L5;

    // L5:
L5:

    // $t3 := $t5
    call $tmp := $CopyOrMoveValue($t5);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(11, 2006, 3, $tmp); }

    // if ($t3) goto L6 else goto L7
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L6; } else { goto L7; }

    // L7:
L7:

    // $t21 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t21 := $tmp;

    // destroy($t21)

    // $t22 := 1
    $tmp := $Integer(1);
    $t22 := $tmp;

    // abort($t22)
    if (true) { assume $DebugTrackAbort(11, 2006); }
    goto Abort;

    // L6:
L6:

    // $t23 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t23 := $tmp;

    // $t24 := false
    $tmp := $Boolean(false);
    $t24 := $tmp;

    // $t25 := pack LibraTimestamp::TimeHasStarted($t24)
    call $tmp := $LibraTimestamp_TimeHasStarted_pack(0, 0, 0, $t24);
    $t25 := $tmp;

    // move_to<LibraTimestamp::TimeHasStarted>($t25, $t23)
    call $MoveTo($LibraTimestamp_TimeHasStarted_type_value(), $t25, $t23);
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 2176);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $LibraTimestamp_set_time_has_started_$direct_inter(lr_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    call $LibraTimestamp_set_time_has_started_$def(lr_account);
}


procedure {:inline 1} $LibraTimestamp_set_time_has_started_$direct_intra(lr_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    call $LibraTimestamp_set_time_has_started_$def(lr_account);
}


procedure {:inline 1} $LibraTimestamp_set_time_has_started(lr_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    call $LibraTimestamp_set_time_has_started_$def(lr_account);
}


procedure {:inline 1} $LibraTimestamp_update_global_time_$def(account: $Value, proposer: $Value, timestamp: $Value) returns ()
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
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $AddressType()
    var $t17: $Reference; // ReferenceType($LibraTimestamp_CurrentTimeMicroseconds_type_value())
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $BooleanType()
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(11, 2734, 0, account); }
    if (true) { assume $DebugTrackLocal(11, 2734, 1, proposer); }
    if (true) { assume $DebugTrackLocal(11, 2734, 2, timestamp); }

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
      assume $DebugTrackAbort(11, 2958);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := CoreAddresses::VM_RESERVED_ADDRESS()
    call $t12 := $CoreAddresses_VM_RESERVED_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 2996);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := ==($t11, $t12)
    $tmp := $Boolean($IsEqual($t11, $t12));
    $t13 := $tmp;

    // $t4 := $t13
    call $tmp := $CopyOrMoveValue($t13);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(11, 2943, 4, $tmp); }

    // if ($t4) goto L0 else goto L1
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t15 := 2
    $tmp := $Integer(2);
    $t15 := $tmp;

    // abort($t15)
    if (true) { assume $DebugTrackAbort(11, 2943); }
    goto Abort;

    // L0:
L0:

    // $t16 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t16 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 3115);
      goto Abort;
    }
    assume is#$Address($t16);


    // $t17 := borrow_global<LibraTimestamp::CurrentTimeMicroseconds>($t16)
    call $t17 := $BorrowGlobal($t16, $LibraTimestamp_CurrentTimeMicroseconds_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 3057);
      goto Abort;
    }
    assume $LibraTimestamp_CurrentTimeMicroseconds_is_well_formed($Dereference($t17));

    // UnpackRef($t17)

    // global_timer := $t17
    call global_timer := $CopyOrMoveRef($t17);
    if (true) { assume $DebugTrackLocal(11, 3042, 3, $Dereference(global_timer)); }

    // $t19 := CoreAddresses::VM_RESERVED_ADDRESS()
    call $t19 := $CoreAddresses_VM_RESERVED_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(11, 3177);
      goto Abort;
    }
    assume is#$Address($t19);


    // $t20 := ==($t41, $t19)
    $tmp := $Boolean($IsEqual($t41, $t19));
    $t20 := $tmp;

    // if ($t20) goto L2 else goto L3
    $tmp := $t20;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // goto L4
    goto L4;

    // L2:
L2:

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
    if (true) { assume $DebugTrackLocal(11, 3295, 6, $tmp); }

    // if ($t6) goto L5 else goto L6
    $tmp := $t6;
    if (b#$Boolean($tmp)) { goto L5; } else { goto L6; }

    // L6:
L6:

    // $t27 := move(global_timer)
    call $t27 := $CopyOrMoveRef(global_timer);

    // destroy($t27)

    // LibraTimestamp::CurrentTimeMicroseconds <- $t27
    call $WritebackToGlobal($t27);

    // PackRef($t27)

    // $t28 := 3
    $tmp := $Integer(3);
    $t28 := $tmp;

    // abort($t28)
    if (true) { assume $DebugTrackAbort(11, 3295); }
    goto Abort;

    // L5:
L5:

    // goto L7
    goto L7;

    // L4:
L4:

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
    if (true) { assume $DebugTrackLocal(11, 3439, 8, $tmp); }

    // if ($t8) goto L7 else goto L8
    $tmp := $t8;
    if (b#$Boolean($tmp)) { goto L7; } else { goto L8; }

    // L8:
L8:

    // $t35 := move(global_timer)
    call $t35 := $CopyOrMoveRef(global_timer);

    // destroy($t35)

    // LibraTimestamp::CurrentTimeMicroseconds <- $t35
    call $WritebackToGlobal($t35);

    // PackRef($t35)

    // $t36 := 3
    $tmp := $Integer(3);
    $t36 := $tmp;

    // abort($t36)
    if (true) { assume $DebugTrackAbort(11, 3439); }
    goto Abort;

    // L7:
L7:

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
    if (true) { assume $DebugTrackLocal(11, 3525, 3, $Dereference(global_timer)); }

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
}

procedure {:inline 1} $LibraTimestamp_update_global_time_$direct_inter(account: $Value, proposer: $Value, timestamp: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(proposer);

    assume $IsValidU64(timestamp);

    call $LibraTimestamp_update_global_time_$def(account, proposer, timestamp);
}


procedure {:inline 1} $LibraTimestamp_update_global_time_$direct_intra(account: $Value, proposer: $Value, timestamp: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(proposer);

    assume $IsValidU64(timestamp);

    call $LibraTimestamp_update_global_time_$def(account, proposer, timestamp);
}


procedure {:inline 1} $LibraTimestamp_update_global_time(account: $Value, proposer: $Value, timestamp: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(proposer);

    assume $IsValidU64(timestamp);

    call $LibraTimestamp_update_global_time_$def(account, proposer, timestamp);
}




// ** spec vars of module Roles



// ** spec funs of module Roles

function {:inline} $Roles_spec_get_role_id($m: $Memory, $txn: $Transaction, account: $Value): $Value {
    (var addr := $Signer_spec_address_of($m, $txn, account); $SelectField($ResourceValue($m, $Roles_RoleId_type_value(), addr), $Roles_RoleId_role_id))
}

function {:inline} $Roles_spec_has_role_id($m: $Memory, $txn: $Transaction, account: $Value, role_id: $Value): $Value {
    (var addr := $Signer_spec_address_of($m, $txn, account); $Boolean(b#$Boolean($ResourceExists($m, $Roles_RoleId_type_value(), addr)) && b#$Boolean($Boolean($IsEqual($SelectField($ResourceValue($m, $Roles_RoleId_type_value(), addr), $Roles_RoleId_role_id), role_id)))))
}

function {:inline} $Roles_SPEC_LIBRA_ROOT_ROLE_ID(): $Value {
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

function {:inline} $Roles_spec_has_libra_root_role($m: $Memory, $txn: $Transaction, account: $Value): $Value {
    $Roles_spec_has_role_id($m, $txn, account, $Roles_SPEC_LIBRA_ROOT_ROLE_ID())
}

function {:inline} $Roles_spec_has_treasury_compliance_role($m: $Memory, $txn: $Transaction, account: $Value): $Value {
    $Roles_spec_has_role_id($m, $txn, account, $Roles_SPEC_TREASURY_COMPLIANCE_ROLE_ID())
}

function {:inline} $Roles_spec_has_designated_dealer_role($m: $Memory, $txn: $Transaction, account: $Value): $Value {
    $Roles_spec_has_role_id($m, $txn, account, $Roles_SPEC_DESIGNATED_DEALER_ROLE_ID())
}

function {:inline} $Roles_spec_has_validator_role($m: $Memory, $txn: $Transaction, account: $Value): $Value {
    $Roles_spec_has_role_id($m, $txn, account, $Roles_SPEC_VALIDATOR_ROLE_ID())
}

function {:inline} $Roles_spec_has_validator_operator_role($m: $Memory, $txn: $Transaction, account: $Value): $Value {
    $Roles_spec_has_role_id($m, $txn, account, $Roles_SPEC_VALIDATOR_OPERATOR_ROLE_ID())
}

function {:inline} $Roles_spec_has_parent_VASP_role($m: $Memory, $txn: $Transaction, account: $Value): $Value {
    $Roles_spec_has_role_id($m, $txn, account, $Roles_SPEC_PARENT_VASP_ROLE_ID())
}

function {:inline} $Roles_spec_has_child_VASP_role($m: $Memory, $txn: $Transaction, account: $Value): $Value {
    $Roles_spec_has_role_id($m, $txn, account, $Roles_SPEC_CHILD_VASP_ROLE_ID())
}

function {:inline} $Roles_spec_has_unhosted_role($m: $Memory, $txn: $Transaction, account: $Value): $Value {
    $Roles_spec_has_role_id($m, $txn, account, $Roles_SPEC_UNHOSTED_ROLE_ID())
}

function {:inline} $Roles_spec_has_register_new_currency_privilege($m: $Memory, $txn: $Transaction, account: $Value): $Value {
    $Roles_spec_has_treasury_compliance_role($m, $txn, account)
}

function {:inline} $Roles_spec_has_update_dual_attestation_threshold_privilege($m: $Memory, $txn: $Transaction, account: $Value): $Value {
    $Roles_spec_has_treasury_compliance_role($m, $txn, account)
}

function {:inline} $Roles_spec_has_on_chain_config_privilege($m: $Memory, $txn: $Transaction, account: $Value): $Value {
    $Roles_spec_has_libra_root_role($m, $txn, account)
}



// ** structs of module Roles

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



// ** functions of module Roles

procedure {:inline 1} $Roles_grant_libra_root_role_$def(lr_account: $Value) returns ()
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
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $Roles_RoleId_type_value()
    var $t21: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 2041, 0, lr_account); }

    // bytecode translation starts here
    // $t21 := move(lr_account)
    call $tmp := $CopyOrMoveValue(lr_account);
    $t21 := $tmp;

    // $t6 := LibraTimestamp::is_genesis()
    call $t6 := $LibraTimestamp_is_genesis();
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 2143);
      goto Abort;
    }
    assume is#$Boolean($t6);


    // $t2 := $t6
    call $tmp := $CopyOrMoveValue($t6);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 2120, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t8 := move($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t8 := $tmp;

    // destroy($t8)

    // $t9 := 0
    $tmp := $Integer(0);
    $t9 := $tmp;

    // abort($t9)
    if (true) { assume $DebugTrackAbort(14, 2120); }
    goto Abort;

    // L0:
L0:

    // $t10 := copy($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t10 := $tmp;

    // $t11 := Signer::address_of($t10)
    call $t11 := $Signer_address_of($t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 2208);
      goto Abort;
    }
    assume is#$Address($t11);


    // owner_address := $t11
    call $tmp := $CopyOrMoveValue($t11);
    owner_address := $tmp;
    if (true) { assume $DebugTrackLocal(14, 2184, 1, $tmp); }

    // $t13 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t13 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 2279);
      goto Abort;
    }
    assume is#$Address($t13);


    // $t14 := ==(owner_address, $t13)
    $tmp := $Boolean($IsEqual(owner_address, $t13));
    $t14 := $tmp;

    // $t4 := $t14
    call $tmp := $CopyOrMoveValue($t14);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 2240, 4, $tmp); }

    // if ($t4) goto L2 else goto L3
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t16 := move($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t16 := $tmp;

    // destroy($t16)

    // $t17 := 1
    $tmp := $Integer(1);
    $t17 := $tmp;

    // abort($t17)
    if (true) { assume $DebugTrackAbort(14, 2240); }
    goto Abort;

    // L2:
L2:

    // $t18 := move($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t18 := $tmp;

    // $t19 := 0
    $tmp := $Integer(0);
    $t19 := $tmp;

    // $t20 := pack Roles::RoleId($t19)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t19);
    $t20 := $tmp;

    // move_to<Roles::RoleId>($t20, $t18)
    call $MoveTo($Roles_RoleId_type_value(), $t20, $t18);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 2385);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Roles_grant_libra_root_role_$direct_inter(lr_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    call $Roles_grant_libra_root_role_$def(lr_account);
}


procedure {:inline 1} $Roles_grant_libra_root_role_$direct_intra(lr_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    call $Roles_grant_libra_root_role_$def(lr_account);
}


procedure {:inline 1} $Roles_grant_libra_root_role(lr_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    call $Roles_grant_libra_root_role_$def(lr_account);
}


procedure {:inline 1} $Roles_grant_treasury_compliance_role_$def(treasury_compliance_account: $Value, lr_account: $Value) returns ()
{
    // declare local variables
    var owner_address: $Value; // $AddressType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $BooleanType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $BooleanType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $AddressType()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $AddressType()
    var $t23: $Value; // $BooleanType()
    var $t24: $Value; // $BooleanType()
    var $t25: $Value; // $AddressType()
    var $t26: $Value; // $IntegerType()
    var $t27: $Value; // $AddressType()
    var $t28: $Value; // $IntegerType()
    var $t29: $Value; // $Roles_RoleId_type_value()
    var $t30: $Value; // $AddressType()
    var $t31: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 3036, 0, treasury_compliance_account); }
    if (true) { assume $DebugTrackLocal(14, 3036, 1, lr_account); }

    // bytecode translation starts here
    // $t30 := move(treasury_compliance_account)
    call $tmp := $CopyOrMoveValue(treasury_compliance_account);
    $t30 := $tmp;

    // $t31 := move(lr_account)
    call $tmp := $CopyOrMoveValue(lr_account);
    $t31 := $tmp;

    // $t9 := LibraTimestamp::is_genesis()
    call $t9 := $LibraTimestamp_is_genesis();
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 3209);
      goto Abort;
    }
    assume is#$Boolean($t9);


    // $t3 := $t9
    call $tmp := $CopyOrMoveValue($t9);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 3186, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t11 := move($t30)
    call $tmp := $CopyOrMoveValue($t30);
    $t11 := $tmp;

    // destroy($t11)

    // $t12 := move($t31)
    call $tmp := $CopyOrMoveValue($t31);
    $t12 := $tmp;

    // destroy($t12)

    // $t13 := 0
    $tmp := $Integer(0);
    $t13 := $tmp;

    // abort($t13)
    if (true) { assume $DebugTrackAbort(14, 3186); }
    goto Abort;

    // L0:
L0:

    // $t14 := move($t31)
    call $tmp := $CopyOrMoveValue($t31);
    $t14 := $tmp;

    // $t15 := Roles::has_libra_root_role($t14)
    call $t15 := $Roles_has_libra_root_role($t14);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10636);
      goto Abort;
    }
    assume is#$Boolean($t15);


    // $t5 := $t15
    call $tmp := $CopyOrMoveValue($t15);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 3246, 5, $tmp); }

    // if ($t5) goto L2 else goto L3
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t17 := move($t30)
    call $tmp := $CopyOrMoveValue($t30);
    $t17 := $tmp;

    // destroy($t17)

    // $t18 := 3
    $tmp := $Integer(3);
    $t18 := $tmp;

    // abort($t18)
    if (true) { assume $DebugTrackAbort(14, 3246); }
    goto Abort;

    // L2:
L2:

    // $t19 := copy($t30)
    call $tmp := $CopyOrMoveValue($t30);
    $t19 := $tmp;

    // $t20 := Signer::address_of($t19)
    call $t20 := $Signer_address_of($t19);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 3345);
      goto Abort;
    }
    assume is#$Address($t20);


    // owner_address := $t20
    call $tmp := $CopyOrMoveValue($t20);
    owner_address := $tmp;
    if (true) { assume $DebugTrackLocal(14, 3321, 2, $tmp); }

    // $t22 := CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()
    call $t22 := $CoreAddresses_TREASURY_COMPLIANCE_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 3433);
      goto Abort;
    }
    assume is#$Address($t22);


    // $t23 := ==(owner_address, $t22)
    $tmp := $Boolean($IsEqual(owner_address, $t22));
    $t23 := $tmp;

    // $t7 := $t23
    call $tmp := $CopyOrMoveValue($t23);
    $t7 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 3394, 7, $tmp); }

    // if ($t7) goto L4 else goto L5
    $tmp := $t7;
    if (b#$Boolean($tmp)) { goto L4; } else { goto L5; }

    // L5:
L5:

    // $t25 := move($t30)
    call $tmp := $CopyOrMoveValue($t30);
    $t25 := $tmp;

    // destroy($t25)

    // $t26 := 2
    $tmp := $Integer(2);
    $t26 := $tmp;

    // abort($t26)
    if (true) { assume $DebugTrackAbort(14, 3394); }
    goto Abort;

    // L4:
L4:

    // $t27 := move($t30)
    call $tmp := $CopyOrMoveValue($t30);
    $t27 := $tmp;

    // $t28 := 1
    $tmp := $Integer(1);
    $t28 := $tmp;

    // $t29 := pack Roles::RoleId($t28)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t28);
    $t29 := $tmp;

    // move_to<Roles::RoleId>($t29, $t27)
    call $MoveTo($Roles_RoleId_type_value(), $t29, $t27);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 3558);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Roles_grant_treasury_compliance_role_$direct_inter(treasury_compliance_account: $Value, lr_account: $Value) returns ()
{
    assume is#$Address(treasury_compliance_account);

    assume is#$Address(lr_account);

    call $Roles_grant_treasury_compliance_role_$def(treasury_compliance_account, lr_account);
}


procedure {:inline 1} $Roles_grant_treasury_compliance_role_$direct_intra(treasury_compliance_account: $Value, lr_account: $Value) returns ()
{
    assume is#$Address(treasury_compliance_account);

    assume is#$Address(lr_account);

    call $Roles_grant_treasury_compliance_role_$def(treasury_compliance_account, lr_account);
}


procedure {:inline 1} $Roles_grant_treasury_compliance_role(treasury_compliance_account: $Value, lr_account: $Value) returns ()
{
    assume is#$Address(treasury_compliance_account);

    assume is#$Address(lr_account);

    call $Roles_grant_treasury_compliance_role_$def(treasury_compliance_account, lr_account);
}


procedure {:inline 1} $Roles_has_child_VASP_role_$def(account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 11453, 0, account); }

    // bytecode translation starts here
    // $t4 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := 6
    $tmp := $Integer(6);
    $t2 := $tmp;

    // $t3 := Roles::has_role($t1, $t2)
    call $t3 := $Roles_has_role($t1, $t2);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10413);
      goto Abort;
    }
    assume is#$Boolean($t3);


    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(14, 11534, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_has_child_VASP_role_$direct_inter(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_child_VASP_role_$def(account);
}


procedure {:inline 1} $Roles_has_child_VASP_role_$direct_intra(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_child_VASP_role_$def(account);
}


procedure {:inline 1} $Roles_has_child_VASP_role(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_child_VASP_role_$def(account);
}


procedure {:inline 1} $Roles_has_designated_dealer_role_$def(account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 10903, 0, account); }

    // bytecode translation starts here
    // $t4 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := 2
    $tmp := $Integer(2);
    $t2 := $tmp;

    // $t3 := Roles::has_role($t1, $t2)
    call $t3 := $Roles_has_role($t1, $t2);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10413);
      goto Abort;
    }
    assume is#$Boolean($t3);


    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(14, 10991, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_has_designated_dealer_role_$direct_inter(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_designated_dealer_role_$def(account);
}


procedure {:inline 1} $Roles_has_designated_dealer_role_$direct_intra(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_designated_dealer_role_$def(account);
}


procedure {:inline 1} $Roles_has_designated_dealer_role(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_designated_dealer_role_$def(account);
}


procedure {:inline 1} $Roles_has_libra_root_role_$def(account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 10625, 0, account); }

    // bytecode translation starts here
    // $t4 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := 0
    $tmp := $Integer(0);
    $t2 := $tmp;

    // $t3 := Roles::has_role($t1, $t2)
    call $t3 := $Roles_has_role($t1, $t2);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10413);
      goto Abort;
    }
    assume is#$Boolean($t3);


    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(14, 10706, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_has_libra_root_role_$direct_inter(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_libra_root_role_$def(account);
}


procedure {:inline 1} $Roles_has_libra_root_role_$direct_intra(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_libra_root_role_$def(account);
}


procedure {:inline 1} $Roles_has_libra_root_role(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_libra_root_role_$def(account);
}


procedure {:inline 1} $Roles_has_on_chain_config_privilege_$def(account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 12002, 0, account); }

    // bytecode translation starts here
    // $t3 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t3 := $tmp;

    // $t1 := move($t3)
    call $tmp := $CopyOrMoveValue($t3);
    $t1 := $tmp;

    // $t2 := Roles::has_libra_root_role($t1)
    call $t2 := $Roles_has_libra_root_role($t1);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10636);
      goto Abort;
    }
    assume is#$Boolean($t2);


    // return $t2
    $ret0 := $t2;
    if (true) { assume $DebugTrackLocal(14, 12094, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_has_on_chain_config_privilege_$direct_inter(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_on_chain_config_privilege_$def(account);
}


procedure {:inline 1} $Roles_has_on_chain_config_privilege_$direct_intra(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_on_chain_config_privilege_$def(account);
}


procedure {:inline 1} $Roles_has_on_chain_config_privilege(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_on_chain_config_privilege_$def(account);
}


procedure {:inline 1} $Roles_has_parent_VASP_role_$def(account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 11321, 0, account); }

    // bytecode translation starts here
    // $t4 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := 5
    $tmp := $Integer(5);
    $t2 := $tmp;

    // $t3 := Roles::has_role($t1, $t2)
    call $t3 := $Roles_has_role($t1, $t2);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10413);
      goto Abort;
    }
    assume is#$Boolean($t3);


    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(14, 11403, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_has_parent_VASP_role_$direct_inter(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_parent_VASP_role_$def(account);
}


procedure {:inline 1} $Roles_has_parent_VASP_role_$direct_intra(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_parent_VASP_role_$def(account);
}


procedure {:inline 1} $Roles_has_parent_VASP_role(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_parent_VASP_role_$def(account);
}


procedure {:inline 1} $Roles_has_register_new_currency_privilege_$def(account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 11709, 0, account); }

    // bytecode translation starts here
    // $t3 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t3 := $tmp;

    // $t1 := move($t3)
    call $tmp := $CopyOrMoveValue($t3);
    $t1 := $tmp;

    // $t2 := Roles::has_libra_root_role($t1)
    call $t2 := $Roles_has_libra_root_role($t1);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10636);
      goto Abort;
    }
    assume is#$Boolean($t2);


    // return $t2
    $ret0 := $t2;
    if (true) { assume $DebugTrackLocal(14, 11807, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_has_register_new_currency_privilege_$direct_inter(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_register_new_currency_privilege_$def(account);
}


procedure {:inline 1} $Roles_has_register_new_currency_privilege_$direct_intra(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_register_new_currency_privilege_$def(account);
}


procedure {:inline 1} $Roles_has_register_new_currency_privilege(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_register_new_currency_privilege_$def(account);
}


procedure {:inline 1} $Roles_has_role_$def(account: $Value, role_id: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var addr: $Value; // $AddressType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $Roles_RoleId_type_value()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $IntegerType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 10402, 0, account); }
    if (true) { assume $DebugTrackLocal(14, 10402, 1, role_id); }

    // bytecode translation starts here
    // $t16 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t16 := $tmp;

    // $t17 := move(role_id)
    call $tmp := $CopyOrMoveValue(role_id);
    $t17 := $tmp;

    // $t4 := move($t16)
    call $tmp := $CopyOrMoveValue($t16);
    $t4 := $tmp;

    // $t5 := Signer::address_of($t4)
    call $t5 := $Signer_address_of($t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10504);
      goto Abort;
    }
    assume is#$Address($t5);


    // addr := $t5
    call $tmp := $CopyOrMoveValue($t5);
    addr := $tmp;
    if (true) { assume $DebugTrackLocal(14, 10489, 2, $tmp); }

    // $t7 := exists<Roles::RoleId>(addr)
    call $tmp := $Exists(addr, $Roles_RoleId_type_value());
    $t7 := $tmp;

    // if ($t7) goto L0 else goto L1
    $tmp := $t7;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t9 := get_global<Roles::RoleId>(addr)
    call $tmp := $GetGlobal(addr, $Roles_RoleId_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10567);
      goto Abort;
    }
    assume $Roles_RoleId_is_well_formed($tmp);
    $t9 := $tmp;

    // $t10 := get_field<Roles::RoleId>.role_id($t9)
    call $tmp := $GetFieldFromValue($t9, $Roles_RoleId_role_id);
    assume $IsValidU64($tmp);
    $t10 := $tmp;

    // $t11 := move($t10)
    call $tmp := $CopyOrMoveValue($t10);
    $t11 := $tmp;

    // $t13 := ==($t11, $t17)
    $tmp := $Boolean($IsEqual($t11, $t17));
    $t13 := $tmp;

    // $t3 := $t13
    call $tmp := $CopyOrMoveValue($t13);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 10532, 3, $tmp); }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t14 := false
    $tmp := $Boolean(false);
    $t14 := $tmp;

    // $t3 := $t14
    call $tmp := $CopyOrMoveValue($t14);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 10532, 3, $tmp); }

    // goto L3
    goto L3;

    // L3:
L3:

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(14, 10532, 18, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_has_role_$direct_inter(account: $Value, role_id: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    assume $IsValidU64(role_id);

    call $ret0 := $Roles_has_role_$def(account, role_id);
}


procedure {:inline 1} $Roles_has_role_$direct_intra(account: $Value, role_id: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    assume $IsValidU64(role_id);

    call $ret0 := $Roles_has_role_$def(account, role_id);
}


procedure {:inline 1} $Roles_has_role(account: $Value, role_id: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    assume $IsValidU64(role_id);

    call $ret0 := $Roles_has_role_$def(account, role_id);
}


procedure {:inline 1} $Roles_has_treasury_compliance_role_$def(account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 10755, 0, account); }

    // bytecode translation starts here
    // $t4 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := 1
    $tmp := $Integer(1);
    $t2 := $tmp;

    // $t3 := Roles::has_role($t1, $t2)
    call $t3 := $Roles_has_role($t1, $t2);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10413);
      goto Abort;
    }
    assume is#$Boolean($t3);


    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(14, 10845, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_has_treasury_compliance_role_$direct_inter(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_treasury_compliance_role_$def(account);
}


procedure {:inline 1} $Roles_has_treasury_compliance_role_$direct_intra(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_treasury_compliance_role_$def(account);
}


procedure {:inline 1} $Roles_has_treasury_compliance_role(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_treasury_compliance_role_$def(account);
}


procedure {:inline 1} $Roles_has_unhosted_role_$def(account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 11583, 0, account); }

    // bytecode translation starts here
    // $t4 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := 7
    $tmp := $Integer(7);
    $t2 := $tmp;

    // $t3 := Roles::has_role($t1, $t2)
    call $t3 := $Roles_has_role($t1, $t2);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10413);
      goto Abort;
    }
    assume is#$Boolean($t3);


    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(14, 11662, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_has_unhosted_role_$direct_inter(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_unhosted_role_$def(account);
}


procedure {:inline 1} $Roles_has_unhosted_role_$direct_intra(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_unhosted_role_$def(account);
}


procedure {:inline 1} $Roles_has_unhosted_role(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_unhosted_role_$def(account);
}


procedure {:inline 1} $Roles_has_update_dual_attestation_limit_privilege_$def(account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 11847, 0, account); }

    // bytecode translation starts here
    // $t3 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t3 := $tmp;

    // $t1 := move($t3)
    call $tmp := $CopyOrMoveValue($t3);
    $t1 := $tmp;

    // $t2 := Roles::has_treasury_compliance_role($t1)
    call $t2 := $Roles_has_treasury_compliance_role($t1);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10766);
      goto Abort;
    }
    assume is#$Boolean($t2);


    // return $t2
    $ret0 := $t2;
    if (true) { assume $DebugTrackLocal(14, 11953, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_has_update_dual_attestation_limit_privilege_$direct_inter(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_update_dual_attestation_limit_privilege_$def(account);
}


procedure {:inline 1} $Roles_has_update_dual_attestation_limit_privilege_$direct_intra(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_update_dual_attestation_limit_privilege_$def(account);
}


procedure {:inline 1} $Roles_has_update_dual_attestation_limit_privilege(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_update_dual_attestation_limit_privilege_$def(account);
}


procedure {:inline 1} $Roles_has_validator_operator_role_$def(account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 11175, 0, account); }

    // bytecode translation starts here
    // $t4 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := 4
    $tmp := $Integer(4);
    $t2 := $tmp;

    // $t3 := Roles::has_role($t1, $t2)
    call $t3 := $Roles_has_role($t1, $t2);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10413);
      goto Abort;
    }
    assume is#$Boolean($t3);


    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(14, 11264, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_has_validator_operator_role_$direct_inter(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_validator_operator_role_$def(account);
}


procedure {:inline 1} $Roles_has_validator_operator_role_$direct_intra(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_validator_operator_role_$def(account);
}


procedure {:inline 1} $Roles_has_validator_operator_role(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_validator_operator_role_$def(account);
}


procedure {:inline 1} $Roles_has_validator_role_$def(account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 11047, 0, account); }

    // bytecode translation starts here
    // $t4 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t4 := $tmp;

    // $t1 := move($t4)
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;

    // $t2 := 3
    $tmp := $Integer(3);
    $t2 := $tmp;

    // $t3 := Roles::has_role($t1, $t2)
    call $t3 := $Roles_has_role($t1, $t2);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10413);
      goto Abort;
    }
    assume is#$Boolean($t3);


    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(14, 11127, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Roles_has_validator_role_$direct_inter(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_validator_role_$def(account);
}


procedure {:inline 1} $Roles_has_validator_role_$direct_intra(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_validator_role_$def(account);
}


procedure {:inline 1} $Roles_has_validator_role(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Roles_has_validator_role_$def(account);
}


procedure {:inline 1} $Roles_new_child_vasp_role_$def(creating_account: $Value, new_account: $Value) returns ()
{
    // declare local variables
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $Roles_RoleId_type_value()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 8243, 0, creating_account); }
    if (true) { assume $DebugTrackLocal(14, 8243, 1, new_account); }

    // bytecode translation starts here
    // $t21 := move(creating_account)
    call $tmp := $CopyOrMoveValue(creating_account);
    $t21 := $tmp;

    // $t22 := move(new_account)
    call $tmp := $CopyOrMoveValue(new_account);
    $t22 := $tmp;

    // $t6 := move($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t6 := $tmp;

    // $t7 := Roles::has_parent_VASP_role($t6)
    call $t7 := $Roles_has_parent_VASP_role($t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 11332);
      goto Abort;
    }
    assume is#$Boolean($t7);


    // $t2 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 8372, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t9 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t9 := $tmp;

    // destroy($t9)

    // $t10 := 3
    $tmp := $Integer(3);
    $t10 := $tmp;

    // abort($t10)
    if (true) { assume $DebugTrackAbort(14, 8372); }
    goto Abort;

    // L0:
L0:

    // $t11 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t11 := $tmp;

    // $t12 := Signer::address_of($t11)
    call $t12 := $Signer_address_of($t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 8554);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := exists<Roles::RoleId>($t12)
    call $tmp := $Exists($t12, $Roles_RoleId_type_value());
    $t13 := $tmp;

    // $t14 := !($t13)
    call $tmp := $Not($t13);
    $t14 := $tmp;

    // $t4 := $t14
    call $tmp := $CopyOrMoveValue($t14);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 8523, 4, $tmp); }

    // if ($t4) goto L2 else goto L3
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t16 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t16 := $tmp;

    // destroy($t16)

    // $t17 := 4
    $tmp := $Integer(4);
    $t17 := $tmp;

    // abort($t17)
    if (true) { assume $DebugTrackAbort(14, 8523); }
    goto Abort;

    // L2:
L2:

    // $t18 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t18 := $tmp;

    // $t19 := 6
    $tmp := $Integer(6);
    $t19 := $tmp;

    // $t20 := pack Roles::RoleId($t19)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t19);
    $t20 := $tmp;

    // move_to<Roles::RoleId>($t20, $t18)
    call $MoveTo($Roles_RoleId_type_value(), $t20, $t18);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 8613);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Roles_new_child_vasp_role_$direct_inter(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_child_vasp_role_$def(creating_account, new_account);
}


procedure {:inline 1} $Roles_new_child_vasp_role_$direct_intra(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_child_vasp_role_$def(creating_account, new_account);
}


procedure {:inline 1} $Roles_new_child_vasp_role(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_child_vasp_role_$def(creating_account, new_account);
}


procedure {:inline 1} $Roles_new_designated_dealer_role_$def(creating_account: $Value, new_account: $Value) returns ()
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
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $Roles_RoleId_type_value()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $Roles_RoleId_type_value()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $IntegerType()
    var $t22: $Value; // $BooleanType()
    var $t23: $Value; // $BooleanType()
    var $t24: $Value; // $AddressType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $AddressType()
    var $t27: $Value; // $IntegerType()
    var $t28: $Value; // $Roles_RoleId_type_value()
    var $t29: $Value; // $AddressType()
    var $t30: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 4540, 0, creating_account); }
    if (true) { assume $DebugTrackLocal(14, 4540, 1, new_account); }

    // bytecode translation starts here
    // $t29 := move(creating_account)
    call $tmp := $CopyOrMoveValue(creating_account);
    $t29 := $tmp;

    // $t30 := move(new_account)
    call $tmp := $CopyOrMoveValue(new_account);
    $t30 := $tmp;

    // $t7 := move($t29)
    call $tmp := $CopyOrMoveValue($t29);
    $t7 := $tmp;

    // $t8 := Signer::address_of($t7)
    call $t8 := $Signer_address_of($t7);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 4725);
      goto Abort;
    }
    assume is#$Address($t8);


    // $t9 := get_global<Roles::RoleId>($t8)
    call $tmp := $GetGlobal($t8, $Roles_RoleId_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 4695);
      goto Abort;
    }
    assume $Roles_RoleId_is_well_formed($tmp);
    $t9 := $tmp;

    // calling_role := $t9
    call $tmp := $CopyOrMoveValue($t9);
    calling_role := $tmp;
    if (true) { assume $DebugTrackLocal(14, 4680, 2, $tmp); }

    // $t10 := copy($t30)
    call $tmp := $CopyOrMoveValue($t30);
    $t10 := $tmp;

    // $t11 := Signer::address_of($t10)
    call $t11 := $Signer_address_of($t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 4868);
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
    if (true) { assume $DebugTrackLocal(14, 4837, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t15 := move($t30)
    call $tmp := $CopyOrMoveValue($t30);
    $t15 := $tmp;

    // destroy($t15)

    // $t16 := move(calling_role)
    call $tmp := $CopyOrMoveValue(calling_role);
    $t16 := $tmp;

    // destroy($t16)

    // $t17 := 4
    $tmp := $Integer(4);
    $t17 := $tmp;

    // abort($t17)
    if (true) { assume $DebugTrackAbort(14, 4837); }
    goto Abort;

    // L0:
L0:

    // $t18 := move(calling_role)
    call $tmp := $CopyOrMoveValue(calling_role);
    $t18 := $tmp;

    // $t19 := get_field<Roles::RoleId>.role_id($t18)
    call $tmp := $GetFieldFromValue($t18, $Roles_RoleId_role_id);
    assume $IsValidU64($tmp);
    $t19 := $tmp;

    // $t20 := move($t19)
    call $tmp := $CopyOrMoveValue($t19);
    $t20 := $tmp;

    // $t21 := 1
    $tmp := $Integer(1);
    $t21 := $tmp;

    // $t22 := ==($t20, $t21)
    $tmp := $Boolean($IsEqual($t20, $t21));
    $t22 := $tmp;

    // $t5 := $t22
    call $tmp := $CopyOrMoveValue($t22);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 4927, 5, $tmp); }

    // if ($t5) goto L2 else goto L3
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t24 := move($t30)
    call $tmp := $CopyOrMoveValue($t30);
    $t24 := $tmp;

    // destroy($t24)

    // $t25 := 3
    $tmp := $Integer(3);
    $t25 := $tmp;

    // abort($t25)
    if (true) { assume $DebugTrackAbort(14, 4927); }
    goto Abort;

    // L2:
L2:

    // $t26 := move($t30)
    call $tmp := $CopyOrMoveValue($t30);
    $t26 := $tmp;

    // $t27 := 2
    $tmp := $Integer(2);
    $t27 := $tmp;

    // $t28 := pack Roles::RoleId($t27)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t27);
    $t28 := $tmp;

    // move_to<Roles::RoleId>($t28, $t26)
    call $MoveTo($Roles_RoleId_type_value(), $t28, $t26);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 5018);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Roles_new_designated_dealer_role_$direct_inter(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_designated_dealer_role_$def(creating_account, new_account);
}


procedure {:inline 1} $Roles_new_designated_dealer_role_$direct_intra(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_designated_dealer_role_$def(creating_account, new_account);
}


procedure {:inline 1} $Roles_new_designated_dealer_role(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_designated_dealer_role_$def(creating_account, new_account);
}


procedure {:inline 1} $Roles_new_parent_vasp_role_$def(creating_account: $Value, new_account: $Value) returns ()
{
    // declare local variables
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $Roles_RoleId_type_value()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 7358, 0, creating_account); }
    if (true) { assume $DebugTrackLocal(14, 7358, 1, new_account); }

    // bytecode translation starts here
    // $t21 := move(creating_account)
    call $tmp := $CopyOrMoveValue(creating_account);
    $t21 := $tmp;

    // $t22 := move(new_account)
    call $tmp := $CopyOrMoveValue(new_account);
    $t22 := $tmp;

    // $t6 := move($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t6 := $tmp;

    // $t7 := Roles::has_libra_root_role($t6)
    call $t7 := $Roles_has_libra_root_role($t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10636);
      goto Abort;
    }
    assume is#$Boolean($t7);


    // $t2 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 7488, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t9 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t9 := $tmp;

    // destroy($t9)

    // $t10 := 3
    $tmp := $Integer(3);
    $t10 := $tmp;

    // abort($t10)
    if (true) { assume $DebugTrackAbort(14, 7488); }
    goto Abort;

    // L0:
L0:

    // $t11 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t11 := $tmp;

    // $t12 := Signer::address_of($t11)
    call $t12 := $Signer_address_of($t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 7669);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := exists<Roles::RoleId>($t12)
    call $tmp := $Exists($t12, $Roles_RoleId_type_value());
    $t13 := $tmp;

    // $t14 := !($t13)
    call $tmp := $Not($t13);
    $t14 := $tmp;

    // $t4 := $t14
    call $tmp := $CopyOrMoveValue($t14);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 7638, 4, $tmp); }

    // if ($t4) goto L2 else goto L3
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t16 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t16 := $tmp;

    // destroy($t16)

    // $t17 := 4
    $tmp := $Integer(4);
    $t17 := $tmp;

    // abort($t17)
    if (true) { assume $DebugTrackAbort(14, 7638); }
    goto Abort;

    // L2:
L2:

    // $t18 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t18 := $tmp;

    // $t19 := 5
    $tmp := $Integer(5);
    $t19 := $tmp;

    // $t20 := pack Roles::RoleId($t19)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t19);
    $t20 := $tmp;

    // move_to<Roles::RoleId>($t20, $t18)
    call $MoveTo($Roles_RoleId_type_value(), $t20, $t18);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 7728);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Roles_new_parent_vasp_role_$direct_inter(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_parent_vasp_role_$def(creating_account, new_account);
}


procedure {:inline 1} $Roles_new_parent_vasp_role_$direct_intra(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_parent_vasp_role_$def(creating_account, new_account);
}


procedure {:inline 1} $Roles_new_parent_vasp_role(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_parent_vasp_role_$def(creating_account, new_account);
}


procedure {:inline 1} $Roles_new_unhosted_role_$def(_creating_account: $Value, new_account: $Value) returns ()
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
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $Roles_RoleId_type_value()
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 9170, 0, _creating_account); }
    if (true) { assume $DebugTrackLocal(14, 9170, 1, new_account); }

    // bytecode translation starts here
    // $t14 := move(_creating_account)
    call $tmp := $CopyOrMoveValue(_creating_account);
    $t14 := $tmp;

    // $t15 := move(new_account)
    call $tmp := $CopyOrMoveValue(new_account);
    $t15 := $tmp;

    // $t4 := copy($t15)
    call $tmp := $CopyOrMoveValue($t15);
    $t4 := $tmp;

    // $t5 := Signer::address_of($t4)
    call $t5 := $Signer_address_of($t4);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 9363);
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
    if (true) { assume $DebugTrackLocal(14, 9332, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t9 := move($t15)
    call $tmp := $CopyOrMoveValue($t15);
    $t9 := $tmp;

    // destroy($t9)

    // $t10 := 4
    $tmp := $Integer(4);
    $t10 := $tmp;

    // abort($t10)
    if (true) { assume $DebugTrackAbort(14, 9332); }
    goto Abort;

    // L0:
L0:

    // $t11 := move($t15)
    call $tmp := $CopyOrMoveValue($t15);
    $t11 := $tmp;

    // $t12 := 7
    $tmp := $Integer(7);
    $t12 := $tmp;

    // $t13 := pack Roles::RoleId($t12)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t12);
    $t13 := $tmp;

    // move_to<Roles::RoleId>($t13, $t11)
    call $MoveTo($Roles_RoleId_type_value(), $t13, $t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 9422);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Roles_new_unhosted_role_$direct_inter(_creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(_creating_account);

    assume is#$Address(new_account);

    call $Roles_new_unhosted_role_$def(_creating_account, new_account);
}


procedure {:inline 1} $Roles_new_unhosted_role_$direct_intra(_creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(_creating_account);

    assume is#$Address(new_account);

    call $Roles_new_unhosted_role_$def(_creating_account, new_account);
}


procedure {:inline 1} $Roles_new_unhosted_role(_creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(_creating_account);

    assume is#$Address(new_account);

    call $Roles_new_unhosted_role_$def(_creating_account, new_account);
}


procedure {:inline 1} $Roles_new_validator_operator_role_$def(creating_account: $Value, new_account: $Value) returns ()
{
    // declare local variables
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $Roles_RoleId_type_value()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 6438, 0, creating_account); }
    if (true) { assume $DebugTrackLocal(14, 6438, 1, new_account); }

    // bytecode translation starts here
    // $t21 := move(creating_account)
    call $tmp := $CopyOrMoveValue(creating_account);
    $t21 := $tmp;

    // $t22 := move(new_account)
    call $tmp := $CopyOrMoveValue(new_account);
    $t22 := $tmp;

    // $t6 := move($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t6 := $tmp;

    // $t7 := Roles::has_libra_root_role($t6)
    call $t7 := $Roles_has_libra_root_role($t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10636);
      goto Abort;
    }
    assume is#$Boolean($t7);


    // $t2 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 6575, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t9 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t9 := $tmp;

    // destroy($t9)

    // $t10 := 3
    $tmp := $Integer(3);
    $t10 := $tmp;

    // abort($t10)
    if (true) { assume $DebugTrackAbort(14, 6575); }
    goto Abort;

    // L0:
L0:

    // $t11 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t11 := $tmp;

    // $t12 := Signer::address_of($t11)
    call $t12 := $Signer_address_of($t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 6756);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := exists<Roles::RoleId>($t12)
    call $tmp := $Exists($t12, $Roles_RoleId_type_value());
    $t13 := $tmp;

    // $t14 := !($t13)
    call $tmp := $Not($t13);
    $t14 := $tmp;

    // $t4 := $t14
    call $tmp := $CopyOrMoveValue($t14);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 6725, 4, $tmp); }

    // if ($t4) goto L2 else goto L3
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t16 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t16 := $tmp;

    // destroy($t16)

    // $t17 := 4
    $tmp := $Integer(4);
    $t17 := $tmp;

    // abort($t17)
    if (true) { assume $DebugTrackAbort(14, 6725); }
    goto Abort;

    // L2:
L2:

    // $t18 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t18 := $tmp;

    // $t19 := 4
    $tmp := $Integer(4);
    $t19 := $tmp;

    // $t20 := pack Roles::RoleId($t19)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t19);
    $t20 := $tmp;

    // move_to<Roles::RoleId>($t20, $t18)
    call $MoveTo($Roles_RoleId_type_value(), $t20, $t18);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 6815);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Roles_new_validator_operator_role_$direct_inter(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_validator_operator_role_$def(creating_account, new_account);
}


procedure {:inline 1} $Roles_new_validator_operator_role_$direct_intra(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_validator_operator_role_$def(creating_account, new_account);
}


procedure {:inline 1} $Roles_new_validator_operator_role(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_validator_operator_role_$def(creating_account, new_account);
}


procedure {:inline 1} $Roles_new_validator_role_$def(creating_account: $Value, new_account: $Value) returns ()
{
    // declare local variables
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $Roles_RoleId_type_value()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(14, 5557, 0, creating_account); }
    if (true) { assume $DebugTrackLocal(14, 5557, 1, new_account); }

    // bytecode translation starts here
    // $t21 := move(creating_account)
    call $tmp := $CopyOrMoveValue(creating_account);
    $t21 := $tmp;

    // $t22 := move(new_account)
    call $tmp := $CopyOrMoveValue(new_account);
    $t22 := $tmp;

    // $t6 := move($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t6 := $tmp;

    // $t7 := Roles::has_libra_root_role($t6)
    call $t7 := $Roles_has_libra_root_role($t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 10636);
      goto Abort;
    }
    assume is#$Boolean($t7);


    // $t2 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 5684, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t9 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t9 := $tmp;

    // destroy($t9)

    // $t10 := 3
    $tmp := $Integer(3);
    $t10 := $tmp;

    // abort($t10)
    if (true) { assume $DebugTrackAbort(14, 5684); }
    goto Abort;

    // L0:
L0:

    // $t11 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t11 := $tmp;

    // $t12 := Signer::address_of($t11)
    call $t12 := $Signer_address_of($t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 5865);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := exists<Roles::RoleId>($t12)
    call $tmp := $Exists($t12, $Roles_RoleId_type_value());
    $t13 := $tmp;

    // $t14 := !($t13)
    call $tmp := $Not($t13);
    $t14 := $tmp;

    // $t4 := $t14
    call $tmp := $CopyOrMoveValue($t14);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(14, 5834, 4, $tmp); }

    // if ($t4) goto L2 else goto L3
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t16 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t16 := $tmp;

    // destroy($t16)

    // $t17 := 4
    $tmp := $Integer(4);
    $t17 := $tmp;

    // abort($t17)
    if (true) { assume $DebugTrackAbort(14, 5834); }
    goto Abort;

    // L2:
L2:

    // $t18 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t18 := $tmp;

    // $t19 := 3
    $tmp := $Integer(3);
    $t19 := $tmp;

    // $t20 := pack Roles::RoleId($t19)
    call $tmp := $Roles_RoleId_pack(0, 0, 0, $t19);
    $t20 := $tmp;

    // move_to<Roles::RoleId>($t20, $t18)
    call $MoveTo($Roles_RoleId_type_value(), $t20, $t18);
    if ($abort_flag) {
      assume $DebugTrackAbort(14, 5924);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Roles_new_validator_role_$direct_inter(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_validator_role_$def(creating_account, new_account);
}


procedure {:inline 1} $Roles_new_validator_role_$direct_intra(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_validator_role_$def(creating_account, new_account);
}


procedure {:inline 1} $Roles_new_validator_role(creating_account: $Value, new_account: $Value) returns ()
{
    assume is#$Address(creating_account);

    assume is#$Address(new_account);

    call $Roles_new_validator_role_$def(creating_account, new_account);
}




// ** spec vars of module Vector



// ** spec funs of module Vector

function {:inline} $Vector_spec_contains($tv0: $TypeValue, v: $Value, e: $Value): $Value {
    $Boolean((var $range_1 := v; (exists $i_0: int :: $InVectorRange($range_1, $i_0) && (var x := $select_vector($range_1, $i_0); b#$Boolean($Boolean($IsEqual(x, e)))))))
}

function {:inline} $Vector_eq_push_back($tv0: $TypeValue, v1: $Value, v2: $Value, e: $Value): $Value {
    $Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($vlen_value(v1), $Integer(i#$Integer($vlen_value(v2)) + i#$Integer($Integer(1)))))) && b#$Boolean($Boolean($IsEqual($select_vector_by_value(v1, $Integer(i#$Integer($vlen_value(v1)) - i#$Integer($Integer(1)))), e))))) && b#$Boolean($Boolean($IsEqual($slice_vector(v1, $Range($Integer(0), $Integer(i#$Integer($vlen_value(v1)) - i#$Integer($Integer(1))))), $slice_vector(v2, $Range($Integer(0), $vlen_value(v2)))))))
}

function {:inline} $Vector_eq_append($tv0: $TypeValue, v: $Value, v1: $Value, v2: $Value): $Value {
    $Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($vlen_value(v), $Integer(i#$Integer($vlen_value(v1)) + i#$Integer($vlen_value(v2)))))) && b#$Boolean($Boolean($IsEqual($slice_vector(v, $Range($Integer(0), $vlen_value(v1))), v1))))) && b#$Boolean($Boolean($IsEqual($slice_vector(v, $Range($vlen_value(v1), $vlen_value(v))), v2))))
}

function {:inline} $Vector_eq_pop_front($tv0: $TypeValue, v1: $Value, v2: $Value): $Value {
    $Boolean(b#$Boolean($Boolean($IsEqual($Integer(i#$Integer($vlen_value(v1)) + i#$Integer($Integer(1))), $vlen_value(v2)))) && b#$Boolean($Boolean($IsEqual(v1, $slice_vector(v2, $Range($Integer(1), $vlen_value(v2)))))))
}



// ** structs of module Vector



// ** functions of module Vector

procedure {:inline 1} $Vector_singleton_$def($tv0: $TypeValue, e: $Value) returns ($ret0: $Value)
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(16, 1249, 0, e); }

    // bytecode translation starts here
    // $t6 := move(e)
    call $tmp := $CopyOrMoveValue(e);
    $t6 := $tmp;

    // $t2 := Vector::empty<#0>()
    call $t2 := $Vector_empty($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(16, 179);
      goto Abort;
    }
    assume $Vector_is_well_formed($t2);


    // v := $t2
    call $tmp := $CopyOrMoveValue($t2);
    v := $tmp;
    if (true) { assume $DebugTrackLocal(16, 1322, 1, $tmp); }

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
      assume $DebugTrackAbort(16, 542);
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
    if (true) { assume $DebugTrackLocal(16, 1373, 8, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Vector_singleton_$direct_inter($tv0: $TypeValue, e: $Value) returns ($ret0: $Value)
{
    call $ret0 := $Vector_singleton_$def($tv0, e);
}


procedure {:inline 1} $Vector_singleton_$direct_intra($tv0: $TypeValue, e: $Value) returns ($ret0: $Value)
{
    call $ret0 := $Vector_singleton_$def($tv0, e);
}


procedure {:inline 1} $Vector_singleton($tv0: $TypeValue, e: $Value) returns ($ret0: $Value)
{
    call $ret0 := $Vector_singleton_$def($tv0, e);
}




// ** spec vars of module LCS



// ** spec funs of module LCS



// ** structs of module LCS



// ** functions of module LCS



// ** spec vars of module Event



// ** spec funs of module Event



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

procedure {:inline 1} $Event_EventHandle_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, counter: $Value, guid: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(counter);
    assume $Vector_is_well_formed(guid) && (forall $$0: int :: {$select_vector(guid,$$0)} $$0 >= 0 && $$0 < $vlen(guid) ==> $IsValidU8($select_vector(guid,$$0)));
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := counter][1 := guid], 2));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Event_EventHandle_unpack($tv0: $TypeValue, $struct: $Value) returns (counter: $Value, guid: $Value)
{
    assume is#$Vector($struct);
    counter := $SelectField($struct, $Event_EventHandle_counter);
    assume $IsValidU64(counter);
    guid := $SelectField($struct, $Event_EventHandle_guid);
    assume $Vector_is_well_formed(guid) && (forall $$0: int :: {$select_vector(guid,$$0)} $$0 >= 0 && $$0 < $vlen(guid) ==> $IsValidU8($select_vector(guid,$$0)));
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

procedure {:inline 1} $Event_EventHandleGenerator_pack($file_id: int, $byte_index: int, $var_idx: int, counter: $Value, addr: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(counter);
    assume is#$Address(addr);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := counter][1 := addr], 2));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Event_EventHandleGenerator_unpack($struct: $Value) returns (counter: $Value, addr: $Value)
{
    assume is#$Vector($struct);
    counter := $SelectField($struct, $Event_EventHandleGenerator_counter);
    assume $IsValidU64(counter);
    addr := $SelectField($struct, $Event_EventHandleGenerator_addr);
    assume is#$Address(addr);
}



// ** functions of module Event



// ** spec vars of module AccountFreezing



// ** spec funs of module AccountFreezing

function {:inline} $AccountFreezing_spec_account_is_frozen($m: $Memory, $txn: $Transaction, addr: $Value): $Value {
    $Boolean(b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), addr)) && b#$Boolean($SelectField($ResourceValue($m, $AccountFreezing_FreezingBit_type_value(), addr), $AccountFreezing_FreezingBit_is_frozen)))
}

function {:inline} $AccountFreezing_spec_account_is_not_frozen($m: $Memory, $txn: $Transaction, addr: $Value): $Value {
    $Boolean(b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), addr)) && b#$Boolean($Boolean(!b#$Boolean($SelectField($ResourceValue($m, $AccountFreezing_FreezingBit_type_value(), addr), $AccountFreezing_FreezingBit_is_frozen)))))
}



// ** structs of module AccountFreezing

const unique $AccountFreezing_FreezeAccountEvent: $TypeName;
const $AccountFreezing_FreezeAccountEvent_initiator_address: $FieldName;
axiom $AccountFreezing_FreezeAccountEvent_initiator_address == 0;
const $AccountFreezing_FreezeAccountEvent_frozen_address: $FieldName;
axiom $AccountFreezing_FreezeAccountEvent_frozen_address == 1;
function $AccountFreezing_FreezeAccountEvent_type_value(): $TypeValue {
    $StructType($AccountFreezing_FreezeAccountEvent, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $AddressType()][1 := $AddressType()], 2))
}
function {:inline} $AccountFreezing_FreezeAccountEvent_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && is#$Address($SelectField($this, $AccountFreezing_FreezeAccountEvent_initiator_address))
      && is#$Address($SelectField($this, $AccountFreezing_FreezeAccountEvent_frozen_address))
}
function {:inline} $AccountFreezing_FreezeAccountEvent_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && is#$Address($SelectField($this, $AccountFreezing_FreezeAccountEvent_initiator_address))
      && is#$Address($SelectField($this, $AccountFreezing_FreezeAccountEvent_frozen_address))
}

procedure {:inline 1} $AccountFreezing_FreezeAccountEvent_pack($file_id: int, $byte_index: int, $var_idx: int, initiator_address: $Value, frozen_address: $Value) returns ($struct: $Value)
{
    assume is#$Address(initiator_address);
    assume is#$Address(frozen_address);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := initiator_address][1 := frozen_address], 2));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $AccountFreezing_FreezeAccountEvent_unpack($struct: $Value) returns (initiator_address: $Value, frozen_address: $Value)
{
    assume is#$Vector($struct);
    initiator_address := $SelectField($struct, $AccountFreezing_FreezeAccountEvent_initiator_address);
    assume is#$Address(initiator_address);
    frozen_address := $SelectField($struct, $AccountFreezing_FreezeAccountEvent_frozen_address);
    assume is#$Address(frozen_address);
}

const unique $AccountFreezing_FreezeEventsHolder: $TypeName;
const $AccountFreezing_FreezeEventsHolder_freeze_event_handle: $FieldName;
axiom $AccountFreezing_FreezeEventsHolder_freeze_event_handle == 0;
const $AccountFreezing_FreezeEventsHolder_unfreeze_event_handle: $FieldName;
axiom $AccountFreezing_FreezeEventsHolder_unfreeze_event_handle == 1;
function $AccountFreezing_FreezeEventsHolder_type_value(): $TypeValue {
    $StructType($AccountFreezing_FreezeEventsHolder, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $Event_EventHandle_type_value($AccountFreezing_FreezeAccountEvent_type_value())][1 := $Event_EventHandle_type_value($AccountFreezing_UnfreezeAccountEvent_type_value())], 2))
}
function {:inline} $AccountFreezing_FreezeEventsHolder_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && $Event_EventHandle_is_well_formed_types($SelectField($this, $AccountFreezing_FreezeEventsHolder_freeze_event_handle))
      && $Event_EventHandle_is_well_formed_types($SelectField($this, $AccountFreezing_FreezeEventsHolder_unfreeze_event_handle))
}
function {:inline} $AccountFreezing_FreezeEventsHolder_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && $Event_EventHandle_is_well_formed($SelectField($this, $AccountFreezing_FreezeEventsHolder_freeze_event_handle))
      && $Event_EventHandle_is_well_formed($SelectField($this, $AccountFreezing_FreezeEventsHolder_unfreeze_event_handle))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $AccountFreezing_FreezeEventsHolder_is_well_formed($ResourceValue(m, $AccountFreezing_FreezeEventsHolder_type_value(), a))
);

procedure {:inline 1} $AccountFreezing_FreezeEventsHolder_pack($file_id: int, $byte_index: int, $var_idx: int, freeze_event_handle: $Value, unfreeze_event_handle: $Value) returns ($struct: $Value)
{
    assume $Event_EventHandle_is_well_formed(freeze_event_handle);
    assume $Event_EventHandle_is_well_formed(unfreeze_event_handle);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := freeze_event_handle][1 := unfreeze_event_handle], 2));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $AccountFreezing_FreezeEventsHolder_unpack($struct: $Value) returns (freeze_event_handle: $Value, unfreeze_event_handle: $Value)
{
    assume is#$Vector($struct);
    freeze_event_handle := $SelectField($struct, $AccountFreezing_FreezeEventsHolder_freeze_event_handle);
    assume $Event_EventHandle_is_well_formed(freeze_event_handle);
    unfreeze_event_handle := $SelectField($struct, $AccountFreezing_FreezeEventsHolder_unfreeze_event_handle);
    assume $Event_EventHandle_is_well_formed(unfreeze_event_handle);
}

const unique $AccountFreezing_FreezingBit: $TypeName;
const $AccountFreezing_FreezingBit_is_frozen: $FieldName;
axiom $AccountFreezing_FreezingBit_is_frozen == 0;
function $AccountFreezing_FreezingBit_type_value(): $TypeValue {
    $StructType($AccountFreezing_FreezingBit, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $BooleanType()], 1))
}
function {:inline} $AccountFreezing_FreezingBit_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $AccountFreezing_FreezingBit_is_frozen))
}
function {:inline} $AccountFreezing_FreezingBit_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $AccountFreezing_FreezingBit_is_frozen))
}

axiom (forall m: $Memory, a: $Value :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $AccountFreezing_FreezingBit_is_well_formed($ResourceValue(m, $AccountFreezing_FreezingBit_type_value(), a))
);

procedure {:inline 1} $AccountFreezing_FreezingBit_pack($file_id: int, $byte_index: int, $var_idx: int, is_frozen: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(is_frozen);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := is_frozen], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $AccountFreezing_FreezingBit_unpack($struct: $Value) returns (is_frozen: $Value)
{
    assume is#$Vector($struct);
    is_frozen := $SelectField($struct, $AccountFreezing_FreezingBit_is_frozen);
    assume is#$Boolean(is_frozen);
}

const unique $AccountFreezing_UnfreezeAccountEvent: $TypeName;
const $AccountFreezing_UnfreezeAccountEvent_initiator_address: $FieldName;
axiom $AccountFreezing_UnfreezeAccountEvent_initiator_address == 0;
const $AccountFreezing_UnfreezeAccountEvent_unfrozen_address: $FieldName;
axiom $AccountFreezing_UnfreezeAccountEvent_unfrozen_address == 1;
function $AccountFreezing_UnfreezeAccountEvent_type_value(): $TypeValue {
    $StructType($AccountFreezing_UnfreezeAccountEvent, $TypeValueArray($MapConstTypeValue($DefaultTypeValue()), 0), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $AddressType()][1 := $AddressType()], 2))
}
function {:inline} $AccountFreezing_UnfreezeAccountEvent_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && is#$Address($SelectField($this, $AccountFreezing_UnfreezeAccountEvent_initiator_address))
      && is#$Address($SelectField($this, $AccountFreezing_UnfreezeAccountEvent_unfrozen_address))
}
function {:inline} $AccountFreezing_UnfreezeAccountEvent_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 2
      && is#$Address($SelectField($this, $AccountFreezing_UnfreezeAccountEvent_initiator_address))
      && is#$Address($SelectField($this, $AccountFreezing_UnfreezeAccountEvent_unfrozen_address))
}

procedure {:inline 1} $AccountFreezing_UnfreezeAccountEvent_pack($file_id: int, $byte_index: int, $var_idx: int, initiator_address: $Value, unfrozen_address: $Value) returns ($struct: $Value)
{
    assume is#$Address(initiator_address);
    assume is#$Address(unfrozen_address);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := initiator_address][1 := unfrozen_address], 2));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $AccountFreezing_UnfreezeAccountEvent_unpack($struct: $Value) returns (initiator_address: $Value, unfrozen_address: $Value)
{
    assume is#$Vector($struct);
    initiator_address := $SelectField($struct, $AccountFreezing_UnfreezeAccountEvent_initiator_address);
    assume is#$Address(initiator_address);
    unfrozen_address := $SelectField($struct, $AccountFreezing_UnfreezeAccountEvent_unfrozen_address);
    assume is#$Address(unfrozen_address);
}



// ** functions of module AccountFreezing

procedure {:inline 1} $AccountFreezing_initialize_$def(lr_account: $Value) returns ()
{
    // declare local variables
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $Event_EventHandle_type_value($AccountFreezing_FreezeAccountEvent_type_value())
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $Event_EventHandle_type_value($AccountFreezing_UnfreezeAccountEvent_type_value())
    var $t21: $Value; // $AccountFreezing_FreezeEventsHolder_type_value()
    var $t22: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(17, 1267, 0, lr_account); }

    // bytecode translation starts here
    // $t22 := move(lr_account)
    call $tmp := $CopyOrMoveValue(lr_account);
    $t22 := $tmp;

    // $t5 := LibraTimestamp::is_genesis()
    call $t5 := $LibraTimestamp_is_genesis();
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 1343);
      goto Abort;
    }
    assume is#$Boolean($t5);


    // $t1 := $t5
    call $tmp := $CopyOrMoveValue($t5);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 1320, 1, $tmp); }

    // if ($t1) goto L0 else goto L1
    $tmp := $t1;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t7 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t7 := $tmp;

    // destroy($t7)

    // $t8 := 0
    $tmp := $Integer(0);
    $t8 := $tmp;

    // abort($t8)
    if (true) { assume $DebugTrackAbort(17, 1320); }
    goto Abort;

    // L0:
L0:

    // $t9 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t9 := $tmp;

    // $t10 := Signer::address_of($t9)
    call $t10 := $Signer_address_of($t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 1408);
      goto Abort;
    }
    assume is#$Address($t10);


    // $t11 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t11 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 1449);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := ==($t10, $t11)
    $tmp := $Boolean($IsEqual($t10, $t11));
    $t12 := $tmp;

    // $t3 := $t12
    call $tmp := $CopyOrMoveValue($t12);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 1380, 3, $tmp); }

    // if ($t3) goto L2 else goto L3
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t14 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t14 := $tmp;

    // destroy($t14)

    // $t15 := 1
    $tmp := $Integer(1);
    $t15 := $tmp;

    // abort($t15)
    if (true) { assume $DebugTrackAbort(17, 1380); }
    goto Abort;

    // L2:
L2:

    // $t16 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t16 := $tmp;

    // $t17 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t17 := $tmp;

    // $t18 := Event::new_event_handle<AccountFreezing::FreezeAccountEvent>($t17)
    call $t18 := $Event_new_event_handle($AccountFreezing_FreezeAccountEvent_type_value(), $t17);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 1610);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t18);


    // $t19 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t19 := $tmp;

    // $t20 := Event::new_event_handle<AccountFreezing::UnfreezeAccountEvent>($t19)
    call $t20 := $Event_new_event_handle($AccountFreezing_UnfreezeAccountEvent_type_value(), $t19);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 1682);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t20);


    // $t21 := pack AccountFreezing::FreezeEventsHolder($t18, $t20)
    call $tmp := $AccountFreezing_FreezeEventsHolder_pack(0, 0, 0, $t18, $t20);
    $t21 := $tmp;

    // move_to<AccountFreezing::FreezeEventsHolder>($t21, $t16)
    call $MoveTo($AccountFreezing_FreezeEventsHolder_type_value(), $t21, $t16);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 1529);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $AccountFreezing_initialize_$direct_inter(lr_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    call $AccountFreezing_initialize_$def(lr_account);
}


procedure {:inline 1} $AccountFreezing_initialize_$direct_intra(lr_account: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))));
{
    assume is#$Address(lr_account);

    call $AccountFreezing_initialize_$def(lr_account);
}


procedure {:inline 1} $AccountFreezing_initialize(lr_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    call $AccountFreezing_initialize_$def(lr_account);
}


procedure {:inline 1} $AccountFreezing_initialize_$def_verify(lr_account: $Value) returns ()
{
    // declare local variables
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $Event_EventHandle_type_value($AccountFreezing_FreezeAccountEvent_type_value())
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $Event_EventHandle_type_value($AccountFreezing_UnfreezeAccountEvent_type_value())
    var $t21: $Value; // $AccountFreezing_FreezeEventsHolder_type_value()
    var $t22: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(17, 1267, 0, lr_account); }

    // bytecode translation starts here
    // $t22 := move(lr_account)
    call $tmp := $CopyOrMoveValue(lr_account);
    $t22 := $tmp;

    // $t5 := LibraTimestamp::is_genesis()
    call $t5 := $LibraTimestamp_is_genesis_$direct_inter();
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 1343);
      goto Abort;
    }
    assume is#$Boolean($t5);


    // $t1 := $t5
    call $tmp := $CopyOrMoveValue($t5);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 1320, 1, $tmp); }

    // if ($t1) goto L0 else goto L1
    $tmp := $t1;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t7 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t7 := $tmp;

    // destroy($t7)

    // $t8 := 0
    $tmp := $Integer(0);
    $t8 := $tmp;

    // abort($t8)
    if (true) { assume $DebugTrackAbort(17, 1320); }
    goto Abort;

    // L0:
L0:

    // $t9 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t9 := $tmp;

    // $t10 := Signer::address_of($t9)
    call $t10 := $Signer_address_of_$direct_inter($t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 1408);
      goto Abort;
    }
    assume is#$Address($t10);


    // $t11 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t11 := $CoreAddresses_LIBRA_ROOT_ADDRESS_$direct_inter();
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 1449);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := ==($t10, $t11)
    $tmp := $Boolean($IsEqual($t10, $t11));
    $t12 := $tmp;

    // $t3 := $t12
    call $tmp := $CopyOrMoveValue($t12);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 1380, 3, $tmp); }

    // if ($t3) goto L2 else goto L3
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t14 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t14 := $tmp;

    // destroy($t14)

    // $t15 := 1
    $tmp := $Integer(1);
    $t15 := $tmp;

    // abort($t15)
    if (true) { assume $DebugTrackAbort(17, 1380); }
    goto Abort;

    // L2:
L2:

    // $t16 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t16 := $tmp;

    // $t17 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t17 := $tmp;

    // $t18 := Event::new_event_handle<AccountFreezing::FreezeAccountEvent>($t17)
    call $t18 := $Event_new_event_handle($AccountFreezing_FreezeAccountEvent_type_value(), $t17);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 1610);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t18);


    // $t19 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t19 := $tmp;

    // $t20 := Event::new_event_handle<AccountFreezing::UnfreezeAccountEvent>($t19)
    call $t20 := $Event_new_event_handle($AccountFreezing_UnfreezeAccountEvent_type_value(), $t19);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 1682);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t20);


    // $t21 := pack AccountFreezing::FreezeEventsHolder($t18, $t20)
    call $tmp := $AccountFreezing_FreezeEventsHolder_pack(0, 0, 0, $t18, $t20);
    $t21 := $tmp;

    // move_to<AccountFreezing::FreezeEventsHolder>($t21, $t16)
    call $MoveTo($AccountFreezing_FreezeEventsHolder_type_value(), $t21, $t16);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 1529);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure $AccountFreezing_initialize_$verify(lr_account: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))))
    || b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn))))
    || b#$Boolean($Boolean(!$IsEqual($Signer_spec_address_of($m, $txn, lr_account), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))
    || b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $Signer_spec_address_of($m, $txn, lr_account)));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))))
    || b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn))))
    || b#$Boolean($Boolean(!$IsEqual($Signer_spec_address_of($m, $txn, lr_account), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))
    || b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $Signer_spec_address_of($m, $txn, lr_account)));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))))
    || b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn))))
    || b#$Boolean($Boolean(!$IsEqual($Signer_spec_address_of($m, $txn, lr_account), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))
    || b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $Signer_spec_address_of($m, $txn, lr_account)));
ensures b#$Boolean(old($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!$IsEqual($Signer_spec_address_of($m, $txn, lr_account), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))) ==> $abort_flag;
ensures b#$Boolean(old($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $Signer_spec_address_of($m, $txn, lr_account)))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn))))))
    || b#$Boolean(old(($Boolean(!$IsEqual($Signer_spec_address_of($m, $txn, lr_account), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))))
    || b#$Boolean(old(($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $Signer_spec_address_of($m, $txn, lr_account))))));
ensures !$abort_flag ==> (b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $Signer_spec_address_of($m, $txn, lr_account))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS())))));
{
    assume is#$Address(lr_account);

    call $InitVerification();
    assume $Memory__is_well_formed($m);
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))));
    call $AccountFreezing_initialize_$def_verify(lr_account);
}


procedure {:inline 1} $AccountFreezing_account_is_frozen_$def(addr: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $AccountFreezing_FreezingBit_type_value()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(17, 4863, 0, addr); }

    // bytecode translation starts here
    // $t10 := move(addr)
    call $tmp := $CopyOrMoveValue(addr);
    $t10 := $tmp;

    // $t3 := exists<AccountFreezing::FreezingBit>($t10)
    call $tmp := $Exists($t10, $AccountFreezing_FreezingBit_type_value());
    $t3 := $tmp;

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t5 := get_global<AccountFreezing::FreezingBit>($t10)
    call $tmp := $GetGlobal($t10, $AccountFreezing_FreezingBit_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 4977);
      goto Abort;
    }
    assume $AccountFreezing_FreezingBit_is_well_formed($tmp);
    $t5 := $tmp;

    // $t6 := get_field<AccountFreezing::FreezingBit>.is_frozen($t5)
    call $tmp := $GetFieldFromValue($t5, $AccountFreezing_FreezingBit_is_frozen);
    assume is#$Boolean($tmp);
    $t6 := $tmp;

    // $t7 := move($t6)
    call $tmp := $CopyOrMoveValue($t6);
    $t7 := $tmp;

    // $t1 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 4948, 1, $tmp); }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t8 := false
    $tmp := $Boolean(false);
    $t8 := $tmp;

    // $t1 := $t8
    call $tmp := $CopyOrMoveValue($t8);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 4948, 1, $tmp); }

    // goto L3
    goto L3;

    // L3:
L3:

    // return $t1
    $ret0 := $t1;
    if (true) { assume $DebugTrackLocal(17, 4948, 11, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $AccountFreezing_account_is_frozen_$direct_inter(addr: $Value) returns ($ret0: $Value)
{
    assume is#$Address(addr);

    call $ret0 := $AccountFreezing_account_is_frozen_$def(addr);
}


procedure {:inline 1} $AccountFreezing_account_is_frozen_$direct_intra(addr: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))));
{
    assume is#$Address(addr);

    call $ret0 := $AccountFreezing_account_is_frozen_$def(addr);
}


procedure {:inline 1} $AccountFreezing_account_is_frozen(addr: $Value) returns ($ret0: $Value)
{
    assume is#$Address(addr);

    call $ret0 := $AccountFreezing_account_is_frozen_$def(addr);
}


procedure {:inline 1} $AccountFreezing_account_is_frozen_$def_verify(addr: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $AccountFreezing_FreezingBit_type_value()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(17, 4863, 0, addr); }

    // bytecode translation starts here
    // $t10 := move(addr)
    call $tmp := $CopyOrMoveValue(addr);
    $t10 := $tmp;

    // $t3 := exists<AccountFreezing::FreezingBit>($t10)
    call $tmp := $Exists($t10, $AccountFreezing_FreezingBit_type_value());
    $t3 := $tmp;

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t5 := get_global<AccountFreezing::FreezingBit>($t10)
    call $tmp := $GetGlobal($t10, $AccountFreezing_FreezingBit_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 4977);
      goto Abort;
    }
    assume $AccountFreezing_FreezingBit_is_well_formed($tmp);
    $t5 := $tmp;

    // $t6 := get_field<AccountFreezing::FreezingBit>.is_frozen($t5)
    call $tmp := $GetFieldFromValue($t5, $AccountFreezing_FreezingBit_is_frozen);
    assume is#$Boolean($tmp);
    $t6 := $tmp;

    // $t7 := move($t6)
    call $tmp := $CopyOrMoveValue($t6);
    $t7 := $tmp;

    // $t1 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 4948, 1, $tmp); }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t8 := false
    $tmp := $Boolean(false);
    $t8 := $tmp;

    // $t1 := $t8
    call $tmp := $CopyOrMoveValue($t8);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 4948, 1, $tmp); }

    // goto L3
    goto L3;

    // L3:
L3:

    // return $t1
    $ret0 := $t1;
    if (true) { assume $DebugTrackLocal(17, 4948, 11, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure $AccountFreezing_account_is_frozen_$verify(addr: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))))
    || b#$Boolean($Boolean(false));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))))
    || b#$Boolean($Boolean(false));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))))
    || b#$Boolean($Boolean(false));
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(false)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $AccountFreezing_spec_account_is_frozen($m, $txn, addr)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS())))));
{
    assume is#$Address(addr);

    call $InitVerification();
    assume $Memory__is_well_formed($m);
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))));
    call $ret0 := $AccountFreezing_account_is_frozen_$def_verify(addr);
}


procedure {:inline 1} $AccountFreezing_create_$def(account: $Value) returns ()
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $AccountFreezing_FreezingBit_type_value()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(17, 2084, 0, account); }

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

    // $t3 := pack AccountFreezing::FreezingBit($t2)
    call $tmp := $AccountFreezing_FreezingBit_pack(0, 0, 0, $t2);
    $t3 := $tmp;

    // move_to<AccountFreezing::FreezingBit>($t3, $t1)
    call $MoveTo($AccountFreezing_FreezingBit_type_value(), $t3, $t1);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 2130);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $AccountFreezing_create_$direct_inter(account: $Value) returns ()
{
    assume is#$Address(account);

    call $AccountFreezing_create_$def(account);
}


procedure {:inline 1} $AccountFreezing_create_$direct_intra(account: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))));
{
    assume is#$Address(account);

    call $AccountFreezing_create_$def(account);
}


procedure {:inline 1} $AccountFreezing_create(account: $Value) returns ()
{
    assume is#$Address(account);

    call $AccountFreezing_create_$def(account);
}


procedure {:inline 1} $AccountFreezing_create_$def_verify(account: $Value) returns ()
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $AccountFreezing_FreezingBit_type_value()
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(17, 2084, 0, account); }

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

    // $t3 := pack AccountFreezing::FreezingBit($t2)
    call $tmp := $AccountFreezing_FreezingBit_pack(0, 0, 0, $t2);
    $t3 := $tmp;

    // move_to<AccountFreezing::FreezingBit>($t3, $t1)
    call $MoveTo($AccountFreezing_FreezingBit_type_value(), $t3, $t1);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 2130);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure $AccountFreezing_create_$verify(account: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))))
    || b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), $Signer_spec_address_of($m, $txn, account)));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))))
    || b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), $Signer_spec_address_of($m, $txn, account)));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))))
    || b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), $Signer_spec_address_of($m, $txn, account)));
ensures b#$Boolean(old($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), $Signer_spec_address_of($m, $txn, account)))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), $Signer_spec_address_of($m, $txn, account))))));
ensures !$abort_flag ==> (b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $Signer_spec_address_of($m, $txn, account))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS())))));
{
    assume is#$Address(account);

    call $InitVerification();
    assume $Memory__is_well_formed($m);
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))));
    call $AccountFreezing_create_$def_verify(account);
}


procedure {:inline 1} $AccountFreezing_freeze_account_$def(account: $Value, frozen_address: $Value) returns ()
{
    // declare local variables
    var initiator_address: $Value; // $AddressType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $BooleanType()
    var $t11: $Value; // $BooleanType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $BooleanType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $AddressType()
    var $t23: $Value; // $BooleanType()
    var $t24: $Value; // $BooleanType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $BooleanType()
    var $t27: $Value; // $AddressType()
    var $t28: $Reference; // ReferenceType($AccountFreezing_FreezingBit_type_value())
    var $t29: $Reference; // ReferenceType($BooleanType())
    var $t30: $Value; // $AddressType()
    var $t31: $Reference; // ReferenceType($AccountFreezing_FreezeEventsHolder_type_value())
    var $t32: $Reference; // ReferenceType($Event_EventHandle_type_value($AccountFreezing_FreezeAccountEvent_type_value()))
    var $t33: $Value; // $AddressType()
    var $t34: $Value; // $AddressType()
    var $t35: $Value; // $AccountFreezing_FreezeAccountEvent_type_value()
    var $t36: $Value; // $AddressType()
    var $t37: $Value; // $AddressType()
    var $t38: $Value; // $Event_EventHandle_type_value($AccountFreezing_FreezeAccountEvent_type_value())
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(17, 2409, 0, account); }
    if (true) { assume $DebugTrackLocal(17, 2409, 1, frozen_address); }

    // bytecode translation starts here
    // $t36 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t36 := $tmp;

    // $t37 := move(frozen_address)
    call $tmp := $CopyOrMoveValue(frozen_address);
    $t37 := $tmp;

    // $t9 := copy($t36)
    call $tmp := $CopyOrMoveValue($t36);
    $t9 := $tmp;

    // $t10 := Roles::has_treasury_compliance_role($t9)
    call $t10 := $Roles_has_treasury_compliance_role($t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 2570);
      goto Abort;
    }
    assume is#$Boolean($t10);


    // $t3 := $t10
    call $tmp := $CopyOrMoveValue($t10);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 2556, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t12 := move($t36)
    call $tmp := $CopyOrMoveValue($t36);
    $t12 := $tmp;

    // destroy($t12)

    // $t13 := 2
    $tmp := $Integer(2);
    $t13 := $tmp;

    // abort($t13)
    if (true) { assume $DebugTrackAbort(17, 2556); }
    goto Abort;

    // L0:
L0:

    // $t14 := move($t36)
    call $tmp := $CopyOrMoveValue($t36);
    $t14 := $tmp;

    // $t15 := Signer::address_of($t14)
    call $t15 := $Signer_address_of($t14);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 2671);
      goto Abort;
    }
    assume is#$Address($t15);


    // initiator_address := $t15
    call $tmp := $CopyOrMoveValue($t15);
    initiator_address := $tmp;
    if (true) { assume $DebugTrackLocal(17, 2643, 2, $tmp); }

    // $t17 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t17 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 2798);
      goto Abort;
    }
    assume is#$Address($t17);


    // $t18 := !=($t37, $t17)
    $tmp := $Boolean(!$IsEqual($t37, $t17));
    $t18 := $tmp;

    // $t5 := $t18
    call $tmp := $CopyOrMoveValue($t18);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 2758, 5, $tmp); }

    // if ($t5) goto L2 else goto L3
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t20 := 3
    $tmp := $Integer(3);
    $t20 := $tmp;

    // abort($t20)
    if (true) { assume $DebugTrackAbort(17, 2758); }
    goto Abort;

    // L2:
L2:

    // $t22 := CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()
    call $t22 := $CoreAddresses_TREASURY_COMPLIANCE_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 2896);
      goto Abort;
    }
    assume is#$Address($t22);


    // $t23 := !=($t37, $t22)
    $tmp := $Boolean(!$IsEqual($t37, $t22));
    $t23 := $tmp;

    // $t7 := $t23
    call $tmp := $CopyOrMoveValue($t23);
    $t7 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 2856, 7, $tmp); }

    // if ($t7) goto L4 else goto L5
    $tmp := $t7;
    if (b#$Boolean($tmp)) { goto L4; } else { goto L5; }

    // L5:
L5:

    // $t25 := 4
    $tmp := $Integer(4);
    $t25 := $tmp;

    // abort($t25)
    if (true) { assume $DebugTrackAbort(17, 2856); }
    goto Abort;

    // L4:
L4:

    // $t26 := true
    $tmp := $Boolean(true);
    $t26 := $tmp;

    // $t28 := borrow_global<AccountFreezing::FreezingBit>($t37)
    call $t28 := $BorrowGlobal($t37, $AccountFreezing_FreezingBit_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 2955);
      goto Abort;
    }
    assume $AccountFreezing_FreezingBit_is_well_formed($Dereference($t28));

    // UnpackRef($t28)

    // $t29 := borrow_field<AccountFreezing::FreezingBit>.is_frozen($t28)
    call $t29 := $BorrowField($t28, $AccountFreezing_FreezingBit_is_frozen);
    assume is#$Boolean($Dereference($t29));

    // AccountFreezing::FreezingBit <- $t28
    call $WritebackToGlobal($t28);

    // UnpackRef($t29)

    // write_ref($t29, $t26)
    call $t29 := $WriteRef($t29, $t26);

    // AccountFreezing::FreezingBit <- $t29
    call $WritebackToGlobal($t29);

    // Reference($t28) <- $t29
    call $t28 := $WritebackToReference($t29, $t28);

    // PackRef($t28)

    // PackRef($t29)

    // $t30 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t30 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 3137);
      goto Abort;
    }
    assume is#$Address($t30);


    // $t31 := borrow_global<AccountFreezing::FreezeEventsHolder>($t30)
    call $t31 := $BorrowGlobal($t30, $AccountFreezing_FreezeEventsHolder_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 3084);
      goto Abort;
    }
    assume $AccountFreezing_FreezeEventsHolder_is_well_formed($Dereference($t31));

    // UnpackRef($t31)

    // $t32 := borrow_field<AccountFreezing::FreezeEventsHolder>.freeze_event_handle($t31)
    call $t32 := $BorrowField($t31, $AccountFreezing_FreezeEventsHolder_freeze_event_handle);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t32));

    // AccountFreezing::FreezeEventsHolder <- $t31
    call $WritebackToGlobal($t31);

    // UnpackRef($t32)

    // $t35 := pack AccountFreezing::FreezeAccountEvent(initiator_address, $t37)
    call $tmp := $AccountFreezing_FreezeAccountEvent_pack(0, 0, 0, initiator_address, $t37);
    $t35 := $tmp;

    // PackRef($t32)

    // $t38 := read_ref($t32)
    call $tmp := $ReadRef($t32);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t38 := $tmp;

    // $t38 := Event::emit_event<AccountFreezing::FreezeAccountEvent>($t38, $t35)
    call $t38 := $Event_emit_event($AccountFreezing_FreezeAccountEvent_type_value(), $t38, $t35);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 3035);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t38);


    // write_ref($t32, $t38)
    call $t32 := $WriteRef($t32, $t38);

    // AccountFreezing::FreezeEventsHolder <- $t32
    call $WritebackToGlobal($t32);

    // Reference($t31) <- $t32
    call $t31 := $WritebackToReference($t32, $t31);

    // UnpackRef($t32)

    // PackRef($t31)

    // PackRef($t32)

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $AccountFreezing_freeze_account_$direct_inter(account: $Value, frozen_address: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(frozen_address);

    call $AccountFreezing_freeze_account_$def(account, frozen_address);
}


procedure {:inline 1} $AccountFreezing_freeze_account_$direct_intra(account: $Value, frozen_address: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))));
{
    assume is#$Address(account);

    assume is#$Address(frozen_address);

    call $AccountFreezing_freeze_account_$def(account, frozen_address);
}


procedure {:inline 1} $AccountFreezing_freeze_account(account: $Value, frozen_address: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(frozen_address);

    call $AccountFreezing_freeze_account_$def(account, frozen_address);
}


procedure {:inline 1} $AccountFreezing_freeze_account_$def_verify(account: $Value, frozen_address: $Value) returns ()
{
    // declare local variables
    var initiator_address: $Value; // $AddressType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $BooleanType()
    var $t11: $Value; // $BooleanType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $BooleanType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $AddressType()
    var $t23: $Value; // $BooleanType()
    var $t24: $Value; // $BooleanType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $BooleanType()
    var $t27: $Value; // $AddressType()
    var $t28: $Reference; // ReferenceType($AccountFreezing_FreezingBit_type_value())
    var $t29: $Reference; // ReferenceType($BooleanType())
    var $t30: $Value; // $AddressType()
    var $t31: $Reference; // ReferenceType($AccountFreezing_FreezeEventsHolder_type_value())
    var $t32: $Reference; // ReferenceType($Event_EventHandle_type_value($AccountFreezing_FreezeAccountEvent_type_value()))
    var $t33: $Value; // $AddressType()
    var $t34: $Value; // $AddressType()
    var $t35: $Value; // $AccountFreezing_FreezeAccountEvent_type_value()
    var $t36: $Value; // $AddressType()
    var $t37: $Value; // $AddressType()
    var $t38: $Value; // $Event_EventHandle_type_value($AccountFreezing_FreezeAccountEvent_type_value())
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(17, 2409, 0, account); }
    if (true) { assume $DebugTrackLocal(17, 2409, 1, frozen_address); }

    // bytecode translation starts here
    // $t36 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t36 := $tmp;

    // $t37 := move(frozen_address)
    call $tmp := $CopyOrMoveValue(frozen_address);
    $t37 := $tmp;

    // $t9 := copy($t36)
    call $tmp := $CopyOrMoveValue($t36);
    $t9 := $tmp;

    // $t10 := Roles::has_treasury_compliance_role($t9)
    call $t10 := $Roles_has_treasury_compliance_role_$direct_inter($t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 2570);
      goto Abort;
    }
    assume is#$Boolean($t10);


    // $t3 := $t10
    call $tmp := $CopyOrMoveValue($t10);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 2556, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t12 := move($t36)
    call $tmp := $CopyOrMoveValue($t36);
    $t12 := $tmp;

    // destroy($t12)

    // $t13 := 2
    $tmp := $Integer(2);
    $t13 := $tmp;

    // abort($t13)
    if (true) { assume $DebugTrackAbort(17, 2556); }
    goto Abort;

    // L0:
L0:

    // $t14 := move($t36)
    call $tmp := $CopyOrMoveValue($t36);
    $t14 := $tmp;

    // $t15 := Signer::address_of($t14)
    call $t15 := $Signer_address_of_$direct_inter($t14);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 2671);
      goto Abort;
    }
    assume is#$Address($t15);


    // initiator_address := $t15
    call $tmp := $CopyOrMoveValue($t15);
    initiator_address := $tmp;
    if (true) { assume $DebugTrackLocal(17, 2643, 2, $tmp); }

    // $t17 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t17 := $CoreAddresses_LIBRA_ROOT_ADDRESS_$direct_inter();
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 2798);
      goto Abort;
    }
    assume is#$Address($t17);


    // $t18 := !=($t37, $t17)
    $tmp := $Boolean(!$IsEqual($t37, $t17));
    $t18 := $tmp;

    // $t5 := $t18
    call $tmp := $CopyOrMoveValue($t18);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 2758, 5, $tmp); }

    // if ($t5) goto L2 else goto L3
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t20 := 3
    $tmp := $Integer(3);
    $t20 := $tmp;

    // abort($t20)
    if (true) { assume $DebugTrackAbort(17, 2758); }
    goto Abort;

    // L2:
L2:

    // $t22 := CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()
    call $t22 := $CoreAddresses_TREASURY_COMPLIANCE_ADDRESS_$direct_inter();
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 2896);
      goto Abort;
    }
    assume is#$Address($t22);


    // $t23 := !=($t37, $t22)
    $tmp := $Boolean(!$IsEqual($t37, $t22));
    $t23 := $tmp;

    // $t7 := $t23
    call $tmp := $CopyOrMoveValue($t23);
    $t7 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 2856, 7, $tmp); }

    // if ($t7) goto L4 else goto L5
    $tmp := $t7;
    if (b#$Boolean($tmp)) { goto L4; } else { goto L5; }

    // L5:
L5:

    // $t25 := 4
    $tmp := $Integer(4);
    $t25 := $tmp;

    // abort($t25)
    if (true) { assume $DebugTrackAbort(17, 2856); }
    goto Abort;

    // L4:
L4:

    // $t26 := true
    $tmp := $Boolean(true);
    $t26 := $tmp;

    // $t28 := borrow_global<AccountFreezing::FreezingBit>($t37)
    call $t28 := $BorrowGlobal($t37, $AccountFreezing_FreezingBit_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 2955);
      goto Abort;
    }
    assume $AccountFreezing_FreezingBit_is_well_formed($Dereference($t28));

    // UnpackRef($t28)

    // $t29 := borrow_field<AccountFreezing::FreezingBit>.is_frozen($t28)
    call $t29 := $BorrowField($t28, $AccountFreezing_FreezingBit_is_frozen);
    assume is#$Boolean($Dereference($t29));

    // AccountFreezing::FreezingBit <- $t28
    call $WritebackToGlobal($t28);

    // UnpackRef($t29)

    // write_ref($t29, $t26)
    call $t29 := $WriteRef($t29, $t26);

    // AccountFreezing::FreezingBit <- $t29
    call $WritebackToGlobal($t29);

    // Reference($t28) <- $t29
    call $t28 := $WritebackToReference($t29, $t28);

    // PackRef($t28)

    // PackRef($t29)

    // $t30 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t30 := $CoreAddresses_LIBRA_ROOT_ADDRESS_$direct_inter();
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 3137);
      goto Abort;
    }
    assume is#$Address($t30);


    // $t31 := borrow_global<AccountFreezing::FreezeEventsHolder>($t30)
    call $t31 := $BorrowGlobal($t30, $AccountFreezing_FreezeEventsHolder_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 3084);
      goto Abort;
    }
    assume $AccountFreezing_FreezeEventsHolder_is_well_formed($Dereference($t31));

    // UnpackRef($t31)

    // $t32 := borrow_field<AccountFreezing::FreezeEventsHolder>.freeze_event_handle($t31)
    call $t32 := $BorrowField($t31, $AccountFreezing_FreezeEventsHolder_freeze_event_handle);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t32));

    // AccountFreezing::FreezeEventsHolder <- $t31
    call $WritebackToGlobal($t31);

    // UnpackRef($t32)

    // $t35 := pack AccountFreezing::FreezeAccountEvent(initiator_address, $t37)
    call $tmp := $AccountFreezing_FreezeAccountEvent_pack(0, 0, 0, initiator_address, $t37);
    $t35 := $tmp;

    // PackRef($t32)

    // $t38 := read_ref($t32)
    call $tmp := $ReadRef($t32);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t38 := $tmp;

    // $t38 := Event::emit_event<AccountFreezing::FreezeAccountEvent>($t38, $t35)
    call $t38 := $Event_emit_event($AccountFreezing_FreezeAccountEvent_type_value(), $t38, $t35);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 3035);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t38);


    // write_ref($t32, $t38)
    call $t32 := $WriteRef($t32, $t38);

    // AccountFreezing::FreezeEventsHolder <- $t32
    call $WritebackToGlobal($t32);

    // Reference($t31) <- $t32
    call $t31 := $WritebackToReference($t32, $t31);

    // UnpackRef($t32)

    // PackRef($t31)

    // PackRef($t32)

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure $AccountFreezing_freeze_account_$verify(account: $Value, frozen_address: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))))
    || b#$Boolean($Boolean(!b#$Boolean($Roles_spec_has_treasury_compliance_role($m, $txn, account))))
    || b#$Boolean($Boolean($IsEqual(frozen_address, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))
    || b#$Boolean($Boolean($IsEqual(frozen_address, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS())))
    || b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), frozen_address))))
    || b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))))
    || b#$Boolean($Boolean(!b#$Boolean($Roles_spec_has_treasury_compliance_role($m, $txn, account))))
    || b#$Boolean($Boolean($IsEqual(frozen_address, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))
    || b#$Boolean($Boolean($IsEqual(frozen_address, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS())))
    || b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), frozen_address))))
    || b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))))
    || b#$Boolean($Boolean(!b#$Boolean($Roles_spec_has_treasury_compliance_role($m, $txn, account))))
    || b#$Boolean($Boolean($IsEqual(frozen_address, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))
    || b#$Boolean($Boolean($IsEqual(frozen_address, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS())))
    || b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), frozen_address))))
    || b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
ensures b#$Boolean(old($Boolean(!b#$Boolean($Roles_spec_has_treasury_compliance_role($m, $txn, account))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean($IsEqual(frozen_address, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean($IsEqual(frozen_address, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS())))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), frozen_address))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(!b#$Boolean($Roles_spec_has_treasury_compliance_role($m, $txn, account))))))
    || b#$Boolean(old(($Boolean($IsEqual(frozen_address, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))))
    || b#$Boolean(old(($Boolean($IsEqual(frozen_address, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS())))))
    || b#$Boolean(old(($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), frozen_address))))))
    || b#$Boolean(old(($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))))));
ensures !$abort_flag ==> (b#$Boolean($AccountFreezing_spec_account_is_frozen($m, $txn, frozen_address)));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS())))));
{
    assume is#$Address(account);

    assume is#$Address(frozen_address);

    call $InitVerification();
    assume $Memory__is_well_formed($m);
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))));
    call $AccountFreezing_freeze_account_$def_verify(account, frozen_address);
}


procedure {:inline 1} $AccountFreezing_unfreeze_account_$def(account: $Value, unfrozen_address: $Value) returns ()
{
    // declare local variables
    var initiator_address: $Value; // $AddressType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $AddressType()
    var $t14: $Reference; // ReferenceType($AccountFreezing_FreezingBit_type_value())
    var $t15: $Reference; // ReferenceType($BooleanType())
    var $t16: $Value; // $AddressType()
    var $t17: $Reference; // ReferenceType($AccountFreezing_FreezeEventsHolder_type_value())
    var $t18: $Reference; // ReferenceType($Event_EventHandle_type_value($AccountFreezing_UnfreezeAccountEvent_type_value()))
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $AddressType()
    var $t21: $Value; // $AccountFreezing_UnfreezeAccountEvent_type_value()
    var $t22: $Value; // $AddressType()
    var $t23: $Value; // $AddressType()
    var $t24: $Value; // $Event_EventHandle_type_value($AccountFreezing_UnfreezeAccountEvent_type_value())
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(17, 3828, 0, account); }
    if (true) { assume $DebugTrackLocal(17, 3828, 1, unfrozen_address); }

    // bytecode translation starts here
    // $t22 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t22 := $tmp;

    // $t23 := move(unfrozen_address)
    call $tmp := $CopyOrMoveValue(unfrozen_address);
    $t23 := $tmp;

    // $t5 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t5 := $tmp;

    // $t6 := Roles::has_treasury_compliance_role($t5)
    call $t6 := $Roles_has_treasury_compliance_role($t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 3993);
      goto Abort;
    }
    assume is#$Boolean($t6);


    // $t3 := $t6
    call $tmp := $CopyOrMoveValue($t6);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 3979, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t8 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t8 := $tmp;

    // destroy($t8)

    // $t9 := 5
    $tmp := $Integer(5);
    $t9 := $tmp;

    // abort($t9)
    if (true) { assume $DebugTrackAbort(17, 3979); }
    goto Abort;

    // L0:
L0:

    // $t10 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t10 := $tmp;

    // $t11 := Signer::address_of($t10)
    call $t11 := $Signer_address_of($t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 4096);
      goto Abort;
    }
    assume is#$Address($t11);


    // initiator_address := $t11
    call $tmp := $CopyOrMoveValue($t11);
    initiator_address := $tmp;
    if (true) { assume $DebugTrackLocal(17, 4068, 2, $tmp); }

    // $t12 := false
    $tmp := $Boolean(false);
    $t12 := $tmp;

    // $t14 := borrow_global<AccountFreezing::FreezingBit>($t23)
    call $t14 := $BorrowGlobal($t23, $AccountFreezing_FreezingBit_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 4125);
      goto Abort;
    }
    assume $AccountFreezing_FreezingBit_is_well_formed($Dereference($t14));

    // UnpackRef($t14)

    // $t15 := borrow_field<AccountFreezing::FreezingBit>.is_frozen($t14)
    call $t15 := $BorrowField($t14, $AccountFreezing_FreezingBit_is_frozen);
    assume is#$Boolean($Dereference($t15));

    // AccountFreezing::FreezingBit <- $t14
    call $WritebackToGlobal($t14);

    // UnpackRef($t15)

    // write_ref($t15, $t12)
    call $t15 := $WriteRef($t15, $t12);

    // AccountFreezing::FreezingBit <- $t15
    call $WritebackToGlobal($t15);

    // Reference($t14) <- $t15
    call $t14 := $WritebackToReference($t15, $t14);

    // PackRef($t14)

    // PackRef($t15)

    // $t16 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t16 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 4312);
      goto Abort;
    }
    assume is#$Address($t16);


    // $t17 := borrow_global<AccountFreezing::FreezeEventsHolder>($t16)
    call $t17 := $BorrowGlobal($t16, $AccountFreezing_FreezeEventsHolder_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 4259);
      goto Abort;
    }
    assume $AccountFreezing_FreezeEventsHolder_is_well_formed($Dereference($t17));

    // UnpackRef($t17)

    // $t18 := borrow_field<AccountFreezing::FreezeEventsHolder>.unfreeze_event_handle($t17)
    call $t18 := $BorrowField($t17, $AccountFreezing_FreezeEventsHolder_unfreeze_event_handle);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t18));

    // AccountFreezing::FreezeEventsHolder <- $t17
    call $WritebackToGlobal($t17);

    // UnpackRef($t18)

    // $t21 := pack AccountFreezing::UnfreezeAccountEvent(initiator_address, $t23)
    call $tmp := $AccountFreezing_UnfreezeAccountEvent_pack(0, 0, 0, initiator_address, $t23);
    $t21 := $tmp;

    // PackRef($t18)

    // $t24 := read_ref($t18)
    call $tmp := $ReadRef($t18);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t24 := $tmp;

    // $t24 := Event::emit_event<AccountFreezing::UnfreezeAccountEvent>($t24, $t21)
    call $t24 := $Event_emit_event($AccountFreezing_UnfreezeAccountEvent_type_value(), $t24, $t21);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 4208);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t24);


    // write_ref($t18, $t24)
    call $t18 := $WriteRef($t18, $t24);

    // AccountFreezing::FreezeEventsHolder <- $t18
    call $WritebackToGlobal($t18);

    // Reference($t17) <- $t18
    call $t17 := $WritebackToReference($t18, $t17);

    // UnpackRef($t18)

    // PackRef($t17)

    // PackRef($t18)

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $AccountFreezing_unfreeze_account_$direct_inter(account: $Value, unfrozen_address: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(unfrozen_address);

    call $AccountFreezing_unfreeze_account_$def(account, unfrozen_address);
}


procedure {:inline 1} $AccountFreezing_unfreeze_account_$direct_intra(account: $Value, unfrozen_address: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))));
{
    assume is#$Address(account);

    assume is#$Address(unfrozen_address);

    call $AccountFreezing_unfreeze_account_$def(account, unfrozen_address);
}


procedure {:inline 1} $AccountFreezing_unfreeze_account(account: $Value, unfrozen_address: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(unfrozen_address);

    call $AccountFreezing_unfreeze_account_$def(account, unfrozen_address);
}


procedure {:inline 1} $AccountFreezing_unfreeze_account_$def_verify(account: $Value, unfrozen_address: $Value) returns ()
{
    // declare local variables
    var initiator_address: $Value; // $AddressType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $AddressType()
    var $t14: $Reference; // ReferenceType($AccountFreezing_FreezingBit_type_value())
    var $t15: $Reference; // ReferenceType($BooleanType())
    var $t16: $Value; // $AddressType()
    var $t17: $Reference; // ReferenceType($AccountFreezing_FreezeEventsHolder_type_value())
    var $t18: $Reference; // ReferenceType($Event_EventHandle_type_value($AccountFreezing_UnfreezeAccountEvent_type_value()))
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $AddressType()
    var $t21: $Value; // $AccountFreezing_UnfreezeAccountEvent_type_value()
    var $t22: $Value; // $AddressType()
    var $t23: $Value; // $AddressType()
    var $t24: $Value; // $Event_EventHandle_type_value($AccountFreezing_UnfreezeAccountEvent_type_value())
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(17, 3828, 0, account); }
    if (true) { assume $DebugTrackLocal(17, 3828, 1, unfrozen_address); }

    // bytecode translation starts here
    // $t22 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t22 := $tmp;

    // $t23 := move(unfrozen_address)
    call $tmp := $CopyOrMoveValue(unfrozen_address);
    $t23 := $tmp;

    // $t5 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t5 := $tmp;

    // $t6 := Roles::has_treasury_compliance_role($t5)
    call $t6 := $Roles_has_treasury_compliance_role_$direct_inter($t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 3993);
      goto Abort;
    }
    assume is#$Boolean($t6);


    // $t3 := $t6
    call $tmp := $CopyOrMoveValue($t6);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(17, 3979, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t8 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t8 := $tmp;

    // destroy($t8)

    // $t9 := 5
    $tmp := $Integer(5);
    $t9 := $tmp;

    // abort($t9)
    if (true) { assume $DebugTrackAbort(17, 3979); }
    goto Abort;

    // L0:
L0:

    // $t10 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t10 := $tmp;

    // $t11 := Signer::address_of($t10)
    call $t11 := $Signer_address_of_$direct_inter($t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 4096);
      goto Abort;
    }
    assume is#$Address($t11);


    // initiator_address := $t11
    call $tmp := $CopyOrMoveValue($t11);
    initiator_address := $tmp;
    if (true) { assume $DebugTrackLocal(17, 4068, 2, $tmp); }

    // $t12 := false
    $tmp := $Boolean(false);
    $t12 := $tmp;

    // $t14 := borrow_global<AccountFreezing::FreezingBit>($t23)
    call $t14 := $BorrowGlobal($t23, $AccountFreezing_FreezingBit_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 4125);
      goto Abort;
    }
    assume $AccountFreezing_FreezingBit_is_well_formed($Dereference($t14));

    // UnpackRef($t14)

    // $t15 := borrow_field<AccountFreezing::FreezingBit>.is_frozen($t14)
    call $t15 := $BorrowField($t14, $AccountFreezing_FreezingBit_is_frozen);
    assume is#$Boolean($Dereference($t15));

    // AccountFreezing::FreezingBit <- $t14
    call $WritebackToGlobal($t14);

    // UnpackRef($t15)

    // write_ref($t15, $t12)
    call $t15 := $WriteRef($t15, $t12);

    // AccountFreezing::FreezingBit <- $t15
    call $WritebackToGlobal($t15);

    // Reference($t14) <- $t15
    call $t14 := $WritebackToReference($t15, $t14);

    // PackRef($t14)

    // PackRef($t15)

    // $t16 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t16 := $CoreAddresses_LIBRA_ROOT_ADDRESS_$direct_inter();
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 4312);
      goto Abort;
    }
    assume is#$Address($t16);


    // $t17 := borrow_global<AccountFreezing::FreezeEventsHolder>($t16)
    call $t17 := $BorrowGlobal($t16, $AccountFreezing_FreezeEventsHolder_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 4259);
      goto Abort;
    }
    assume $AccountFreezing_FreezeEventsHolder_is_well_formed($Dereference($t17));

    // UnpackRef($t17)

    // $t18 := borrow_field<AccountFreezing::FreezeEventsHolder>.unfreeze_event_handle($t17)
    call $t18 := $BorrowField($t17, $AccountFreezing_FreezeEventsHolder_unfreeze_event_handle);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t18));

    // AccountFreezing::FreezeEventsHolder <- $t17
    call $WritebackToGlobal($t17);

    // UnpackRef($t18)

    // $t21 := pack AccountFreezing::UnfreezeAccountEvent(initiator_address, $t23)
    call $tmp := $AccountFreezing_UnfreezeAccountEvent_pack(0, 0, 0, initiator_address, $t23);
    $t21 := $tmp;

    // PackRef($t18)

    // $t24 := read_ref($t18)
    call $tmp := $ReadRef($t18);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t24 := $tmp;

    // $t24 := Event::emit_event<AccountFreezing::UnfreezeAccountEvent>($t24, $t21)
    call $t24 := $Event_emit_event($AccountFreezing_UnfreezeAccountEvent_type_value(), $t24, $t21);
    if ($abort_flag) {
      assume $DebugTrackAbort(17, 4208);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t24);


    // write_ref($t18, $t24)
    call $t18 := $WriteRef($t18, $t24);

    // AccountFreezing::FreezeEventsHolder <- $t18
    call $WritebackToGlobal($t18);

    // Reference($t17) <- $t18
    call $t17 := $WritebackToReference($t18, $t17);

    // UnpackRef($t18)

    // PackRef($t17)

    // PackRef($t18)

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure $AccountFreezing_unfreeze_account_$verify(account: $Value, unfrozen_address: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))))
    || b#$Boolean($Boolean(!b#$Boolean($Roles_spec_has_treasury_compliance_role($m, $txn, account))))
    || b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), unfrozen_address))))
    || b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))))
    || b#$Boolean($Boolean(!b#$Boolean($Roles_spec_has_treasury_compliance_role($m, $txn, account))))
    || b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), unfrozen_address))))
    || b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))))
    || b#$Boolean($Boolean(!b#$Boolean($Roles_spec_has_treasury_compliance_role($m, $txn, account))))
    || b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), unfrozen_address))))
    || b#$Boolean($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
ensures b#$Boolean(old($Boolean(!b#$Boolean($Roles_spec_has_treasury_compliance_role($m, $txn, account))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), unfrozen_address))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old(($Boolean(!b#$Boolean($Roles_spec_has_treasury_compliance_role($m, $txn, account))))))
    || b#$Boolean(old(($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezingBit_type_value(), unfrozen_address))))))
    || b#$Boolean(old(($Boolean(!b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(!b#$Boolean($AccountFreezing_spec_account_is_frozen($m, $txn, unfrozen_address)))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS())))));
{
    assume is#$Address(account);

    assume is#$Address(unfrozen_address);

    call $InitVerification();
    assume $Memory__is_well_formed($m);
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($ResourceExists($m, $AccountFreezing_FreezeEventsHolder_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()))));
    assume b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($AccountFreezing_spec_account_is_not_frozen($m, $txn, $CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS()))));
    call $AccountFreezing_unfreeze_account_$def_verify(account, unfrozen_address);
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

procedure {:inline 1} $Offer_address_of_$def($tv0: $TypeValue, offer_address: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $Offer_Offer_type_value($tv0)
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(12, 1494, 0, offer_address); }

    // bytecode translation starts here
    // $t5 := move(offer_address)
    call $tmp := $CopyOrMoveValue(offer_address);
    $t5 := $tmp;

    // $t2 := get_global<Offer::Offer<#0>>($t5)
    call $tmp := $GetGlobal($t5, $Offer_Offer_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(12, 1579);
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
    if (true) { assume $DebugTrackLocal(12, 1579, 6, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Offer_address_of_$direct_inter($tv0: $TypeValue, offer_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(offer_address);

    call $ret0 := $Offer_address_of_$def($tv0, offer_address);
}


procedure {:inline 1} $Offer_address_of_$direct_intra($tv0: $TypeValue, offer_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(offer_address);

    call $ret0 := $Offer_address_of_$def($tv0, offer_address);
}


procedure {:inline 1} $Offer_address_of($tv0: $TypeValue, offer_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(offer_address);

    call $ret0 := $Offer_address_of_$def($tv0, offer_address);
}


procedure {:inline 1} $Offer_create_$def($tv0: $TypeValue, account: $Value, offered: $Value, for: $Value) returns ()
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(12, 501, 0, account); }
    if (true) { assume $DebugTrackLocal(12, 501, 1, offered); }
    if (true) { assume $DebugTrackLocal(12, 501, 2, for); }

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
      assume $DebugTrackAbort(12, 584);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Offer_create_$direct_inter($tv0: $TypeValue, account: $Value, offered: $Value, for: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(for);

    call $Offer_create_$def($tv0, account, offered, for);
}


procedure {:inline 1} $Offer_create_$direct_intra($tv0: $TypeValue, account: $Value, offered: $Value, for: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(for);

    call $Offer_create_$def($tv0, account, offered, for);
}


procedure {:inline 1} $Offer_create($tv0: $TypeValue, account: $Value, offered: $Value, for: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(for);

    call $Offer_create_$def($tv0, account, offered, for);
}


procedure {:inline 1} $Offer_exists_at_$def($tv0: $TypeValue, offer_address: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(12, 1272, 0, offer_address); }

    // bytecode translation starts here
    // $t3 := move(offer_address)
    call $tmp := $CopyOrMoveValue(offer_address);
    $t3 := $tmp;

    // $t2 := exists<Offer::Offer<#0>>($t3)
    call $tmp := $Exists($t3, $Offer_Offer_type_value($tv0));
    $t2 := $tmp;

    // return $t2
    $ret0 := $t2;
    if (true) { assume $DebugTrackLocal(12, 1338, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Offer_exists_at_$direct_inter($tv0: $TypeValue, offer_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(offer_address);

    call $ret0 := $Offer_exists_at_$def($tv0, offer_address);
}


procedure {:inline 1} $Offer_exists_at_$direct_intra($tv0: $TypeValue, offer_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(offer_address);

    call $ret0 := $Offer_exists_at_$def($tv0, offer_address);
}


procedure {:inline 1} $Offer_exists_at($tv0: $TypeValue, offer_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(offer_address);

    call $ret0 := $Offer_exists_at_$def($tv0, offer_address);
}


procedure {:inline 1} $Offer_redeem_$def($tv0: $TypeValue, account: $Value, offer_address: $Value) returns ($ret0: $Value)
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
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $tv0
    var $t25: $Value; // $AddressType()
    var $t26: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(12, 875, 0, account); }
    if (true) { assume $DebugTrackLocal(12, 875, 1, offer_address); }

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
      assume $DebugTrackAbort(12, 1012);
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
    if (true) { assume $DebugTrackLocal(12, 1004, 2, $tmp); }

    // offered := $t10
    call $tmp := $CopyOrMoveValue($t10);
    offered := $tmp;
    if (true) { assume $DebugTrackLocal(12, 995, 3, $tmp); }

    // $t12 := move($t25)
    call $tmp := $CopyOrMoveValue($t25);
    $t12 := $tmp;

    // $t13 := Signer::address_of($t12)
    call $t13 := $Signer_address_of($t12);
    if ($abort_flag) {
      assume $DebugTrackAbort(12, 1079);
      goto Abort;
    }
    assume is#$Address($t13);


    // sender := $t13
    call $tmp := $CopyOrMoveValue($t13);
    sender := $tmp;
    if (true) { assume $DebugTrackLocal(12, 1062, 4, $tmp); }

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
    if (true) { assume $DebugTrackLocal(12, 1111, 7, $tmp); }

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
    if (true) { assume $DebugTrackLocal(12, 1111, 7, $tmp); }

    // goto L3
    goto L3;

    // L3:
L3:

    // $t5 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(12, 1104, 5, $tmp); }

    // if ($t5) goto L4 else goto L5
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L4; } else { goto L5; }

    // L5:
L5:

    // $t23 := 0
    $tmp := $Integer(0);
    $t23 := $tmp;

    // abort($t23)
    if (true) { assume $DebugTrackAbort(12, 1104); }
    goto Abort;

    // L4:
L4:

    // return offered
    $ret0 := offered;
    if (true) { assume $DebugTrackLocal(12, 1182, 27, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Offer_redeem_$direct_inter($tv0: $TypeValue, account: $Value, offer_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    assume is#$Address(offer_address);

    call $ret0 := $Offer_redeem_$def($tv0, account, offer_address);
}


procedure {:inline 1} $Offer_redeem_$direct_intra($tv0: $TypeValue, account: $Value, offer_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    assume is#$Address(offer_address);

    call $ret0 := $Offer_redeem_$def($tv0, account, offer_address);
}


procedure {:inline 1} $Offer_redeem($tv0: $TypeValue, account: $Value, offer_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    assume is#$Address(offer_address);

    call $ret0 := $Offer_redeem_$def($tv0, account, offer_address);
}




// ** spec vars of module LibraConfig



// ** spec funs of module LibraConfig

function {:inline} $LibraConfig_spec_has_config($m: $Memory, $txn: $Transaction): $Value {
    $ResourceExists($m, $LibraConfig_Configuration_type_value(), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())
}

function {:inline} $LibraConfig_spec_get($m: $Memory, $txn: $Transaction, $tv0: $TypeValue): $Value {
    $SelectField($ResourceValue($m, $LibraConfig_LibraConfig_type_value($tv0), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS()), $LibraConfig_LibraConfig_payload)
}

function {:inline} $LibraConfig_spec_is_published($m: $Memory, $txn: $Transaction, $tv0: $TypeValue): $Value {
    $ResourceExists($m, $LibraConfig_LibraConfig_type_value($tv0), $CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS())
}

function {:inline} $LibraConfig_spec_has_on_chain_config_privilege($m: $Memory, $txn: $Transaction, account: $Value): $Value {
    $Roles_spec_has_libra_root_role($m, $txn, account)
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

procedure {:inline 1} $LibraConfig_initialize_$def(config_account: $Value) returns ()
{
    // declare local variables
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $Event_EventHandle_type_value($LibraConfig_NewEpochEvent_type_value())
    var $t21: $Value; // $LibraConfig_Configuration_type_value()
    var $t22: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 1177, 0, config_account); }

    // bytecode translation starts here
    // $t22 := move(config_account)
    call $tmp := $CopyOrMoveValue(config_account);
    $t22 := $tmp;

    // $t5 := LibraTimestamp::is_genesis()
    call $t5 := $LibraTimestamp_is_genesis();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 1272);
      goto Abort;
    }
    assume is#$Boolean($t5);


    // $t1 := $t5
    call $tmp := $CopyOrMoveValue($t5);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 1249, 1, $tmp); }

    // if ($t1) goto L0 else goto L1
    $tmp := $t1;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t7 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t7 := $tmp;

    // destroy($t7)

    // $t8 := 0
    $tmp := $Integer(0);
    $t8 := $tmp;

    // abort($t8)
    if (true) { assume $DebugTrackAbort(10, 1249); }
    goto Abort;

    // L0:
L0:

    // $t9 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t9 := $tmp;

    // $t10 := Signer::address_of($t9)
    call $t10 := $Signer_address_of($t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 1358);
      goto Abort;
    }
    assume is#$Address($t10);


    // $t11 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t11 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 1403);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := ==($t10, $t11)
    $tmp := $Boolean($IsEqual($t10, $t11));
    $t12 := $tmp;

    // $t3 := $t12
    call $tmp := $CopyOrMoveValue($t12);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 1343, 3, $tmp); }

    // if ($t3) goto L2 else goto L3
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t14 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t14 := $tmp;

    // destroy($t14)

    // $t15 := 2
    $tmp := $Integer(2);
    $t15 := $tmp;

    // abort($t15)
    if (true) { assume $DebugTrackAbort(10, 1343); }
    goto Abort;

    // L2:
L2:

    // $t16 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t16 := $tmp;

    // $t17 := 0
    $tmp := $Integer(0);
    $t17 := $tmp;

    // $t18 := 0
    $tmp := $Integer(0);
    $t18 := $tmp;

    // $t19 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t19 := $tmp;

    // $t20 := Event::new_event_handle<LibraConfig::NewEpochEvent>($t19)
    call $t20 := $Event_new_event_handle($LibraConfig_NewEpochEvent_type_value(), $t19);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 1645);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t20);


    // $t21 := pack LibraConfig::Configuration($t17, $t18, $t20)
    call $tmp := $LibraConfig_Configuration_pack(0, 0, 0, $t17, $t18, $t20);
    $t21 := $tmp;

    // move_to<LibraConfig::Configuration>($t21, $t16)
    call $MoveTo($LibraConfig_Configuration_type_value(), $t21, $t16);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 1462);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $LibraConfig_initialize_$direct_inter(config_account: $Value) returns ()
{
    assume is#$Address(config_account);

    call $LibraConfig_initialize_$def(config_account);
}


procedure {:inline 1} $LibraConfig_initialize_$direct_intra(config_account: $Value) returns ()
{
    assume is#$Address(config_account);

    call $LibraConfig_initialize_$def(config_account);
}


procedure {:inline 1} $LibraConfig_initialize(config_account: $Value) returns ()
{
    assume is#$Address(config_account);

    call $LibraConfig_initialize_$def(config_account);
}


procedure {:inline 1} $LibraConfig_claim_delegated_modify_config_$def($tv0: $TypeValue, account: $Value, offer_address: $Value) returns ()
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $LibraConfig_ModifyConfigCapability_type_value($tv0)
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 6417, 0, account); }
    if (true) { assume $DebugTrackLocal(10, 6417, 1, offer_address); }

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
      assume $DebugTrackAbort(10, 6542);
      goto Abort;
    }
    assume $LibraConfig_ModifyConfigCapability_is_well_formed($t5);


    // move_to<LibraConfig::ModifyConfigCapability<#0>>($t5, $t2)
    call $MoveTo($LibraConfig_ModifyConfigCapability_type_value($tv0), $t5, $t2);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 6518);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $LibraConfig_claim_delegated_modify_config_$direct_inter($tv0: $TypeValue, account: $Value, offer_address: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(offer_address);

    call $LibraConfig_claim_delegated_modify_config_$def($tv0, account, offer_address);
}


procedure {:inline 1} $LibraConfig_claim_delegated_modify_config_$direct_intra($tv0: $TypeValue, account: $Value, offer_address: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(offer_address);

    call $LibraConfig_claim_delegated_modify_config_$def($tv0, account, offer_address);
}


procedure {:inline 1} $LibraConfig_claim_delegated_modify_config($tv0: $TypeValue, account: $Value, offer_address: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(offer_address);

    call $LibraConfig_claim_delegated_modify_config_$def($tv0, account, offer_address);
}


procedure {:inline 1} $LibraConfig_emit_reconfiguration_event_$def() returns ()
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t1 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t1 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 8716);
      goto Abort;
    }
    assume is#$Address($t1);


    // $t2 := borrow_global<LibraConfig::Configuration>($t1)
    call $t2 := $BorrowGlobal($t1, $LibraConfig_Configuration_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 8668);
      goto Abort;
    }
    assume $LibraConfig_Configuration_is_well_formed($Dereference($t2));

    // UnpackRef($t2)

    // config_ref := $t2
    call config_ref := $CopyOrMoveRef($t2);
    if (true) { assume $DebugTrackLocal(10, 8655, 0, $Dereference(config_ref)); }

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
      assume $DebugTrackAbort(10, 8783);
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
    if (true) { assume $DebugTrackLocal(10, 8747, 0, $Dereference(config_ref)); }

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

    // $t16 := read_ref($t11)
    call $tmp := $ReadRef($t11);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t16 := $tmp;

    // $t16 := Event::emit_event<LibraConfig::NewEpochEvent>($t16, $t15)
    call $t16 := $Event_emit_event($LibraConfig_NewEpochEvent_type_value(), $t16, $t15);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 8804);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t16);


    // write_ref($t11, $t16)
    call $t11 := $WriteRef($t11, $t16);
    if (true) { assume $DebugTrackLocal(10, 8585, 0, $Dereference(config_ref)); }

    // LibraConfig::Configuration <- $t11
    call $WritebackToGlobal($t11);

    // Reference($t10) <- $t11
    call $t10 := $WritebackToReference($t11, $t10);

    // Reference($t12) <- $t11
    call $t12 := $WritebackToReference($t11, $t12);

    // UnpackRef($t11)

    // PackRef($t11)

    // PackRef($t12)

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $LibraConfig_emit_reconfiguration_event_$direct_intra() returns ()
{
    call $LibraConfig_emit_reconfiguration_event_$def();
}


procedure {:inline 1} $LibraConfig_emit_reconfiguration_event() returns ()
{
    call $LibraConfig_emit_reconfiguration_event_$def();
}


procedure {:inline 1} $LibraConfig_get_$def($tv0: $TypeValue) returns ($ret0: $Value)
{
    // declare local variables
    var addr: $Value; // $AddressType()
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $LibraConfig_LibraConfig_type_value($tv0)
    var $t10: $Value; // $tv0
    var $t11: $Value; // $tv0
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t3 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t3 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 1891);
      goto Abort;
    }
    assume is#$Address($t3);


    // addr := $t3
    call $tmp := $CopyOrMoveValue($t3);
    addr := $tmp;
    if (true) { assume $DebugTrackLocal(10, 1869, 0, $tmp); }

    // $t5 := exists<LibraConfig::LibraConfig<#0>>(addr)
    call $tmp := $Exists(addr, $LibraConfig_LibraConfig_type_value($tv0));
    $t5 := $tmp;

    // $t1 := $t5
    call $tmp := $CopyOrMoveValue($t5);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 1921, 1, $tmp); }

    // if ($t1) goto L0 else goto L1
    $tmp := $t1;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t7 := 3
    $tmp := $Integer(3);
    $t7 := $tmp;

    // abort($t7)
    if (true) { assume $DebugTrackAbort(10, 1921); }
    goto Abort;

    // L0:
L0:

    // $t9 := get_global<LibraConfig::LibraConfig<#0>>(addr)
    call $tmp := $GetGlobal(addr, $LibraConfig_LibraConfig_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 1998);
      goto Abort;
    }
    assume $LibraConfig_LibraConfig_is_well_formed($tmp);
    $t9 := $tmp;

    // $t10 := get_field<LibraConfig::LibraConfig<#0>>.payload($t9)
    call $tmp := $GetFieldFromValue($t9, $LibraConfig_LibraConfig_payload);
    $t10 := $tmp;

    // $t11 := move($t10)
    call $tmp := $CopyOrMoveValue($t10);
    $t11 := $tmp;

    // return $t11
    $ret0 := $t11;
    if (true) { assume $DebugTrackLocal(10, 1996, 12, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $LibraConfig_get_$direct_inter($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $LibraConfig_get_$def($tv0);
}


procedure {:inline 1} $LibraConfig_get_$direct_intra($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $LibraConfig_get_$def($tv0);
}


procedure {:inline 1} $LibraConfig_get($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $LibraConfig_get_$def($tv0);
}


procedure {:inline 1} $LibraConfig_publish_new_config_$def($tv0: $TypeValue, config_account: $Value, payload: $Value) returns ()
{
    // declare local variables
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $LibraConfig_ModifyConfigCapability_type_value($tv0)
    var $t21: $Value; // $AddressType()
    var $t22: $Value; // $tv0
    var $t23: $Value; // $LibraConfig_LibraConfig_type_value($tv0)
    var $t24: $Value; // $AddressType()
    var $t25: $Value; // $tv0
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 4690, 0, config_account); }
    if (true) { assume $DebugTrackLocal(10, 4690, 1, payload); }

    // bytecode translation starts here
    // $t24 := move(config_account)
    call $tmp := $CopyOrMoveValue(config_account);
    $t24 := $tmp;

    // $t25 := move(payload)
    call $tmp := $CopyOrMoveValue(payload);
    $t25 := $tmp;

    // $t6 := copy($t24)
    call $tmp := $CopyOrMoveValue($t24);
    $t6 := $tmp;

    // $t7 := Roles::has_on_chain_config_privilege($t6)
    call $t7 := $Roles_has_on_chain_config_privilege($t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 4826);
      goto Abort;
    }
    assume is#$Boolean($t7);


    // $t2 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 4812, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t9 := move($t24)
    call $tmp := $CopyOrMoveValue($t24);
    $t9 := $tmp;

    // destroy($t9)

    // $t10 := 5
    $tmp := $Integer(5);
    $t10 := $tmp;

    // abort($t10)
    if (true) { assume $DebugTrackAbort(10, 4812); }
    goto Abort;

    // L0:
L0:

    // $t11 := copy($t24)
    call $tmp := $CopyOrMoveValue($t24);
    $t11 := $tmp;

    // $t12 := Signer::address_of($t11)
    call $t12 := $Signer_address_of($t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 4919);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t13 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 4964);
      goto Abort;
    }
    assume is#$Address($t13);


    // $t14 := ==($t12, $t13)
    $tmp := $Boolean($IsEqual($t12, $t13));
    $t14 := $tmp;

    // $t4 := $t14
    call $tmp := $CopyOrMoveValue($t14);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 4904, 4, $tmp); }

    // if ($t4) goto L2 else goto L3
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t16 := move($t24)
    call $tmp := $CopyOrMoveValue($t24);
    $t16 := $tmp;

    // destroy($t16)

    // $t17 := 2
    $tmp := $Integer(2);
    $t17 := $tmp;

    // abort($t17)
    if (true) { assume $DebugTrackAbort(10, 4904); }
    goto Abort;

    // L2:
L2:

    // $t18 := copy($t24)
    call $tmp := $CopyOrMoveValue($t24);
    $t18 := $tmp;

    // $t19 := false
    $tmp := $Boolean(false);
    $t19 := $tmp;

    // $t20 := pack LibraConfig::ModifyConfigCapability<#0>($t19)
    call $tmp := $LibraConfig_ModifyConfigCapability_pack(0, 0, 0, $tv0, $t19);
    $t20 := $tmp;

    // move_to<LibraConfig::ModifyConfigCapability<#0>>($t20, $t18)
    call $MoveTo($LibraConfig_ModifyConfigCapability_type_value($tv0), $t20, $t18);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 5023);
      goto Abort;
    }

    // $t21 := move($t24)
    call $tmp := $CopyOrMoveValue($t24);
    $t21 := $tmp;

    // $t23 := pack LibraConfig::LibraConfig<#0>($t25)
    call $tmp := $LibraConfig_LibraConfig_pack(0, 0, 0, $tv0, $t25);
    $t23 := $tmp;

    // move_to<LibraConfig::LibraConfig<#0>>($t23, $t21)
    call $MoveTo($LibraConfig_LibraConfig_type_value($tv0), $t23, $t21);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 5091);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $LibraConfig_publish_new_config_$direct_inter($tv0: $TypeValue, config_account: $Value, payload: $Value) returns ()
{
    assume is#$Address(config_account);

    call $LibraConfig_publish_new_config_$def($tv0, config_account, payload);
}


procedure {:inline 1} $LibraConfig_publish_new_config_$direct_intra($tv0: $TypeValue, config_account: $Value, payload: $Value) returns ()
{
    assume is#$Address(config_account);

    call $LibraConfig_publish_new_config_$def($tv0, config_account, payload);
}


procedure {:inline 1} $LibraConfig_publish_new_config($tv0: $TypeValue, config_account: $Value, payload: $Value) returns ()
{
    assume is#$Address(config_account);

    call $LibraConfig_publish_new_config_$def($tv0, config_account, payload);
}


procedure {:inline 1} $LibraConfig_publish_new_config_with_capability_$def($tv0: $TypeValue, config_account: $Value, payload: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $tv0
    var $t20: $Value; // $LibraConfig_LibraConfig_type_value($tv0)
    var $t21: $Value; // $BooleanType()
    var $t22: $Value; // $LibraConfig_ModifyConfigCapability_type_value($tv0)
    var $t23: $Value; // $AddressType()
    var $t24: $Value; // $tv0
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 3398, 0, config_account); }
    if (true) { assume $DebugTrackLocal(10, 3398, 1, payload); }

    // bytecode translation starts here
    // $t23 := move(config_account)
    call $tmp := $CopyOrMoveValue(config_account);
    $t23 := $tmp;

    // $t24 := move(payload)
    call $tmp := $CopyOrMoveValue(payload);
    $t24 := $tmp;

    // $t6 := copy($t23)
    call $tmp := $CopyOrMoveValue($t23);
    $t6 := $tmp;

    // $t7 := Roles::has_on_chain_config_privilege($t6)
    call $t7 := $Roles_has_on_chain_config_privilege($t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 3583);
      goto Abort;
    }
    assume is#$Boolean($t7);


    // $t2 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 3569, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t9 := move($t23)
    call $tmp := $CopyOrMoveValue($t23);
    $t9 := $tmp;

    // destroy($t9)

    // $t10 := 5
    $tmp := $Integer(5);
    $t10 := $tmp;

    // abort($t10)
    if (true) { assume $DebugTrackAbort(10, 3569); }
    goto Abort;

    // L0:
L0:

    // $t11 := copy($t23)
    call $tmp := $CopyOrMoveValue($t23);
    $t11 := $tmp;

    // $t12 := Signer::address_of($t11)
    call $t12 := $Signer_address_of($t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 3676);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t13 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 3721);
      goto Abort;
    }
    assume is#$Address($t13);


    // $t14 := ==($t12, $t13)
    $tmp := $Boolean($IsEqual($t12, $t13));
    $t14 := $tmp;

    // $t4 := $t14
    call $tmp := $CopyOrMoveValue($t14);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 3661, 4, $tmp); }

    // if ($t4) goto L2 else goto L3
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t16 := move($t23)
    call $tmp := $CopyOrMoveValue($t23);
    $t16 := $tmp;

    // destroy($t16)

    // $t17 := 2
    $tmp := $Integer(2);
    $t17 := $tmp;

    // abort($t17)
    if (true) { assume $DebugTrackAbort(10, 3661); }
    goto Abort;

    // L2:
L2:

    // $t18 := move($t23)
    call $tmp := $CopyOrMoveValue($t23);
    $t18 := $tmp;

    // $t20 := pack LibraConfig::LibraConfig<#0>($t24)
    call $tmp := $LibraConfig_LibraConfig_pack(0, 0, 0, $tv0, $t24);
    $t20 := $tmp;

    // move_to<LibraConfig::LibraConfig<#0>>($t20, $t18)
    call $MoveTo($LibraConfig_LibraConfig_type_value($tv0), $t20, $t18);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 3780);
      goto Abort;
    }

    // $t21 := false
    $tmp := $Boolean(false);
    $t21 := $tmp;

    // $t22 := pack LibraConfig::ModifyConfigCapability<#0>($t21)
    call $tmp := $LibraConfig_ModifyConfigCapability_pack(0, 0, 0, $tv0, $t21);
    $t22 := $tmp;

    // return $t22
    $ret0 := $t22;
    if (true) { assume $DebugTrackLocal(10, 4106, 25, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $LibraConfig_publish_new_config_with_capability_$direct_inter($tv0: $TypeValue, config_account: $Value, payload: $Value) returns ($ret0: $Value)
{
    assume is#$Address(config_account);

    call $ret0 := $LibraConfig_publish_new_config_with_capability_$def($tv0, config_account, payload);
}


procedure {:inline 1} $LibraConfig_publish_new_config_with_capability_$direct_intra($tv0: $TypeValue, config_account: $Value, payload: $Value) returns ($ret0: $Value)
{
    assume is#$Address(config_account);

    call $ret0 := $LibraConfig_publish_new_config_with_capability_$def($tv0, config_account, payload);
}


procedure {:inline 1} $LibraConfig_publish_new_config_with_capability($tv0: $TypeValue, config_account: $Value, payload: $Value) returns ($ret0: $Value)
{
    assume is#$Address(config_account);

    call $ret0 := $LibraConfig_publish_new_config_with_capability_$def($tv0, config_account, payload);
}


procedure {:inline 1} $LibraConfig_publish_new_config_with_delegate_$def($tv0: $TypeValue, config_account: $Value, payload: $Value, delegate: $Value) returns ()
{
    // declare local variables
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $BooleanType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $BooleanType()
    var $t21: $Value; // $LibraConfig_ModifyConfigCapability_type_value($tv0)
    var $t22: $Value; // $AddressType()
    var $t23: $Value; // $AddressType()
    var $t24: $Value; // $tv0
    var $t25: $Value; // $LibraConfig_LibraConfig_type_value($tv0)
    var $t26: $Value; // $AddressType()
    var $t27: $Value; // $tv0
    var $t28: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 5535, 0, config_account); }
    if (true) { assume $DebugTrackLocal(10, 5535, 1, payload); }
    if (true) { assume $DebugTrackLocal(10, 5535, 2, delegate); }

    // bytecode translation starts here
    // $t26 := move(config_account)
    call $tmp := $CopyOrMoveValue(config_account);
    $t26 := $tmp;

    // $t27 := move(payload)
    call $tmp := $CopyOrMoveValue(payload);
    $t27 := $tmp;

    // $t28 := move(delegate)
    call $tmp := $CopyOrMoveValue(delegate);
    $t28 := $tmp;

    // $t7 := copy($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t7 := $tmp;

    // $t8 := Roles::has_on_chain_config_privilege($t7)
    call $t8 := $Roles_has_on_chain_config_privilege($t7);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 5713);
      goto Abort;
    }
    assume is#$Boolean($t8);


    // $t3 := $t8
    call $tmp := $CopyOrMoveValue($t8);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 5699, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t10 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t10 := $tmp;

    // destroy($t10)

    // $t11 := 5
    $tmp := $Integer(5);
    $t11 := $tmp;

    // abort($t11)
    if (true) { assume $DebugTrackAbort(10, 5699); }
    goto Abort;

    // L0:
L0:

    // $t12 := copy($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t12 := $tmp;

    // $t13 := Signer::address_of($t12)
    call $t13 := $Signer_address_of($t12);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 5806);
      goto Abort;
    }
    assume is#$Address($t13);


    // $t14 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t14 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 5851);
      goto Abort;
    }
    assume is#$Address($t14);


    // $t15 := ==($t13, $t14)
    $tmp := $Boolean($IsEqual($t13, $t14));
    $t15 := $tmp;

    // $t5 := $t15
    call $tmp := $CopyOrMoveValue($t15);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 5791, 5, $tmp); }

    // if ($t5) goto L2 else goto L3
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t17 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t17 := $tmp;

    // destroy($t17)

    // $t18 := 2
    $tmp := $Integer(2);
    $t18 := $tmp;

    // abort($t18)
    if (true) { assume $DebugTrackAbort(10, 5791); }
    goto Abort;

    // L2:
L2:

    // $t19 := copy($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t19 := $tmp;

    // $t20 := false
    $tmp := $Boolean(false);
    $t20 := $tmp;

    // $t21 := pack LibraConfig::ModifyConfigCapability<#0>($t20)
    call $tmp := $LibraConfig_ModifyConfigCapability_pack(0, 0, 0, $tv0, $t20);
    $t21 := $tmp;

    // Offer::create<LibraConfig::ModifyConfigCapability<#0>>($t19, $t21, $t28)
    call $Offer_create($LibraConfig_ModifyConfigCapability_type_value($tv0), $t19, $t21, $t28);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 5917);
      goto Abort;
    }

    // $t23 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t23 := $tmp;

    // $t25 := pack LibraConfig::LibraConfig<#0>($t27)
    call $tmp := $LibraConfig_LibraConfig_pack(0, 0, 0, $tv0, $t27);
    $t25 := $tmp;

    // move_to<LibraConfig::LibraConfig<#0>>($t25, $t23)
    call $MoveTo($LibraConfig_LibraConfig_type_value($tv0), $t25, $t23);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 5993);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $LibraConfig_publish_new_config_with_delegate_$direct_inter($tv0: $TypeValue, config_account: $Value, payload: $Value, delegate: $Value) returns ()
{
    assume is#$Address(config_account);

    assume is#$Address(delegate);

    call $LibraConfig_publish_new_config_with_delegate_$def($tv0, config_account, payload, delegate);
}


procedure {:inline 1} $LibraConfig_publish_new_config_with_delegate_$direct_intra($tv0: $TypeValue, config_account: $Value, payload: $Value, delegate: $Value) returns ()
{
    assume is#$Address(config_account);

    assume is#$Address(delegate);

    call $LibraConfig_publish_new_config_with_delegate_$def($tv0, config_account, payload, delegate);
}


procedure {:inline 1} $LibraConfig_publish_new_config_with_delegate($tv0: $TypeValue, config_account: $Value, payload: $Value, delegate: $Value) returns ()
{
    assume is#$Address(config_account);

    assume is#$Address(delegate);

    call $LibraConfig_publish_new_config_with_delegate_$def($tv0, config_account, payload, delegate);
}


procedure {:inline 1} $LibraConfig_publish_new_treasury_compliance_config_$def($tv0: $TypeValue, config_account: $Value, tc_account: $Value, payload: $Value) returns ()
{
    // declare local variables
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $tv0
    var $t13: $Value; // $LibraConfig_LibraConfig_type_value($tv0)
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $LibraConfig_ModifyConfigCapability_type_value($tv0)
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $tv0
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 4219, 0, config_account); }
    if (true) { assume $DebugTrackLocal(10, 4219, 1, tc_account); }
    if (true) { assume $DebugTrackLocal(10, 4219, 2, payload); }

    // bytecode translation starts here
    // $t17 := move(config_account)
    call $tmp := $CopyOrMoveValue(config_account);
    $t17 := $tmp;

    // $t18 := move(tc_account)
    call $tmp := $CopyOrMoveValue(tc_account);
    $t18 := $tmp;

    // $t19 := move(payload)
    call $tmp := $CopyOrMoveValue(payload);
    $t19 := $tmp;

    // $t5 := copy($t17)
    call $tmp := $CopyOrMoveValue($t17);
    $t5 := $tmp;

    // $t6 := Roles::has_on_chain_config_privilege($t5)
    call $t6 := $Roles_has_on_chain_config_privilege($t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 4405);
      goto Abort;
    }
    assume is#$Boolean($t6);


    // $t3 := $t6
    call $tmp := $CopyOrMoveValue($t6);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 4391, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t8 := move($t18)
    call $tmp := $CopyOrMoveValue($t18);
    $t8 := $tmp;

    // destroy($t8)

    // $t9 := move($t17)
    call $tmp := $CopyOrMoveValue($t17);
    $t9 := $tmp;

    // destroy($t9)

    // $t10 := 5
    $tmp := $Integer(5);
    $t10 := $tmp;

    // abort($t10)
    if (true) { assume $DebugTrackAbort(10, 4391); }
    goto Abort;

    // L0:
L0:

    // $t11 := move($t17)
    call $tmp := $CopyOrMoveValue($t17);
    $t11 := $tmp;

    // $t13 := pack LibraConfig::LibraConfig<#0>($t19)
    call $tmp := $LibraConfig_LibraConfig_pack(0, 0, 0, $tv0, $t19);
    $t13 := $tmp;

    // move_to<LibraConfig::LibraConfig<#0>>($t13, $t11)
    call $MoveTo($LibraConfig_LibraConfig_type_value($tv0), $t13, $t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 4483);
      goto Abort;
    }

    // $t14 := move($t18)
    call $tmp := $CopyOrMoveValue($t18);
    $t14 := $tmp;

    // $t15 := false
    $tmp := $Boolean(false);
    $t15 := $tmp;

    // $t16 := pack LibraConfig::ModifyConfigCapability<#0>($t15)
    call $tmp := $LibraConfig_ModifyConfigCapability_pack(0, 0, 0, $tv0, $t15);
    $t16 := $tmp;

    // move_to<LibraConfig::ModifyConfigCapability<#0>>($t16, $t14)
    call $MoveTo($LibraConfig_ModifyConfigCapability_type_value($tv0), $t16, $t14);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 4541);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $LibraConfig_publish_new_treasury_compliance_config_$direct_inter($tv0: $TypeValue, config_account: $Value, tc_account: $Value, payload: $Value) returns ()
{
    assume is#$Address(config_account);

    assume is#$Address(tc_account);

    call $LibraConfig_publish_new_treasury_compliance_config_$def($tv0, config_account, tc_account, payload);
}


procedure {:inline 1} $LibraConfig_publish_new_treasury_compliance_config_$direct_intra($tv0: $TypeValue, config_account: $Value, tc_account: $Value, payload: $Value) returns ()
{
    assume is#$Address(config_account);

    assume is#$Address(tc_account);

    call $LibraConfig_publish_new_treasury_compliance_config_$def($tv0, config_account, tc_account, payload);
}


procedure {:inline 1} $LibraConfig_publish_new_treasury_compliance_config($tv0: $TypeValue, config_account: $Value, tc_account: $Value, payload: $Value) returns ()
{
    assume is#$Address(config_account);

    assume is#$Address(tc_account);

    call $LibraConfig_publish_new_treasury_compliance_config_$def($tv0, config_account, tc_account, payload);
}


procedure {:inline 1} $LibraConfig_reconfigure_$def(lr_account: $Value) returns ()
{
    // declare local variables
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 6617, 0, lr_account); }

    // bytecode translation starts here
    // $t7 := move(lr_account)
    call $tmp := $CopyOrMoveValue(lr_account);
    $t7 := $tmp;

    // $t3 := move($t7)
    call $tmp := $CopyOrMoveValue($t7);
    $t3 := $tmp;

    // $t4 := Roles::has_libra_root_role($t3)
    call $t4 := $Roles_has_libra_root_role($t3);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 6795);
      goto Abort;
    }
    assume is#$Boolean($t4);


    // $t1 := $t4
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 6781, 1, $tmp); }

    // if ($t1) goto L0 else goto L1
    $tmp := $t1;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t6 := 1
    $tmp := $Integer(1);
    $t6 := $tmp;

    // abort($t6)
    if (true) { assume $DebugTrackAbort(10, 6781); }
    goto Abort;

    // L0:
L0:

    // LibraConfig::reconfigure_()
    call $LibraConfig_reconfigure_();
    assume $abort_flag == false;

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $LibraConfig_reconfigure_$direct_inter(lr_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    call $LibraConfig_reconfigure_$def(lr_account);
}


procedure {:inline 1} $LibraConfig_reconfigure_$direct_intra(lr_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    call $LibraConfig_reconfigure_$def(lr_account);
}


procedure {:inline 1} $LibraConfig_reconfigure(lr_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    call $LibraConfig_reconfigure_$def(lr_account);
}


procedure {:inline 1} $LibraConfig_reconfigure__$def() returns ()
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
    var $t14: $Reference; // ReferenceType($LibraConfig_Configuration_type_value())
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Reference; // ReferenceType($LibraConfig_Configuration_type_value())
    var $t18: $Reference; // ReferenceType($IntegerType())
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t4 := LibraTimestamp::is_not_initialized()
    call $t4 := $LibraTimestamp_is_not_initialized();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 7055);
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

    // $t5 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t5 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 7183);
      goto Abort;
    }
    assume is#$Address($t5);


    // $t6 := borrow_global<LibraConfig::Configuration>($t5)
    call $t6 := $BorrowGlobal($t5, $LibraConfig_Configuration_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 7135);
      goto Abort;
    }
    assume $LibraConfig_Configuration_is_well_formed($Dereference($t6));

    // UnpackRef($t6)

    // config_ref := $t6
    call config_ref := $CopyOrMoveRef($t6);
    if (true) { assume $DebugTrackLocal(10, 7122, 0, $Dereference(config_ref)); }

    // $t7 := LibraTimestamp::now_microseconds()
    call $t7 := $LibraTimestamp_now_microseconds();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 7457);
      goto Abort;
    }
    assume $IsValidU64($t7);


    // current_block_time := $t7
    call $tmp := $CopyOrMoveValue($t7);
    current_block_time := $tmp;
    if (true) { assume $DebugTrackLocal(10, 7420, 1, $tmp); }

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
    if (true) { assume $DebugTrackLocal(10, 7484, 2, $tmp); }

    // if ($t2) goto L3 else goto L4
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // $t14 := move(config_ref)
    call $t14 := $CopyOrMoveRef(config_ref);

    // destroy($t14)

    // LibraConfig::Configuration <- $t14
    call $WritebackToGlobal($t14);

    // PackRef($t14)

    // $t15 := 6
    $tmp := $Integer(6);
    $t15 := $tmp;

    // abort($t15)
    if (true) { assume $DebugTrackAbort(10, 7484); }
    goto Abort;

    // L3:
L3:

    // $t17 := move(config_ref)
    call $t17 := $CopyOrMoveRef(config_ref);

    // $t18 := borrow_field<LibraConfig::Configuration>.last_reconfiguration_time($t17)
    call $t18 := $BorrowField($t17, $LibraConfig_Configuration_last_reconfiguration_time);
    assume $IsValidU64($Dereference($t18));

    // LibraConfig::Configuration <- $t17
    call $WritebackToGlobal($t17);

    // UnpackRef($t18)

    // write_ref($t18, current_block_time)
    call $t18 := $WriteRef($t18, current_block_time);
    if (true) { assume $DebugTrackLocal(10, 7579, 0, $Dereference(config_ref)); }

    // LibraConfig::Configuration <- $t18
    call $WritebackToGlobal($t18);

    // Reference($t17) <- $t18
    call $t17 := $WritebackToReference($t18, $t17);

    // PackRef($t17)

    // PackRef($t18)

    // LibraConfig::emit_reconfiguration_event()
    call $LibraConfig_emit_reconfiguration_event();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 8589);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $LibraConfig_reconfigure__$direct_intra() returns ()
{
    call $LibraConfig_reconfigure__$def();
}


procedure {:inline 1} $LibraConfig_reconfigure_() returns ()
{
    call $LibraConfig_reconfigure__$def();
}


procedure {:inline 1} $LibraConfig_set_$def($tv0: $TypeValue, account: $Value, payload: $Value) returns ()
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
    var $t14: $Value; // $IntegerType()
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $BooleanType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $AddressType()
    var $t22: $Reference; // ReferenceType($LibraConfig_LibraConfig_type_value($tv0))
    var $t23: $Value; // $tv0
    var $t24: $Reference; // ReferenceType($LibraConfig_LibraConfig_type_value($tv0))
    var $t25: $Reference; // ReferenceType($tv0)
    var $t26: $Value; // $AddressType()
    var $t27: $Value; // $tv0
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 2192, 0, account); }
    if (true) { assume $DebugTrackLocal(10, 2192, 1, payload); }

    // bytecode translation starts here
    // $t26 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t26 := $tmp;

    // $t27 := move(payload)
    call $tmp := $CopyOrMoveValue(payload);
    $t27 := $tmp;

    // $t9 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t9 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 2336);
      goto Abort;
    }
    assume is#$Address($t9);


    // addr := $t9
    call $tmp := $CopyOrMoveValue($t9);
    addr := $tmp;
    if (true) { assume $DebugTrackLocal(10, 2314, 2, $tmp); }

    // $t11 := exists<LibraConfig::LibraConfig<#0>>(addr)
    call $tmp := $Exists(addr, $LibraConfig_LibraConfig_type_value($tv0));
    $t11 := $tmp;

    // $t5 := $t11
    call $tmp := $CopyOrMoveValue($t11);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 2366, 5, $tmp); }

    // if ($t5) goto L0 else goto L1
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t13 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t13 := $tmp;

    // destroy($t13)

    // $t14 := 3
    $tmp := $Integer(3);
    $t14 := $tmp;

    // abort($t14)
    if (true) { assume $DebugTrackAbort(10, 2366); }
    goto Abort;

    // L0:
L0:

    // $t15 := move($t26)
    call $tmp := $CopyOrMoveValue($t26);
    $t15 := $tmp;

    // $t16 := Signer::address_of($t15)
    call $t16 := $Signer_address_of($t15);
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 2470);
      goto Abort;
    }
    assume is#$Address($t16);


    // signer_address := $t16
    call $tmp := $CopyOrMoveValue($t16);
    signer_address := $tmp;
    if (true) { assume $DebugTrackLocal(10, 2445, 4, $tmp); }

    // $t18 := exists<LibraConfig::ModifyConfigCapability<#0>>(signer_address)
    call $tmp := $Exists(signer_address, $LibraConfig_ModifyConfigCapability_type_value($tv0));
    $t18 := $tmp;

    // $t7 := $t18
    call $tmp := $CopyOrMoveValue($t18);
    $t7 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 2499, 7, $tmp); }

    // if ($t7) goto L2 else goto L3
    $tmp := $t7;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t20 := 4
    $tmp := $Integer(4);
    $t20 := $tmp;

    // abort($t20)
    if (true) { assume $DebugTrackAbort(10, 2499); }
    goto Abort;

    // L2:
L2:

    // $t22 := borrow_global<LibraConfig::LibraConfig<#0>>(addr)
    call $t22 := $BorrowGlobal(addr, $LibraConfig_LibraConfig_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 2614);
      goto Abort;
    }
    assume $LibraConfig_LibraConfig_is_well_formed($Dereference($t22));

    // UnpackRef($t22)

    // config := $t22
    call config := $CopyOrMoveRef($t22);
    if (true) { assume $DebugTrackLocal(10, 2605, 3, $Dereference(config)); }

    // $t24 := move(config)
    call $t24 := $CopyOrMoveRef(config);

    // $t25 := borrow_field<LibraConfig::LibraConfig<#0>>.payload($t24)
    call $t25 := $BorrowField($t24, $LibraConfig_LibraConfig_payload);

    // LibraConfig::LibraConfig <- $t24
    call $WritebackToGlobal($t24);

    // UnpackRef($t25)

    // write_ref($t25, $t27)
    call $t25 := $WriteRef($t25, $t27);
    if (true) { assume $DebugTrackLocal(10, 2668, 3, $Dereference(config)); }

    // LibraConfig::LibraConfig <- $t25
    call $WritebackToGlobal($t25);

    // Reference($t24) <- $t25
    call $t24 := $WritebackToReference($t25, $t24);

    // PackRef($t24)

    // PackRef($t25)

    // LibraConfig::reconfigure_()
    call $LibraConfig_reconfigure_();
    assume $abort_flag == false;

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $LibraConfig_set_$direct_inter($tv0: $TypeValue, account: $Value, payload: $Value) returns ()
{
    assume is#$Address(account);

    call $LibraConfig_set_$def($tv0, account, payload);
}


procedure {:inline 1} $LibraConfig_set_$direct_intra($tv0: $TypeValue, account: $Value, payload: $Value) returns ()
{
    assume is#$Address(account);

    call $LibraConfig_set_$def($tv0, account, payload);
}


procedure {:inline 1} $LibraConfig_set($tv0: $TypeValue, account: $Value, payload: $Value) returns ()
{
    assume is#$Address(account);

    call $LibraConfig_set_$def($tv0, account, payload);
}


procedure {:inline 1} $LibraConfig_set_with_capability_$def($tv0: $TypeValue, _cap: $Value, payload: $Value) returns ()
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
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $AddressType()
    var $t12: $Reference; // ReferenceType($LibraConfig_LibraConfig_type_value($tv0))
    var $t13: $Value; // $tv0
    var $t14: $Reference; // ReferenceType($LibraConfig_LibraConfig_type_value($tv0))
    var $t15: $Reference; // ReferenceType($tv0)
    var $t16: $Value; // $LibraConfig_ModifyConfigCapability_type_value($tv0)
    var $t17: $Value; // $tv0
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(10, 2801, 0, _cap); }
    if (true) { assume $DebugTrackLocal(10, 2801, 1, payload); }

    // bytecode translation starts here
    // $t16 := move(_cap)
    call $tmp := $CopyOrMoveValue(_cap);
    $t16 := $tmp;

    // $t17 := move(payload)
    call $tmp := $CopyOrMoveValue(payload);
    $t17 := $tmp;

    // $t6 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t6 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 3000);
      goto Abort;
    }
    assume is#$Address($t6);


    // addr := $t6
    call $tmp := $CopyOrMoveValue($t6);
    addr := $tmp;
    if (true) { assume $DebugTrackLocal(10, 2978, 2, $tmp); }

    // $t8 := exists<LibraConfig::LibraConfig<#0>>(addr)
    call $tmp := $Exists(addr, $LibraConfig_LibraConfig_type_value($tv0));
    $t8 := $tmp;

    // $t4 := $t8
    call $tmp := $CopyOrMoveValue($t8);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(10, 3030, 4, $tmp); }

    // if ($t4) goto L0 else goto L1
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t10 := 3
    $tmp := $Integer(3);
    $t10 := $tmp;

    // abort($t10)
    if (true) { assume $DebugTrackAbort(10, 3030); }
    goto Abort;

    // L0:
L0:

    // $t12 := borrow_global<LibraConfig::LibraConfig<#0>>(addr)
    call $t12 := $BorrowGlobal(addr, $LibraConfig_LibraConfig_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(10, 3118);
      goto Abort;
    }
    assume $LibraConfig_LibraConfig_is_well_formed($Dereference($t12));

    // UnpackRef($t12)

    // config := $t12
    call config := $CopyOrMoveRef($t12);
    if (true) { assume $DebugTrackLocal(10, 3109, 3, $Dereference(config)); }

    // $t14 := move(config)
    call $t14 := $CopyOrMoveRef(config);

    // $t15 := borrow_field<LibraConfig::LibraConfig<#0>>.payload($t14)
    call $t15 := $BorrowField($t14, $LibraConfig_LibraConfig_payload);

    // LibraConfig::LibraConfig <- $t14
    call $WritebackToGlobal($t14);

    // UnpackRef($t15)

    // write_ref($t15, $t17)
    call $t15 := $WriteRef($t15, $t17);
    if (true) { assume $DebugTrackLocal(10, 3172, 3, $Dereference(config)); }

    // LibraConfig::LibraConfig <- $t15
    call $WritebackToGlobal($t15);

    // Reference($t14) <- $t15
    call $t14 := $WritebackToReference($t15, $t14);

    // PackRef($t14)

    // PackRef($t15)

    // LibraConfig::reconfigure_()
    call $LibraConfig_reconfigure_();
    assume $abort_flag == false;

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $LibraConfig_set_with_capability_$direct_inter($tv0: $TypeValue, _cap: $Value, payload: $Value) returns ()
{
    assume $LibraConfig_ModifyConfigCapability_is_well_formed(_cap);

    call $LibraConfig_set_with_capability_$def($tv0, _cap, payload);
}


procedure {:inline 1} $LibraConfig_set_with_capability_$direct_intra($tv0: $TypeValue, _cap: $Value, payload: $Value) returns ()
{
    assume $LibraConfig_ModifyConfigCapability_is_well_formed(_cap);

    call $LibraConfig_set_with_capability_$def($tv0, _cap, payload);
}


procedure {:inline 1} $LibraConfig_set_with_capability($tv0: $TypeValue, _cap: $Value, payload: $Value) returns ()
{
    assume $LibraConfig_ModifyConfigCapability_is_well_formed(_cap);

    call $LibraConfig_set_with_capability_$def($tv0, _cap, payload);
}




// ** spec vars of module RegisteredCurrencies



// ** spec funs of module RegisteredCurrencies

function {:inline} $RegisteredCurrencies_get_currency_codes($m: $Memory, $txn: $Transaction): $Value {
    $SelectField($LibraConfig_spec_get($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value()), $RegisteredCurrencies_RegisteredCurrencies_currency_codes)
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

procedure {:inline 1} $RegisteredCurrencies_initialize_$def(config_account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var cap: $Value; // $LibraConfig_ModifyConfigCapability_type_value($RegisteredCurrencies_RegisteredCurrencies_type_value())
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $Vector_type_value($Vector_type_value($IntegerType()))
    var $t19: $Value; // $RegisteredCurrencies_RegisteredCurrencies_type_value()
    var $t20: $Value; // $LibraConfig_ModifyConfigCapability_type_value($RegisteredCurrencies_RegisteredCurrencies_type_value())
    var $t21: $Value; // $LibraConfig_ModifyConfigCapability_type_value($RegisteredCurrencies_RegisteredCurrencies_type_value())
    var $t22: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t23: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 937, 0, config_account); }

    // bytecode translation starts here
    // $t23 := move(config_account)
    call $tmp := $CopyOrMoveValue(config_account);
    $t23 := $tmp;

    // $t6 := LibraTimestamp::is_genesis()
    call $t6 := $LibraTimestamp_is_genesis();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 1056);
      goto Abort;
    }
    assume is#$Boolean($t6);


    // $t2 := $t6
    call $tmp := $CopyOrMoveValue($t6);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 1033, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t8 := move($t23)
    call $tmp := $CopyOrMoveValue($t23);
    $t8 := $tmp;

    // destroy($t8)

    // $t9 := 0
    $tmp := $Integer(0);
    $t9 := $tmp;

    // abort($t9)
    if (true) { assume $DebugTrackAbort(13, 1033); }
    goto Abort;

    // L0:
L0:

    // $t10 := copy($t23)
    call $tmp := $CopyOrMoveValue($t23);
    $t10 := $tmp;

    // $t11 := Signer::address_of($t10)
    call $t11 := $Signer_address_of($t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 1122);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t12 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 1167);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := ==($t11, $t12)
    $tmp := $Boolean($IsEqual($t11, $t12));
    $t13 := $tmp;

    // $t4 := $t13
    call $tmp := $CopyOrMoveValue($t13);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 1094, 4, $tmp); }

    // if ($t4) goto L2 else goto L3
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t15 := move($t23)
    call $tmp := $CopyOrMoveValue($t23);
    $t15 := $tmp;

    // destroy($t15)

    // $t16 := 1
    $tmp := $Integer(1);
    $t16 := $tmp;

    // abort($t16)
    if (true) { assume $DebugTrackAbort(13, 1094); }
    goto Abort;

    // L2:
L2:

    // $t17 := move($t23)
    call $tmp := $CopyOrMoveValue($t23);
    $t17 := $tmp;

    // $t18 := Vector::empty<vector<u8>>()
    call $t18 := $Vector_empty($Vector_type_value($IntegerType()));
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 1393);
      goto Abort;
    }
    assume $Vector_is_well_formed($t18) && (forall $$0: int :: {$select_vector($t18,$$0)} $$0 >= 0 && $$0 < $vlen($t18) ==> $Vector_is_well_formed($select_vector($t18,$$0)) && (forall $$1: int :: {$select_vector($select_vector($t18,$$0),$$1)} $$1 >= 0 && $$1 < $vlen($select_vector($t18,$$0)) ==> $IsValidU8($select_vector($select_vector($t18,$$0),$$1))));


    // $t19 := pack RegisteredCurrencies::RegisteredCurrencies($t18)
    call $tmp := $RegisteredCurrencies_RegisteredCurrencies_pack(0, 0, 0, $t18);
    $t19 := $tmp;

    // $t20 := LibraConfig::publish_new_config_with_capability<RegisteredCurrencies::RegisteredCurrencies>($t17, $t19)
    call $t20 := $LibraConfig_publish_new_config_with_capability($RegisteredCurrencies_RegisteredCurrencies_type_value(), $t17, $t19);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 1270);
      goto Abort;
    }
    assume $LibraConfig_ModifyConfigCapability_is_well_formed($t20);


    // cap := $t20
    call $tmp := $CopyOrMoveValue($t20);
    cap := $tmp;
    if (true) { assume $DebugTrackLocal(13, 1251, 1, $tmp); }

    // $t22 := pack RegisteredCurrencies::RegistrationCapability(cap)
    call $tmp := $RegisteredCurrencies_RegistrationCapability_pack(0, 0, 0, cap);
    $t22 := $tmp;

    // return $t22
    $ret0 := $t22;
    if (true) { assume $DebugTrackLocal(13, 1423, 24, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $RegisteredCurrencies_initialize_$direct_inter(config_account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(config_account);

    call $ret0 := $RegisteredCurrencies_initialize_$def(config_account);
}


procedure {:inline 1} $RegisteredCurrencies_initialize_$direct_intra(config_account: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value()))));
{
    assume is#$Address(config_account);

    call $ret0 := $RegisteredCurrencies_initialize_$def(config_account);
}


procedure {:inline 1} $RegisteredCurrencies_initialize(config_account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(config_account);

    call $ret0 := $RegisteredCurrencies_initialize_$def(config_account);
}


procedure {:inline 1} $RegisteredCurrencies_add_currency_code_$def(currency_code: $Value, cap: $Value) returns ()
{
    // declare local variables
    var config: $Value; // $RegisteredCurrencies_RegisteredCurrencies_type_value()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $RegisteredCurrencies_RegisteredCurrencies_type_value()
    var $t6: $Value; // $RegisteredCurrencies_RegisteredCurrencies_type_value()
    var $t7: $Value; // $Vector_type_value($Vector_type_value($IntegerType()))
    var $t8: $Value; // $Vector_type_value($IntegerType())
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $BooleanType()
    var $t11: $Value; // $BooleanType()
    var $t12: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t13: $Value; // $IntegerType()
    var $t14: $Reference; // ReferenceType($RegisteredCurrencies_RegisteredCurrencies_type_value())
    var $t15: $Reference; // ReferenceType($Vector_type_value($Vector_type_value($IntegerType())))
    var $t16: $Value; // $Vector_type_value($IntegerType())
    var $t17: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t18: $Value; // $LibraConfig_ModifyConfigCapability_type_value($RegisteredCurrencies_RegisteredCurrencies_type_value())
    var $t19: $Value; // $RegisteredCurrencies_RegisteredCurrencies_type_value()
    var $t20: $Value; // $Vector_type_value($IntegerType())
    var $t21: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t22: $Value; // $Vector_type_value($Vector_type_value($IntegerType()))
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(13, 2225, 0, currency_code); }
    if (true) { assume $DebugTrackLocal(13, 2225, 1, cap); }

    // bytecode translation starts here
    // $t20 := move(currency_code)
    call $tmp := $CopyOrMoveValue(currency_code);
    $t20 := $tmp;

    // $t21 := move(cap)
    call $tmp := $CopyOrMoveValue(cap);
    $t21 := $tmp;

    // $t5 := LibraConfig::get<RegisteredCurrencies::RegisteredCurrencies>()
    call $t5 := $LibraConfig_get($RegisteredCurrencies_RegisteredCurrencies_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2370);
      goto Abort;
    }
    assume $RegisteredCurrencies_RegisteredCurrencies_is_well_formed($t5);


    // config := $t5
    call $tmp := $CopyOrMoveValue($t5);
    config := $tmp;
    if (true) { assume $DebugTrackLocal(13, 2348, 2, $tmp); }

    // $t6 := copy(config)
    call $tmp := $CopyOrMoveValue(config);
    $t6 := $tmp;

    // $t7 := get_field<RegisteredCurrencies::RegisteredCurrencies>.currency_codes($t6)
    call $tmp := $GetFieldFromValue($t6, $RegisteredCurrencies_RegisteredCurrencies_currency_codes);
    assume $Vector_is_well_formed($tmp) && (forall $$0: int :: {$select_vector($tmp,$$0)} $$0 >= 0 && $$0 < $vlen($tmp) ==> $Vector_is_well_formed($select_vector($tmp,$$0)) && (forall $$1: int :: {$select_vector($select_vector($tmp,$$0),$$1)} $$1 >= 0 && $$1 < $vlen($select_vector($tmp,$$0)) ==> $IsValidU8($select_vector($select_vector($tmp,$$0),$$1))));
    $t7 := $tmp;

    // $t8 := copy($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t8 := $tmp;

    // $t9 := Vector::contains<vector<u8>>($t7, $t8)
    call $t9 := $Vector_contains($Vector_type_value($IntegerType()), $t7, $t8);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2436);
      goto Abort;
    }
    assume is#$Boolean($t9);


    // $t10 := !($t9)
    call $tmp := $Not($t9);
    $t10 := $tmp;

    // $t3 := $t10
    call $tmp := $CopyOrMoveValue($t10);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(13, 2407, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t12 := move($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t12 := $tmp;

    // destroy($t12)

    // $t13 := 2
    $tmp := $Integer(2);
    $t13 := $tmp;

    // abort($t13)
    if (true) { assume $DebugTrackAbort(13, 2407); }
    goto Abort;

    // L0:
L0:

    // $t14 := borrow_local(config)
    call $t14 := $BorrowLoc(2, config);
    assume $RegisteredCurrencies_RegisteredCurrencies_is_well_formed($Dereference($t14));

    // UnpackRef($t14)

    // $t15 := borrow_field<RegisteredCurrencies::RegisteredCurrencies>.currency_codes($t14)
    call $t15 := $BorrowField($t14, $RegisteredCurrencies_RegisteredCurrencies_currency_codes);
    assume $Vector_is_well_formed($Dereference($t15)) && (forall $$1: int :: {$select_vector($Dereference($t15),$$1)} $$1 >= 0 && $$1 < $vlen($Dereference($t15)) ==> $Vector_is_well_formed($select_vector($Dereference($t15),$$1)) && (forall $$2: int :: {$select_vector($select_vector($Dereference($t15),$$1),$$2)} $$2 >= 0 && $$2 < $vlen($select_vector($Dereference($t15),$$1)) ==> $IsValidU8($select_vector($select_vector($Dereference($t15),$$1),$$2))));

    // LocalRoot(config) <- $t14
    call config := $WritebackToValue($t14, 2, config);

    // UnpackRef($t15)

    // PackRef($t15)

    // $t22 := read_ref($t15)
    call $tmp := $ReadRef($t15);
    assume $Vector_is_well_formed($tmp) && (forall $$0: int :: {$select_vector($tmp,$$0)} $$0 >= 0 && $$0 < $vlen($tmp) ==> $Vector_is_well_formed($select_vector($tmp,$$0)) && (forall $$1: int :: {$select_vector($select_vector($tmp,$$0),$$1)} $$1 >= 0 && $$1 < $vlen($select_vector($tmp,$$0)) ==> $IsValidU8($select_vector($select_vector($tmp,$$0),$$1))));
    $t22 := $tmp;

    // $t22 := Vector::push_back<vector<u8>>($t22, $t20)
    call $t22 := $Vector_push_back($Vector_type_value($IntegerType()), $t22, $t20);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2554);
      goto Abort;
    }
    assume $Vector_is_well_formed($t22) && (forall $$0: int :: {$select_vector($t22,$$0)} $$0 >= 0 && $$0 < $vlen($t22) ==> $Vector_is_well_formed($select_vector($t22,$$0)) && (forall $$1: int :: {$select_vector($select_vector($t22,$$0),$$1)} $$1 >= 0 && $$1 < $vlen($select_vector($t22,$$0)) ==> $IsValidU8($select_vector($select_vector($t22,$$0),$$1))));


    // write_ref($t15, $t22)
    call $t15 := $WriteRef($t15, $t22);

    // LocalRoot(config) <- $t15
    call config := $WritebackToValue($t15, 2, config);

    // Reference($t14) <- $t15
    call $t14 := $WritebackToReference($t15, $t14);

    // UnpackRef($t15)

    // PackRef($t14)

    // PackRef($t15)

    // $t17 := move($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t17 := $tmp;

    // $t18 := get_field<RegisteredCurrencies::RegistrationCapability>.cap($t17)
    call $tmp := $GetFieldFromValue($t17, $RegisteredCurrencies_RegistrationCapability_cap);
    assume $LibraConfig_ModifyConfigCapability_is_well_formed($tmp);
    $t18 := $tmp;

    // LibraConfig::set_with_capability<RegisteredCurrencies::RegisteredCurrencies>($t18, config)
    call $LibraConfig_set_with_capability($RegisteredCurrencies_RegisteredCurrencies_type_value(), $t18, config);
    if ($abort_flag) {
      assume $DebugTrackAbort(13, 2629);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $RegisteredCurrencies_add_currency_code_$direct_inter(currency_code: $Value, cap: $Value) returns ()
{
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));

    assume $RegisteredCurrencies_RegistrationCapability_is_well_formed(cap);

    call $RegisteredCurrencies_add_currency_code_$def(currency_code, cap);
}


procedure {:inline 1} $RegisteredCurrencies_add_currency_code_$direct_intra(currency_code: $Value, cap: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($LibraTimestamp_spec_is_genesis($m, $txn)))) ==> b#$Boolean($LibraConfig_spec_is_published($m, $txn, $RegisteredCurrencies_RegisteredCurrencies_type_value()))));
{
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));

    assume $RegisteredCurrencies_RegistrationCapability_is_well_formed(cap);

    call $RegisteredCurrencies_add_currency_code_$def(currency_code, cap);
}


procedure {:inline 1} $RegisteredCurrencies_add_currency_code(currency_code: $Value, cap: $Value) returns ()
{
    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));

    assume $RegisteredCurrencies_RegistrationCapability_is_well_formed(cap);

    call $RegisteredCurrencies_add_currency_code_$def(currency_code, cap);
}




// ** spec vars of module FixedPoint32



// ** spec funs of module FixedPoint32

function {:inline} $FixedPoint32_spec_multiply_u64(val: $Value, multiplier: $Value): $Value;
axiom (forall val: $Value, multiplier: $Value :: $IsValidU64($FixedPoint32_spec_multiply_u64(val, multiplier)));
function {:inline} $FixedPoint32_spec_divide_u64(val: $Value, divisor: $Value): $Value;
axiom (forall val: $Value, divisor: $Value :: $IsValidU64($FixedPoint32_spec_divide_u64(val, divisor)));
function {:inline} $FixedPoint32_spec_create_from_rational(numerator: $Value, denominator: $Value): $Value;
axiom (forall numerator: $Value, denominator: $Value :: $FixedPoint32_FixedPoint32_is_well_formed($FixedPoint32_spec_create_from_rational(numerator, denominator)));


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

procedure {:inline 1} $FixedPoint32_create_from_rational_$def(numerator: $Value, denominator: $Value) returns ($ret0: $Value)
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
    var $t30: $Value; // $IntegerType()
    var $t31: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t32: $Value; // $IntegerType()
    var $t33: $Value; // $IntegerType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(6, 3072, 0, numerator); }
    if (true) { assume $DebugTrackLocal(6, 3072, 1, denominator); }

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
      assume $DebugTrackAbort(6, 3364);
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
    if (true) { assume $DebugTrackLocal(6, 3345, 4, $tmp); }

    // $t13 := (u128)($t33)
    call $tmp := $CastU128($t33);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 3424);
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
    if (true) { assume $DebugTrackLocal(6, 3403, 3, $tmp); }

    // $t18 := /(scaled_numerator, scaled_denominator)
    call $tmp := $Div(scaled_numerator, scaled_denominator);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 3587);
      goto Abort;
    }
    $t18 := $tmp;

    // quotient := $t18
    call $tmp := $CopyOrMoveValue($t18);
    quotient := $tmp;
    if (true) { assume $DebugTrackLocal(6, 3559, 2, $tmp); }

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
    if (true) { assume $DebugTrackLocal(6, 3730, 7, $tmp); }

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
    if (true) { assume $DebugTrackLocal(6, 3730, 7, $tmp); }

    // goto L3
    goto L3;

    // L3:
L3:

    // $t5 := $t7
    call $tmp := $CopyOrMoveValue($t7);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(6, 3723, 5, $tmp); }

    // if ($t5) goto L4 else goto L5
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L4; } else { goto L5; }

    // L5:
L5:

    // $t28 := 0
    $tmp := $Integer(0);
    $t28 := $tmp;

    // abort($t28)
    if (true) { assume $DebugTrackAbort(6, 3723); }
    goto Abort;

    // L4:
L4:

    // $t30 := (u64)(quotient)
    call $tmp := $CastU64(quotient);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 3952);
      goto Abort;
    }
    $t30 := $tmp;

    // $t31 := pack FixedPoint32::FixedPoint32($t30)
    call $tmp := $FixedPoint32_FixedPoint32_pack(0, 0, 0, $t30);
    $t31 := $tmp;

    // return $t31
    $ret0 := $t31;
    if (true) { assume $DebugTrackLocal(6, 3930, 34, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $FixedPoint32_create_from_rational_$direct_inter(numerator: $Value, denominator: $Value) returns ($ret0: $Value)
;ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $FixedPoint32_spec_create_from_rational(numerator, denominator)))));

procedure {:inline 1} $FixedPoint32_create_from_rational_$direct_intra(numerator: $Value, denominator: $Value) returns ($ret0: $Value)
;ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $FixedPoint32_spec_create_from_rational(numerator, denominator)))));

procedure {:inline 1} $FixedPoint32_create_from_rational(numerator: $Value, denominator: $Value) returns ($ret0: $Value)
;ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $FixedPoint32_spec_create_from_rational(numerator, denominator)))));

procedure {:inline 1} $FixedPoint32_create_from_raw_value_$def(value: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t3: $Value; // $IntegerType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(6, 4211, 0, value); }

    // bytecode translation starts here
    // $t3 := move(value)
    call $tmp := $CopyOrMoveValue(value);
    $t3 := $tmp;

    // $t2 := pack FixedPoint32::FixedPoint32($t3)
    call $tmp := $FixedPoint32_FixedPoint32_pack(0, 0, 0, $t3);
    $t2 := $tmp;

    // return $t2
    $ret0 := $t2;
    if (true) { assume $DebugTrackLocal(6, 4280, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $FixedPoint32_create_from_raw_value_$direct_inter(value: $Value) returns ($ret0: $Value)
{
    assume $IsValidU64(value);

    call $ret0 := $FixedPoint32_create_from_raw_value_$def(value);
}


procedure {:inline 1} $FixedPoint32_create_from_raw_value_$direct_intra(value: $Value) returns ($ret0: $Value)
{
    assume $IsValidU64(value);

    call $ret0 := $FixedPoint32_create_from_raw_value_$def(value);
}


procedure {:inline 1} $FixedPoint32_create_from_raw_value(value: $Value) returns ($ret0: $Value)
{
    assume $IsValidU64(value);

    call $ret0 := $FixedPoint32_create_from_raw_value_$def(value);
}


procedure {:inline 1} $FixedPoint32_divide_u64_$def(num: $Value, divisor: $Value) returns ($ret0: $Value)
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(6, 2025, 0, num); }
    if (true) { assume $DebugTrackLocal(6, 2025, 1, divisor); }

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
      assume $DebugTrackAbort(6, 2230);
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
    if (true) { assume $DebugTrackLocal(6, 2215, 3, $tmp); }

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
      assume $DebugTrackAbort(6, 2344);
      goto Abort;
    }
    $t12 := $tmp;

    // $t13 := /(scaled_value, $t12)
    call $tmp := $Div(scaled_value, $t12);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 2342);
      goto Abort;
    }
    $t13 := $tmp;

    // quotient := $t13
    call $tmp := $CopyOrMoveValue($t13);
    quotient := $tmp;
    if (true) { assume $DebugTrackLocal(6, 2318, 2, $tmp); }

    // $t15 := (u64)(quotient)
    call $tmp := $CastU64(quotient);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 2487);
      goto Abort;
    }
    $t15 := $tmp;

    // return $t15
    $ret0 := $t15;
    if (true) { assume $DebugTrackLocal(6, 2487, 18, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $FixedPoint32_divide_u64_$direct_inter(num: $Value, divisor: $Value) returns ($ret0: $Value)
;ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $FixedPoint32_spec_divide_u64(num, divisor)))));

procedure {:inline 1} $FixedPoint32_divide_u64_$direct_intra(num: $Value, divisor: $Value) returns ($ret0: $Value)
;ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $FixedPoint32_spec_divide_u64(num, divisor)))));

procedure {:inline 1} $FixedPoint32_divide_u64(num: $Value, divisor: $Value) returns ($ret0: $Value)
;ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $FixedPoint32_spec_divide_u64(num, divisor)))));

procedure {:inline 1} $FixedPoint32_get_raw_value_$def(num: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(6, 4494, 0, num); }

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
    if (true) { assume $DebugTrackLocal(6, 4553, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $FixedPoint32_get_raw_value_$direct_inter(num: $Value) returns ($ret0: $Value)
{
    assume $FixedPoint32_FixedPoint32_is_well_formed(num);

    call $ret0 := $FixedPoint32_get_raw_value_$def(num);
}


procedure {:inline 1} $FixedPoint32_get_raw_value_$direct_intra(num: $Value) returns ($ret0: $Value)
{
    assume $FixedPoint32_FixedPoint32_is_well_formed(num);

    call $ret0 := $FixedPoint32_get_raw_value_$def(num);
}


procedure {:inline 1} $FixedPoint32_get_raw_value(num: $Value) returns ($ret0: $Value)
{
    assume $FixedPoint32_FixedPoint32_is_well_formed(num);

    call $ret0 := $FixedPoint32_get_raw_value_$def(num);
}


procedure {:inline 1} $FixedPoint32_multiply_u64_$def(num: $Value, multiplier: $Value) returns ($ret0: $Value)
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(6, 445, 0, num); }
    if (true) { assume $DebugTrackLocal(6, 445, 1, multiplier); }

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
      assume $DebugTrackAbort(6, 729);
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
      assume $DebugTrackAbort(6, 745);
      goto Abort;
    }
    $t9 := $tmp;

    // $t10 := *($t5, $t9)
    call $tmp := $MulU128($t5, $t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 743);
      goto Abort;
    }
    $t10 := $tmp;

    // unscaled_product := $t10
    call $tmp := $CopyOrMoveValue($t10);
    unscaled_product := $tmp;
    if (true) { assume $DebugTrackLocal(6, 710, 3, $tmp); }

    // $t12 := 32
    $tmp := $Integer(32);
    $t12 := $tmp;

    // $t13 := <<(unscaled_product, $t12)
    call $tmp := $Shr(unscaled_product, $t12);
    $t13 := $tmp;

    // product := $t13
    call $tmp := $CopyOrMoveValue($t13);
    product := $tmp;
    if (true) { assume $DebugTrackLocal(6, 918, 2, $tmp); }

    // $t15 := (u64)(product)
    call $tmp := $CastU64(product);
    if ($abort_flag) {
      assume $DebugTrackAbort(6, 1070);
      goto Abort;
    }
    $t15 := $tmp;

    // return $t15
    $ret0 := $t15;
    if (true) { assume $DebugTrackLocal(6, 1070, 18, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $FixedPoint32_multiply_u64_$direct_inter(num: $Value, multiplier: $Value) returns ($ret0: $Value)
;ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $FixedPoint32_spec_multiply_u64(num, multiplier)))));

procedure {:inline 1} $FixedPoint32_multiply_u64_$direct_intra(num: $Value, multiplier: $Value) returns ($ret0: $Value)
;ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $FixedPoint32_spec_multiply_u64(num, multiplier)))));

procedure {:inline 1} $FixedPoint32_multiply_u64(num: $Value, multiplier: $Value) returns ($ret0: $Value)
;ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $FixedPoint32_spec_multiply_u64(num, multiplier)))));



// ** spec vars of module Libra

var $Libra_sum_of_coin_values : [$TypeValue]$Value where (forall $tv0: $TypeValue :: $IsValidNum($Libra_sum_of_coin_values[$tv0]));


// ** spec funs of module Libra

function {:inline} $Libra_spec_is_currency($m: $Memory, $txn: $Transaction, $tv0: $TypeValue): $Value {
    $ResourceExists($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS())
}

function {:inline} $Libra_spec_currency_info($m: $Memory, $txn: $Transaction, $tv0: $TypeValue): $Value {
    $ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS())
}

function {:inline} $Libra_spec_approx_lbr_for_value($m: $Memory, $txn: $Transaction, $tv0: $TypeValue, value: $Value): $Value {
    $FixedPoint32_spec_multiply_u64(value, $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_to_lbr_exchange_rate))
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
const $Libra_Preburn_to_burn: $FieldName;
axiom $Libra_Preburn_to_burn == 0;
function $Libra_Preburn_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Libra_Preburn, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $Libra_Libra_type_value($tv0)], 1))
}
function {:inline} $Libra_Preburn_is_well_formed_types($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $Libra_Libra_is_well_formed_types($SelectField($this, $Libra_Preburn_to_burn))
}
function {:inline} $Libra_Preburn_is_well_formed($this: $Value): bool {
    $Vector_is_well_formed($this)
    && $vlen($this) == 1
      && $Libra_Libra_is_well_formed($SelectField($this, $Libra_Preburn_to_burn))
}

axiom (forall m: $Memory, a: $Value, $tv0: $TypeValue :: $Memory__is_well_formed(m) && is#$Address(a) ==>
    $Libra_Preburn_is_well_formed($ResourceValue(m, $Libra_Preburn_type_value($tv0), a))
);

procedure {:inline 1} $Libra_Preburn_before_update_inv($tv0: $TypeValue, $before: $Value) {
    call $Libra_Libra_before_update_inv($tv0, $SelectField($before, $Libra_Preburn_to_burn));
}

procedure {:inline 1} $Libra_Preburn_after_update_inv($tv0: $TypeValue, $after: $Value) {
    call $Libra_Libra_after_update_inv($tv0, $SelectField($after, $Libra_Preburn_to_burn));
}

procedure {:inline 1} $Libra_Preburn_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, to_burn: $Value) returns ($struct: $Value)
{
    assume $Libra_Libra_is_well_formed(to_burn);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := to_burn], 1));
    if ($byte_index > 0) { assume $DebugTrackLocal($file_id, $byte_index, $var_idx, $struct); }
}

procedure {:inline 1} $Libra_Preburn_unpack($tv0: $TypeValue, $struct: $Value) returns (to_burn: $Value)
{
    assume is#$Vector($struct);
    to_burn := $SelectField($struct, $Libra_Preburn_to_burn);
    assume $Libra_Libra_is_well_formed(to_burn);
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

procedure {:inline 1} $Libra_initialize_$def(config_account: $Value) returns ()
{
    // declare local variables
    var cap: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t21: $Value; // $Libra_CurrencyRegistrationCapability_type_value()
    var $t22: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 10444, 0, config_account); }

    // bytecode translation starts here
    // $t22 := move(config_account)
    call $tmp := $CopyOrMoveValue(config_account);
    $t22 := $tmp;

    // $t6 := LibraTimestamp::is_genesis()
    call $t6 := $LibraTimestamp_is_genesis();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 10539);
      goto Abort;
    }
    assume is#$Boolean($t6);


    // $t2 := $t6
    call $tmp := $CopyOrMoveValue($t6);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 10516, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t8 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t8 := $tmp;

    // destroy($t8)

    // $t9 := 0
    $tmp := $Integer(0);
    $t9 := $tmp;

    // abort($t9)
    if (true) { assume $DebugTrackAbort(9, 10516); }
    goto Abort;

    // L0:
L0:

    // $t10 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t10 := $tmp;

    // $t11 := Signer::address_of($t10)
    call $t11 := $Signer_address_of($t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 10638);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t12 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 10683);
      goto Abort;
    }
    assume is#$Address($t12);


    // $t13 := ==($t11, $t12)
    $tmp := $Boolean($IsEqual($t11, $t12));
    $t13 := $tmp;

    // $t4 := $t13
    call $tmp := $CopyOrMoveValue($t13);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 10610, 4, $tmp); }

    // if ($t4) goto L2 else goto L3
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t15 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t15 := $tmp;

    // destroy($t15)

    // $t16 := 1
    $tmp := $Integer(1);
    $t16 := $tmp;

    // abort($t16)
    if (true) { assume $DebugTrackAbort(9, 10610); }
    goto Abort;

    // L2:
L2:

    // $t17 := copy($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t17 := $tmp;

    // $t18 := RegisteredCurrencies::initialize($t17)
    call $t18 := $RegisteredCurrencies_initialize($t17);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 10795);
      goto Abort;
    }
    assume $RegisteredCurrencies_RegistrationCapability_is_well_formed($t18);


    // cap := $t18
    call $tmp := $CopyOrMoveValue($t18);
    cap := $tmp;
    if (true) { assume $DebugTrackLocal(9, 10767, 1, $tmp); }

    // $t19 := move($t22)
    call $tmp := $CopyOrMoveValue($t22);
    $t19 := $tmp;

    // $t21 := pack Libra::CurrencyRegistrationCapability(cap)
    call $tmp := $Libra_CurrencyRegistrationCapability_pack(0, 0, 0, cap);
    $t21 := $tmp;

    // move_to<Libra::CurrencyRegistrationCapability>($t21, $t19)
    call $MoveTo($Libra_CurrencyRegistrationCapability_type_value(), $t21, $t19);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 10831);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Libra_initialize_$direct_inter(config_account: $Value) returns ()
{
    assume is#$Address(config_account);

    call $Libra_initialize_$def(config_account);
}


procedure {:inline 1} $Libra_initialize_$direct_intra(config_account: $Value) returns ()
{
    assume is#$Address(config_account);

    call $Libra_initialize_$def(config_account);
}


procedure {:inline 1} $Libra_initialize(config_account: $Value) returns ()
{
    assume is#$Address(config_account);

    call $Libra_initialize_$def(config_account);
}


procedure {:inline 1} $Libra_currency_code_$def($tv0: $TypeValue) returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t2: $Value; // $Vector_type_value($IntegerType())
    var $t3: $Value; // $Vector_type_value($IntegerType())
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 35861);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := get_global<Libra::CurrencyInfo<#0>>($t0)
    call $tmp := $GetGlobal($t0, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 35808);
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
    if (true) { assume $DebugTrackLocal(9, 35806, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_currency_code_$direct_inter($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_currency_code_$def($tv0);
}


procedure {:inline 1} $Libra_currency_code_$direct_intra($tv0: $TypeValue) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    call $ret0 := $Libra_currency_code_$def($tv0);
}


procedure {:inline 1} $Libra_currency_code($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_currency_code_$def($tv0);
}


procedure {:inline 1} $Libra_value_$def($tv0: $TypeValue, coin: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $Libra_Libra_type_value($tv0)
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 26950, 0, coin); }

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
    if (true) { assume $DebugTrackLocal(9, 27016, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_value_$direct_inter($tv0: $TypeValue, coin: $Value) returns ($ret0: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    call $ret0 := $Libra_value_$def($tv0, coin);
}


procedure {:inline 1} $Libra_value_$direct_intra($tv0: $TypeValue, coin: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume $Libra_Libra_is_well_formed(coin);

    call $ret0 := $Libra_value_$def($tv0, coin);
}


procedure {:inline 1} $Libra_value($tv0: $TypeValue, coin: $Value) returns ($ret0: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    call $ret0 := $Libra_value_$def($tv0, coin);
}


procedure {:inline 1} $Libra_approx_lbr_for_coin_$def($tv0: $TypeValue, coin: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var from_value: $Value; // $IntegerType()
    var $t2: $Value; // $Libra_Libra_type_value($tv0)
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 34151, 0, coin); }

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
      assume $DebugTrackAbort(9, 26961);
      goto Abort;
    }
    assume $IsValidU64($t3);


    // from_value := $t3
    call $tmp := $CopyOrMoveValue($t3);
    from_value := $tmp;
    if (true) { assume $DebugTrackLocal(9, 34269, 1, $tmp); }

    // $t5 := Libra::approx_lbr_for_value<#0>(from_value)
    call $t5 := $Libra_approx_lbr_for_value($tv0, from_value);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 33744);
      goto Abort;
    }
    assume $IsValidU64($t5);


    // return $t5
    $ret0 := $t5;
    if (true) { assume $DebugTrackLocal(9, 34303, 7, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_approx_lbr_for_coin_$direct_inter($tv0: $TypeValue, coin: $Value) returns ($ret0: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    call $ret0 := $Libra_approx_lbr_for_coin_$def($tv0, coin);
}


procedure {:inline 1} $Libra_approx_lbr_for_coin_$direct_intra($tv0: $TypeValue, coin: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume $Libra_Libra_is_well_formed(coin);

    call $ret0 := $Libra_approx_lbr_for_coin_$def($tv0, coin);
}


procedure {:inline 1} $Libra_approx_lbr_for_coin($tv0: $TypeValue, coin: $Value) returns ($ret0: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    call $ret0 := $Libra_approx_lbr_for_coin_$def($tv0, coin);
}


procedure {:inline 1} $Libra_approx_lbr_for_value_$def($tv0: $TypeValue, from_value: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var lbr_exchange_rate: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t2: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 33733, 0, from_value); }

    // bytecode translation starts here
    // $t6 := move(from_value)
    call $tmp := $CopyOrMoveValue(from_value);
    $t6 := $tmp;

    // $t2 := Libra::lbr_exchange_rate<#0>()
    call $t2 := $Libra_lbr_exchange_rate($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 36934);
      goto Abort;
    }
    assume $FixedPoint32_FixedPoint32_is_well_formed($t2);


    // lbr_exchange_rate := $t2
    call $tmp := $CopyOrMoveValue($t2);
    lbr_exchange_rate := $tmp;
    if (true) { assume $DebugTrackLocal(9, 33841, 1, $tmp); }

    // $t5 := FixedPoint32::multiply_u64($t6, lbr_exchange_rate)
    call $t5 := $FixedPoint32_multiply_u64($t6, lbr_exchange_rate);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 33918);
      goto Abort;
    }
    assume $IsValidU64($t5);


    // return $t5
    $ret0 := $t5;
    if (true) { assume $DebugTrackLocal(9, 33904, 7, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_approx_lbr_for_value_$direct_inter($tv0: $TypeValue, from_value: $Value) returns ($ret0: $Value)
{
    assume $IsValidU64(from_value);

    call $ret0 := $Libra_approx_lbr_for_value_$def($tv0, from_value);
}


procedure {:inline 1} $Libra_approx_lbr_for_value_$direct_intra($tv0: $TypeValue, from_value: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume $IsValidU64(from_value);

    call $ret0 := $Libra_approx_lbr_for_value_$def($tv0, from_value);
}


procedure {:inline 1} $Libra_approx_lbr_for_value($tv0: $TypeValue, from_value: $Value) returns ($ret0: $Value)
{
    assume $IsValidU64(from_value);

    call $ret0 := $Libra_approx_lbr_for_value_$def($tv0, from_value);
}


procedure {:inline 1} $Libra_assert_is_currency_$def($tv0: $TypeValue) returns ()
{
    // declare local variables
    var $t0: $Value; // $BooleanType()
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t2 := Libra::is_currency<#0>()
    call $t2 := $Libra_is_currency($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 34479);
      goto Abort;
    }
    assume is#$Boolean($t2);


    // $t0 := $t2
    call $tmp := $CopyOrMoveValue($t2);
    $t0 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 38370, 0, $tmp); }

    // if ($t0) goto L0 else goto L1
    $tmp := $t0;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t4 := 8
    $tmp := $Integer(8);
    $t4 := $tmp;

    // abort($t4)
    if (true) { assume $DebugTrackAbort(9, 38370); }
    goto Abort;

    // L0:
L0:

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Libra_assert_is_currency_$direct_intra($tv0: $TypeValue) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    call $Libra_assert_is_currency_$def($tv0);
}


procedure {:inline 1} $Libra_assert_is_currency($tv0: $TypeValue) returns ()
{
    call $Libra_assert_is_currency_$def($tv0);
}


procedure {:inline 1} $Libra_burn_$def($tv0: $TypeValue, account: $Value, preburn_address: $Value) returns ()
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 12894, 0, account); }
    if (true) { assume $DebugTrackLocal(9, 12894, 1, preburn_address); }

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
      assume $DebugTrackAbort(9, 13154);
      goto Abort;
    }
    assume is#$Address($t4);


    // $t5 := get_global<Libra::BurnCapability<#0>>($t4)
    call $tmp := $GetGlobal($t4, $Libra_BurnCapability_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 13106);
      goto Abort;
    }
    assume $Libra_BurnCapability_is_well_formed($tmp);
    $t5 := $tmp;

    // Libra::burn_with_capability<#0>($t7, $t5)
    call $Libra_burn_with_capability($tv0, $t7, $t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 20412);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Libra_burn_$direct_inter($tv0: $TypeValue, account: $Value, preburn_address: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(preburn_address);

    call $Libra_burn_$def($tv0, account, preburn_address);
}


procedure {:inline 1} $Libra_burn_$direct_intra($tv0: $TypeValue, account: $Value, preburn_address: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(account);

    assume is#$Address(preburn_address);

    call $Libra_burn_$def($tv0, account, preburn_address);
}


procedure {:inline 1} $Libra_burn($tv0: $TypeValue, account: $Value, preburn_address: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(preburn_address);

    call $Libra_burn_$def($tv0, account, preburn_address);
}


procedure {:inline 1} $Libra_burn_with_capability_$def($tv0: $TypeValue, preburn_address: $Value, capability: $Value) returns ()
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 20401, 0, preburn_address); }
    if (true) { assume $DebugTrackLocal(9, 20401, 1, capability); }

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
      assume $DebugTrackAbort(9, 20663);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($Dereference($t3));

    // UnpackRef($t3)
    call $Libra_Preburn_before_update_inv($tv0, $Dereference($t3));

    // $t5 := move($t7)
    call $tmp := $CopyOrMoveValue($t7);
    $t5 := $tmp;

    // PackRef($t3)
    call $Libra_Preburn_after_update_inv($tv0, $Dereference($t3));

    // $t8 := read_ref($t3)
    call $tmp := $ReadRef($t3);
    assume $Libra_Preburn_is_well_formed($tmp);
    $t8 := $tmp;

    // $t8 := Libra::burn_with_resource_cap<#0>($t8, $t6, $t5)
    call $t8 := $Libra_burn_with_resource_cap($tv0, $t8, $t6, $t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 21546);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($t8);


    // write_ref($t3, $t8)
    call $t3 := $WriteRef($t3, $t8);

    // Libra::Preburn <- $t3
    call $WritebackToGlobal($t3);

    // UnpackRef($t3)
    call $Libra_Preburn_before_update_inv($tv0, $Dereference($t3));

    // PackRef($t3)
    call $Libra_Preburn_after_update_inv($tv0, $Dereference($t3));

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Libra_burn_with_capability_$direct_inter($tv0: $TypeValue, preburn_address: $Value, capability: $Value) returns ()
{
    assume is#$Address(preburn_address);

    assume $Libra_BurnCapability_is_well_formed(capability);

    call $Libra_burn_with_capability_$def($tv0, preburn_address, capability);
}


procedure {:inline 1} $Libra_burn_with_capability_$direct_intra($tv0: $TypeValue, preburn_address: $Value, capability: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(preburn_address);

    assume $Libra_BurnCapability_is_well_formed(capability);

    call $Libra_burn_with_capability_$def($tv0, preburn_address, capability);
}


procedure {:inline 1} $Libra_burn_with_capability($tv0: $TypeValue, preburn_address: $Value, capability: $Value) returns ()
{
    assume is#$Address(preburn_address);

    assume $Libra_BurnCapability_is_well_formed(capability);

    call $Libra_burn_with_capability_$def($tv0, preburn_address, capability);
}


procedure {:inline 1} $Libra_burn_with_resource_cap_$def($tv0: $TypeValue, preburn: $Value, preburn_address: $Value, _capability: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var currency_code: $Value; // $Vector_type_value($IntegerType())
    var info: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var value: $Value; // $IntegerType()
    var $t8: $Value; // $Vector_type_value($IntegerType())
    var $t9: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t10: $Value; // $Libra_Libra_type_value($tv0)
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t17: $Value; // $IntegerType()
    var $t18: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t19: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t20: $Value; // $Libra_Libra_type_value($tv0)
    var $t21: $Value; // $IntegerType()
    var $t22: $Value; // $AddressType()
    var $t23: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t24: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $IntegerType()
    var $t27: $Value; // $IntegerType()
    var $t28: $Value; // $IntegerType()
    var $t29: $Value; // $IntegerType()
    var $t30: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t31: $Reference; // ReferenceType($IntegerType())
    var $t32: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t33: $Value; // $IntegerType()
    var $t34: $Value; // $IntegerType()
    var $t35: $Value; // $IntegerType()
    var $t36: $Value; // $IntegerType()
    var $t37: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t38: $Reference; // ReferenceType($IntegerType())
    var $t39: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t40: $Value; // $BooleanType()
    var $t41: $Value; // $BooleanType()
    var $t42: $Value; // $BooleanType()
    var $t43: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t44: $Reference; // ReferenceType($Event_EventHandle_type_value($Libra_BurnEvent_type_value()))
    var $t45: $Value; // $IntegerType()
    var $t46: $Value; // $Vector_type_value($IntegerType())
    var $t47: $Value; // $AddressType()
    var $t48: $Value; // $Libra_BurnEvent_type_value()
    var $t49: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t50: $Value; // $Libra_Preburn_type_value($tv0)
    var $t51: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t52: $Value; // $AddressType()
    var $t53: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t54: $Value; // $Event_EventHandle_type_value($Libra_BurnEvent_type_value())
    var $t55: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 21535, 0, preburn); }
    if (true) { assume $DebugTrackLocal(9, 21535, 1, preburn_address); }
    if (true) { assume $DebugTrackLocal(9, 21535, 2, _capability); }

    // bytecode translation starts here
    // $t50 := move(preburn)
    call $tmp := $CopyOrMoveValue(preburn);
    $t50 := $tmp;

    // $t52 := move(preburn_address)
    call $tmp := $CopyOrMoveValue(preburn_address);
    $t52 := $tmp;

    // $t53 := move(_capability)
    call $tmp := $CopyOrMoveValue(_capability);
    $t53 := $tmp;

    // $t51 := borrow_local($t50)
    call $t51 := $BorrowLoc(50, $t50);
    assume $Libra_Preburn_is_well_formed($Dereference($t51));

    // UnpackRef($t51)
    call $Libra_Preburn_before_update_inv($tv0, $Dereference($t51));

    // $t8 := Libra::currency_code<#0>()
    call $t8 := $Libra_currency_code($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 35732);
      goto Abort;
    }
    assume $Vector_is_well_formed($t8) && (forall $$0: int :: {$select_vector($t8,$$0)} $$0 >= 0 && $$0 < $vlen($t8) ==> $IsValidU8($select_vector($t8,$$0)));


    // currency_code := $t8
    call $tmp := $CopyOrMoveValue($t8);
    currency_code := $tmp;
    if (true) { assume $DebugTrackLocal(9, 21744, 3, $tmp); }

    // $t9 := copy($t51)
    call $t9 := $CopyOrMoveRef($t51);

    // $t10 := get_field<Libra::Preburn<#0>>.to_burn($t9)
    call $tmp := $GetFieldFromReference($t9, $Libra_Preburn_to_burn);
    assume $Libra_Libra_is_well_formed($tmp);
    $t10 := $tmp;

    // Reference($t51) <- $t9
    call $t51 := $WritebackToReference($t9, $t51);

    // $t11 := get_field<Libra::Libra<#0>>.value($t10)
    call $tmp := $GetFieldFromValue($t10, $Libra_Libra_value);
    assume $IsValidU64($tmp);
    $t11 := $tmp;

    // $t12 := move($t11)
    call $tmp := $CopyOrMoveValue($t11);
    $t12 := $tmp;

    // $t13 := 0
    $tmp := $Integer(0);
    $t13 := $tmp;

    // $t14 := >($t12, $t13)
    call $tmp := $Gt($t12, $t13);
    $t14 := $tmp;

    // $t5 := $t14
    call $tmp := $CopyOrMoveValue($t14);
    $t5 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 21847, 5, $tmp); }

    // if ($t5) goto L0 else goto L1
    $tmp := $t5;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t16 := move($t51)
    call $t16 := $CopyOrMoveRef($t51);

    // destroy($t16)

    // LocalRoot($t50) <- $t16
    call $t50 := $WritebackToValue($t16, 50, $t50);

    // PackRef($t16)
    call $Libra_Preburn_after_update_inv($tv0, $Dereference($t16));

    // $t17 := 7
    $tmp := $Integer(7);
    $t17 := $tmp;

    // abort($t17)
    if (true) { assume $DebugTrackAbort(9, 21847); }
    goto Abort;

    // L0:
L0:

    // $t18 := move($t51)
    call $t18 := $CopyOrMoveRef($t51);

    // $t19 := borrow_field<Libra::Preburn<#0>>.to_burn($t18)
    call $t19 := $BorrowField($t18, $Libra_Preburn_to_burn);
    assume $Libra_Libra_is_well_formed_types($Dereference($t19));

    // LocalRoot($t50) <- $t18
    call $t50 := $WritebackToValue($t18, 50, $t50);

    // UnpackRef($t19)
    call $Libra_Libra_before_update_inv($tv0, $Dereference($t19));

    // PackRef($t19)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t19));

    // $t55 := read_ref($t19)
    call $tmp := $ReadRef($t19);
    assume $Libra_Libra_is_well_formed($tmp);
    $t55 := $tmp;

    // ($t20, $t55) := Libra::withdraw_all<#0>($t55)
    call $t20, $t55 := $Libra_withdraw_all($tv0, $t55);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 28601);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t20);

    assume $Libra_Libra_is_well_formed($t55);


    // write_ref($t19, $t55)
    call $t19 := $WriteRef($t19, $t55);
    if (true) { assume $DebugTrackLocal(9, 21535, 4, $Dereference(info)); }

    // LocalRoot($t50) <- $t19
    call $t50 := $WritebackToValue($t19, 50, $t50);

    // Reference($t18) <- $t19
    call $t18 := $WritebackToReference($t19, $t18);

    // UnpackRef($t19)
    call $Libra_Libra_before_update_inv($tv0, $Dereference($t19));

    // PackRef($t18)
    call $Libra_Preburn_after_update_inv($tv0, $Dereference($t18));

    // PackRef($t19)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t19));

    // $t21 := unpack Libra::Libra<#0>($t20)
    call $t21 := $Libra_Libra_unpack($tv0, $t20);
    $t21 := $t21;

    // value := $t21
    call $tmp := $CopyOrMoveValue($t21);
    value := $tmp;
    if (true) { assume $DebugTrackLocal(9, 21949, 7, $tmp); }

    // $t22 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t22 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 22114);
      goto Abort;
    }
    assume is#$Address($t22);


    // $t23 := borrow_global<Libra::CurrencyInfo<#0>>($t22)
    call $t23 := $BorrowGlobal($t22, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 22057);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($Dereference($t23));

    // UnpackRef($t23)

    // info := $t23
    call info := $CopyOrMoveRef($t23);
    if (true) { assume $DebugTrackLocal(9, 22050, 4, $Dereference(info)); }

    // $t24 := copy(info)
    call $t24 := $CopyOrMoveRef(info);

    // $t25 := get_field<Libra::CurrencyInfo<#0>>.total_value($t24)
    call $tmp := $GetFieldFromReference($t24, $Libra_CurrencyInfo_total_value);
    assume $IsValidU128($tmp);
    $t25 := $tmp;

    // Reference(info) <- $t24
    call info := $WritebackToReference($t24, info);

    // $t26 := move($t25)
    call $tmp := $CopyOrMoveValue($t25);
    $t26 := $tmp;

    // $t28 := (u128)(value)
    call $tmp := $CastU128(value);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 22186);
      goto Abort;
    }
    $t28 := $tmp;

    // $t29 := -($t26, $t28)
    call $tmp := $Sub($t26, $t28);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 22184);
      goto Abort;
    }
    $t29 := $tmp;

    // $t30 := copy(info)
    call $t30 := $CopyOrMoveRef(info);

    // $t31 := borrow_field<Libra::CurrencyInfo<#0>>.total_value($t30)
    call $t31 := $BorrowField($t30, $Libra_CurrencyInfo_total_value);
    assume $IsValidU128($Dereference($t31));

    // Reference(info) <- $t30
    call info := $WritebackToReference($t30, info);

    // UnpackRef($t31)

    // write_ref($t31, $t29)
    call $t31 := $WriteRef($t31, $t29);
    if (true) { assume $DebugTrackLocal(9, 22148, 4, $Dereference(info)); }

    // Reference(info) <- $t31
    call info := $WritebackToReference($t31, info);

    // Reference($t30) <- $t31
    call $t30 := $WritebackToReference($t31, $t30);

    // PackRef($t31)

    // $t32 := copy(info)
    call $t32 := $CopyOrMoveRef(info);

    // $t33 := get_field<Libra::CurrencyInfo<#0>>.preburn_value($t32)
    call $tmp := $GetFieldFromReference($t32, $Libra_CurrencyInfo_preburn_value);
    assume $IsValidU64($tmp);
    $t33 := $tmp;

    // Reference(info) <- $t32
    call info := $WritebackToReference($t32, info);

    // $t34 := move($t33)
    call $tmp := $CopyOrMoveValue($t33);
    $t34 := $tmp;

    // $t36 := -($t34, value)
    call $tmp := $Sub($t34, value);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 22251);
      goto Abort;
    }
    $t36 := $tmp;

    // $t37 := copy(info)
    call $t37 := $CopyOrMoveRef(info);

    // $t38 := borrow_field<Libra::CurrencyInfo<#0>>.preburn_value($t37)
    call $t38 := $BorrowField($t37, $Libra_CurrencyInfo_preburn_value);
    assume $IsValidU64($Dereference($t38));

    // Reference(info) <- $t37
    call info := $WritebackToReference($t37, info);

    // UnpackRef($t38)

    // write_ref($t38, $t36)
    call $t38 := $WriteRef($t38, $t36);
    if (true) { assume $DebugTrackLocal(9, 22211, 4, $Dereference(info)); }

    // Reference(info) <- $t38
    call info := $WritebackToReference($t38, info);

    // Reference($t37) <- $t38
    call $t37 := $WritebackToReference($t38, $t37);

    // PackRef($t38)

    // $t39 := copy(info)
    call $t39 := $CopyOrMoveRef(info);

    // $t40 := get_field<Libra::CurrencyInfo<#0>>.is_synthetic($t39)
    call $tmp := $GetFieldFromReference($t39, $Libra_CurrencyInfo_is_synthetic);
    assume is#$Boolean($tmp);
    $t40 := $tmp;

    // Reference(info) <- $t39
    call info := $WritebackToReference($t39, info);

    // $t41 := move($t40)
    call $tmp := $CopyOrMoveValue($t40);
    $t41 := $tmp;

    // $t42 := !($t41)
    call $tmp := $Not($t41);
    $t42 := $tmp;

    // if ($t42) goto L2 else goto L3
    $tmp := $t42;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // goto L4
    goto L4;

    // L2:
L2:

    // $t43 := move(info)
    call $t43 := $CopyOrMoveRef(info);

    // $t44 := borrow_field<Libra::CurrencyInfo<#0>>.burn_events($t43)
    call $t44 := $BorrowField($t43, $Libra_CurrencyInfo_burn_events);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t44));

    // Libra::CurrencyInfo <- $t43
    call $WritebackToGlobal($t43);

    // UnpackRef($t44)

    // $t48 := pack Libra::BurnEvent(value, currency_code, $t52)
    call $tmp := $Libra_BurnEvent_pack(0, 0, 0, value, currency_code, $t52);
    $t48 := $tmp;

    // PackRef($t44)

    // $t54 := read_ref($t44)
    call $tmp := $ReadRef($t44);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t54 := $tmp;

    // $t54 := Event::emit_event<Libra::BurnEvent>($t54, $t48)
    call $t54 := $Event_emit_event($Libra_BurnEvent_type_value(), $t54, $t48);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 22372);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t54);


    // write_ref($t44, $t54)
    call $t44 := $WriteRef($t44, $t54);
    if (true) { assume $DebugTrackLocal(9, 21535, 4, $Dereference(info)); }

    // Libra::CurrencyInfo <- $t44
    call $WritebackToGlobal($t44);

    // Reference($t43) <- $t44
    call $t43 := $WritebackToReference($t44, $t43);

    // UnpackRef($t44)

    // PackRef($t43)

    // PackRef($t44)

    // goto L5
    goto L5;

    // L4:
L4:

    // $t49 := move(info)
    call $t49 := $CopyOrMoveRef(info);

    // destroy($t49)

    // Libra::CurrencyInfo <- $t49
    call $WritebackToGlobal($t49);

    // PackRef($t49)

    // goto L5
    goto L5;

    // L5:
L5:

    // return $t50
    $ret0 := $t50;
    if (true) { assume $DebugTrackLocal(9, 22600, 56, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_burn_with_resource_cap_$direct_inter($tv0: $TypeValue, preburn: $Value, preburn_address: $Value, _capability: $Value) returns ($ret0: $Value)
{
    assume $Libra_Preburn_is_well_formed(preburn);

    assume is#$Address(preburn_address);

    assume $Libra_BurnCapability_is_well_formed(_capability);

    call $ret0 := $Libra_burn_with_resource_cap_$def($tv0, preburn, preburn_address, _capability);
}


procedure {:inline 1} $Libra_burn_with_resource_cap_$direct_intra($tv0: $TypeValue, preburn: $Value, preburn_address: $Value, _capability: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume $Libra_Preburn_is_well_formed(preburn);

    assume is#$Address(preburn_address);

    assume $Libra_BurnCapability_is_well_formed(_capability);

    call $ret0 := $Libra_burn_with_resource_cap_$def($tv0, preburn, preburn_address, _capability);
}


procedure {:inline 1} $Libra_burn_with_resource_cap($tv0: $TypeValue, preburn: $Value, preburn_address: $Value, _capability: $Value) returns ($ret0: $Value)
{
    assume $Libra_Preburn_is_well_formed(preburn);

    assume is#$Address(preburn_address);

    assume $Libra_BurnCapability_is_well_formed(_capability);

    call $ret0 := $Libra_burn_with_resource_cap_$def($tv0, preburn, preburn_address, _capability);
}


procedure {:inline 1} $Libra_cancel_burn_$def($tv0: $TypeValue, account: $Value, preburn_address: $Value) returns ($ret0: $Value)
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 13691, 0, account); }
    if (true) { assume $DebugTrackLocal(9, 13691, 1, preburn_address); }

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
      assume $DebugTrackAbort(9, 13982);
      goto Abort;
    }
    assume is#$Address($t4);


    // $t5 := get_global<Libra::BurnCapability<#0>>($t4)
    call $tmp := $GetGlobal($t4, $Libra_BurnCapability_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 13934);
      goto Abort;
    }
    assume $Libra_BurnCapability_is_well_formed($tmp);
    $t5 := $tmp;

    // $t6 := Libra::cancel_burn_with_capability<#0>($t8, $t5)
    call $t6 := $Libra_cancel_burn_with_capability($tv0, $t8, $t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 24134);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t6);


    // return $t6
    $ret0 := $t6;
    if (true) { assume $DebugTrackLocal(9, 13864, 9, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_cancel_burn_$direct_inter($tv0: $TypeValue, account: $Value, preburn_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    assume is#$Address(preburn_address);

    call $ret0 := $Libra_cancel_burn_$def($tv0, account, preburn_address);
}


procedure {:inline 1} $Libra_cancel_burn_$direct_intra($tv0: $TypeValue, account: $Value, preburn_address: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(account);

    assume is#$Address(preburn_address);

    call $ret0 := $Libra_cancel_burn_$def($tv0, account, preburn_address);
}


procedure {:inline 1} $Libra_cancel_burn($tv0: $TypeValue, account: $Value, preburn_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    assume is#$Address(preburn_address);

    call $ret0 := $Libra_cancel_burn_$def($tv0, account, preburn_address);
}


procedure {:inline 1} $Libra_cancel_burn_with_capability_$def($tv0: $TypeValue, preburn_address: $Value, _capability: $Value) returns ($ret0: $Value)
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
    var $t10: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t11: $Value; // $Libra_Libra_type_value($tv0)
    var $t12: $Value; // $Vector_type_value($IntegerType())
    var $t13: $Value; // $AddressType()
    var $t14: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t15: $Value; // $Libra_Libra_type_value($tv0)
    var $t16: $Value; // $IntegerType()
    var $t17: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $IntegerType()
    var $t22: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t23: $Reference; // ReferenceType($IntegerType())
    var $t24: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t25: $Value; // $BooleanType()
    var $t26: $Value; // $BooleanType()
    var $t27: $Value; // $BooleanType()
    var $t28: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t29: $Reference; // ReferenceType($Event_EventHandle_type_value($Libra_CancelBurnEvent_type_value()))
    var $t30: $Value; // $IntegerType()
    var $t31: $Value; // $Vector_type_value($IntegerType())
    var $t32: $Value; // $AddressType()
    var $t33: $Value; // $Libra_CancelBurnEvent_type_value()
    var $t34: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t35: $Value; // $Libra_Libra_type_value($tv0)
    var $t36: $Value; // $AddressType()
    var $t37: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t38: $Value; // $Event_EventHandle_type_value($Libra_CancelBurnEvent_type_value())
    var $t39: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 24123, 0, preburn_address); }
    if (true) { assume $DebugTrackLocal(9, 24123, 1, _capability); }

    // bytecode translation starts here
    // $t36 := move(preburn_address)
    call $tmp := $CopyOrMoveValue(preburn_address);
    $t36 := $tmp;

    // $t37 := move(_capability)
    call $tmp := $CopyOrMoveValue(_capability);
    $t37 := $tmp;

    // $t8 := borrow_global<Libra::Preburn<#0>>($t36)
    call $t8 := $BorrowGlobal($t36, $Libra_Preburn_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 24380);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($Dereference($t8));

    // UnpackRef($t8)
    call $Libra_Preburn_before_update_inv($tv0, $Dereference($t8));

    // preburn := $t8
    call preburn := $CopyOrMoveRef($t8);
    if (true) { assume $DebugTrackLocal(9, 24370, 6, $Dereference(preburn)); }

    // $t9 := move(preburn)
    call $t9 := $CopyOrMoveRef(preburn);

    // $t10 := borrow_field<Libra::Preburn<#0>>.to_burn($t9)
    call $t10 := $BorrowField($t9, $Libra_Preburn_to_burn);
    assume $Libra_Libra_is_well_formed_types($Dereference($t10));

    // Libra::Preburn <- $t9
    call $WritebackToGlobal($t9);

    // UnpackRef($t10)
    call $Libra_Libra_before_update_inv($tv0, $Dereference($t10));

    // PackRef($t10)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t10));

    // $t39 := read_ref($t10)
    call $tmp := $ReadRef($t10);
    assume $Libra_Libra_is_well_formed($tmp);
    $t39 := $tmp;

    // ($t11, $t39) := Libra::withdraw_all<#0>($t39)
    call $t11, $t39 := $Libra_withdraw_all($tv0, $t39);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 28601);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t11);

    assume $Libra_Libra_is_well_formed($t39);


    // write_ref($t10, $t39)
    call $t10 := $WriteRef($t10, $t39);
    if (true) { assume $DebugTrackLocal(9, 25226, 5, $Dereference(info)); }
    if (true) { assume $DebugTrackLocal(9, 25226, 6, $Dereference(preburn)); }

    // Libra::Preburn <- $t10
    call $WritebackToGlobal($t10);

    // Reference($t9) <- $t10
    call $t9 := $WritebackToReference($t10, $t9);

    // UnpackRef($t10)
    call $Libra_Libra_before_update_inv($tv0, $Dereference($t10));

    // PackRef($t9)
    call $Libra_Preburn_after_update_inv($tv0, $Dereference($t9));

    // PackRef($t10)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t10));

    // coin := $t11
    call $tmp := $CopyOrMoveValue($t11);
    coin := $tmp;
    if (true) { assume $DebugTrackLocal(9, 24447, 3, $tmp); }

    // $t12 := Libra::currency_code<#0>()
    call $t12 := $Libra_currency_code($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 35732);
      goto Abort;
    }
    assume $Vector_is_well_formed($t12) && (forall $$0: int :: {$select_vector($t12,$$0)} $$0 >= 0 && $$0 < $vlen($t12) ==> $IsValidU8($select_vector($t12,$$0)));


    // currency_code := $t12
    call $tmp := $CopyOrMoveValue($t12);
    currency_code := $tmp;
    if (true) { assume $DebugTrackLocal(9, 24545, 4, $tmp); }

    // $t13 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t13 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 24664);
      goto Abort;
    }
    assume is#$Address($t13);


    // $t14 := borrow_global<Libra::CurrencyInfo<#0>>($t13)
    call $t14 := $BorrowGlobal($t13, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 24607);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($Dereference($t14));

    // UnpackRef($t14)

    // info := $t14
    call info := $CopyOrMoveRef($t14);
    if (true) { assume $DebugTrackLocal(9, 24600, 5, $Dereference(info)); }

    // $t15 := copy(coin)
    call $tmp := $CopyOrMoveValue(coin);
    $t15 := $tmp;

    // $t16 := Libra::value<#0>($t15)
    call $t16 := $Libra_value($tv0, $t15);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 26961);
      goto Abort;
    }
    assume $IsValidU64($t16);


    // amount := $t16
    call $tmp := $CopyOrMoveValue($t16);
    amount := $tmp;
    if (true) { assume $DebugTrackLocal(9, 24702, 2, $tmp); }

    // $t17 := copy(info)
    call $t17 := $CopyOrMoveRef(info);

    // $t18 := get_field<Libra::CurrencyInfo<#0>>.preburn_value($t17)
    call $tmp := $GetFieldFromReference($t17, $Libra_CurrencyInfo_preburn_value);
    assume $IsValidU64($tmp);
    $t18 := $tmp;

    // Reference(info) <- $t17
    call info := $WritebackToReference($t17, info);

    // $t19 := move($t18)
    call $tmp := $CopyOrMoveValue($t18);
    $t19 := $tmp;

    // $t21 := -($t19, amount)
    call $tmp := $Sub($t19, amount);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 24773);
      goto Abort;
    }
    $t21 := $tmp;

    // $t22 := copy(info)
    call $t22 := $CopyOrMoveRef(info);

    // $t23 := borrow_field<Libra::CurrencyInfo<#0>>.preburn_value($t22)
    call $t23 := $BorrowField($t22, $Libra_CurrencyInfo_preburn_value);
    assume $IsValidU64($Dereference($t23));

    // Reference(info) <- $t22
    call info := $WritebackToReference($t22, info);

    // UnpackRef($t23)

    // write_ref($t23, $t21)
    call $t23 := $WriteRef($t23, $t21);
    if (true) { assume $DebugTrackLocal(9, 24733, 5, $Dereference(info)); }
    if (true) { assume $DebugTrackLocal(9, 24733, 6, $Dereference(preburn)); }

    // Reference(info) <- $t23
    call info := $WritebackToReference($t23, info);

    // Reference($t22) <- $t23
    call $t22 := $WritebackToReference($t23, $t22);

    // PackRef($t23)

    // $t24 := copy(info)
    call $t24 := $CopyOrMoveRef(info);

    // $t25 := get_field<Libra::CurrencyInfo<#0>>.is_synthetic($t24)
    call $tmp := $GetFieldFromReference($t24, $Libra_CurrencyInfo_is_synthetic);
    assume is#$Boolean($tmp);
    $t25 := $tmp;

    // Reference(info) <- $t24
    call info := $WritebackToReference($t24, info);

    // $t26 := move($t25)
    call $tmp := $CopyOrMoveValue($t25);
    $t26 := $tmp;

    // $t27 := !($t26)
    call $tmp := $Not($t26);
    $t27 := $tmp;

    // if ($t27) goto L0 else goto L1
    $tmp := $t27;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t28 := move(info)
    call $t28 := $CopyOrMoveRef(info);

    // $t29 := borrow_field<Libra::CurrencyInfo<#0>>.cancel_burn_events($t28)
    call $t29 := $BorrowField($t28, $Libra_CurrencyInfo_cancel_burn_events);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t29));

    // Libra::CurrencyInfo <- $t28
    call $WritebackToGlobal($t28);

    // UnpackRef($t29)

    // $t33 := pack Libra::CancelBurnEvent(amount, currency_code, $t36)
    call $tmp := $Libra_CancelBurnEvent_pack(0, 0, 0, amount, currency_code, $t36);
    $t33 := $tmp;

    // PackRef($t29)

    // $t38 := read_ref($t29)
    call $tmp := $ReadRef($t29);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t38 := $tmp;

    // $t38 := Event::emit_event<Libra::CancelBurnEvent>($t38, $t33)
    call $t38 := $Event_emit_event($Libra_CancelBurnEvent_type_value(), $t38, $t33);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 24981);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t38);


    // write_ref($t29, $t38)
    call $t29 := $WriteRef($t29, $t38);
    if (true) { assume $DebugTrackLocal(9, 24123, 5, $Dereference(info)); }
    if (true) { assume $DebugTrackLocal(9, 24123, 6, $Dereference(preburn)); }

    // Libra::CurrencyInfo <- $t29
    call $WritebackToGlobal($t29);

    // Reference($t28) <- $t29
    call $t28 := $WritebackToReference($t29, $t28);

    // UnpackRef($t29)

    // PackRef($t28)

    // PackRef($t29)

    // goto L3
    goto L3;

    // L2:
L2:

    // $t34 := move(info)
    call $t34 := $CopyOrMoveRef(info);

    // destroy($t34)

    // Libra::CurrencyInfo <- $t34
    call $WritebackToGlobal($t34);

    // PackRef($t34)

    // goto L3
    goto L3;

    // L3:
L3:

    // return coin
    $ret0 := coin;
    if (true) { assume $DebugTrackLocal(9, 25226, 40, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_cancel_burn_with_capability_$direct_inter($tv0: $TypeValue, preburn_address: $Value, _capability: $Value) returns ($ret0: $Value)
{
    assume is#$Address(preburn_address);

    assume $Libra_BurnCapability_is_well_formed(_capability);

    call $ret0 := $Libra_cancel_burn_with_capability_$def($tv0, preburn_address, _capability);
}


procedure {:inline 1} $Libra_cancel_burn_with_capability_$direct_intra($tv0: $TypeValue, preburn_address: $Value, _capability: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(preburn_address);

    assume $Libra_BurnCapability_is_well_formed(_capability);

    call $ret0 := $Libra_cancel_burn_with_capability_$def($tv0, preburn_address, _capability);
}


procedure {:inline 1} $Libra_cancel_burn_with_capability($tv0: $TypeValue, preburn_address: $Value, _capability: $Value) returns ($ret0: $Value)
{
    assume is#$Address(preburn_address);

    assume $Libra_BurnCapability_is_well_formed(_capability);

    call $ret0 := $Libra_cancel_burn_with_capability_$def($tv0, preburn_address, _capability);
}


procedure {:inline 1} $Libra_create_preburn_$def($tv0: $TypeValue, tc_account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $Libra_Libra_type_value($tv0)
    var $t8: $Value; // $Libra_Preburn_type_value($tv0)
    var $t9: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 18173, 0, tc_account); }

    // bytecode translation starts here
    // $t9 := move(tc_account)
    call $tmp := $CopyOrMoveValue(tc_account);
    $t9 := $tmp;

    // $t3 := move($t9)
    call $tmp := $CopyOrMoveValue($t9);
    $t3 := $tmp;

    // $t4 := Roles::has_treasury_compliance_role($t3)
    call $t4 := $Roles_has_treasury_compliance_role($t3);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 18287);
      goto Abort;
    }
    assume is#$Boolean($t4);


    // $t1 := $t4
    call $tmp := $CopyOrMoveValue($t4);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 18273, 1, $tmp); }

    // if ($t1) goto L0 else goto L1
    $tmp := $t1;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t6 := 2
    $tmp := $Integer(2);
    $t6 := $tmp;

    // abort($t6)
    if (true) { assume $DebugTrackAbort(9, 18273); }
    goto Abort;

    // L0:
L0:

    // Libra::assert_is_currency<#0>()
    call $Libra_assert_is_currency($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 38329);
      goto Abort;
    }

    // $t7 := Libra::zero<#0>()
    call $t7 := $Libra_zero($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 26670);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t7);


    // $t8 := pack Libra::Preburn<#0>($t7)
    call $tmp := $Libra_Preburn_pack(0, 0, 0, $tv0, $t7);
    $t8 := $tmp;

    // return $t8
    $ret0 := $t8;
    if (true) { assume $DebugTrackLocal(9, 18404, 10, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_create_preburn_$direct_inter($tv0: $TypeValue, tc_account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(tc_account);

    call $ret0 := $Libra_create_preburn_$def($tv0, tc_account);
}


procedure {:inline 1} $Libra_create_preburn_$direct_intra($tv0: $TypeValue, tc_account: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(tc_account);

    call $ret0 := $Libra_create_preburn_$def($tv0, tc_account);
}


procedure {:inline 1} $Libra_create_preburn($tv0: $TypeValue, tc_account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(tc_account);

    call $ret0 := $Libra_create_preburn_$def($tv0, tc_account);
}


procedure {:inline 1} $Libra_deposit_$def($tv0: $TypeValue, coin: $Value, check: $Value) returns ($ret0: $Value)
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 29437, 0, coin); }
    if (true) { assume $DebugTrackLocal(9, 29437, 1, check); }

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
    if (true) { assume $DebugTrackLocal(9, 29540, 2, $tmp); }

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
      assume $DebugTrackAbort(9, 29589);
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
    if (true) { assume $DebugTrackLocal(9, 29596, 15, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_deposit_$direct_inter($tv0: $TypeValue, coin: $Value, check: $Value) returns ($ret0: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    assume $Libra_Libra_is_well_formed(check);

    call $ret0 := $Libra_deposit_$def($tv0, coin, check);
}


procedure {:inline 1} $Libra_deposit_$direct_intra($tv0: $TypeValue, coin: $Value, check: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume $Libra_Libra_is_well_formed(coin);

    assume $Libra_Libra_is_well_formed(check);

    call $ret0 := $Libra_deposit_$def($tv0, coin, check);
}


procedure {:inline 1} $Libra_deposit($tv0: $TypeValue, coin: $Value, check: $Value) returns ($ret0: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    assume $Libra_Libra_is_well_formed(check);

    call $ret0 := $Libra_deposit_$def($tv0, coin, check);
}


procedure {:inline 1} $Libra_destroy_zero_$def($tv0: $TypeValue, coin: $Value) returns ()
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 29842, 0, coin); }

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
    if (true) { assume $DebugTrackLocal(9, 29921, 3, $tmp); }

    // $t7 := 0
    $tmp := $Integer(0);
    $t7 := $tmp;

    // $t8 := ==(value, $t7)
    $tmp := $Boolean($IsEqual(value, $t7));
    $t8 := $tmp;

    // $t1 := $t8
    call $tmp := $CopyOrMoveValue($t8);
    $t1 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 29945, 1, $tmp); }

    // if ($t1) goto L0 else goto L1
    $tmp := $t1;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t10 := 6
    $tmp := $Integer(6);
    $t10 := $tmp;

    // abort($t10)
    if (true) { assume $DebugTrackAbort(9, 29945); }
    goto Abort;

    // L0:
L0:

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Libra_destroy_zero_$direct_inter($tv0: $TypeValue, coin: $Value) returns ()
{
    assume $Libra_Libra_is_well_formed(coin);

    call $Libra_destroy_zero_$def($tv0, coin);
}


procedure {:inline 1} $Libra_destroy_zero_$direct_intra($tv0: $TypeValue, coin: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume $Libra_Libra_is_well_formed(coin);

    call $Libra_destroy_zero_$def($tv0, coin);
}


procedure {:inline 1} $Libra_destroy_zero($tv0: $TypeValue, coin: $Value) returns ()
{
    assume $Libra_Libra_is_well_formed(coin);

    call $Libra_destroy_zero_$def($tv0, coin);
}


procedure {:inline 1} $Libra_fractional_part_$def($tv0: $TypeValue) returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 35556);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := get_global<Libra::CurrencyInfo<#0>>($t0)
    call $tmp := $GetGlobal($t0, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 35503);
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
    if (true) { assume $DebugTrackLocal(9, 35503, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_fractional_part_$direct_inter($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_fractional_part_$def($tv0);
}


procedure {:inline 1} $Libra_fractional_part_$direct_intra($tv0: $TypeValue) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    call $ret0 := $Libra_fractional_part_$def($tv0);
}


procedure {:inline 1} $Libra_fractional_part($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_fractional_part_$def($tv0);
}


procedure {:inline 1} $Libra_is_currency_$def($tv0: $TypeValue) returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $BooleanType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 34565);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := exists<Libra::CurrencyInfo<#0>>($t0)
    call $tmp := $Exists($t0, $Libra_CurrencyInfo_type_value($tv0));
    $t1 := $tmp;

    // return $t1
    $ret0 := $t1;
    if (true) { assume $DebugTrackLocal(9, 34519, 2, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_is_currency_$direct_inter($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_is_currency_$def($tv0);
}


procedure {:inline 1} $Libra_is_currency_$direct_intra($tv0: $TypeValue) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    call $ret0 := $Libra_is_currency_$def($tv0);
}


procedure {:inline 1} $Libra_is_currency($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_is_currency_$def($tv0);
}


procedure {:inline 1} $Libra_is_synthetic_currency_$def($tv0: $TypeValue) returns ($ret0: $Value)
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t2 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t2 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 34844);
      goto Abort;
    }
    assume is#$Address($t2);


    // addr := $t2
    call $tmp := $CopyOrMoveValue($t2);
    addr := $tmp;
    if (true) { assume $DebugTrackLocal(9, 34822, 0, $tmp); }

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
      assume $DebugTrackAbort(9, 34929);
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
    if (true) { assume $DebugTrackLocal(9, 34877, 1, $tmp); }

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
    if (true) { assume $DebugTrackLocal(9, 34877, 1, $tmp); }

    // goto L3
    goto L3;

    // L3:
L3:

    // return $t1
    $ret0 := $t1;
    if (true) { assume $DebugTrackLocal(9, 34877, 11, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_is_synthetic_currency_$direct_inter($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_is_synthetic_currency_$def($tv0);
}


procedure {:inline 1} $Libra_is_synthetic_currency_$direct_intra($tv0: $TypeValue) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    call $ret0 := $Libra_is_synthetic_currency_$def($tv0);
}


procedure {:inline 1} $Libra_is_synthetic_currency($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_is_synthetic_currency_$def($tv0);
}


procedure {:inline 1} $Libra_join_$def($tv0: $TypeValue, coin1: $Value, coin2: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t2: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t3: $Value; // $Libra_Libra_type_value($tv0)
    var $t4: $Value; // $Libra_Libra_type_value($tv0)
    var $t5: $Value; // $Libra_Libra_type_value($tv0)
    var $t6: $Value; // $Libra_Libra_type_value($tv0)
    var $t7: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 28963, 0, coin1); }
    if (true) { assume $DebugTrackLocal(9, 28963, 1, coin2); }

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
      assume $DebugTrackAbort(9, 29448);
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
    if (true) { assume $DebugTrackLocal(9, 29101, 8, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_join_$direct_inter($tv0: $TypeValue, coin1: $Value, coin2: $Value) returns ($ret0: $Value)
{
    assume $Libra_Libra_is_well_formed(coin1);

    assume $Libra_Libra_is_well_formed(coin2);

    call $ret0 := $Libra_join_$def($tv0, coin1, coin2);
}


procedure {:inline 1} $Libra_join_$direct_intra($tv0: $TypeValue, coin1: $Value, coin2: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume $Libra_Libra_is_well_formed(coin1);

    assume $Libra_Libra_is_well_formed(coin2);

    call $ret0 := $Libra_join_$def($tv0, coin1, coin2);
}


procedure {:inline 1} $Libra_join($tv0: $TypeValue, coin1: $Value, coin2: $Value) returns ($ret0: $Value)
{
    assume $Libra_Libra_is_well_formed(coin1);

    assume $Libra_Libra_is_well_formed(coin2);

    call $ret0 := $Libra_join_$def($tv0, coin1, coin2);
}


procedure {:inline 1} $Libra_lbr_exchange_rate_$def($tv0: $TypeValue) returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t2: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t3: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 37069);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := get_global<Libra::CurrencyInfo<#0>>($t0)
    call $tmp := $GetGlobal($t0, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 37016);
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
    if (true) { assume $DebugTrackLocal(9, 37014, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_lbr_exchange_rate_$direct_inter($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_lbr_exchange_rate_$def($tv0);
}


procedure {:inline 1} $Libra_lbr_exchange_rate_$direct_intra($tv0: $TypeValue) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    call $ret0 := $Libra_lbr_exchange_rate_$def($tv0);
}


procedure {:inline 1} $Libra_lbr_exchange_rate($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_lbr_exchange_rate_$def($tv0);
}


procedure {:inline 1} $Libra_market_cap_$def($tv0: $TypeValue) returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 33505);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := get_global<Libra::CurrencyInfo<#0>>($t0)
    call $tmp := $GetGlobal($t0, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 33452);
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
    if (true) { assume $DebugTrackLocal(9, 33452, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_market_cap_$direct_inter($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_market_cap_$def($tv0);
}


procedure {:inline 1} $Libra_market_cap_$direct_intra($tv0: $TypeValue) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    call $ret0 := $Libra_market_cap_$def($tv0);
}


procedure {:inline 1} $Libra_market_cap($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_market_cap_$def($tv0);
}


procedure {:inline 1} $Libra_mint_$def($tv0: $TypeValue, account: $Value, value: $Value) returns ($ret0: $Value)
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 12190, 0, account); }
    if (true) { assume $DebugTrackLocal(9, 12190, 1, value); }

    // bytecode translation starts here
    // $t7 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t7 := $tmp;

    // $t8 := move(value)
    call $tmp := $CopyOrMoveValue(value);
    $t8 := $tmp;

    // $t3 := move($t7)
    call $tmp := $CopyOrMoveValue($t7);
    $t3 := $tmp;

    // $t4 := Signer::address_of($t3)
    call $t4 := $Signer_address_of($t3);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 12416);
      goto Abort;
    }
    assume is#$Address($t4);


    // $t5 := get_global<Libra::MintCapability<#0>>($t4)
    call $tmp := $GetGlobal($t4, $Libra_MintCapability_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 12368);
      goto Abort;
    }
    assume $Libra_MintCapability_is_well_formed($tmp);
    $t5 := $tmp;

    // $t6 := Libra::mint_with_capability<#0>($t8, $t5)
    call $t6 := $Libra_mint_with_capability($tv0, $t8, $t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 14287);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t6);


    // return $t6
    $ret0 := $t6;
    if (true) { assume $DebugTrackLocal(9, 12315, 9, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_mint_$direct_inter($tv0: $TypeValue, account: $Value, value: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    assume $IsValidU64(value);

    call $ret0 := $Libra_mint_$def($tv0, account, value);
}


procedure {:inline 1} $Libra_mint_$direct_intra($tv0: $TypeValue, account: $Value, value: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(account);

    assume $IsValidU64(value);

    call $ret0 := $Libra_mint_$def($tv0, account, value);
}


procedure {:inline 1} $Libra_mint($tv0: $TypeValue, account: $Value, value: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    assume $IsValidU64(value);

    call $ret0 := $Libra_mint_$def($tv0, account, value);
}


procedure {:inline 1} $Libra_mint_with_capability_$def($tv0: $TypeValue, value: $Value, _capability: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var currency_code: $Value; // $Vector_type_value($IntegerType())
    var info: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $Vector_type_value($IntegerType())
    var $t7: $Value; // $AddressType()
    var $t8: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t9: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t10: $Value; // $BooleanType()
    var $t11: $Value; // $BooleanType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t14: $Value; // $IntegerType()
    var $t15: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t22: $Reference; // ReferenceType($IntegerType())
    var $t23: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t24: $Value; // $BooleanType()
    var $t25: $Value; // $BooleanType()
    var $t26: $Value; // $BooleanType()
    var $t27: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t28: $Reference; // ReferenceType($Event_EventHandle_type_value($Libra_MintEvent_type_value()))
    var $t29: $Value; // $IntegerType()
    var $t30: $Value; // $Vector_type_value($IntegerType())
    var $t31: $Value; // $Libra_MintEvent_type_value()
    var $t32: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t33: $Value; // $IntegerType()
    var $t34: $Value; // $Libra_Libra_type_value($tv0)
    var $t35: $Value; // $IntegerType()
    var $t36: $Value; // $Libra_MintCapability_type_value($tv0)
    var $t37: $Value; // $Event_EventHandle_type_value($Libra_MintEvent_type_value())
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 14276, 0, value); }
    if (true) { assume $DebugTrackLocal(9, 14276, 1, _capability); }

    // bytecode translation starts here
    // $t35 := move(value)
    call $tmp := $CopyOrMoveValue(value);
    $t35 := $tmp;

    // $t36 := move(_capability)
    call $tmp := $CopyOrMoveValue(_capability);
    $t36 := $tmp;

    // Libra::assert_is_currency<#0>()
    call $Libra_assert_is_currency($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 38329);
      goto Abort;
    }

    // $t6 := Libra::currency_code<#0>()
    call $t6 := $Libra_currency_code($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 35732);
      goto Abort;
    }
    assume $Vector_is_well_formed($t6) && (forall $$0: int :: {$select_vector($t6,$$0)} $$0 >= 0 && $$0 < $vlen($t6) ==> $IsValidU8($select_vector($t6,$$0)));


    // currency_code := $t6
    call $tmp := $CopyOrMoveValue($t6);
    currency_code := $tmp;
    if (true) { assume $DebugTrackLocal(9, 14485, 2, $tmp); }

    // $t7 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t7 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 14661);
      goto Abort;
    }
    assume is#$Address($t7);


    // $t8 := borrow_global<Libra::CurrencyInfo<#0>>($t7)
    call $t8 := $BorrowGlobal($t7, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 14604);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($Dereference($t8));

    // UnpackRef($t8)

    // info := $t8
    call info := $CopyOrMoveRef($t8);
    if (true) { assume $DebugTrackLocal(9, 14597, 3, $Dereference(info)); }

    // $t9 := copy(info)
    call $t9 := $CopyOrMoveRef(info);

    // $t10 := get_field<Libra::CurrencyInfo<#0>>.can_mint($t9)
    call $tmp := $GetFieldFromReference($t9, $Libra_CurrencyInfo_can_mint);
    assume is#$Boolean($tmp);
    $t10 := $tmp;

    // Reference(info) <- $t9
    call info := $WritebackToReference($t9, info);

    // $t11 := move($t10)
    call $tmp := $CopyOrMoveValue($t10);
    $t11 := $tmp;

    // $t4 := $t11
    call $tmp := $CopyOrMoveValue($t11);
    $t4 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 14695, 4, $tmp); }

    // if ($t4) goto L0 else goto L1
    $tmp := $t4;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t13 := move(info)
    call $t13 := $CopyOrMoveRef(info);

    // destroy($t13)

    // Libra::CurrencyInfo <- $t13
    call $WritebackToGlobal($t13);

    // PackRef($t13)

    // $t14 := 3
    $tmp := $Integer(3);
    $t14 := $tmp;

    // abort($t14)
    if (true) { assume $DebugTrackAbort(9, 14695); }
    goto Abort;

    // L0:
L0:

    // $t15 := copy(info)
    call $t15 := $CopyOrMoveRef(info);

    // $t16 := get_field<Libra::CurrencyInfo<#0>>.total_value($t15)
    call $tmp := $GetFieldFromReference($t15, $Libra_CurrencyInfo_total_value);
    assume $IsValidU128($tmp);
    $t16 := $tmp;

    // Reference(info) <- $t15
    call info := $WritebackToReference($t15, info);

    // $t17 := move($t16)
    call $tmp := $CopyOrMoveValue($t16);
    $t17 := $tmp;

    // $t19 := (u128)($t35)
    call $tmp := $CastU128($t35);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 14786);
      goto Abort;
    }
    $t19 := $tmp;

    // $t20 := +($t17, $t19)
    call $tmp := $AddU128($t17, $t19);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 14784);
      goto Abort;
    }
    $t20 := $tmp;

    // $t21 := copy(info)
    call $t21 := $CopyOrMoveRef(info);

    // $t22 := borrow_field<Libra::CurrencyInfo<#0>>.total_value($t21)
    call $t22 := $BorrowField($t21, $Libra_CurrencyInfo_total_value);
    assume $IsValidU128($Dereference($t22));

    // Reference(info) <- $t21
    call info := $WritebackToReference($t21, info);

    // UnpackRef($t22)

    // write_ref($t22, $t20)
    call $t22 := $WriteRef($t22, $t20);
    if (true) { assume $DebugTrackLocal(9, 14748, 3, $Dereference(info)); }

    // Reference(info) <- $t22
    call info := $WritebackToReference($t22, info);

    // Reference($t21) <- $t22
    call $t21 := $WritebackToReference($t22, $t21);

    // PackRef($t22)

    // $t23 := copy(info)
    call $t23 := $CopyOrMoveRef(info);

    // $t24 := get_field<Libra::CurrencyInfo<#0>>.is_synthetic($t23)
    call $tmp := $GetFieldFromReference($t23, $Libra_CurrencyInfo_is_synthetic);
    assume is#$Boolean($tmp);
    $t24 := $tmp;

    // Reference(info) <- $t23
    call info := $WritebackToReference($t23, info);

    // $t25 := move($t24)
    call $tmp := $CopyOrMoveValue($t24);
    $t25 := $tmp;

    // $t26 := !($t25)
    call $tmp := $Not($t25);
    $t26 := $tmp;

    // if ($t26) goto L2 else goto L3
    $tmp := $t26;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // goto L4
    goto L4;

    // L2:
L2:

    // $t27 := move(info)
    call $t27 := $CopyOrMoveRef(info);

    // $t28 := borrow_field<Libra::CurrencyInfo<#0>>.mint_events($t27)
    call $t28 := $BorrowField($t27, $Libra_CurrencyInfo_mint_events);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t28));

    // Libra::CurrencyInfo <- $t27
    call $WritebackToGlobal($t27);

    // UnpackRef($t28)

    // $t31 := pack Libra::MintEvent($t35, currency_code)
    call $tmp := $Libra_MintEvent_pack(0, 0, 0, $t35, currency_code);
    $t31 := $tmp;

    // PackRef($t28)

    // $t37 := read_ref($t28)
    call $tmp := $ReadRef($t28);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t37 := $tmp;

    // $t37 := Event::emit_event<Libra::MintEvent>($t37, $t31)
    call $t37 := $Event_emit_event($Libra_MintEvent_type_value(), $t37, $t31);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 14915);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t37);


    // write_ref($t28, $t37)
    call $t28 := $WriteRef($t28, $t37);
    if (true) { assume $DebugTrackLocal(9, 15116, 3, $Dereference(info)); }

    // Libra::CurrencyInfo <- $t28
    call $WritebackToGlobal($t28);

    // Reference($t27) <- $t28
    call $t27 := $WritebackToReference($t28, $t27);

    // UnpackRef($t28)

    // PackRef($t27)

    // PackRef($t28)

    // goto L5
    goto L5;

    // L4:
L4:

    // $t32 := move(info)
    call $t32 := $CopyOrMoveRef(info);

    // destroy($t32)

    // Libra::CurrencyInfo <- $t32
    call $WritebackToGlobal($t32);

    // PackRef($t32)

    // goto L5
    goto L5;

    // L5:
L5:

    // $t34 := pack Libra::Libra<#0>($t35)
    call $tmp := $Libra_Libra_pack(0, 0, 0, $tv0, $t35);
    $t34 := $tmp;

    // return $t34
    $ret0 := $t34;
    if (true) { assume $DebugTrackLocal(9, 15116, 38, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_mint_with_capability_$direct_inter($tv0: $TypeValue, value: $Value, _capability: $Value) returns ($ret0: $Value)
{
    assume $IsValidU64(value);

    assume $Libra_MintCapability_is_well_formed(_capability);

    call $ret0 := $Libra_mint_with_capability_$def($tv0, value, _capability);
}


procedure {:inline 1} $Libra_mint_with_capability_$direct_intra($tv0: $TypeValue, value: $Value, _capability: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume $IsValidU64(value);

    assume $Libra_MintCapability_is_well_formed(_capability);

    call $ret0 := $Libra_mint_with_capability_$def($tv0, value, _capability);
}


procedure {:inline 1} $Libra_mint_with_capability($tv0: $TypeValue, value: $Value, _capability: $Value) returns ($ret0: $Value)
{
    assume $IsValidU64(value);

    assume $Libra_MintCapability_is_well_formed(_capability);

    call $ret0 := $Libra_mint_with_capability_$def($tv0, value, _capability);
}


procedure {:inline 1} $Libra_preburn_to_$def($tv0: $TypeValue, account: $Value, coin: $Value) returns ()
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 19413, 0, account); }
    if (true) { assume $DebugTrackLocal(9, 19413, 1, coin); }

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
      assume $DebugTrackAbort(9, 19557);
      goto Abort;
    }
    assume is#$Address($t4);


    // sender := $t4
    call $tmp := $CopyOrMoveValue($t4);
    sender := $tmp;
    if (true) { assume $DebugTrackLocal(9, 19540, 2, $tmp); }

    // $t7 := borrow_global<Libra::Preburn<#0>>(sender)
    call $t7 := $BorrowGlobal(sender, $Libra_Preburn_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 19614);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($Dereference($t7));

    // UnpackRef($t7)
    call $Libra_Preburn_before_update_inv($tv0, $Dereference($t7));

    // PackRef($t7)
    call $Libra_Preburn_after_update_inv($tv0, $Dereference($t7));

    // $t11 := read_ref($t7)
    call $tmp := $ReadRef($t7);
    assume $Libra_Preburn_is_well_formed($tmp);
    $t11 := $tmp;

    // $t11 := Libra::preburn_with_resource<#0>($t10, $t11, sender)
    call $t11 := $Libra_preburn_with_resource($tv0, $t10, $t11, sender);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 16258);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($t11);


    // write_ref($t7, $t11)
    call $t7 := $WriteRef($t7, $t11);

    // Libra::Preburn <- $t7
    call $WritebackToGlobal($t7);

    // UnpackRef($t7)
    call $Libra_Preburn_before_update_inv($tv0, $Dereference($t7));

    // PackRef($t7)
    call $Libra_Preburn_after_update_inv($tv0, $Dereference($t7));

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Libra_preburn_to_$direct_inter($tv0: $TypeValue, account: $Value, coin: $Value) returns ()
{
    assume is#$Address(account);

    assume $Libra_Libra_is_well_formed(coin);

    call $Libra_preburn_to_$def($tv0, account, coin);
}


procedure {:inline 1} $Libra_preburn_to_$direct_intra($tv0: $TypeValue, account: $Value, coin: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(account);

    assume $Libra_Libra_is_well_formed(coin);

    call $Libra_preburn_to_$def($tv0, account, coin);
}


procedure {:inline 1} $Libra_preburn_to($tv0: $TypeValue, account: $Value, coin: $Value) returns ()
{
    assume is#$Address(account);

    assume $Libra_Libra_is_well_formed(coin);

    call $Libra_preburn_to_$def($tv0, account, coin);
}


procedure {:inline 1} $Libra_preburn_value_$def($tv0: $TypeValue) returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 26446);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := get_global<Libra::CurrencyInfo<#0>>($t0)
    call $tmp := $GetGlobal($t0, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 26393);
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
    if (true) { assume $DebugTrackLocal(9, 26393, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_preburn_value_$direct_inter($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_preburn_value_$def($tv0);
}


procedure {:inline 1} $Libra_preburn_value_$direct_intra($tv0: $TypeValue) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    call $ret0 := $Libra_preburn_value_$def($tv0);
}


procedure {:inline 1} $Libra_preburn_value($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_preburn_value_$def($tv0);
}


procedure {:inline 1} $Libra_preburn_with_resource_$def($tv0: $TypeValue, coin: $Value, preburn: $Value, preburn_address: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var coin_value: $Value; // $IntegerType()
    var currency_code: $Value; // $Vector_type_value($IntegerType())
    var info: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $Libra_Libra_type_value($tv0)
    var $t9: $Value; // $IntegerType()
    var $t10: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t11: $Value; // $Libra_Libra_type_value($tv0)
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t17: $Value; // $IntegerType()
    var $t18: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t19: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t20: $Value; // $Libra_Libra_type_value($tv0)
    var $t21: $Value; // $Vector_type_value($IntegerType())
    var $t22: $Value; // $AddressType()
    var $t23: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t24: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $IntegerType()
    var $t27: $Value; // $IntegerType()
    var $t28: $Value; // $IntegerType()
    var $t29: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t30: $Reference; // ReferenceType($IntegerType())
    var $t31: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t32: $Value; // $BooleanType()
    var $t33: $Value; // $BooleanType()
    var $t34: $Value; // $BooleanType()
    var $t35: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t36: $Reference; // ReferenceType($Event_EventHandle_type_value($Libra_PreburnEvent_type_value()))
    var $t37: $Value; // $IntegerType()
    var $t38: $Value; // $Vector_type_value($IntegerType())
    var $t39: $Value; // $AddressType()
    var $t40: $Value; // $Libra_PreburnEvent_type_value()
    var $t41: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t42: $Value; // $Libra_Libra_type_value($tv0)
    var $t43: $Value; // $Libra_Preburn_type_value($tv0)
    var $t44: $Reference; // ReferenceType($Libra_Preburn_type_value($tv0))
    var $t45: $Value; // $AddressType()
    var $t46: $Value; // $Event_EventHandle_type_value($Libra_PreburnEvent_type_value())
    var $t47: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 16247, 0, coin); }
    if (true) { assume $DebugTrackLocal(9, 16247, 1, preburn); }
    if (true) { assume $DebugTrackLocal(9, 16247, 2, preburn_address); }

    // bytecode translation starts here
    // $t42 := move(coin)
    call $tmp := $CopyOrMoveValue(coin);
    $t42 := $tmp;

    // $t43 := move(preburn)
    call $tmp := $CopyOrMoveValue(preburn);
    $t43 := $tmp;

    // $t45 := move(preburn_address)
    call $tmp := $CopyOrMoveValue(preburn_address);
    $t45 := $tmp;

    // $t44 := borrow_local($t43)
    call $t44 := $BorrowLoc(43, $t43);
    assume $Libra_Preburn_is_well_formed($Dereference($t44));

    // UnpackRef($t44)
    call $Libra_Preburn_before_update_inv($tv0, $Dereference($t44));

    // $t8 := copy($t42)
    call $tmp := $CopyOrMoveValue($t42);
    $t8 := $tmp;

    // $t9 := Libra::value<#0>($t8)
    call $t9 := $Libra_value($tv0, $t8);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 26961);
      goto Abort;
    }
    assume $IsValidU64($t9);


    // coin_value := $t9
    call $tmp := $CopyOrMoveValue($t9);
    coin_value := $tmp;
    if (true) { assume $DebugTrackLocal(9, 16439, 3, $tmp); }

    // $t10 := copy($t44)
    call $t10 := $CopyOrMoveRef($t44);

    // $t11 := get_field<Libra::Preburn<#0>>.to_burn($t10)
    call $tmp := $GetFieldFromReference($t10, $Libra_Preburn_to_burn);
    assume $Libra_Libra_is_well_formed($tmp);
    $t11 := $tmp;

    // Reference($t44) <- $t10
    call $t44 := $WritebackToReference($t10, $t44);

    // $t12 := Libra::value<#0>($t11)
    call $t12 := $Libra_value($tv0, $t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 26961);
      goto Abort;
    }
    assume $IsValidU64($t12);


    // $t13 := 0
    $tmp := $Integer(0);
    $t13 := $tmp;

    // $t14 := ==($t12, $t13)
    $tmp := $Boolean($IsEqual($t12, $t13));
    $t14 := $tmp;

    // $t6 := $t14
    call $tmp := $CopyOrMoveValue($t14);
    $t6 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 16511, 6, $tmp); }

    // if ($t6) goto L0 else goto L1
    $tmp := $t6;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t16 := move($t44)
    call $t16 := $CopyOrMoveRef($t44);

    // destroy($t16)

    // LocalRoot($t43) <- $t16
    call $t43 := $WritebackToValue($t16, 43, $t43);

    // PackRef($t16)
    call $Libra_Preburn_after_update_inv($tv0, $Dereference($t16));

    // $t17 := 6
    $tmp := $Integer(6);
    $t17 := $tmp;

    // abort($t17)
    if (true) { assume $DebugTrackAbort(9, 16511); }
    goto Abort;

    // L0:
L0:

    // $t18 := move($t44)
    call $t18 := $CopyOrMoveRef($t44);

    // $t19 := borrow_field<Libra::Preburn<#0>>.to_burn($t18)
    call $t19 := $BorrowField($t18, $Libra_Preburn_to_burn);
    assume $Libra_Libra_is_well_formed_types($Dereference($t19));

    // LocalRoot($t43) <- $t18
    call $t43 := $WritebackToValue($t18, 43, $t43);

    // UnpackRef($t19)
    call $Libra_Libra_before_update_inv($tv0, $Dereference($t19));

    // PackRef($t19)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t19));

    // $t47 := read_ref($t19)
    call $tmp := $ReadRef($t19);
    assume $Libra_Libra_is_well_formed($tmp);
    $t47 := $tmp;

    // $t47 := Libra::deposit<#0>($t47, $t42)
    call $t47 := $Libra_deposit($tv0, $t47, $t42);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 29448);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t47);


    // write_ref($t19, $t47)
    call $t19 := $WriteRef($t19, $t47);
    if (true) { assume $DebugTrackLocal(9, 16247, 5, $Dereference(info)); }

    // LocalRoot($t43) <- $t19
    call $t43 := $WritebackToValue($t19, 43, $t43);

    // Reference($t18) <- $t19
    call $t18 := $WritebackToReference($t19, $t18);

    // UnpackRef($t19)
    call $Libra_Libra_before_update_inv($tv0, $Dereference($t19));

    // PackRef($t18)
    call $Libra_Preburn_after_update_inv($tv0, $Dereference($t18));

    // PackRef($t19)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t19));

    // $t21 := Libra::currency_code<#0>()
    call $t21 := $Libra_currency_code($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 35732);
      goto Abort;
    }
    assume $Vector_is_well_formed($t21) && (forall $$0: int :: {$select_vector($t21,$$0)} $$0 >= 0 && $$0 < $vlen($t21) ==> $IsValidU8($select_vector($t21,$$0)));


    // currency_code := $t21
    call $tmp := $CopyOrMoveValue($t21);
    currency_code := $tmp;
    if (true) { assume $DebugTrackLocal(9, 16609, 4, $tmp); }

    // $t22 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t22 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 16728);
      goto Abort;
    }
    assume is#$Address($t22);


    // $t23 := borrow_global<Libra::CurrencyInfo<#0>>($t22)
    call $t23 := $BorrowGlobal($t22, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 16671);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($Dereference($t23));

    // UnpackRef($t23)

    // info := $t23
    call info := $CopyOrMoveRef($t23);
    if (true) { assume $DebugTrackLocal(9, 16664, 5, $Dereference(info)); }

    // $t24 := copy(info)
    call $t24 := $CopyOrMoveRef(info);

    // $t25 := get_field<Libra::CurrencyInfo<#0>>.preburn_value($t24)
    call $tmp := $GetFieldFromReference($t24, $Libra_CurrencyInfo_preburn_value);
    assume $IsValidU64($tmp);
    $t25 := $tmp;

    // Reference(info) <- $t24
    call info := $WritebackToReference($t24, info);

    // $t26 := move($t25)
    call $tmp := $CopyOrMoveValue($t25);
    $t26 := $tmp;

    // $t28 := +($t26, coin_value)
    call $tmp := $AddU64($t26, coin_value);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 16802);
      goto Abort;
    }
    $t28 := $tmp;

    // $t29 := copy(info)
    call $t29 := $CopyOrMoveRef(info);

    // $t30 := borrow_field<Libra::CurrencyInfo<#0>>.preburn_value($t29)
    call $t30 := $BorrowField($t29, $Libra_CurrencyInfo_preburn_value);
    assume $IsValidU64($Dereference($t30));

    // Reference(info) <- $t29
    call info := $WritebackToReference($t29, info);

    // UnpackRef($t30)

    // write_ref($t30, $t28)
    call $t30 := $WriteRef($t30, $t28);
    if (true) { assume $DebugTrackLocal(9, 16762, 5, $Dereference(info)); }

    // Reference(info) <- $t30
    call info := $WritebackToReference($t30, info);

    // Reference($t29) <- $t30
    call $t29 := $WritebackToReference($t30, $t29);

    // PackRef($t30)

    // $t31 := copy(info)
    call $t31 := $CopyOrMoveRef(info);

    // $t32 := get_field<Libra::CurrencyInfo<#0>>.is_synthetic($t31)
    call $tmp := $GetFieldFromReference($t31, $Libra_CurrencyInfo_is_synthetic);
    assume is#$Boolean($tmp);
    $t32 := $tmp;

    // Reference(info) <- $t31
    call info := $WritebackToReference($t31, info);

    // $t33 := move($t32)
    call $tmp := $CopyOrMoveValue($t32);
    $t33 := $tmp;

    // $t34 := !($t33)
    call $tmp := $Not($t33);
    $t34 := $tmp;

    // if ($t34) goto L2 else goto L3
    $tmp := $t34;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // goto L4
    goto L4;

    // L2:
L2:

    // $t35 := move(info)
    call $t35 := $CopyOrMoveRef(info);

    // $t36 := borrow_field<Libra::CurrencyInfo<#0>>.preburn_events($t35)
    call $t36 := $BorrowField($t35, $Libra_CurrencyInfo_preburn_events);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t36));

    // Libra::CurrencyInfo <- $t35
    call $WritebackToGlobal($t35);

    // UnpackRef($t36)

    // $t40 := pack Libra::PreburnEvent(coin_value, currency_code, $t45)
    call $tmp := $Libra_PreburnEvent_pack(0, 0, 0, coin_value, currency_code, $t45);
    $t40 := $tmp;

    // PackRef($t36)

    // $t46 := read_ref($t36)
    call $tmp := $ReadRef($t36);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t46 := $tmp;

    // $t46 := Event::emit_event<Libra::PreburnEvent>($t46, $t40)
    call $t46 := $Event_emit_event($Libra_PreburnEvent_type_value(), $t46, $t40);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 16931);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t46);


    // write_ref($t36, $t46)
    call $t36 := $WriteRef($t36, $t46);
    if (true) { assume $DebugTrackLocal(9, 16247, 5, $Dereference(info)); }

    // Libra::CurrencyInfo <- $t36
    call $WritebackToGlobal($t36);

    // Reference($t35) <- $t36
    call $t35 := $WritebackToReference($t36, $t35);

    // UnpackRef($t36)

    // PackRef($t35)

    // PackRef($t36)

    // goto L5
    goto L5;

    // L4:
L4:

    // $t41 := move(info)
    call $t41 := $CopyOrMoveRef(info);

    // destroy($t41)

    // Libra::CurrencyInfo <- $t41
    call $WritebackToGlobal($t41);

    // PackRef($t41)

    // goto L5
    goto L5;

    // L5:
L5:

    // return $t43
    $ret0 := $t43;
    if (true) { assume $DebugTrackLocal(9, 17169, 48, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_preburn_with_resource_$direct_inter($tv0: $TypeValue, coin: $Value, preburn: $Value, preburn_address: $Value) returns ($ret0: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    assume $Libra_Preburn_is_well_formed(preburn);

    assume is#$Address(preburn_address);

    call $ret0 := $Libra_preburn_with_resource_$def($tv0, coin, preburn, preburn_address);
}


procedure {:inline 1} $Libra_preburn_with_resource_$direct_intra($tv0: $TypeValue, coin: $Value, preburn: $Value, preburn_address: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume $Libra_Libra_is_well_formed(coin);

    assume $Libra_Preburn_is_well_formed(preburn);

    assume is#$Address(preburn_address);

    call $ret0 := $Libra_preburn_with_resource_$def($tv0, coin, preburn, preburn_address);
}


procedure {:inline 1} $Libra_preburn_with_resource($tv0: $TypeValue, coin: $Value, preburn: $Value, preburn_address: $Value) returns ($ret0: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    assume $Libra_Preburn_is_well_formed(preburn);

    assume is#$Address(preburn_address);

    call $ret0 := $Libra_preburn_with_resource_$def($tv0, coin, preburn, preburn_address);
}


procedure {:inline 1} $Libra_publish_burn_capability_$def($tv0: $TypeValue, account: $Value, cap: $Value, tc_account: $Value) returns ()
{
    // declare local variables
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t14: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 11668, 0, account); }
    if (true) { assume $DebugTrackLocal(9, 11668, 1, cap); }
    if (true) { assume $DebugTrackLocal(9, 11668, 2, tc_account); }

    // bytecode translation starts here
    // $t12 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t12 := $tmp;

    // $t13 := move(cap)
    call $tmp := $CopyOrMoveValue(cap);
    $t13 := $tmp;

    // $t14 := move(tc_account)
    call $tmp := $CopyOrMoveValue(tc_account);
    $t14 := $tmp;

    // $t5 := move($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t5 := $tmp;

    // $t6 := Roles::has_treasury_compliance_role($t5)
    call $t6 := $Roles_has_treasury_compliance_role($t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 11838);
      goto Abort;
    }
    assume is#$Boolean($t6);


    // $t3 := $t6
    call $tmp := $CopyOrMoveValue($t6);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 11824, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t8 := move($t12)
    call $tmp := $CopyOrMoveValue($t12);
    $t8 := $tmp;

    // destroy($t8)

    // $t9 := 2
    $tmp := $Integer(2);
    $t9 := $tmp;

    // abort($t9)
    if (true) { assume $DebugTrackAbort(9, 11824); }
    goto Abort;

    // L0:
L0:

    // Libra::assert_is_currency<#0>()
    call $Libra_assert_is_currency($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 38329);
      goto Abort;
    }

    // $t10 := move($t12)
    call $tmp := $CopyOrMoveValue($t12);
    $t10 := $tmp;

    // move_to<Libra::BurnCapability<#0>>($t13, $t10)
    call $MoveTo($Libra_BurnCapability_type_value($tv0), $t13, $t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 11955);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Libra_publish_burn_capability_$direct_inter($tv0: $TypeValue, account: $Value, cap: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(account);

    assume $Libra_BurnCapability_is_well_formed(cap);

    assume is#$Address(tc_account);

    call $Libra_publish_burn_capability_$def($tv0, account, cap, tc_account);
}


procedure {:inline 1} $Libra_publish_burn_capability_$direct_intra($tv0: $TypeValue, account: $Value, cap: $Value, tc_account: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(account);

    assume $Libra_BurnCapability_is_well_formed(cap);

    assume is#$Address(tc_account);

    call $Libra_publish_burn_capability_$def($tv0, account, cap, tc_account);
}


procedure {:inline 1} $Libra_publish_burn_capability($tv0: $TypeValue, account: $Value, cap: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(account);

    assume $Libra_BurnCapability_is_well_formed(cap);

    assume is#$Address(tc_account);

    call $Libra_publish_burn_capability_$def($tv0, account, cap, tc_account);
}


procedure {:inline 1} $Libra_publish_mint_capability_$def($tv0: $TypeValue, publish_account: $Value, cap: $Value, tc_account: $Value) returns ()
{
    // declare local variables
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $Libra_MintCapability_type_value($tv0)
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $Libra_MintCapability_type_value($tv0)
    var $t14: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 11119, 0, publish_account); }
    if (true) { assume $DebugTrackLocal(9, 11119, 1, cap); }
    if (true) { assume $DebugTrackLocal(9, 11119, 2, tc_account); }

    // bytecode translation starts here
    // $t12 := move(publish_account)
    call $tmp := $CopyOrMoveValue(publish_account);
    $t12 := $tmp;

    // $t13 := move(cap)
    call $tmp := $CopyOrMoveValue(cap);
    $t13 := $tmp;

    // $t14 := move(tc_account)
    call $tmp := $CopyOrMoveValue(tc_account);
    $t14 := $tmp;

    // $t5 := move($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t5 := $tmp;

    // $t6 := Roles::has_treasury_compliance_role($t5)
    call $t6 := $Roles_has_treasury_compliance_role($t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 11297);
      goto Abort;
    }
    assume is#$Boolean($t6);


    // $t3 := $t6
    call $tmp := $CopyOrMoveValue($t6);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 11283, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t8 := move($t12)
    call $tmp := $CopyOrMoveValue($t12);
    $t8 := $tmp;

    // destroy($t8)

    // $t9 := 2
    $tmp := $Integer(2);
    $t9 := $tmp;

    // abort($t9)
    if (true) { assume $DebugTrackAbort(9, 11283); }
    goto Abort;

    // L0:
L0:

    // Libra::assert_is_currency<#0>()
    call $Libra_assert_is_currency($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 38329);
      goto Abort;
    }

    // $t10 := move($t12)
    call $tmp := $CopyOrMoveValue($t12);
    $t10 := $tmp;

    // move_to<Libra::MintCapability<#0>>($t13, $t10)
    call $MoveTo($Libra_MintCapability_type_value($tv0), $t13, $t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 11414);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Libra_publish_mint_capability_$direct_inter($tv0: $TypeValue, publish_account: $Value, cap: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(publish_account);

    assume $Libra_MintCapability_is_well_formed(cap);

    assume is#$Address(tc_account);

    call $Libra_publish_mint_capability_$def($tv0, publish_account, cap, tc_account);
}


procedure {:inline 1} $Libra_publish_mint_capability_$direct_intra($tv0: $TypeValue, publish_account: $Value, cap: $Value, tc_account: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(publish_account);

    assume $Libra_MintCapability_is_well_formed(cap);

    assume is#$Address(tc_account);

    call $Libra_publish_mint_capability_$def($tv0, publish_account, cap, tc_account);
}


procedure {:inline 1} $Libra_publish_mint_capability($tv0: $TypeValue, publish_account: $Value, cap: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(publish_account);

    assume $Libra_MintCapability_is_well_formed(cap);

    assume is#$Address(tc_account);

    call $Libra_publish_mint_capability_$def($tv0, publish_account, cap, tc_account);
}


procedure {:inline 1} $Libra_publish_preburn_to_account_$def($tv0: $TypeValue, account: $Value, tc_account: $Value) returns ()
{
    // declare local variables
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $Libra_Preburn_type_value($tv0)
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 18777, 0, account); }
    if (true) { assume $DebugTrackLocal(9, 18777, 1, tc_account); }

    // bytecode translation starts here
    // $t13 := move(account)
    call $tmp := $CopyOrMoveValue(account);
    $t13 := $tmp;

    // $t14 := move(tc_account)
    call $tmp := $CopyOrMoveValue(tc_account);
    $t14 := $tmp;

    // $t4 := Libra::is_synthetic_currency<#0>()
    call $t4 := $Libra_is_synthetic_currency($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 34742);
      goto Abort;
    }
    assume is#$Boolean($t4);


    // $t5 := !($t4)
    call $tmp := $Not($t4);
    $t5 := $tmp;

    // $t2 := $t5
    call $tmp := $CopyOrMoveValue($t5);
    $t2 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 18918, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t7 := move($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t7 := $tmp;

    // destroy($t7)

    // $t8 := move($t13)
    call $tmp := $CopyOrMoveValue($t13);
    $t8 := $tmp;

    // destroy($t8)

    // $t9 := 4
    $tmp := $Integer(4);
    $t9 := $tmp;

    // abort($t9)
    if (true) { assume $DebugTrackAbort(9, 18918); }
    goto Abort;

    // L0:
L0:

    // $t10 := move($t13)
    call $tmp := $CopyOrMoveValue($t13);
    $t10 := $tmp;

    // $t11 := move($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t11 := $tmp;

    // $t12 := Libra::create_preburn<#0>($t11)
    call $t12 := $Libra_create_preburn($tv0, $t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 18184);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($t12);


    // move_to<Libra::Preburn<#0>>($t12, $t10)
    call $MoveTo($Libra_Preburn_type_value($tv0), $t12, $t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 18994);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Libra_publish_preburn_to_account_$direct_inter($tv0: $TypeValue, account: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(tc_account);

    call $Libra_publish_preburn_to_account_$def($tv0, account, tc_account);
}


procedure {:inline 1} $Libra_publish_preburn_to_account_$direct_intra($tv0: $TypeValue, account: $Value, tc_account: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(account);

    assume is#$Address(tc_account);

    call $Libra_publish_preburn_to_account_$def($tv0, account, tc_account);
}


procedure {:inline 1} $Libra_publish_preburn_to_account($tv0: $TypeValue, account: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(tc_account);

    call $Libra_publish_preburn_to_account_$def($tv0, account, tc_account);
}


procedure {:inline 1} $Libra_register_currency_$def($tv0: $TypeValue, lr_account: $Value, to_lbr_exchange_rate: $Value, is_synthetic: $Value, scaling_factor: $Value, fractional_part: $Value, currency_code: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    // declare local variables
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $BooleanType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $AddressType()
    var $t14: $Value; // $IntegerType()
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $BooleanType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $AddressType()
    var $t21: $Value; // $IntegerType()
    var $t22: $Value; // $AddressType()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $IntegerType()
    var $t25: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t26: $Value; // $BooleanType()
    var $t27: $Value; // $IntegerType()
    var $t28: $Value; // $IntegerType()
    var $t29: $Value; // $Vector_type_value($IntegerType())
    var $t30: $Value; // $BooleanType()
    var $t31: $Value; // $AddressType()
    var $t32: $Value; // $Event_EventHandle_type_value($Libra_MintEvent_type_value())
    var $t33: $Value; // $AddressType()
    var $t34: $Value; // $Event_EventHandle_type_value($Libra_BurnEvent_type_value())
    var $t35: $Value; // $AddressType()
    var $t36: $Value; // $Event_EventHandle_type_value($Libra_PreburnEvent_type_value())
    var $t37: $Value; // $AddressType()
    var $t38: $Value; // $Event_EventHandle_type_value($Libra_CancelBurnEvent_type_value())
    var $t39: $Value; // $AddressType()
    var $t40: $Value; // $Event_EventHandle_type_value($Libra_ToLBRExchangeRateUpdateEvent_type_value())
    var $t41: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t42: $Value; // $Vector_type_value($IntegerType())
    var $t43: $Value; // $AddressType()
    var $t44: $Value; // $Libra_CurrencyRegistrationCapability_type_value()
    var $t45: $Value; // $RegisteredCurrencies_RegistrationCapability_type_value()
    var $t46: $Value; // $BooleanType()
    var $t47: $Value; // $Libra_MintCapability_type_value($tv0)
    var $t48: $Value; // $BooleanType()
    var $t49: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t50: $Value; // $AddressType()
    var $t51: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t52: $Value; // $BooleanType()
    var $t53: $Value; // $IntegerType()
    var $t54: $Value; // $IntegerType()
    var $t55: $Value; // $Vector_type_value($IntegerType())
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 31053, 0, lr_account); }
    if (true) { assume $DebugTrackLocal(9, 31053, 1, to_lbr_exchange_rate); }
    if (true) { assume $DebugTrackLocal(9, 31053, 2, is_synthetic); }
    if (true) { assume $DebugTrackLocal(9, 31053, 3, scaling_factor); }
    if (true) { assume $DebugTrackLocal(9, 31053, 4, fractional_part); }
    if (true) { assume $DebugTrackLocal(9, 31053, 5, currency_code); }

    // bytecode translation starts here
    // $t50 := move(lr_account)
    call $tmp := $CopyOrMoveValue(lr_account);
    $t50 := $tmp;

    // $t51 := move(to_lbr_exchange_rate)
    call $tmp := $CopyOrMoveValue(to_lbr_exchange_rate);
    $t51 := $tmp;

    // $t52 := move(is_synthetic)
    call $tmp := $CopyOrMoveValue(is_synthetic);
    $t52 := $tmp;

    // $t53 := move(scaling_factor)
    call $tmp := $CopyOrMoveValue(scaling_factor);
    $t53 := $tmp;

    // $t54 := move(fractional_part)
    call $tmp := $CopyOrMoveValue(fractional_part);
    $t54 := $tmp;

    // $t55 := move(currency_code)
    call $tmp := $CopyOrMoveValue(currency_code);
    $t55 := $tmp;

    // $t10 := copy($t50)
    call $tmp := $CopyOrMoveValue($t50);
    $t10 := $tmp;

    // $t11 := Roles::has_register_new_currency_privilege($t10)
    call $t11 := $Roles_has_register_new_currency_privilege($t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 31416);
      goto Abort;
    }
    assume is#$Boolean($t11);


    // $t6 := $t11
    call $tmp := $CopyOrMoveValue($t11);
    $t6 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 31402, 6, $tmp); }

    // if ($t6) goto L0 else goto L1
    $tmp := $t6;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t13 := move($t50)
    call $tmp := $CopyOrMoveValue($t50);
    $t13 := $tmp;

    // destroy($t13)

    // $t14 := 7
    $tmp := $Integer(7);
    $t14 := $tmp;

    // abort($t14)
    if (true) { assume $DebugTrackAbort(9, 31402); }
    goto Abort;

    // L0:
L0:

    // $t15 := copy($t50)
    call $tmp := $CopyOrMoveValue($t50);
    $t15 := $tmp;

    // $t16 := Signer::address_of($t15)
    call $t16 := $Signer_address_of($t15);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 31624);
      goto Abort;
    }
    assume is#$Address($t16);


    // $t17 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t17 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 31665);
      goto Abort;
    }
    assume is#$Address($t17);


    // $t18 := ==($t16, $t17)
    $tmp := $Boolean($IsEqual($t16, $t17));
    $t18 := $tmp;

    // $t8 := $t18
    call $tmp := $CopyOrMoveValue($t18);
    $t8 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 31596, 8, $tmp); }

    // if ($t8) goto L2 else goto L3
    $tmp := $t8;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t20 := move($t50)
    call $tmp := $CopyOrMoveValue($t50);
    $t20 := $tmp;

    // destroy($t20)

    // $t21 := 1
    $tmp := $Integer(1);
    $t21 := $tmp;

    // abort($t21)
    if (true) { assume $DebugTrackAbort(9, 31596); }
    goto Abort;

    // L2:
L2:

    // $t22 := copy($t50)
    call $tmp := $CopyOrMoveValue($t50);
    $t22 := $tmp;

    // $t23 := 0
    $tmp := $Integer(0);
    $t23 := $tmp;

    // $t24 := 0
    $tmp := $Integer(0);
    $t24 := $tmp;

    // $t30 := true
    $tmp := $Boolean(true);
    $t30 := $tmp;

    // $t31 := copy($t50)
    call $tmp := $CopyOrMoveValue($t50);
    $t31 := $tmp;

    // $t32 := Event::new_event_handle<Libra::MintEvent>($t31)
    call $t32 := $Event_new_event_handle($Libra_MintEvent_type_value(), $t31);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 32076);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t32);


    // $t33 := copy($t50)
    call $tmp := $CopyOrMoveValue($t50);
    $t33 := $tmp;

    // $t34 := Event::new_event_handle<Libra::BurnEvent>($t33)
    call $t34 := $Event_new_event_handle($Libra_BurnEvent_type_value(), $t33);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 32149);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t34);


    // $t35 := copy($t50)
    call $tmp := $CopyOrMoveValue($t50);
    $t35 := $tmp;

    // $t36 := Event::new_event_handle<Libra::PreburnEvent>($t35)
    call $t36 := $Event_new_event_handle($Libra_PreburnEvent_type_value(), $t35);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 32225);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t36);


    // $t37 := copy($t50)
    call $tmp := $CopyOrMoveValue($t50);
    $t37 := $tmp;

    // $t38 := Event::new_event_handle<Libra::CancelBurnEvent>($t37)
    call $t38 := $Event_new_event_handle($Libra_CancelBurnEvent_type_value(), $t37);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 32308);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t38);


    // $t39 := move($t50)
    call $tmp := $CopyOrMoveValue($t50);
    $t39 := $tmp;

    // $t40 := Event::new_event_handle<Libra::ToLBRExchangeRateUpdateEvent>($t39)
    call $t40 := $Event_new_event_handle($Libra_ToLBRExchangeRateUpdateEvent_type_value(), $t39);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 32403);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t40);


    // $t41 := pack Libra::CurrencyInfo<#0>($t23, $t24, $t51, $t52, $t53, $t54, $t55, $t30, $t32, $t34, $t36, $t38, $t40)
    call $tmp := $Libra_CurrencyInfo_pack(0, 0, 0, $tv0, $t23, $t24, $t51, $t52, $t53, $t54, $t55, $t30, $t32, $t34, $t36, $t38, $t40);
    $t41 := $tmp;

    // move_to<Libra::CurrencyInfo<#0>>($t41, $t22)
    call $MoveTo($Libra_CurrencyInfo_type_value($tv0), $t41, $t22);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 31749);
      goto Abort;
    }

    // $t43 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t43 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 32624);
      goto Abort;
    }
    assume is#$Address($t43);


    // $t44 := get_global<Libra::CurrencyRegistrationCapability>($t43)
    call $tmp := $GetGlobal($t43, $Libra_CurrencyRegistrationCapability_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 32563);
      goto Abort;
    }
    assume $Libra_CurrencyRegistrationCapability_is_well_formed($tmp);
    $t44 := $tmp;

    // $t45 := get_field<Libra::CurrencyRegistrationCapability>.cap($t44)
    call $tmp := $GetFieldFromValue($t44, $Libra_CurrencyRegistrationCapability_cap);
    assume $RegisteredCurrencies_RegistrationCapability_is_well_formed($tmp);
    $t45 := $tmp;

    // RegisteredCurrencies::add_currency_code($t55, $t45)
    call $RegisteredCurrencies_add_currency_code($t55, $t45);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 32504);
      goto Abort;
    }

    // $t46 := false
    $tmp := $Boolean(false);
    $t46 := $tmp;

    // $t47 := pack Libra::MintCapability<#0>($t46)
    call $tmp := $Libra_MintCapability_pack(0, 0, 0, $tv0, $t46);
    $t47 := $tmp;

    // $t48 := false
    $tmp := $Boolean(false);
    $t48 := $tmp;

    // $t49 := pack Libra::BurnCapability<#0>($t48)
    call $tmp := $Libra_BurnCapability_pack(0, 0, 0, $tv0, $t48);
    $t49 := $tmp;

    // return ($t47, $t49)
    $ret0 := $t47;
    if (true) { assume $DebugTrackLocal(9, 32669, 56, $ret0); }
    $ret1 := $t49;
    if (true) { assume $DebugTrackLocal(9, 32669, 57, $ret1); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}

procedure {:inline 1} $Libra_register_currency_$direct_inter($tv0: $TypeValue, lr_account: $Value, to_lbr_exchange_rate: $Value, is_synthetic: $Value, scaling_factor: $Value, fractional_part: $Value, currency_code: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    assume is#$Address(lr_account);

    assume $FixedPoint32_FixedPoint32_is_well_formed(to_lbr_exchange_rate);

    assume is#$Boolean(is_synthetic);

    assume $IsValidU64(scaling_factor);

    assume $IsValidU64(fractional_part);

    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));

    call $ret0, $ret1 := $Libra_register_currency_$def($tv0, lr_account, to_lbr_exchange_rate, is_synthetic, scaling_factor, fractional_part, currency_code);
}


procedure {:inline 1} $Libra_register_currency_$direct_intra($tv0: $TypeValue, lr_account: $Value, to_lbr_exchange_rate: $Value, is_synthetic: $Value, scaling_factor: $Value, fractional_part: $Value, currency_code: $Value) returns ($ret0: $Value, $ret1: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(lr_account);

    assume $FixedPoint32_FixedPoint32_is_well_formed(to_lbr_exchange_rate);

    assume is#$Boolean(is_synthetic);

    assume $IsValidU64(scaling_factor);

    assume $IsValidU64(fractional_part);

    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));

    call $ret0, $ret1 := $Libra_register_currency_$def($tv0, lr_account, to_lbr_exchange_rate, is_synthetic, scaling_factor, fractional_part, currency_code);
}


procedure {:inline 1} $Libra_register_currency($tv0: $TypeValue, lr_account: $Value, to_lbr_exchange_rate: $Value, is_synthetic: $Value, scaling_factor: $Value, fractional_part: $Value, currency_code: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    assume is#$Address(lr_account);

    assume $FixedPoint32_FixedPoint32_is_well_formed(to_lbr_exchange_rate);

    assume is#$Boolean(is_synthetic);

    assume $IsValidU64(scaling_factor);

    assume $IsValidU64(fractional_part);

    assume $Vector_is_well_formed(currency_code) && (forall $$0: int :: {$select_vector(currency_code,$$0)} $$0 >= 0 && $$0 < $vlen(currency_code) ==> $IsValidU8($select_vector(currency_code,$$0)));

    call $ret0, $ret1 := $Libra_register_currency_$def($tv0, lr_account, to_lbr_exchange_rate, is_synthetic, scaling_factor, fractional_part, currency_code);
}


procedure {:inline 1} $Libra_scaling_factor_$def($tv0: $TypeValue) returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $Libra_CurrencyInfo_type_value($tv0)
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 35234);
      goto Abort;
    }
    assume is#$Address($t0);


    // $t1 := get_global<Libra::CurrencyInfo<#0>>($t0)
    call $tmp := $GetGlobal($t0, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 35181);
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
    if (true) { assume $DebugTrackLocal(9, 35181, 4, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_scaling_factor_$direct_inter($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_scaling_factor_$def($tv0);
}


procedure {:inline 1} $Libra_scaling_factor_$direct_intra($tv0: $TypeValue) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    call $ret0 := $Libra_scaling_factor_$def($tv0);
}


procedure {:inline 1} $Libra_scaling_factor($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_scaling_factor_$def($tv0);
}


procedure {:inline 1} $Libra_remove_burn_capability_$def($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $Libra_BurnCapability_type_value($tv0)
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 25880, 0, account); }

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
      assume $DebugTrackAbort(9, 26050);
      goto Abort;
    }
    assume is#$Address($t2);


    // $t3 := move_from<Libra::BurnCapability<#0>>($t2)
    call $tmp := $MoveFrom($t2, $Libra_BurnCapability_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 26006);
      goto Abort;
    }
    assume $Libra_BurnCapability_is_well_formed($tmp);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(9, 26006, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_remove_burn_capability_$direct_inter($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Libra_remove_burn_capability_$def($tv0, account);
}


procedure {:inline 1} $Libra_remove_burn_capability_$direct_intra($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(account);

    call $ret0 := $Libra_remove_burn_capability_$def($tv0, account);
}


procedure {:inline 1} $Libra_remove_burn_capability($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Libra_remove_burn_capability_$def($tv0, account);
}


procedure {:inline 1} $Libra_remove_mint_capability_$def($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $Libra_MintCapability_type_value($tv0)
    var $t4: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 25460, 0, account); }

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
      assume $DebugTrackAbort(9, 25630);
      goto Abort;
    }
    assume is#$Address($t2);


    // $t3 := move_from<Libra::MintCapability<#0>>($t2)
    call $tmp := $MoveFrom($t2, $Libra_MintCapability_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 25586);
      goto Abort;
    }
    assume $Libra_MintCapability_is_well_formed($tmp);
    $t3 := $tmp;

    // return $t3
    $ret0 := $t3;
    if (true) { assume $DebugTrackLocal(9, 25586, 5, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_remove_mint_capability_$direct_inter($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Libra_remove_mint_capability_$def($tv0, account);
}


procedure {:inline 1} $Libra_remove_mint_capability_$direct_intra($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(account);

    call $ret0 := $Libra_remove_mint_capability_$def($tv0, account);
}


procedure {:inline 1} $Libra_remove_mint_capability($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Libra_remove_mint_capability_$def($tv0, account);
}


procedure {:inline 1} $Libra_split_$def($tv0: $TypeValue, coin: $Value, amount: $Value) returns ($ret0: $Value, $ret1: $Value)
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 27275, 0, coin); }
    if (true) { assume $DebugTrackLocal(9, 27275, 1, amount); }

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
      assume $DebugTrackAbort(9, 28003);
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
    if (true) { assume $DebugTrackLocal(9, 27388, 2, $tmp); }

    // return ($t8, other)
    $ret0 := $t8;
    if (true) { assume $DebugTrackLocal(9, 27433, 11, $ret0); }
    $ret1 := other;
    if (true) { assume $DebugTrackLocal(9, 27433, 12, $ret1); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}

procedure {:inline 1} $Libra_split_$direct_inter($tv0: $TypeValue, coin: $Value, amount: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    assume $IsValidU64(amount);

    call $ret0, $ret1 := $Libra_split_$def($tv0, coin, amount);
}


procedure {:inline 1} $Libra_split_$direct_intra($tv0: $TypeValue, coin: $Value, amount: $Value) returns ($ret0: $Value, $ret1: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume $Libra_Libra_is_well_formed(coin);

    assume $IsValidU64(amount);

    call $ret0, $ret1 := $Libra_split_$def($tv0, coin, amount);
}


procedure {:inline 1} $Libra_split($tv0: $TypeValue, coin: $Value, amount: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    assume $IsValidU64(amount);

    call $ret0, $ret1 := $Libra_split_$def($tv0, coin, amount);
}


procedure {:inline 1} $Libra_update_lbr_exchange_rate_$def($tv0: $TypeValue, tr_account: $Value, lbr_exchange_rate: $Value) returns ()
{
    // declare local variables
    var currency_info: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $AddressType()
    var $t10: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t11: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t12: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t13: $Reference; // ReferenceType($FixedPoint32_FixedPoint32_type_value())
    var $t14: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t15: $Reference; // ReferenceType($Event_EventHandle_type_value($Libra_ToLBRExchangeRateUpdateEvent_type_value()))
    var $t16: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t17: $Value; // $Vector_type_value($IntegerType())
    var $t18: $Value; // $Vector_type_value($IntegerType())
    var $t19: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t20: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t21: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t22: $Value; // $IntegerType()
    var $t23: $Value; // $Libra_ToLBRExchangeRateUpdateEvent_type_value()
    var $t24: $Value; // $AddressType()
    var $t25: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t26: $Value; // $Event_EventHandle_type_value($Libra_ToLBRExchangeRateUpdateEvent_type_value())
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 36050, 0, tr_account); }
    if (true) { assume $DebugTrackLocal(9, 36050, 1, lbr_exchange_rate); }

    // bytecode translation starts here
    // $t24 := move(tr_account)
    call $tmp := $CopyOrMoveValue(tr_account);
    $t24 := $tmp;

    // $t25 := move(lbr_exchange_rate)
    call $tmp := $CopyOrMoveValue(lbr_exchange_rate);
    $t25 := $tmp;

    // $t5 := move($t24)
    call $tmp := $CopyOrMoveValue($t24);
    $t5 := $tmp;

    // $t6 := Roles::has_treasury_compliance_role($t5)
    call $t6 := $Roles_has_treasury_compliance_role($t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 36222);
      goto Abort;
    }
    assume is#$Boolean($t6);


    // $t3 := $t6
    call $tmp := $CopyOrMoveValue($t6);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 36208, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t8 := 2
    $tmp := $Integer(2);
    $t8 := $tmp;

    // abort($t8)
    if (true) { assume $DebugTrackAbort(9, 36208); }
    goto Abort;

    // L0:
L0:

    // Libra::assert_is_currency<#0>()
    call $Libra_assert_is_currency($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 38329);
      goto Abort;
    }

    // $t9 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t9 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 36424);
      goto Abort;
    }
    assume is#$Address($t9);


    // $t10 := borrow_global<Libra::CurrencyInfo<#0>>($t9)
    call $t10 := $BorrowGlobal($t9, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 36363);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($Dereference($t10));

    // UnpackRef($t10)

    // currency_info := $t10
    call currency_info := $CopyOrMoveRef($t10);
    if (true) { assume $DebugTrackLocal(9, 36347, 2, $Dereference(currency_info)); }

    // $t12 := copy(currency_info)
    call $t12 := $CopyOrMoveRef(currency_info);

    // $t13 := borrow_field<Libra::CurrencyInfo<#0>>.to_lbr_exchange_rate($t12)
    call $t13 := $BorrowField($t12, $Libra_CurrencyInfo_to_lbr_exchange_rate);
    assume $FixedPoint32_FixedPoint32_is_well_formed_types($Dereference($t13));

    // Reference(currency_info) <- $t12
    call currency_info := $WritebackToReference($t12, currency_info);

    // UnpackRef($t13)

    // write_ref($t13, $t25)
    call $t13 := $WriteRef($t13, $t25);
    if (true) { assume $DebugTrackLocal(9, 36458, 2, $Dereference(currency_info)); }

    // Reference(currency_info) <- $t13
    call currency_info := $WritebackToReference($t13, currency_info);

    // Reference($t12) <- $t13
    call $t12 := $WritebackToReference($t13, $t12);

    // PackRef($t13)

    // $t14 := copy(currency_info)
    call $t14 := $CopyOrMoveRef(currency_info);

    // $t15 := borrow_field<Libra::CurrencyInfo<#0>>.exchange_rate_update_events($t14)
    call $t15 := $BorrowField($t14, $Libra_CurrencyInfo_exchange_rate_update_events);
    assume $Event_EventHandle_is_well_formed_types($Dereference($t15));

    // Reference(currency_info) <- $t14
    call currency_info := $WritebackToReference($t14, currency_info);

    // UnpackRef($t15)

    // $t16 := copy(currency_info)
    call $t16 := $CopyOrMoveRef(currency_info);

    // $t17 := get_field<Libra::CurrencyInfo<#0>>.currency_code($t16)
    call $tmp := $GetFieldFromReference($t16, $Libra_CurrencyInfo_currency_code);
    assume $Vector_is_well_formed($tmp) && (forall $$0: int :: {$select_vector($tmp,$$0)} $$0 >= 0 && $$0 < $vlen($tmp) ==> $IsValidU8($select_vector($tmp,$$0)));
    $t17 := $tmp;

    // Reference(currency_info) <- $t16
    call currency_info := $WritebackToReference($t16, currency_info);

    // $t18 := move($t17)
    call $tmp := $CopyOrMoveValue($t17);
    $t18 := $tmp;

    // $t19 := move(currency_info)
    call $t19 := $CopyOrMoveRef(currency_info);

    // $t20 := get_field<Libra::CurrencyInfo<#0>>.to_lbr_exchange_rate($t19)
    call $tmp := $GetFieldFromReference($t19, $Libra_CurrencyInfo_to_lbr_exchange_rate);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t20 := $tmp;

    // Libra::CurrencyInfo <- $t19
    call $WritebackToGlobal($t19);

    // $t21 := move($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t21 := $tmp;

    // $t22 := FixedPoint32::get_raw_value($t21)
    call $t22 := $FixedPoint32_get_raw_value($t21);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 36762);
      goto Abort;
    }
    assume $IsValidU64($t22);


    // $t23 := pack Libra::ToLBRExchangeRateUpdateEvent($t18, $t22)
    call $tmp := $Libra_ToLBRExchangeRateUpdateEvent_pack(0, 0, 0, $t18, $t22);
    $t23 := $tmp;

    // PackRef($t15)

    // $t26 := read_ref($t15)
    call $tmp := $ReadRef($t15);
    assume $Event_EventHandle_is_well_formed($tmp);
    $t26 := $tmp;

    // $t26 := Event::emit_event<Libra::ToLBRExchangeRateUpdateEvent>($t26, $t23)
    call $t26 := $Event_emit_event($Libra_ToLBRExchangeRateUpdateEvent_type_value(), $t26, $t23);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 36529);
      goto Abort;
    }
    assume $Event_EventHandle_is_well_formed($t26);


    // write_ref($t15, $t26)
    call $t15 := $WriteRef($t15, $t26);
    if (true) { assume $DebugTrackLocal(9, 36050, 2, $Dereference(currency_info)); }

    // Libra::CurrencyInfo <- $t15
    call $WritebackToGlobal($t15);

    // Reference($t14) <- $t15
    call $t14 := $WritebackToReference($t15, $t14);

    // Reference($t16) <- $t15
    call $t16 := $WritebackToReference($t15, $t16);

    // Reference($t19) <- $t15
    call $t19 := $WritebackToReference($t15, $t19);

    // UnpackRef($t15)

    // PackRef($t15)

    // PackRef($t19)

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Libra_update_lbr_exchange_rate_$direct_inter($tv0: $TypeValue, tr_account: $Value, lbr_exchange_rate: $Value) returns ()
{
    assume is#$Address(tr_account);

    assume $FixedPoint32_FixedPoint32_is_well_formed(lbr_exchange_rate);

    call $Libra_update_lbr_exchange_rate_$def($tv0, tr_account, lbr_exchange_rate);
}


procedure {:inline 1} $Libra_update_lbr_exchange_rate_$direct_intra($tv0: $TypeValue, tr_account: $Value, lbr_exchange_rate: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(tr_account);

    assume $FixedPoint32_FixedPoint32_is_well_formed(lbr_exchange_rate);

    call $Libra_update_lbr_exchange_rate_$def($tv0, tr_account, lbr_exchange_rate);
}


procedure {:inline 1} $Libra_update_lbr_exchange_rate($tv0: $TypeValue, tr_account: $Value, lbr_exchange_rate: $Value) returns ()
{
    assume is#$Address(tr_account);

    assume $FixedPoint32_FixedPoint32_is_well_formed(lbr_exchange_rate);

    call $Libra_update_lbr_exchange_rate_$def($tv0, tr_account, lbr_exchange_rate);
}


procedure {:inline 1} $Libra_update_minting_ability_$def($tv0: $TypeValue, tr_account: $Value, can_mint: $Value) returns ()
{
    // declare local variables
    var currency_info: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $AddressType()
    var $t10: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t11: $Value; // $BooleanType()
    var $t12: $Reference; // ReferenceType($Libra_CurrencyInfo_type_value($tv0))
    var $t13: $Reference; // ReferenceType($BooleanType())
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $BooleanType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 37650, 0, tr_account); }
    if (true) { assume $DebugTrackLocal(9, 37650, 1, can_mint); }

    // bytecode translation starts here
    // $t14 := move(tr_account)
    call $tmp := $CopyOrMoveValue(tr_account);
    $t14 := $tmp;

    // $t15 := move(can_mint)
    call $tmp := $CopyOrMoveValue(can_mint);
    $t15 := $tmp;

    // $t5 := move($t14)
    call $tmp := $CopyOrMoveValue($t14);
    $t5 := $tmp;

    // $t6 := Roles::has_treasury_compliance_role($t5)
    call $t6 := $Roles_has_treasury_compliance_role($t5);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 37808);
      goto Abort;
    }
    assume is#$Boolean($t6);


    // $t3 := $t6
    call $tmp := $CopyOrMoveValue($t6);
    $t3 := $tmp;
    if (true) { assume $DebugTrackLocal(9, 37794, 3, $tmp); }

    // if ($t3) goto L0 else goto L1
    $tmp := $t3;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t8 := 2
    $tmp := $Integer(2);
    $t8 := $tmp;

    // abort($t8)
    if (true) { assume $DebugTrackAbort(9, 37794); }
    goto Abort;

    // L0:
L0:

    // Libra::assert_is_currency<#0>()
    call $Libra_assert_is_currency($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 38329);
      goto Abort;
    }

    // $t9 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t9 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 38002);
      goto Abort;
    }
    assume is#$Address($t9);


    // $t10 := borrow_global<Libra::CurrencyInfo<#0>>($t9)
    call $t10 := $BorrowGlobal($t9, $Libra_CurrencyInfo_type_value($tv0));
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 37945);
      goto Abort;
    }
    assume $Libra_CurrencyInfo_is_well_formed($Dereference($t10));

    // UnpackRef($t10)

    // currency_info := $t10
    call currency_info := $CopyOrMoveRef($t10);
    if (true) { assume $DebugTrackLocal(9, 37929, 2, $Dereference(currency_info)); }

    // $t12 := move(currency_info)
    call $t12 := $CopyOrMoveRef(currency_info);

    // $t13 := borrow_field<Libra::CurrencyInfo<#0>>.can_mint($t12)
    call $t13 := $BorrowField($t12, $Libra_CurrencyInfo_can_mint);
    assume is#$Boolean($Dereference($t13));

    // Libra::CurrencyInfo <- $t12
    call $WritebackToGlobal($t12);

    // UnpackRef($t13)

    // write_ref($t13, $t15)
    call $t13 := $WriteRef($t13, $t15);
    if (true) { assume $DebugTrackLocal(9, 38036, 2, $Dereference(currency_info)); }

    // Libra::CurrencyInfo <- $t13
    call $WritebackToGlobal($t13);

    // Reference($t12) <- $t13
    call $t12 := $WritebackToReference($t13, $t12);

    // PackRef($t12)

    // PackRef($t13)

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Libra_update_minting_ability_$direct_inter($tv0: $TypeValue, tr_account: $Value, can_mint: $Value) returns ()
{
    assume is#$Address(tr_account);

    assume is#$Boolean(can_mint);

    call $Libra_update_minting_ability_$def($tv0, tr_account, can_mint);
}


procedure {:inline 1} $Libra_update_minting_ability_$direct_intra($tv0: $TypeValue, tr_account: $Value, can_mint: $Value) returns ()
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume is#$Address(tr_account);

    assume is#$Boolean(can_mint);

    call $Libra_update_minting_ability_$def($tv0, tr_account, can_mint);
}


procedure {:inline 1} $Libra_update_minting_ability($tv0: $TypeValue, tr_account: $Value, can_mint: $Value) returns ()
{
    assume is#$Address(tr_account);

    assume is#$Boolean(can_mint);

    call $Libra_update_minting_ability_$def($tv0, tr_account, can_mint);
}


procedure {:inline 1} $Libra_withdraw_$def($tv0: $TypeValue, coin: $Value, amount: $Value) returns ($ret0: $Value, $ret1: $Value)
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
    var $t12: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $IntegerType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t18: $Reference; // ReferenceType($IntegerType())
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $Libra_Libra_type_value($tv0)
    var $t21: $Value; // $Libra_Libra_type_value($tv0)
    var $t22: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t23: $Value; // $IntegerType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 27992, 0, coin); }
    if (true) { assume $DebugTrackLocal(9, 27992, 1, amount); }

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
    if (true) { assume $DebugTrackLocal(9, 28151, 2, $tmp); }

    // if ($t2) goto L0 else goto L1
    $tmp := $t2;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t10 := move($t22)
    call $t10 := $CopyOrMoveRef($t22);

    // destroy($t10)

    // LocalRoot($t21) <- $t10
    call $t21 := $WritebackToValue($t10, 21, $t21);

    // PackRef($t10)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t10));

    // $t11 := 5
    $tmp := $Integer(5);
    $t11 := $tmp;

    // abort($t11)
    if (true) { assume $DebugTrackAbort(9, 28151); }
    goto Abort;

    // L0:
L0:

    // $t12 := copy($t22)
    call $t12 := $CopyOrMoveRef($t22);

    // $t13 := get_field<Libra::Libra<#0>>.value($t12)
    call $tmp := $GetFieldFromReference($t12, $Libra_Libra_value);
    assume $IsValidU64($tmp);
    $t13 := $tmp;

    // Reference($t22) <- $t12
    call $t22 := $WritebackToReference($t12, $t22);

    // $t14 := move($t13)
    call $tmp := $CopyOrMoveValue($t13);
    $t14 := $tmp;

    // $t16 := -($t14, $t23)
    call $tmp := $Sub($t14, $t23);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 28241);
      goto Abort;
    }
    $t16 := $tmp;

    // $t17 := move($t22)
    call $t17 := $CopyOrMoveRef($t22);

    // $t18 := borrow_field<Libra::Libra<#0>>.value($t17)
    call $t18 := $BorrowField($t17, $Libra_Libra_value);
    assume $IsValidU64($Dereference($t18));

    // LocalRoot($t21) <- $t17
    call $t21 := $WritebackToValue($t17, 21, $t21);

    // UnpackRef($t18)

    // write_ref($t18, $t16)
    call $t18 := $WriteRef($t18, $t16);

    // LocalRoot($t21) <- $t18
    call $t21 := $WritebackToValue($t18, 21, $t21);

    // Reference($t17) <- $t18
    call $t17 := $WritebackToReference($t18, $t17);

    // PackRef($t17)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t17));

    // PackRef($t18)

    // $t20 := pack Libra::Libra<#0>($t23)
    call $tmp := $Libra_Libra_pack(0, 0, 0, $tv0, $t23);
    $t20 := $tmp;

    // return ($t20, $t21)
    $ret0 := $t20;
    if (true) { assume $DebugTrackLocal(9, 28259, 24, $ret0); }
    $ret1 := $t21;
    if (true) { assume $DebugTrackLocal(9, 28259, 25, $ret1); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}

procedure {:inline 1} $Libra_withdraw_$direct_inter($tv0: $TypeValue, coin: $Value, amount: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    assume $IsValidU64(amount);

    call $ret0, $ret1 := $Libra_withdraw_$def($tv0, coin, amount);
}


procedure {:inline 1} $Libra_withdraw_$direct_intra($tv0: $TypeValue, coin: $Value, amount: $Value) returns ($ret0: $Value, $ret1: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume $Libra_Libra_is_well_formed(coin);

    assume $IsValidU64(amount);

    call $ret0, $ret1 := $Libra_withdraw_$def($tv0, coin, amount);
}


procedure {:inline 1} $Libra_withdraw($tv0: $TypeValue, coin: $Value, amount: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    assume $IsValidU64(amount);

    call $ret0, $ret1 := $Libra_withdraw_$def($tv0, coin, amount);
}


procedure {:inline 1} $Libra_withdraw_all_$def($tv0: $TypeValue, coin: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    // declare local variables
    var val: $Value; // $IntegerType()
    var $t2: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $Libra_Libra_type_value($tv0)
    var $t8: $Value; // $Libra_Libra_type_value($tv0)
    var $t9: $Reference; // ReferenceType($Libra_Libra_type_value($tv0))
    var $t10: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(9, 28590, 0, coin); }

    // bytecode translation starts here
    // $t8 := move(coin)
    call $tmp := $CopyOrMoveValue(coin);
    $t8 := $tmp;

    // $t9 := borrow_local($t8)
    call $t9 := $BorrowLoc(8, $t8);
    assume $Libra_Libra_is_well_formed($Dereference($t9));

    // UnpackRef($t9)
    call $Libra_Libra_before_update_inv($tv0, $Dereference($t9));

    // $t2 := copy($t9)
    call $t2 := $CopyOrMoveRef($t9);

    // $t3 := get_field<Libra::Libra<#0>>.value($t2)
    call $tmp := $GetFieldFromReference($t2, $Libra_Libra_value);
    assume $IsValidU64($tmp);
    $t3 := $tmp;

    // Reference($t9) <- $t2
    call $t9 := $WritebackToReference($t2, $t9);

    // $t4 := move($t3)
    call $tmp := $CopyOrMoveValue($t3);
    $t4 := $tmp;

    // val := $t4
    call $tmp := $CopyOrMoveValue($t4);
    val := $tmp;
    if (true) { assume $DebugTrackLocal(9, 28683, 1, $tmp); }

    // $t5 := move($t9)
    call $t5 := $CopyOrMoveRef($t9);

    // PackRef($t5)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t5));

    // $t10 := read_ref($t5)
    call $tmp := $ReadRef($t5);
    assume $Libra_Libra_is_well_formed($tmp);
    $t10 := $tmp;

    // ($t7, $t10) := Libra::withdraw<#0>($t10, val)
    call $t7, $t10 := $Libra_withdraw($tv0, $t10, val);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 28003);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t7);

    assume $Libra_Libra_is_well_formed($t10);


    // write_ref($t5, $t10)
    call $t5 := $WriteRef($t5, $t10);

    // LocalRoot($t8) <- $t5
    call $t8 := $WritebackToValue($t5, 8, $t8);

    // UnpackRef($t5)
    call $Libra_Libra_before_update_inv($tv0, $Dereference($t5));

    // PackRef($t5)
    call $Libra_Libra_after_update_inv($tv0, $Dereference($t5));

    // return ($t7, $t8)
    $ret0 := $t7;
    if (true) { assume $DebugTrackLocal(9, 28709, 11, $ret0); }
    $ret1 := $t8;
    if (true) { assume $DebugTrackLocal(9, 28709, 12, $ret1); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}

procedure {:inline 1} $Libra_withdraw_all_$direct_inter($tv0: $TypeValue, coin: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    call $ret0, $ret1 := $Libra_withdraw_all_$def($tv0, coin);
}


procedure {:inline 1} $Libra_withdraw_all_$direct_intra($tv0: $TypeValue, coin: $Value) returns ($ret0: $Value, $ret1: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    assume $Libra_Libra_is_well_formed(coin);

    call $ret0, $ret1 := $Libra_withdraw_all_$def($tv0, coin);
}


procedure {:inline 1} $Libra_withdraw_all($tv0: $TypeValue, coin: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    call $ret0, $ret1 := $Libra_withdraw_all_$def($tv0, coin);
}


procedure {:inline 1} $Libra_zero_$def($tv0: $TypeValue) returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $t1: $Value; // $Libra_Libra_type_value($tv0)
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // Libra::assert_is_currency<#0>()
    call $Libra_assert_is_currency($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(9, 38329);
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
    if (true) { assume $DebugTrackLocal(9, 26754, 2, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Libra_zero_$direct_inter($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_zero_$def($tv0);
}


procedure {:inline 1} $Libra_zero_$direct_intra($tv0: $TypeValue) returns ($ret0: $Value)
requires b#$Boolean($Boolean(b#$Boolean($Boolean(!b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)))) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $Integer(0))))));
requires b#$Boolean($Boolean(b#$Boolean($Libra_spec_is_currency($m, $txn, $tv0)) ==> b#$Boolean($Boolean($IsEqual($Libra_sum_of_coin_values[$tv0], $SelectField($ResourceValue($m, $Libra_CurrencyInfo_type_value($tv0), $CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS()), $Libra_CurrencyInfo_total_value))))));
{
    call $ret0 := $Libra_zero_$def($tv0);
}


procedure {:inline 1} $Libra_zero($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $Libra_zero_$def($tv0);
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

procedure {:inline 1} $Coin1_initialize_$def(lr_account: $Value, tc_account: $Value) returns ()
{
    // declare local variables
    var coin1_burn_cap: $Value; // $Libra_BurnCapability_type_value($Coin1_Coin1_type_value())
    var coin1_mint_cap: $Value; // $Libra_MintCapability_type_value($Coin1_Coin1_type_value())
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $Vector_type_value($IntegerType())
    var $t12: $Value; // $Libra_MintCapability_type_value($Coin1_Coin1_type_value())
    var $t13: $Value; // $Libra_BurnCapability_type_value($Coin1_Coin1_type_value())
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $Libra_MintCapability_type_value($Coin1_Coin1_type_value())
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $Libra_BurnCapability_type_value($Coin1_Coin1_type_value())
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $AddressType()
    var $t21: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(2, 104, 0, lr_account); }
    if (true) { assume $DebugTrackLocal(2, 104, 1, tc_account); }

    // bytecode translation starts here
    // $t20 := move(lr_account)
    call $tmp := $CopyOrMoveValue(lr_account);
    $t20 := $tmp;

    // $t21 := move(tc_account)
    call $tmp := $CopyOrMoveValue(tc_account);
    $t21 := $tmp;

    // $t4 := move($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t4 := $tmp;

    // $t5 := 1
    $tmp := $Integer(1);
    $t5 := $tmp;

    // $t6 := 2
    $tmp := $Integer(2);
    $t6 := $tmp;

    // $t7 := FixedPoint32::create_from_rational($t5, $t6)
    call $t7 := $FixedPoint32_create_from_rational($t5, $t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(2, 383);
      goto Abort;
    }
    assume $FixedPoint32_FixedPoint32_is_well_formed($t7);


    // $t8 := false
    $tmp := $Boolean(false);
    $t8 := $tmp;

    // $t9 := 1000000
    $tmp := $Integer(1000000);
    $t9 := $tmp;

    // $t10 := 100
    $tmp := $Integer(100);
    $t10 := $tmp;

    // $t11 := [67, 111, 105, 110, 49]
    $tmp := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := $Integer(67)][1 := $Integer(111)][2 := $Integer(105)][3 := $Integer(110)][4 := $Integer(49)], 5));
    $t11 := $tmp;

    // ($t12, $t13) := Libra::register_currency<Coin1::Coin1>($t4, $t7, $t8, $t9, $t10, $t11)
    call $t12, $t13 := $Libra_register_currency($Coin1_Coin1_type_value(), $t4, $t7, $t8, $t9, $t10, $t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(2, 299);
      goto Abort;
    }
    assume $Libra_MintCapability_is_well_formed($t12);

    assume $Libra_BurnCapability_is_well_formed($t13);


    // coin1_burn_cap := $t13
    call $tmp := $CopyOrMoveValue($t13);
    coin1_burn_cap := $tmp;
    if (true) { assume $DebugTrackLocal(2, 262, 2, $tmp); }

    // coin1_mint_cap := $t12
    call $tmp := $CopyOrMoveValue($t12);
    coin1_mint_cap := $tmp;
    if (true) { assume $DebugTrackLocal(2, 246, 3, $tmp); }

    // $t14 := copy($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t14 := $tmp;

    // $t16 := copy($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t16 := $tmp;

    // Libra::publish_mint_capability<Coin1::Coin1>($t14, coin1_mint_cap, $t16)
    call $Libra_publish_mint_capability($Coin1_Coin1_type_value(), $t14, coin1_mint_cap, $t16);
    if ($abort_flag) {
      assume $DebugTrackAbort(2, 632);
      goto Abort;
    }

    // $t17 := copy($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t17 := $tmp;

    // $t19 := move($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t19 := $tmp;

    // Libra::publish_burn_capability<Coin1::Coin1>($t17, coin1_burn_cap, $t19)
    call $Libra_publish_burn_capability($Coin1_Coin1_type_value(), $t17, coin1_burn_cap, $t19);
    if ($abort_flag) {
      assume $DebugTrackAbort(2, 719);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Coin1_initialize_$direct_inter(lr_account: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    assume is#$Address(tc_account);

    call $Coin1_initialize_$def(lr_account, tc_account);
}


procedure {:inline 1} $Coin1_initialize_$direct_intra(lr_account: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    assume is#$Address(tc_account);

    call $Coin1_initialize_$def(lr_account, tc_account);
}


procedure {:inline 1} $Coin1_initialize(lr_account: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    assume is#$Address(tc_account);

    call $Coin1_initialize_$def(lr_account, tc_account);
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

procedure {:inline 1} $Coin2_initialize_$def(lr_account: $Value, tc_account: $Value) returns ()
{
    // declare local variables
    var coin2_burn_cap: $Value; // $Libra_BurnCapability_type_value($Coin2_Coin2_type_value())
    var coin2_mint_cap: $Value; // $Libra_MintCapability_type_value($Coin2_Coin2_type_value())
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $Vector_type_value($IntegerType())
    var $t12: $Value; // $Libra_MintCapability_type_value($Coin2_Coin2_type_value())
    var $t13: $Value; // $Libra_BurnCapability_type_value($Coin2_Coin2_type_value())
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $Libra_MintCapability_type_value($Coin2_Coin2_type_value())
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $Libra_BurnCapability_type_value($Coin2_Coin2_type_value())
    var $t19: $Value; // $AddressType()
    var $t20: $Value; // $AddressType()
    var $t21: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(3, 104, 0, lr_account); }
    if (true) { assume $DebugTrackLocal(3, 104, 1, tc_account); }

    // bytecode translation starts here
    // $t20 := move(lr_account)
    call $tmp := $CopyOrMoveValue(lr_account);
    $t20 := $tmp;

    // $t21 := move(tc_account)
    call $tmp := $CopyOrMoveValue(tc_account);
    $t21 := $tmp;

    // $t4 := move($t20)
    call $tmp := $CopyOrMoveValue($t20);
    $t4 := $tmp;

    // $t5 := 1
    $tmp := $Integer(1);
    $t5 := $tmp;

    // $t6 := 2
    $tmp := $Integer(2);
    $t6 := $tmp;

    // $t7 := FixedPoint32::create_from_rational($t5, $t6)
    call $t7 := $FixedPoint32_create_from_rational($t5, $t6);
    if ($abort_flag) {
      assume $DebugTrackAbort(3, 383);
      goto Abort;
    }
    assume $FixedPoint32_FixedPoint32_is_well_formed($t7);


    // $t8 := false
    $tmp := $Boolean(false);
    $t8 := $tmp;

    // $t9 := 1000000
    $tmp := $Integer(1000000);
    $t9 := $tmp;

    // $t10 := 100
    $tmp := $Integer(100);
    $t10 := $tmp;

    // $t11 := [67, 111, 105, 110, 50]
    $tmp := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := $Integer(67)][1 := $Integer(111)][2 := $Integer(105)][3 := $Integer(110)][4 := $Integer(50)], 5));
    $t11 := $tmp;

    // ($t12, $t13) := Libra::register_currency<Coin2::Coin2>($t4, $t7, $t8, $t9, $t10, $t11)
    call $t12, $t13 := $Libra_register_currency($Coin2_Coin2_type_value(), $t4, $t7, $t8, $t9, $t10, $t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(3, 299);
      goto Abort;
    }
    assume $Libra_MintCapability_is_well_formed($t12);

    assume $Libra_BurnCapability_is_well_formed($t13);


    // coin2_burn_cap := $t13
    call $tmp := $CopyOrMoveValue($t13);
    coin2_burn_cap := $tmp;
    if (true) { assume $DebugTrackLocal(3, 262, 2, $tmp); }

    // coin2_mint_cap := $t12
    call $tmp := $CopyOrMoveValue($t12);
    coin2_mint_cap := $tmp;
    if (true) { assume $DebugTrackLocal(3, 246, 3, $tmp); }

    // $t14 := copy($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t14 := $tmp;

    // $t16 := copy($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t16 := $tmp;

    // Libra::publish_mint_capability<Coin2::Coin2>($t14, coin2_mint_cap, $t16)
    call $Libra_publish_mint_capability($Coin2_Coin2_type_value(), $t14, coin2_mint_cap, $t16);
    if ($abort_flag) {
      assume $DebugTrackAbort(3, 633);
      goto Abort;
    }

    // $t17 := copy($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t17 := $tmp;

    // $t19 := move($t21)
    call $tmp := $CopyOrMoveValue($t21);
    $t19 := $tmp;

    // Libra::publish_burn_capability<Coin2::Coin2>($t17, coin2_burn_cap, $t19)
    call $Libra_publish_burn_capability($Coin2_Coin2_type_value(), $t17, coin2_burn_cap, $t19);
    if ($abort_flag) {
      assume $DebugTrackAbort(3, 720);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Coin2_initialize_$direct_inter(lr_account: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    assume is#$Address(tc_account);

    call $Coin2_initialize_$def(lr_account, tc_account);
}


procedure {:inline 1} $Coin2_initialize_$direct_intra(lr_account: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    assume is#$Address(tc_account);

    call $Coin2_initialize_$def(lr_account, tc_account);
}


procedure {:inline 1} $Coin2_initialize(lr_account: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    assume is#$Address(tc_account);

    call $Coin2_initialize_$def(lr_account, tc_account);
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
    call $Libra_Preburn_before_update_inv($LBR_LBR_type_value(), $SelectField($before, $LBR_Reserve_preburn_cap));
    call $LBR_ReserveComponent_before_update_inv($Coin1_Coin1_type_value(), $SelectField($before, $LBR_Reserve_coin1));
    call $LBR_ReserveComponent_before_update_inv($Coin2_Coin2_type_value(), $SelectField($before, $LBR_Reserve_coin2));
}

procedure {:inline 1} $LBR_Reserve_after_update_inv($after: $Value) {
    call $Libra_Preburn_after_update_inv($LBR_LBR_type_value(), $SelectField($after, $LBR_Reserve_preburn_cap));
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

procedure {:inline 1} $LBR_initialize_$def(lr_account: $Value, tc_account: $Value) returns ()
{
    // declare local variables
    var burn_cap: $Value; // $Libra_BurnCapability_type_value($LBR_LBR_type_value())
    var coin1: $Value; // $LBR_ReserveComponent_type_value($Coin1_Coin1_type_value())
    var coin2: $Value; // $LBR_ReserveComponent_type_value($Coin2_Coin2_type_value())
    var mint_cap: $Value; // $Libra_MintCapability_type_value($LBR_LBR_type_value())
    var preburn_cap: $Value; // $Libra_Preburn_type_value($LBR_LBR_type_value())
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $AddressType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $BooleanType()
    var $t13: $Value; // $BooleanType()
    var $t14: $Value; // $AddressType()
    var $t15: $Value; // $AddressType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $AddressType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t21: $Value; // $BooleanType()
    var $t22: $Value; // $IntegerType()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $Vector_type_value($IntegerType())
    var $t25: $Value; // $Libra_MintCapability_type_value($LBR_LBR_type_value())
    var $t26: $Value; // $Libra_BurnCapability_type_value($LBR_LBR_type_value())
    var $t27: $Value; // $AddressType()
    var $t28: $Value; // $Libra_Preburn_type_value($LBR_LBR_type_value())
    var $t29: $Value; // $IntegerType()
    var $t30: $Value; // $IntegerType()
    var $t31: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t32: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t33: $Value; // $LBR_ReserveComponent_type_value($Coin1_Coin1_type_value())
    var $t34: $Value; // $IntegerType()
    var $t35: $Value; // $IntegerType()
    var $t36: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t37: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t38: $Value; // $LBR_ReserveComponent_type_value($Coin2_Coin2_type_value())
    var $t39: $Value; // $AddressType()
    var $t40: $Value; // $Libra_MintCapability_type_value($LBR_LBR_type_value())
    var $t41: $Value; // $Libra_BurnCapability_type_value($LBR_LBR_type_value())
    var $t42: $Value; // $Libra_Preburn_type_value($LBR_LBR_type_value())
    var $t43: $Value; // $LBR_ReserveComponent_type_value($Coin1_Coin1_type_value())
    var $t44: $Value; // $LBR_ReserveComponent_type_value($Coin2_Coin2_type_value())
    var $t45: $Value; // $LBR_Reserve_type_value()
    var $t46: $Value; // $AddressType()
    var $t47: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(7, 3871, 0, lr_account); }
    if (true) { assume $DebugTrackLocal(7, 3871, 1, tc_account); }

    // bytecode translation starts here
    // $t46 := move(lr_account)
    call $tmp := $CopyOrMoveValue(lr_account);
    $t46 := $tmp;

    // $t47 := move(tc_account)
    call $tmp := $CopyOrMoveValue(tc_account);
    $t47 := $tmp;

    // $t9 := copy($t46)
    call $tmp := $CopyOrMoveValue($t46);
    $t9 := $tmp;

    // $t10 := Signer::address_of($t9)
    call $t10 := $Signer_address_of($t9);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4017);
      goto Abort;
    }
    assume is#$Address($t10);


    // $t11 := LBR::reserve_address()
    call $t11 := $LBR_reserve_address();
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 8817);
      goto Abort;
    }
    assume is#$Address($t11);


    // $t12 := ==($t10, $t11)
    $tmp := $Boolean($IsEqual($t10, $t11));
    $t12 := $tmp;

    // $t7 := $t12
    call $tmp := $CopyOrMoveValue($t12);
    $t7 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 4002, 7, $tmp); }

    // if ($t7) goto L0 else goto L1
    $tmp := $t7;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t14 := move($t47)
    call $tmp := $CopyOrMoveValue($t47);
    $t14 := $tmp;

    // destroy($t14)

    // $t15 := move($t46)
    call $tmp := $CopyOrMoveValue($t46);
    $t15 := $tmp;

    // destroy($t15)

    // $t16 := 0
    $tmp := $Integer(0);
    $t16 := $tmp;

    // abort($t16)
    if (true) { assume $DebugTrackAbort(7, 4002); }
    goto Abort;

    // L0:
L0:

    // $t17 := copy($t46)
    call $tmp := $CopyOrMoveValue($t46);
    $t17 := $tmp;

    // $t18 := 1
    $tmp := $Integer(1);
    $t18 := $tmp;

    // $t19 := 1
    $tmp := $Integer(1);
    $t19 := $tmp;

    // $t20 := FixedPoint32::create_from_rational($t18, $t19)
    call $t20 := $FixedPoint32_create_from_rational($t18, $t19);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4247);
      goto Abort;
    }
    assume $FixedPoint32_FixedPoint32_is_well_formed($t20);


    // $t21 := true
    $tmp := $Boolean(true);
    $t21 := $tmp;

    // $t22 := 1000000
    $tmp := $Integer(1000000);
    $t22 := $tmp;

    // $t23 := 1000
    $tmp := $Integer(1000);
    $t23 := $tmp;

    // $t24 := [76, 66, 82]
    $tmp := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := $Integer(76)][1 := $Integer(66)][2 := $Integer(82)], 3));
    $t24 := $tmp;

    // ($t25, $t26) := Libra::register_currency<LBR::LBR>($t17, $t20, $t21, $t22, $t23, $t24)
    call $t25, $t26 := $Libra_register_currency($LBR_LBR_type_value(), $t17, $t20, $t21, $t22, $t23, $t24);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4173);
      goto Abort;
    }
    assume $Libra_MintCapability_is_well_formed($t25);

    assume $Libra_BurnCapability_is_well_formed($t26);


    // burn_cap := $t26
    call $tmp := $CopyOrMoveValue($t26);
    burn_cap := $tmp;
    if (true) { assume $DebugTrackLocal(7, 4154, 2, $tmp); }

    // mint_cap := $t25
    call $tmp := $CopyOrMoveValue($t25);
    mint_cap := $tmp;
    if (true) { assume $DebugTrackLocal(7, 4144, 5, $tmp); }

    // $t27 := move($t47)
    call $tmp := $CopyOrMoveValue($t47);
    $t27 := $tmp;

    // $t28 := Libra::create_preburn<LBR::LBR>($t27)
    call $t28 := $Libra_create_preburn($LBR_LBR_type_value(), $t27);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4492);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($t28);


    // preburn_cap := $t28
    call $tmp := $CopyOrMoveValue($t28);
    preburn_cap := $tmp;
    if (true) { assume $DebugTrackLocal(7, 4471, 6, $tmp); }

    // $t29 := 1
    $tmp := $Integer(1);
    $t29 := $tmp;

    // $t30 := 2
    $tmp := $Integer(2);
    $t30 := $tmp;

    // $t31 := FixedPoint32::create_from_rational($t29, $t30)
    call $t31 := $FixedPoint32_create_from_rational($t29, $t30);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4604);
      goto Abort;
    }
    assume $FixedPoint32_FixedPoint32_is_well_formed($t31);


    // $t32 := Libra::zero<Coin1::Coin1>()
    call $t32 := $Libra_zero($Coin1_Coin1_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4660);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t32);


    // $t33 := pack LBR::ReserveComponent<Coin1::Coin1>($t31, $t32)
    call $tmp := $LBR_ReserveComponent_pack(0, 0, 0, $Coin1_Coin1_type_value(), $t31, $t32);
    $t33 := $tmp;

    // coin1 := $t33
    call $tmp := $CopyOrMoveValue($t33);
    coin1 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 4537, 3, $tmp); }

    // $t34 := 1
    $tmp := $Integer(1);
    $t34 := $tmp;

    // $t35 := 2
    $tmp := $Integer(2);
    $t35 := $tmp;

    // $t36 := FixedPoint32::create_from_rational($t34, $t35)
    call $t36 := $FixedPoint32_create_from_rational($t34, $t35);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4765);
      goto Abort;
    }
    assume $FixedPoint32_FixedPoint32_is_well_formed($t36);


    // $t37 := Libra::zero<Coin2::Coin2>()
    call $t37 := $Libra_zero($Coin2_Coin2_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4821);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t37);


    // $t38 := pack LBR::ReserveComponent<Coin2::Coin2>($t36, $t37)
    call $tmp := $LBR_ReserveComponent_pack(0, 0, 0, $Coin2_Coin2_type_value(), $t36, $t37);
    $t38 := $tmp;

    // coin2 := $t38
    call $tmp := $CopyOrMoveValue($t38);
    coin2 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 4698, 4, $tmp); }

    // $t39 := move($t46)
    call $tmp := $CopyOrMoveValue($t46);
    $t39 := $tmp;

    // $t45 := pack LBR::Reserve(mint_cap, burn_cap, preburn_cap, coin1, coin2)
    call $tmp := $LBR_Reserve_pack(0, 0, 0, mint_cap, burn_cap, preburn_cap, coin1, coin2);
    $t45 := $tmp;

    // move_to<LBR::Reserve>($t45, $t39)
    call $MoveTo($LBR_Reserve_type_value(), $t45, $t39);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 4855);
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $LBR_initialize_$direct_inter(lr_account: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    assume is#$Address(tc_account);

    call $LBR_initialize_$def(lr_account, tc_account);
}


procedure {:inline 1} $LBR_initialize_$direct_intra(lr_account: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    assume is#$Address(tc_account);

    call $LBR_initialize_$def(lr_account, tc_account);
}


procedure {:inline 1} $LBR_initialize(lr_account: $Value, tc_account: $Value) returns ()
{
    assume is#$Address(lr_account);

    assume is#$Address(tc_account);

    call $LBR_initialize_$def(lr_account, tc_account);
}


procedure {:inline 1} $LBR_create_$def(amount_lbr: $Value, coin1: $Value, coin2: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var num_coin1: $Value; // $IntegerType()
    var num_coin2: $Value; // $IntegerType()
    var reserve: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $BooleanType()
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $AddressType()
    var $t21: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t22: $Value; // $IntegerType()
    var $t23: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t24: $Value; // $IntegerType()
    var $t25: $Value; // $BooleanType()
    var $t26: $Value; // $BooleanType()
    var $t27: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t28: $Value; // $IntegerType()
    var $t29: $Value; // $IntegerType()
    var $t30: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t31: $Value; // $IntegerType()
    var $t32: $Value; // $BooleanType()
    var $t33: $Value; // $BooleanType()
    var $t34: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t35: $Value; // $IntegerType()
    var $t36: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t37: $Reference; // ReferenceType($LBR_ReserveComponent_type_value($Coin1_Coin1_type_value()))
    var $t38: $Reference; // ReferenceType($Libra_Libra_type_value($Coin1_Coin1_type_value()))
    var $t39: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t40: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t41: $Reference; // ReferenceType($LBR_ReserveComponent_type_value($Coin2_Coin2_type_value()))
    var $t42: $Reference; // ReferenceType($Libra_Libra_type_value($Coin2_Coin2_type_value()))
    var $t43: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t44: $Value; // $IntegerType()
    var $t45: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t46: $Value; // $Libra_MintCapability_type_value($LBR_LBR_type_value())
    var $t47: $Value; // $Libra_Libra_type_value($LBR_LBR_type_value())
    var $t48: $Value; // $IntegerType()
    var $t49: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t50: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t51: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t52: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(7, 6437, 0, amount_lbr); }
    if (true) { assume $DebugTrackLocal(7, 6437, 1, coin1); }
    if (true) { assume $DebugTrackLocal(7, 6437, 2, coin2); }

    // bytecode translation starts here
    // $t48 := move(amount_lbr)
    call $tmp := $CopyOrMoveValue(amount_lbr);
    $t48 := $tmp;

    // $t49 := move(coin1)
    call $tmp := $CopyOrMoveValue(coin1);
    $t49 := $tmp;

    // $t50 := move(coin2)
    call $tmp := $CopyOrMoveValue(coin2);
    $t50 := $tmp;

    // $t13 := 0
    $tmp := $Integer(0);
    $t13 := $tmp;

    // $t14 := >($t48, $t13)
    call $tmp := $Gt($t48, $t13);
    $t14 := $tmp;

    // $t6 := $t14
    call $tmp := $CopyOrMoveValue($t14);
    $t6 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 6587, 6, $tmp); }

    // if ($t6) goto L0 else goto L1
    $tmp := $t6;
    if (b#$Boolean($tmp)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // $t16 := 1
    $tmp := $Integer(1);
    $t16 := $tmp;

    // abort($t16)
    if (true) { assume $DebugTrackAbort(7, 6587); }
    goto Abort;

    // L0:
L0:

    // ($t18, $t19) := LBR::calculate_component_amounts_for_lbr($t48)
    call $t18, $t19 := $LBR_calculate_component_amounts_for_lbr($t48);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5492);
      goto Abort;
    }
    assume $IsValidU64($t18);

    assume $IsValidU64($t19);


    // num_coin2 := $t19
    call $tmp := $CopyOrMoveValue($t19);
    num_coin2 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 6663, 4, $tmp); }

    // num_coin1 := $t18
    call $tmp := $CopyOrMoveValue($t18);
    num_coin1 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 6652, 3, $tmp); }

    // $t20 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t20 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 6789);
      goto Abort;
    }
    assume is#$Address($t20);


    // $t21 := borrow_global<LBR::Reserve>($t20)
    call $t21 := $BorrowGlobal($t20, $LBR_Reserve_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 6747);
      goto Abort;
    }
    assume $LBR_Reserve_is_well_formed($Dereference($t21));

    // UnpackRef($t21)
    call $LBR_Reserve_before_update_inv($Dereference($t21));

    // reserve := $t21
    call reserve := $CopyOrMoveRef($t21);
    if (true) { assume $DebugTrackLocal(7, 6737, 5, $Dereference(reserve)); }

    // $t23 := copy($t49)
    call $tmp := $CopyOrMoveValue($t49);
    $t23 := $tmp;

    // $t24 := Libra::value<Coin1::Coin1>($t23)
    call $t24 := $Libra_value($Coin1_Coin1_type_value(), $t23);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 6847);
      goto Abort;
    }
    assume $IsValidU64($t24);


    // $t25 := ==(num_coin1, $t24)
    $tmp := $Boolean($IsEqual(num_coin1, $t24));
    $t25 := $tmp;

    // $t8 := $t25
    call $tmp := $CopyOrMoveValue($t25);
    $t8 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 6820, 8, $tmp); }

    // if ($t8) goto L2 else goto L3
    $tmp := $t8;
    if (b#$Boolean($tmp)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // $t27 := move(reserve)
    call $t27 := $CopyOrMoveRef(reserve);

    // destroy($t27)

    // LBR::Reserve <- $t27
    call $WritebackToGlobal($t27);

    // PackRef($t27)
    call $LBR_Reserve_after_update_inv($Dereference($t27));

    // $t28 := 2
    $tmp := $Integer(2);
    $t28 := $tmp;

    // abort($t28)
    if (true) { assume $DebugTrackAbort(7, 6820); }
    goto Abort;

    // L2:
L2:

    // $t30 := copy($t50)
    call $tmp := $CopyOrMoveValue($t50);
    $t30 := $tmp;

    // $t31 := Libra::value<Coin2::Coin2>($t30)
    call $t31 := $Libra_value($Coin2_Coin2_type_value(), $t30);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 6921);
      goto Abort;
    }
    assume $IsValidU64($t31);


    // $t32 := ==(num_coin2, $t31)
    $tmp := $Boolean($IsEqual(num_coin2, $t31));
    $t32 := $tmp;

    // $t10 := $t32
    call $tmp := $CopyOrMoveValue($t32);
    $t10 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 6894, 10, $tmp); }

    // if ($t10) goto L4 else goto L5
    $tmp := $t10;
    if (b#$Boolean($tmp)) { goto L4; } else { goto L5; }

    // L5:
L5:

    // $t34 := move(reserve)
    call $t34 := $CopyOrMoveRef(reserve);

    // destroy($t34)

    // LBR::Reserve <- $t34
    call $WritebackToGlobal($t34);

    // PackRef($t34)
    call $LBR_Reserve_after_update_inv($Dereference($t34));

    // $t35 := 3
    $tmp := $Integer(3);
    $t35 := $tmp;

    // abort($t35)
    if (true) { assume $DebugTrackAbort(7, 6894); }
    goto Abort;

    // L4:
L4:

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

    // $t51 := read_ref($t38)
    call $tmp := $ReadRef($t38);
    assume $Libra_Libra_is_well_formed($tmp);
    $t51 := $tmp;

    // $t51 := Libra::deposit<Coin1::Coin1>($t51, $t49)
    call $t51 := $Libra_deposit($Coin1_Coin1_type_value(), $t51, $t49);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 7022);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t51);


    // write_ref($t38, $t51)
    call $t38 := $WriteRef($t38, $t51);
    if (true) { assume $DebugTrackLocal(7, 7081, 5, $Dereference(reserve)); }

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

    // $t40 := copy(reserve)
    call $t40 := $CopyOrMoveRef(reserve);

    // $t41 := borrow_field<LBR::Reserve>.coin2($t40)
    call $t41 := $BorrowField($t40, $LBR_Reserve_coin2);
    assume $LBR_ReserveComponent_is_well_formed_types($Dereference($t41));

    // Reference(reserve) <- $t40
    call reserve := $WritebackToReference($t40, reserve);

    // UnpackRef($t41)
    call $LBR_ReserveComponent_before_update_inv($Coin2_Coin2_type_value(), $Dereference($t41));

    // $t42 := borrow_field<LBR::ReserveComponent<Coin2::Coin2>>.backing($t41)
    call $t42 := $BorrowField($t41, $LBR_ReserveComponent_backing);
    assume $Libra_Libra_is_well_formed_types($Dereference($t42));

    // Reference(reserve) <- $t41
    call reserve := $WritebackToReference($t41, reserve);

    // Reference($t40) <- $t41
    call $t40 := $WritebackToReference($t41, $t40);

    // UnpackRef($t42)
    call $Libra_Libra_before_update_inv($Coin2_Coin2_type_value(), $Dereference($t42));

    // PackRef($t42)
    call $Libra_Libra_after_update_inv($Coin2_Coin2_type_value(), $Dereference($t42));

    // $t52 := read_ref($t42)
    call $tmp := $ReadRef($t42);
    assume $Libra_Libra_is_well_formed($tmp);
    $t52 := $tmp;

    // $t52 := Libra::deposit<Coin2::Coin2>($t52, $t50)
    call $t52 := $Libra_deposit($Coin2_Coin2_type_value(), $t52, $t50);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 7081);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t52);


    // write_ref($t42, $t52)
    call $t42 := $WriteRef($t42, $t52);
    if (true) { assume $DebugTrackLocal(7, 7261, 5, $Dereference(reserve)); }

    // Reference(reserve) <- $t42
    call reserve := $WritebackToReference($t42, reserve);

    // Reference($t41) <- $t42
    call $t41 := $WritebackToReference($t42, $t41);

    // Reference($t40) <- $t41
    call $t40 := $WritebackToReference($t41, $t40);

    // UnpackRef($t42)
    call $Libra_Libra_before_update_inv($Coin2_Coin2_type_value(), $Dereference($t42));

    // PackRef($t41)
    call $LBR_ReserveComponent_after_update_inv($Coin2_Coin2_type_value(), $Dereference($t41));

    // PackRef($t42)
    call $Libra_Libra_after_update_inv($Coin2_Coin2_type_value(), $Dereference($t42));

    // $t45 := move(reserve)
    call $t45 := $CopyOrMoveRef(reserve);

    // $t46 := get_field<LBR::Reserve>.mint_cap($t45)
    call $tmp := $GetFieldFromReference($t45, $LBR_Reserve_mint_cap);
    assume $Libra_MintCapability_is_well_formed($tmp);
    $t46 := $tmp;

    // LBR::Reserve <- $t45
    call $WritebackToGlobal($t45);

    // PackRef($t45)
    call $LBR_Reserve_after_update_inv($Dereference($t45));

    // $t47 := Libra::mint_with_capability<LBR::LBR>($t48, $t46)
    call $t47 := $Libra_mint_with_capability($LBR_LBR_type_value(), $t48, $t46);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 7222);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t47);


    // return $t47
    $ret0 := $t47;
    if (true) { assume $DebugTrackLocal(7, 7215, 53, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $LBR_create_$direct_inter(amount_lbr: $Value, coin1: $Value, coin2: $Value) returns ($ret0: $Value)
{
    assume $IsValidU64(amount_lbr);

    assume $Libra_Libra_is_well_formed(coin1);

    assume $Libra_Libra_is_well_formed(coin2);

    call $ret0 := $LBR_create_$def(amount_lbr, coin1, coin2);
}


procedure {:inline 1} $LBR_create_$direct_intra(amount_lbr: $Value, coin1: $Value, coin2: $Value) returns ($ret0: $Value)
{
    assume $IsValidU64(amount_lbr);

    assume $Libra_Libra_is_well_formed(coin1);

    assume $Libra_Libra_is_well_formed(coin2);

    call $ret0 := $LBR_create_$def(amount_lbr, coin1, coin2);
}


procedure {:inline 1} $LBR_create(amount_lbr: $Value, coin1: $Value, coin2: $Value) returns ($ret0: $Value)
{
    assume $IsValidU64(amount_lbr);

    assume $Libra_Libra_is_well_formed(coin1);

    assume $Libra_Libra_is_well_formed(coin2);

    call $ret0 := $LBR_create_$def(amount_lbr, coin1, coin2);
}


procedure {:inline 1} $LBR_calculate_component_amounts_for_lbr_$def(amount_lbr: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    // declare local variables
    var num_coin1: $Value; // $IntegerType()
    var num_coin2: $Value; // $IntegerType()
    var reserve: $Value; // $LBR_Reserve_type_value()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $LBR_Reserve_type_value()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $LBR_Reserve_type_value()
    var $t9: $Value; // $LBR_ReserveComponent_type_value($Coin1_Coin1_type_value())
    var $t10: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t11: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $IntegerType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $LBR_Reserve_type_value()
    var $t17: $Value; // $LBR_ReserveComponent_type_value($Coin2_Coin2_type_value())
    var $t18: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t19: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $IntegerType()
    var $t22: $Value; // $IntegerType()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $IntegerType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(7, 5481, 0, amount_lbr); }

    // bytecode translation starts here
    // $t24 := move(amount_lbr)
    call $tmp := $CopyOrMoveValue(amount_lbr);
    $t24 := $tmp;

    // $t4 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t4 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5640);
      goto Abort;
    }
    assume is#$Address($t4);


    // $t5 := get_global<LBR::Reserve>($t4)
    call $tmp := $GetGlobal($t4, $LBR_Reserve_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5602);
      goto Abort;
    }
    assume $LBR_Reserve_is_well_formed($tmp);
    $t5 := $tmp;

    // reserve := $t5
    call $tmp := $CopyOrMoveValue($t5);
    reserve := $tmp;
    if (true) { assume $DebugTrackLocal(7, 5592, 3, $tmp); }

    // $t6 := 1
    $tmp := $Integer(1);
    $t6 := $tmp;

    // $t8 := copy(reserve)
    call $tmp := $CopyOrMoveValue(reserve);
    $t8 := $tmp;

    // $t9 := get_field<LBR::Reserve>.coin1($t8)
    call $tmp := $GetFieldFromValue($t8, $LBR_Reserve_coin1);
    assume $LBR_ReserveComponent_is_well_formed($tmp);
    $t9 := $tmp;

    // $t10 := get_field<LBR::ReserveComponent<Coin1::Coin1>>.ratio($t9)
    call $tmp := $GetFieldFromValue($t9, $LBR_ReserveComponent_ratio);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t10 := $tmp;

    // $t11 := move($t10)
    call $tmp := $CopyOrMoveValue($t10);
    $t11 := $tmp;

    // $t12 := FixedPoint32::multiply_u64($t24, $t11)
    call $t12 := $FixedPoint32_multiply_u64($t24, $t11);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5705);
      goto Abort;
    }
    assume $IsValidU64($t12);


    // $t13 := +($t6, $t12)
    call $tmp := $AddU64($t6, $t12);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5689);
      goto Abort;
    }
    $t13 := $tmp;

    // num_coin1 := $t13
    call $tmp := $CopyOrMoveValue($t13);
    num_coin1 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 5675, 1, $tmp); }

    // $t14 := 1
    $tmp := $Integer(1);
    $t14 := $tmp;

    // $t16 := move(reserve)
    call $tmp := $CopyOrMoveValue(reserve);
    $t16 := $tmp;

    // $t17 := get_field<LBR::Reserve>.coin2($t16)
    call $tmp := $GetFieldFromValue($t16, $LBR_Reserve_coin2);
    assume $LBR_ReserveComponent_is_well_formed($tmp);
    $t17 := $tmp;

    // $t18 := get_field<LBR::ReserveComponent<Coin2::Coin2>>.ratio($t17)
    call $tmp := $GetFieldFromValue($t17, $LBR_ReserveComponent_ratio);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t18 := $tmp;

    // $t19 := move($t18)
    call $tmp := $CopyOrMoveValue($t18);
    $t19 := $tmp;

    // $t20 := FixedPoint32::multiply_u64($t24, $t19)
    call $t20 := $FixedPoint32_multiply_u64($t24, $t19);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5796);
      goto Abort;
    }
    assume $IsValidU64($t20);


    // $t21 := +($t14, $t20)
    call $tmp := $AddU64($t14, $t20);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5780);
      goto Abort;
    }
    $t21 := $tmp;

    // num_coin2 := $t21
    call $tmp := $CopyOrMoveValue($t21);
    num_coin2 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 5766, 2, $tmp); }

    // return (num_coin1, num_coin2)
    $ret0 := num_coin1;
    if (true) { assume $DebugTrackLocal(7, 5853, 25, $ret0); }
    $ret1 := num_coin2;
    if (true) { assume $DebugTrackLocal(7, 5853, 26, $ret1); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}

procedure {:inline 1} $LBR_calculate_component_amounts_for_lbr_$direct_inter(amount_lbr: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    assume $IsValidU64(amount_lbr);

    call $ret0, $ret1 := $LBR_calculate_component_amounts_for_lbr_$def(amount_lbr);
}


procedure {:inline 1} $LBR_calculate_component_amounts_for_lbr_$direct_intra(amount_lbr: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    assume $IsValidU64(amount_lbr);

    call $ret0, $ret1 := $LBR_calculate_component_amounts_for_lbr_$def(amount_lbr);
}


procedure {:inline 1} $LBR_calculate_component_amounts_for_lbr(amount_lbr: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    assume $IsValidU64(amount_lbr);

    call $ret0, $ret1 := $LBR_calculate_component_amounts_for_lbr_$def(amount_lbr);
}


procedure {:inline 1} $LBR_is_lbr_$def($tv0: $TypeValue) returns ($ret0: $Value)
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

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t1 := Libra::is_currency<#0>()
    call $t1 := $Libra_is_currency($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5048);
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
    call $t2 := $Libra_currency_code($tv0);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5094);
      goto Abort;
    }
    assume $Vector_is_well_formed($t2) && (forall $$0: int :: {$select_vector($t2,$$0)} $$0 >= 0 && $$0 < $vlen($t2) ==> $IsValidU8($select_vector($t2,$$0)));


    // $t3 := Libra::currency_code<LBR::LBR>()
    call $t3 := $Libra_currency_code($LBR_LBR_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 5130);
      goto Abort;
    }
    assume $Vector_is_well_formed($t3) && (forall $$0: int :: {$select_vector($t3,$$0)} $$0 >= 0 && $$0 < $vlen($t3) ==> $IsValidU8($select_vector($t3,$$0)));


    // $t4 := ==($t2, $t3)
    $tmp := $Boolean($IsEqual($t2, $t3));
    $t4 := $tmp;

    // $t0 := $t4
    call $tmp := $CopyOrMoveValue($t4);
    $t0 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 5041, 0, $tmp); }

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
    if (true) { assume $DebugTrackLocal(7, 5041, 0, $tmp); }

    // goto L3
    goto L3;

    // L3:
L3:

    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(7, 5041, 7, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $LBR_is_lbr_$direct_inter($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $LBR_is_lbr_$def($tv0);
}


procedure {:inline 1} $LBR_is_lbr_$direct_intra($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $LBR_is_lbr_$def($tv0);
}


procedure {:inline 1} $LBR_is_lbr($tv0: $TypeValue) returns ($ret0: $Value)
{
    call $ret0 := $LBR_is_lbr_$def($tv0);
}


procedure {:inline 1} $LBR_reserve_address_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::CURRENCY_INFO_ADDRESS()
    call $t0 := $CoreAddresses_CURRENCY_INFO_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 8869);
      goto Abort;
    }
    assume is#$Address($t0);


    // return $t0
    $ret0 := $t0;
    if (true) { assume $DebugTrackLocal(7, 8854, 1, $ret0); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $LBR_reserve_address_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $LBR_reserve_address_$def();
}


procedure {:inline 1} $LBR_reserve_address_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $LBR_reserve_address_$def();
}


procedure {:inline 1} $LBR_reserve_address() returns ($ret0: $Value)
{
    call $ret0 := $LBR_reserve_address_$def();
}


procedure {:inline 1} $LBR_unpack_$def(coin: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    // declare local variables
    var coin1: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var coin1_amount: $Value; // $IntegerType()
    var coin2: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var coin2_amount: $Value; // $IntegerType()
    var ratio_multiplier: $Value; // $IntegerType()
    var reserve: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var sender: $Value; // $AddressType()
    var $t8: $Value; // $AddressType()
    var $t9: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t10: $Value; // $Libra_Libra_type_value($LBR_LBR_type_value())
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $AddressType()
    var $t13: $Value; // $Libra_Libra_type_value($LBR_LBR_type_value())
    var $t14: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t15: $Reference; // ReferenceType($Libra_Preburn_type_value($LBR_LBR_type_value()))
    var $t16: $Value; // $AddressType()
    var $t17: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t18: $Reference; // ReferenceType($Libra_Preburn_type_value($LBR_LBR_type_value()))
    var $t19: $Value; // $AddressType()
    var $t20: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t21: $Value; // $Libra_BurnCapability_type_value($LBR_LBR_type_value())
    var $t22: $Value; // $IntegerType()
    var $t23: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t24: $Value; // $LBR_ReserveComponent_type_value($Coin1_Coin1_type_value())
    var $t25: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t26: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t27: $Value; // $IntegerType()
    var $t28: $Value; // $IntegerType()
    var $t29: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t30: $Value; // $LBR_ReserveComponent_type_value($Coin2_Coin2_type_value())
    var $t31: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t32: $Value; // $FixedPoint32_FixedPoint32_type_value()
    var $t33: $Value; // $IntegerType()
    var $t34: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t35: $Reference; // ReferenceType($LBR_ReserveComponent_type_value($Coin1_Coin1_type_value()))
    var $t36: $Reference; // ReferenceType($Libra_Libra_type_value($Coin1_Coin1_type_value()))
    var $t37: $Value; // $IntegerType()
    var $t38: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t39: $Reference; // ReferenceType($LBR_Reserve_type_value())
    var $t40: $Reference; // ReferenceType($LBR_ReserveComponent_type_value($Coin2_Coin2_type_value()))
    var $t41: $Reference; // ReferenceType($Libra_Libra_type_value($Coin2_Coin2_type_value()))
    var $t42: $Value; // $IntegerType()
    var $t43: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t44: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t45: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t46: $Value; // $Libra_Libra_type_value($LBR_LBR_type_value())
    var $t47: $Value; // $Libra_Libra_type_value($Coin1_Coin1_type_value())
    var $t48: $Value; // $Libra_Libra_type_value($Coin2_Coin2_type_value())
    var $t49: $Value; // $Libra_Preburn_type_value($LBR_LBR_type_value())
    var $tmp: $Value;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) { assume $DebugTrackLocal(7, 7891, 0, coin); }

    // bytecode translation starts here
    // $t46 := move(coin)
    call $tmp := $CopyOrMoveValue(coin);
    $t46 := $tmp;

    // $t8 := CoreAddresses::LIBRA_ROOT_ADDRESS()
    call $t8 := $CoreAddresses_LIBRA_ROOT_ADDRESS();
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 8044);
      goto Abort;
    }
    assume is#$Address($t8);


    // $t9 := borrow_global<LBR::Reserve>($t8)
    call $t9 := $BorrowGlobal($t8, $LBR_Reserve_type_value());
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 8002);
      goto Abort;
    }
    assume $LBR_Reserve_is_well_formed($Dereference($t9));

    // UnpackRef($t9)
    call $LBR_Reserve_before_update_inv($Dereference($t9));

    // reserve := $t9
    call reserve := $CopyOrMoveRef($t9);
    if (true) { assume $DebugTrackLocal(7, 7992, 6, $Dereference(reserve)); }

    // $t10 := copy($t46)
    call $tmp := $CopyOrMoveValue($t46);
    $t10 := $tmp;

    // $t11 := Libra::value<LBR::LBR>($t10)
    call $t11 := $Libra_value($LBR_LBR_type_value(), $t10);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 8105);
      goto Abort;
    }
    assume $IsValidU64($t11);


    // ratio_multiplier := $t11
    call $tmp := $CopyOrMoveValue($t11);
    ratio_multiplier := $tmp;
    if (true) { assume $DebugTrackLocal(7, 8079, 5, $tmp); }

    // $t12 := LBR::reserve_address()
    call $t12 := $LBR_reserve_address();
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 8817);
      goto Abort;
    }
    assume is#$Address($t12);


    // sender := $t12
    call $tmp := $CopyOrMoveValue($t12);
    sender := $tmp;
    if (true) { assume $DebugTrackLocal(7, 8131, 7, $tmp); }

    // $t14 := copy(reserve)
    call $t14 := $CopyOrMoveRef(reserve);

    // $t15 := borrow_field<LBR::Reserve>.preburn_cap($t14)
    call $t15 := $BorrowField($t14, $LBR_Reserve_preburn_cap);
    assume $Libra_Preburn_is_well_formed_types($Dereference($t15));

    // Reference(reserve) <- $t14
    call reserve := $WritebackToReference($t14, reserve);

    // UnpackRef($t15)
    call $Libra_Preburn_before_update_inv($LBR_LBR_type_value(), $Dereference($t15));

    // PackRef($t15)
    call $Libra_Preburn_after_update_inv($LBR_LBR_type_value(), $Dereference($t15));

    // $t49 := read_ref($t15)
    call $tmp := $ReadRef($t15);
    assume $Libra_Preburn_is_well_formed($tmp);
    $t49 := $tmp;

    // $t49 := Libra::preburn_with_resource<LBR::LBR>($t46, $t49, sender)
    call $t49 := $Libra_preburn_with_resource($LBR_LBR_type_value(), $t46, $t49, sender);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 8174);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($t49);


    // write_ref($t15, $t49)
    call $t15 := $WriteRef($t15, $t49);
    if (true) { assume $DebugTrackLocal(7, 8636, 6, $Dereference(reserve)); }

    // Reference(reserve) <- $t15
    call reserve := $WritebackToReference($t15, reserve);

    // Reference($t14) <- $t15
    call $t14 := $WritebackToReference($t15, $t14);

    // UnpackRef($t15)
    call $Libra_Preburn_before_update_inv($LBR_LBR_type_value(), $Dereference($t15));

    // PackRef($t15)
    call $Libra_Preburn_after_update_inv($LBR_LBR_type_value(), $Dereference($t15));

    // $t17 := copy(reserve)
    call $t17 := $CopyOrMoveRef(reserve);

    // $t18 := borrow_field<LBR::Reserve>.preburn_cap($t17)
    call $t18 := $BorrowField($t17, $LBR_Reserve_preburn_cap);
    assume $Libra_Preburn_is_well_formed_types($Dereference($t18));

    // Reference(reserve) <- $t17
    call reserve := $WritebackToReference($t17, reserve);

    // UnpackRef($t18)
    call $Libra_Preburn_before_update_inv($LBR_LBR_type_value(), $Dereference($t18));

    // $t20 := copy(reserve)
    call $t20 := $CopyOrMoveRef(reserve);

    // $t21 := get_field<LBR::Reserve>.burn_cap($t20)
    call $tmp := $GetFieldFromReference($t20, $LBR_Reserve_burn_cap);
    assume $Libra_BurnCapability_is_well_formed($tmp);
    $t21 := $tmp;

    // Reference(reserve) <- $t20
    call reserve := $WritebackToReference($t20, reserve);

    // PackRef($t18)
    call $Libra_Preburn_after_update_inv($LBR_LBR_type_value(), $Dereference($t18));

    // $t49 := read_ref($t18)
    call $tmp := $ReadRef($t18);
    assume $Libra_Preburn_is_well_formed($tmp);
    $t49 := $tmp;

    // $t49 := Libra::burn_with_resource_cap<LBR::LBR>($t49, sender, $t21)
    call $t49 := $Libra_burn_with_resource_cap($LBR_LBR_type_value(), $t49, sender, $t21);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 8252);
      goto Abort;
    }
    assume $Libra_Preburn_is_well_formed($t49);


    // write_ref($t18, $t49)
    call $t18 := $WriteRef($t18, $t49);
    if (true) { assume $DebugTrackLocal(7, 8627, 6, $Dereference(reserve)); }

    // Reference(reserve) <- $t18
    call reserve := $WritebackToReference($t18, reserve);

    // Reference($t17) <- $t18
    call $t17 := $WritebackToReference($t18, $t17);

    // Reference($t20) <- $t18
    call $t20 := $WritebackToReference($t18, $t20);

    // UnpackRef($t18)
    call $Libra_Preburn_before_update_inv($LBR_LBR_type_value(), $Dereference($t18));

    // PackRef($t18)
    call $Libra_Preburn_after_update_inv($LBR_LBR_type_value(), $Dereference($t18));

    // $t23 := copy(reserve)
    call $t23 := $CopyOrMoveRef(reserve);

    // $t24 := get_field<LBR::Reserve>.coin1($t23)
    call $tmp := $GetFieldFromReference($t23, $LBR_Reserve_coin1);
    assume $LBR_ReserveComponent_is_well_formed($tmp);
    $t24 := $tmp;

    // Reference(reserve) <- $t23
    call reserve := $WritebackToReference($t23, reserve);

    // $t25 := get_field<LBR::ReserveComponent<Coin1::Coin1>>.ratio($t24)
    call $tmp := $GetFieldFromValue($t24, $LBR_ReserveComponent_ratio);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t25 := $tmp;

    // $t26 := move($t25)
    call $tmp := $CopyOrMoveValue($t25);
    $t26 := $tmp;

    // $t27 := FixedPoint32::multiply_u64(ratio_multiplier, $t26)
    call $t27 := $FixedPoint32_multiply_u64(ratio_multiplier, $t26);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 8370);
      goto Abort;
    }
    assume $IsValidU64($t27);


    // coin1_amount := $t27
    call $tmp := $CopyOrMoveValue($t27);
    coin1_amount := $tmp;
    if (true) { assume $DebugTrackLocal(7, 8341, 2, $tmp); }

    // $t29 := copy(reserve)
    call $t29 := $CopyOrMoveRef(reserve);

    // $t30 := get_field<LBR::Reserve>.coin2($t29)
    call $tmp := $GetFieldFromReference($t29, $LBR_Reserve_coin2);
    assume $LBR_ReserveComponent_is_well_formed($tmp);
    $t30 := $tmp;

    // Reference(reserve) <- $t29
    call reserve := $WritebackToReference($t29, reserve);

    // $t31 := get_field<LBR::ReserveComponent<Coin2::Coin2>>.ratio($t30)
    call $tmp := $GetFieldFromValue($t30, $LBR_ReserveComponent_ratio);
    assume $FixedPoint32_FixedPoint32_is_well_formed($tmp);
    $t31 := $tmp;

    // $t32 := move($t31)
    call $tmp := $CopyOrMoveValue($t31);
    $t32 := $tmp;

    // $t33 := FixedPoint32::multiply_u64(ratio_multiplier, $t32)
    call $t33 := $FixedPoint32_multiply_u64(ratio_multiplier, $t32);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 8466);
      goto Abort;
    }
    assume $IsValidU64($t33);


    // coin2_amount := $t33
    call $tmp := $CopyOrMoveValue($t33);
    coin2_amount := $tmp;
    if (true) { assume $DebugTrackLocal(7, 8437, 4, $tmp); }

    // $t34 := copy(reserve)
    call $t34 := $CopyOrMoveRef(reserve);

    // $t35 := borrow_field<LBR::Reserve>.coin1($t34)
    call $t35 := $BorrowField($t34, $LBR_Reserve_coin1);
    assume $LBR_ReserveComponent_is_well_formed_types($Dereference($t35));

    // Reference(reserve) <- $t34
    call reserve := $WritebackToReference($t34, reserve);

    // UnpackRef($t35)
    call $LBR_ReserveComponent_before_update_inv($Coin1_Coin1_type_value(), $Dereference($t35));

    // $t36 := borrow_field<LBR::ReserveComponent<Coin1::Coin1>>.backing($t35)
    call $t36 := $BorrowField($t35, $LBR_ReserveComponent_backing);
    assume $Libra_Libra_is_well_formed_types($Dereference($t36));

    // Reference(reserve) <- $t35
    call reserve := $WritebackToReference($t35, reserve);

    // Reference($t34) <- $t35
    call $t34 := $WritebackToReference($t35, $t34);

    // UnpackRef($t36)
    call $Libra_Libra_before_update_inv($Coin1_Coin1_type_value(), $Dereference($t36));

    // PackRef($t36)
    call $Libra_Libra_after_update_inv($Coin1_Coin1_type_value(), $Dereference($t36));

    // $t47 := read_ref($t36)
    call $tmp := $ReadRef($t36);
    assume $Libra_Libra_is_well_formed($tmp);
    $t47 := $tmp;

    // ($t38, $t47) := Libra::withdraw<Coin1::Coin1>($t47, coin1_amount)
    call $t38, $t47 := $Libra_withdraw($Coin1_Coin1_type_value(), $t47, coin1_amount);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 8548);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t38);

    assume $Libra_Libra_is_well_formed($t47);


    // write_ref($t36, $t47)
    call $t36 := $WriteRef($t36, $t47);
    if (true) { assume $DebugTrackLocal(7, 8688, 6, $Dereference(reserve)); }

    // Reference(reserve) <- $t36
    call reserve := $WritebackToReference($t36, reserve);

    // Reference($t35) <- $t36
    call $t35 := $WritebackToReference($t36, $t35);

    // Reference($t34) <- $t35
    call $t34 := $WritebackToReference($t35, $t34);

    // UnpackRef($t36)
    call $Libra_Libra_before_update_inv($Coin1_Coin1_type_value(), $Dereference($t36));

    // PackRef($t35)
    call $LBR_ReserveComponent_after_update_inv($Coin1_Coin1_type_value(), $Dereference($t35));

    // PackRef($t36)
    call $Libra_Libra_after_update_inv($Coin1_Coin1_type_value(), $Dereference($t36));

    // coin1 := $t38
    call $tmp := $CopyOrMoveValue($t38);
    coin1 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 8533, 1, $tmp); }

    // $t39 := move(reserve)
    call $t39 := $CopyOrMoveRef(reserve);

    // $t40 := borrow_field<LBR::Reserve>.coin2($t39)
    call $t40 := $BorrowField($t39, $LBR_Reserve_coin2);
    assume $LBR_ReserveComponent_is_well_formed_types($Dereference($t40));

    // LBR::Reserve <- $t39
    call $WritebackToGlobal($t39);

    // UnpackRef($t40)
    call $LBR_ReserveComponent_before_update_inv($Coin2_Coin2_type_value(), $Dereference($t40));

    // $t41 := borrow_field<LBR::ReserveComponent<Coin2::Coin2>>.backing($t40)
    call $t41 := $BorrowField($t40, $LBR_ReserveComponent_backing);
    assume $Libra_Libra_is_well_formed_types($Dereference($t41));

    // LBR::Reserve <- $t40
    call $WritebackToGlobal($t40);

    // Reference($t39) <- $t40
    call $t39 := $WritebackToReference($t40, $t39);

    // UnpackRef($t41)
    call $Libra_Libra_before_update_inv($Coin2_Coin2_type_value(), $Dereference($t41));

    // PackRef($t41)
    call $Libra_Libra_after_update_inv($Coin2_Coin2_type_value(), $Dereference($t41));

    // $t48 := read_ref($t41)
    call $tmp := $ReadRef($t41);
    assume $Libra_Libra_is_well_formed($tmp);
    $t48 := $tmp;

    // ($t43, $t48) := Libra::withdraw<Coin2::Coin2>($t48, coin2_amount)
    call $t43, $t48 := $Libra_withdraw($Coin2_Coin2_type_value(), $t48, coin2_amount);
    if ($abort_flag) {
      assume $DebugTrackAbort(7, 8627);
      goto Abort;
    }
    assume $Libra_Libra_is_well_formed($t43);

    assume $Libra_Libra_is_well_formed($t48);


    // write_ref($t41, $t48)
    call $t41 := $WriteRef($t41, $t48);
    if (true) { assume $DebugTrackLocal(7, 8687, 6, $Dereference(reserve)); }

    // LBR::Reserve <- $t41
    call $WritebackToGlobal($t41);

    // Reference($t40) <- $t41
    call $t40 := $WritebackToReference($t41, $t40);

    // Reference($t39) <- $t40
    call $t39 := $WritebackToReference($t40, $t39);

    // UnpackRef($t41)
    call $Libra_Libra_before_update_inv($Coin2_Coin2_type_value(), $Dereference($t41));

    // PackRef($t39)
    call $LBR_Reserve_after_update_inv($Dereference($t39));

    // PackRef($t40)
    call $LBR_ReserveComponent_after_update_inv($Coin2_Coin2_type_value(), $Dereference($t40));

    // PackRef($t41)
    call $Libra_Libra_after_update_inv($Coin2_Coin2_type_value(), $Dereference($t41));

    // coin2 := $t43
    call $tmp := $CopyOrMoveValue($t43);
    coin2 := $tmp;
    if (true) { assume $DebugTrackLocal(7, 8612, 3, $tmp); }

    // return (coin1, coin2)
    $ret0 := coin1;
    if (true) { assume $DebugTrackLocal(7, 8687, 50, $ret0); }
    $ret1 := coin2;
    if (true) { assume $DebugTrackLocal(7, 8687, 51, $ret1); }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}

procedure {:inline 1} $LBR_unpack_$direct_inter(coin: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    call $ret0, $ret1 := $LBR_unpack_$def(coin);
}


procedure {:inline 1} $LBR_unpack_$direct_intra(coin: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    call $ret0, $ret1 := $LBR_unpack_$def(coin);
}


procedure {:inline 1} $LBR_unpack(coin: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    assume $Libra_Libra_is_well_formed(coin);

    call $ret0, $ret1 := $LBR_unpack_$def(coin);
}
