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
function $DebugTrackAbort(file_id: int, byte_index: int, code: int) : bool {
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
function {:constructor} $StructType(name: $TypeName, ts: $TypeValueArray) : $TypeValue;
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
function {:constructor} $Vector(v: $ValueArray): $Value; // used to both represent Move Struct and Vector
function {:constructor} $Range(lb: $Value, ub: $Value): $Value;
function {:constructor} $Type(t: $TypeValue): $Value;
function {:constructor} $Error(): $Value;

function {:inline} $DefaultValue(): $Value { $Error() }
function {:builtin "MapConst"} $MapConstValue(v: $Value): [int]$Value;

function {:inline} $IsValidU8(v: $Value): bool {
  is#$Integer(v) && i#$Integer(v) >= 0 && i#$Integer(v) <= $MAX_U8
}

function {:inline} $IsValidU8Vector(vec: $Value): bool {
  $Vector_$is_well_formed(vec)
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


{{#if backend.vector_using_sequences}}

// This is the implementation of $ValueArray using sequences

type {:datatype} {:builtin "(Seq T@$Value)"} $ValueArray;

function {:builtin "(as seq.empty (Seq T@$Value))"} $EmptyValueArray(): $ValueArray;

function {:builtin "seq.nth"} $ReadValueArray(a: $ValueArray, i: int): $Value;

function {:builtin "seq.len"} $LenValueArray(a: $ValueArray): int;

function {:builtin "seq.extract"} $ValueArrayExtract(a: $ValueArray, offset: int, length: int): $ValueArray;

function {:builtin "seq.++"} $ConcatValueArray(a: $ValueArray, b:$ValueArray): $ValueArray;

function {:builtin "seq.unit"} $UnitValueArray(v: $Value): $ValueArray;


function {{backend.func_inline}} $RemoveValueArray(a: $ValueArray): $ValueArray {
    (
        $ValueArrayExtract(a, 0, $LenValueArray(a) -1)
    )
}


function {{backend.func_inline}} $RemoveIndexValueArray(a: $ValueArray, i: int): $ValueArray {
    (
        var prefix := $ValueArrayExtract(a, 0, i); (
        var suffix := $ValueArrayExtract(a, i+1, $LenValueArray(a)-i-1);
        $ConcatValueArray(prefix, suffix)
    ))
}

// Note: The axioms for $ReverseValueArray are written assuming specific degenerate behaviour in the z3 operators underneath the functions used.
// This behaviour could change with a different version of z3 or a different solver.
// The implementation favours seq.extract over seq.nth because of weird behaviour in the latter for accesses outside the length of the sequence.
function {{backend.func_inline}} $ReverseValueArray(a: $ValueArray): $ValueArray;
axiom (forall a: $ValueArray :: $LenValueArray($ReverseValueArray(a)) == $LenValueArray(a));
axiom (forall a: $ValueArray :: (forall i: int :: $ValueArrayExtract(a, i, 1) == $ValueArrayExtract(a, $LenValueArray(a)-1-i, 1)));

function {{backend.func_inline}} $SliceValueArray(a: $ValueArray, i: int, j: int): $ValueArray { // return the sliced vector of a for the range [i, j)
    (
        $ValueArrayExtract(a, i, j-i)
    )
}

function {{backend.func_inline}} $ExtendValueArray(a: $ValueArray, elem: $Value): $ValueArray {
    (
        $ConcatValueArray(a, $UnitValueArray(elem))
    )
}

function {{backend.func_inline}} $UpdateValueArray(a: $ValueArray, i: int, elem: $Value): $ValueArray {
    (
        var prefix := $ValueArrayExtract(a, 0, i); (
        var suffix := $ValueArrayExtract(a, i+1, $LenValueArray(a)-i-1);
        $ConcatValueArray(prefix, $ConcatValueArray($UnitValueArray(elem), suffix))
    ))
}

function {{backend.func_inline}} $SwapValueArray(a: $ValueArray, i: int, j: int): $ValueArray {
    (
        var lower := if i < j then i else j; (
        var upper := if i < j then j else i; (
        var beginning := $ValueArrayExtract(a, 0, lower); (
        var middle := $ValueArrayExtract(a, lower+1, (upper-lower)-1); (
        var end := $ValueArrayExtract(a, upper+1, $LenValueArray(a)-upper-1); (
        var swap_first_part := $ConcatValueArray(beginning, $ValueArrayExtract(a,upper,1)); (
        var swap_last_part := $ConcatValueArray($ValueArrayExtract(a,lower,1), end);
        $ConcatValueArray(swap_first_part, $ConcatValueArray(middle, swap_last_part))
    )))))))
}

function {:inline} $IsEmpty(a: $ValueArray): bool {
    $LenValueArray(a) == 0
}

// All invalid elements of array are DefaultValue. This is useful in specialized
// cases. This is used to defined normalization for $Vector
// For sequences this is true by default
function {:inline} $IsNormalizedValueArray(a: $ValueArray, len: int): bool {
    (
        true
    )
}


{{else}}


// This is the implementation of $ValueArray using integer maps

type {:datatype} $ValueArray;

function {:constructor} $ValueArray(v: [int]$Value, l: int): $ValueArray;

function $EmptyValueArray(): $ValueArray;
axiom l#$ValueArray($EmptyValueArray()) == 0;
axiom v#$ValueArray($EmptyValueArray()) == $MapConstValue($Error());

function {{backend.func_inline}} $ReadValueArray(a: $ValueArray, i: int): $Value {
    (
        v#$ValueArray(a)[i]
    )
}

function {{backend.func_inline}} $LenValueArray(a: $ValueArray): int {
    (
        l#$ValueArray(a)
    )
}

function {{backend.func_inline}} $RemoveValueArray(a: $ValueArray): $ValueArray {
    (
        var l := l#$ValueArray(a) - 1;
        $ValueArray(
            (lambda i: int ::
                if i >= 0 && i < l then v#$ValueArray(a)[i] else $DefaultValue()),
            l
        )
    )
}

function {{backend.func_inline}} $SingleValueArray(v: $Value): $ValueArray {
    $ValueArray($MapConstValue($DefaultValue())[0 := v], 1)
}

function {{backend.func_inline}} $RemoveIndexValueArray(a: $ValueArray, i: int): $ValueArray {
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

function {{backend.func_inline}} $ConcatValueArray(a1: $ValueArray, a2: $ValueArray): $ValueArray {
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

function {{backend.func_inline}} $ReverseValueArray(a: $ValueArray): $ValueArray {
    (
        var l := l#$ValueArray(a);
        $ValueArray(
            (lambda i: int :: if 0 <= i && i < l then v#$ValueArray(a)[l - i - 1] else $DefaultValue()),
            l
        )
    )
}

function {{backend.func_inline}} $SliceValueArray(a: $ValueArray, i: int, j: int): $ValueArray { // return the sliced vector of a for the range [i, j)
    $ValueArray((lambda k:int :: if 0 <= k && k < j-i then v#$ValueArray(a)[i+k] else $DefaultValue()), (if j-i < 0 then 0 else j-i))
}

function {{backend.func_inline}} $ExtendValueArray(a: $ValueArray, elem: $Value): $ValueArray {
    (var len := l#$ValueArray(a);
     $ValueArray(v#$ValueArray(a)[len := elem], len + 1))
}

function {{backend.func_inline}} $UpdateValueArray(a: $ValueArray, i: int, elem: $Value): $ValueArray {
    $ValueArray(v#$ValueArray(a)[i := elem], l#$ValueArray(a))
}

function {{backend.func_inline}} $SwapValueArray(a: $ValueArray, i: int, j: int): $ValueArray {
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


{{/if}} //end of backend.vector_using_sequences


// Stratified Functions on Values
// ------------------------------

// TODO: templatize this or move it back to the translator. For now we
//   prefer to handcode this so its easier to evolve the model independent of the
//   translator.

const $StratificationDepth: int;
axiom $StratificationDepth == {{backend.stratification_depth}};

{{#if backend.native_equality}}

// Map IsEqual to native Boogie equality. This only works with extensional arrays as provided
// by the array theory.
function {:inline} $IsEqual(v1: $Value, v2: $Value): bool {
    v1 == v2
}

{{else}}

// Generate a stratified version of IsEqual for depth of {{backend.stratification_depth}}.
{{#stratified}}
function {{backend.aggressive_func_inline}} $IsEqual_{{@this_suffix}}(v1: $Value, v2: $Value): bool {
    (v1 == v2) ||
    (is#$Vector(v1) &&
     is#$Vector(v2) &&
     $vlen(v1) == $vlen(v2) &&
     (forall i: int :: 0 <= i && i < $vlen(v1) ==> $IsEqual_{{@next_suffix}}($select_vector(v1,i), $select_vector(v2,i))))
}
{{else}}
function {:inline} $IsEqual_{{@this_suffix}}(v1: $Value, v2: $Value): bool {
    v1 == v2
}
{{/stratified}}

function {:inline} $IsEqual(v1: $Value, v2: $Value): bool {
    $IsEqual_stratified(v1, v2)
}

{{/if}}

// Generate stratified ReadValue for the depth of {{backend.stratification_depth}}.

{{#stratified}}
function {{backend.aggressive_func_inline}} $ReadValue_{{@this_suffix}}(p: $Path, v: $Value) : $Value {
    if ({{@this_level}} == size#$Path(p)) then
        v
    else
        $ReadValue_{{@next_suffix}}(p, $select_vector(v,$path_index_at(p, {{@this_level}})))
}
{{else}}
function {:inline} $ReadValue_{{@this_suffix}}(p: $Path, v: $Value): $Value {
    v
}
{{/stratified}}

function {:inline} $ReadValue(p: $Path, v: $Value): $Value {
    $ReadValue_stratified(p, v)
}

// Generate stratified $UpdateValue for the depth of {{backend.stratification_depth}}.

{{#stratified}}
function {{backend.aggressive_func_inline}} $UpdateValue_{{@this_suffix}}(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    (var poffset := offset + {{@this_level}};
    if (poffset == size#$Path(p)) then
        new_v
    else
        $update_vector(v, $path_index_at(p, poffset),
                       $UpdateValue_{{@next_suffix}}(p, offset, $select_vector(v,$path_index_at(p, poffset)), new_v)))
}
{{else}}
function {:inline} $UpdateValue_{{@this_suffix}}(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    new_v
}
{{/stratified}}

function {:inline} $UpdateValue(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    $UpdateValue_stratified(p, offset, v, new_v)
}

// Generate stratified $IsPathPrefix for the depth of {{backend.stratification_depth}}.

{{#stratified}}
function {{backend.aggressive_func_inline}} $IsPathPrefix_{{@this_suffix}}(p1: $Path, p2: $Path): bool {
    if ({{@this_level}} == size#$Path(p1)) then
        true
    else if (p#$Path(p1)[{{@this_level}}] == p#$Path(p2)[{{@this_level}}]) then
        $IsPathPrefix_{{@next_suffix}}(p1, p2)
    else
        false
}
{{else}}
function {:inline} $IsPathPrefix_{{@this_suffix}}(p1: $Path, p2: $Path): bool {
    true
}
{{/stratified}}

function {:inline} $IsPathPrefix(p1: $Path, p2: $Path): bool {
    $IsPathPrefix_stratified(p1, p2)
}

// Generate stratified $ConcatPath for the depth of {{backend.stratification_depth}}.

{{#stratified}}
function {{backend.aggressive_func_inline}} $ConcatPath_{{@this_suffix}}(p1: $Path, p2: $Path): $Path {
    if ({{@this_level}} == size#$Path(p2)) then
        p1
    else
        $ConcatPath_{{@next_suffix}}($Path(p#$Path(p1)[size#$Path(p1) := p#$Path(p2)[{{@this_level}}]], size#$Path(p1) + 1), p2)
}
{{else}}
function {:inline} $ConcatPath_{{@this_suffix}}(p1: $Path, p2: $Path): $Path {
    p1
}
{{/stratified}}

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
function {:inline} $single_vector(v: $Value): $Value {
    $Vector($SingleValueArray(v))
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

// A global resource location within the statically known resource type's memory.
// `ts` are the type parameters for the outer type, and `a` is the address.
function {:constructor} $Global(ts: $TypeValueArray, a: int): $Location;

// A local location. `i` is the unique index of the local.
function {:constructor} $Local(i: int): $Location;

// The location of a reference outside of the verification scope, for example, a `&mut` parameter
// of the function being verified. References with these locations don't need to be written back
// when mutation ends.
function {:constructor} $Param(i: int): $Location;


// A mutable reference which also carries its current value. Since mutable references
// are single threaded in Move, we can keep them together and treat them as a value
// during mutation until the point they are stored back to their original location.
type {:datatype} $Mutation;
function {:constructor} $Mutation(l: $Location, p: $Path, v: $Value): $Mutation;
const $DefaultMutation: $Mutation;

// Representation of memory for a given type. The maps take the content of a Global location.
type {:datatype} $Memory;
function {:constructor} $Memory(domain: [$TypeValueArray, int]bool, contents: [$TypeValueArray, int]$Value): $Memory;

function {:inline} $Memory__is_well_formed(m: $Memory): bool {
    true
}

function {:builtin "MapConst"} $ConstMemoryDomain(v: bool): [$TypeValueArray, int]bool;
function {:builtin "MapConst"} $ConstMemoryContent(v: $Value): [$TypeValueArray, int]$Value;
axiom $ConstMemoryDomain(false) == (lambda ta: $TypeValueArray, i: int :: false);
axiom $ConstMemoryDomain(true) == (lambda ta: $TypeValueArray, i: int :: true);

const $EmptyMemory: $Memory;
axiom domain#$Memory($EmptyMemory) == $ConstMemoryDomain(false);
axiom contents#$Memory($EmptyMemory) == $ConstMemoryContent($DefaultValue());

var $abort_flag: bool;
var $abort_code: int;

function {:inline} $process_abort_code(code: int): int {
    code
}

const $EXEC_FAILURE_CODE: int;
axiom $EXEC_FAILURE_CODE == -1;

// TODO(wrwg): currently we map aborts of native functions like those for vectors also to
//   execution failure. This may need to be aligned with what the runtime actually does.

procedure {:inline 1} $ExecFailureAbort() {
    $abort_flag := true;
    $abort_code := $EXEC_FAILURE_CODE;
}

procedure {:inline 1} $InitVerification() {
  // Set abort_flag to false, and havoc abort_code
  $abort_flag := false;
  havoc $abort_code;
}

// ============================================================================================
// Functional APIs

// TODO: unify some of this with instruction procedures to avoid duplication

// Tests whether resource exists.
function {:inline} $ResourceExistsRaw(m: $Memory, args: $TypeValueArray, addr: int): bool {
    domain#$Memory(m)[args, addr]
}
function {:inline} $ResourceExists(m: $Memory, args: $TypeValueArray, addr: $Value): $Value {
    $Boolean($ResourceExistsRaw(m, args, a#$Address(addr)))
}

// Obtains Value of given resource.
function {:inline} $ResourceValue(m: $Memory, args: $TypeValueArray, addr: $Value): $Value {
  contents#$Memory(m)[args, a#$Address(addr)]
}

// Applies a field selection to a Value.
function {:inline} $SelectField(val: $Value, field: $FieldName): $Value {
    $select_vector(val, field)
}

// Updates a field.
function {:inline} $UpdateField(val: $Value, field: $FieldName, new_value: $Value): $Value {
    $update_vector(val, field, new_value)
}


// Dereferences a reference.
function {:inline} $Dereference(ref: $Mutation): $Value {
    v#$Mutation(ref)
}

// ============================================================================================
// Instructions

procedure {:inline 1} $MoveToRaw(m: $Memory, ta: $TypeValueArray, a: int, v: $Value) returns (m': $Memory)
{
    if ($ResourceExistsRaw(m, ta, a)) {
        call $ExecFailureAbort();
        return;
    }
    m' := $Memory(domain#$Memory(m)[ta, a := true], contents#$Memory(m)[ta, a := v]);
}

procedure {:inline 1} $MoveTo(m: $Memory, ta: $TypeValueArray, v: $Value, signer: $Value) returns (m': $Memory)
{
    var address: $Value;
    var a: int;

    call address := $Signer_borrow_address(signer);
    a := a#$Address(address);
    call m' := $MoveToRaw(m, ta, a, v);
}

procedure {:inline 1} $MoveFrom(m: $Memory, address: $Value, ta: $TypeValueArray) returns (m': $Memory, dst: $Value)
{{backend.type_requires}} is#$Address(address);
{
    var a: int;

    a := a#$Address(address);
    if (!$ResourceExistsRaw(m, ta, a)) {
        call $ExecFailureAbort();
        return;
    }
    dst := contents#$Memory(m)[ta, a];
    m' := $Memory(domain#$Memory(m)[ta, a := false], contents#$Memory(m)[ta, a := $DefaultValue()]);
}

procedure {:inline 1} $BorrowGlobal(m: $Memory, address: $Value, ta: $TypeValueArray) returns (dst: $Mutation)
{{backend.type_requires}} is#$Address(address);
{
    var a: int;

    a := a#$Address(address);
    if (!$ResourceExistsRaw(m, ta, a)) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Mutation($Global(ta, a), $EmptyPath, contents#$Memory(m)[ta, a]);
}

procedure {:inline 1} $BorrowLoc(l: int, v: $Value) returns (dst: $Mutation)
{
    dst := $Mutation($Local(l), $EmptyPath, v);
}

procedure {:inline 1} $BorrowField(src: $Mutation, f: $FieldName) returns (dst: $Mutation)
{
    var p: $Path;
    var size: int;

    p := p#$Mutation(src);
    size := size#$Path(p);
    p := $Path(p#$Path(p)[size := f], size+1);
    dst := $Mutation(l#$Mutation(src), p, $select_vector(v#$Mutation(src), f));
}

procedure {:inline 1} $GetGlobal(m: $Memory, address: $Value, ta: $TypeValueArray) returns (dst: $Value)
{{backend.type_requires}} is#$Address(address);
{
    var a: int;
    a := a#$Address(address);
    if (!$ResourceExistsRaw(m, ta, a)) {
        call $ExecFailureAbort();
        return;
    }
    dst := $ResourceValue(m, ta, address);
}

procedure {:inline 1} $GetFieldFromReference(src: $Mutation, f: $FieldName) returns (dst: $Value)
{
    var r: $Mutation;

    call r := $BorrowField(src, f);
    call dst := $ReadRef(r);
}

procedure {:inline 1} $GetFieldFromValue(src: $Value, f: $FieldName) returns (dst: $Value)
{
    dst := $select_vector(src, f);
}

procedure {:inline 1} $WriteRef(to: $Mutation, new_v: $Value) returns (to': $Mutation)
{
    to' := $Mutation(l#$Mutation(to), p#$Mutation(to), new_v);
}

procedure {:inline 1} $ReadRef(from: $Mutation) returns (v: $Value)
{
    v := v#$Mutation(from);
}

procedure {:inline 1} $CopyOrMoveRef(local: $Mutation) returns (dst: $Mutation)
{
    dst := local;
}

procedure {:inline 1} $CopyOrMoveValue(local: $Value) returns (dst: $Value)
{
    dst := local;
}

procedure {:inline 1} $WritebackToGlobal(m: $Memory, src: $Mutation) returns (m': $Memory)
{
    var l: $Location;
    var ta: $TypeValueArray;
    var a: int;
    var v: $Value;

    l := l#$Mutation(src);
    if (is#$Global(l)) {
        ta := ts#$Global(l);
        a := a#$Global(l);
        v := $UpdateValue(p#$Mutation(src), 0, contents#$Memory(m)[ta, a], v#$Mutation(src));
        m' := $Memory(domain#$Memory(m), contents#$Memory(m)[ta, a := v]);
    } else {
        m' := m;
    }
}

procedure {:inline 1} $WritebackToValue(src: $Mutation, idx: int, vdst: $Value) returns (vdst': $Value)
{
    if (l#$Mutation(src) == $Local(idx)) {
        vdst' := $UpdateValue(p#$Mutation(src), 0, vdst, v#$Mutation(src));
    } else {
        vdst' := vdst;
    }
}

procedure {:inline 1} $WritebackToReference(src: $Mutation, dst: $Mutation) returns (dst': $Mutation)
{
    var srcPath, dstPath: $Path;

    srcPath := p#$Mutation(src);
    dstPath := p#$Mutation(dst);
    if (l#$Mutation(dst) == l#$Mutation(src) && size#$Path(dstPath) <= size#$Path(srcPath) && $IsPathPrefix(dstPath, srcPath)) {
        dst' := $Mutation(
                    l#$Mutation(dst),
                    dstPath,
                    $UpdateValue(srcPath, size#$Path(dstPath), v#$Mutation(dst), v#$Mutation(src)));
    } else {
        dst' := dst;
    }
}

procedure {:inline 1} $Splice1(idx1: int, src1: $Mutation, dst: $Mutation) returns (dst': $Mutation) {
    dst' := $Mutation(l#$Mutation(src1), $ConcatPath(p#$Mutation(src1), p#$Mutation(dst)), v#$Mutation(dst));
}

procedure {:inline 1} $CastU8(src: $Value) returns (dst: $Value)
{{backend.type_requires}} is#$Integer(src);
{
    if (i#$Integer(src) > $MAX_U8) {
        call $ExecFailureAbort();
        return;
    }
    dst := src;
}

procedure {:inline 1} $CastU64(src: $Value) returns (dst: $Value)
{{backend.type_requires}} is#$Integer(src);
{
    if (i#$Integer(src) > $MAX_U64) {
        call $ExecFailureAbort();
        return;
    }
    dst := src;
}

procedure {:inline 1} $CastU128(src: $Value) returns (dst: $Value)
{{backend.type_requires}} is#$Integer(src);
{
    if (i#$Integer(src) > $MAX_U128) {
        call $ExecFailureAbort();
        return;
    }
    dst := src;
}

procedure {:inline 1} $AddU8(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} $IsValidU8(src1) && $IsValidU8(src2);
{
    if (i#$Integer(src1) + i#$Integer(src2) > $MAX_U8) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) + i#$Integer(src2));
}

procedure {:inline 1} $AddU64(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} $IsValidU64(src1) && $IsValidU64(src2);
{
    if (i#$Integer(src1) + i#$Integer(src2) > $MAX_U64) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) + i#$Integer(src2));
}

procedure {:inline 1} $AddU64_unchecked(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} $IsValidU64(src1) && $IsValidU64(src2);
{
    dst := $Integer(i#$Integer(src1) + i#$Integer(src2));
}

procedure {:inline 1} $AddU128(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} $IsValidU128(src1) && $IsValidU128(src2);
{
    if (i#$Integer(src1) + i#$Integer(src2) > $MAX_U128) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) + i#$Integer(src2));
}

procedure {:inline 1} $AddU128_unchecked(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} $IsValidU128(src1) && $IsValidU128(src2);
{
    dst := $Integer(i#$Integer(src1) + i#$Integer(src2));
}

procedure {:inline 1} $Sub(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} is#$Integer(src1) && is#$Integer(src2);
{
    if (i#$Integer(src1) < i#$Integer(src2)) {
        call $ExecFailureAbort();
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
     if p == 8 then 256
     else if p == 16 then 65536
     else if p == 32 then 4294967296
     else if p == 64 then 18446744073709551616
     // Value is undefined, otherwise.
     else -1
     )
}

function $shl(src1: $Value, src2: $Value): $Value {
   (var po2 := $power_of_2(src2);
    $Integer(i#$Integer(src1) * po2)
   )
}

function $shr(src1: $Value, src2: $Value): $Value {
   (var po2 := $power_of_2(src2);
    $Integer(i#$Integer(src1) div po2)
   )
}

// TODO: fix this and $Shr to drop bits on overflow. Requires $Shl8, $Shl64, and $Shl128
procedure {:inline 1} $Shl(src1: $Value, src2: $Value) returns (dst: $Value)
requires is#$Integer(src1) && is#$Integer(src2);
{
    var po2: int;
    po2 := $power_of_2(src2);
    assert po2 >= 1;   // restriction: shift argument must be 8, 16, 32, or 64
    dst := $Integer(i#$Integer(src1) * po2);
}

procedure {:inline 1} $Shr(src1: $Value, src2: $Value) returns (dst: $Value)
requires is#$Integer(src1) && is#$Integer(src2);
{
    var po2: int;
    po2 := $power_of_2(src2);
    assert po2 >= 1;   // restriction: shift argument must be 8, 16, 32, or 64
    dst := $Integer(i#$Integer(src1) div po2);
}

procedure {:inline 1} $MulU8(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} $IsValidU8(src1) && $IsValidU8(src2);
{
    if (i#$Integer(src1) * i#$Integer(src2) > $MAX_U8) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) * i#$Integer(src2));
}

procedure {:inline 1} $MulU64(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} $IsValidU64(src1) && $IsValidU64(src2);
{
    if (i#$Integer(src1) * i#$Integer(src2) > $MAX_U64) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) * i#$Integer(src2));
}

procedure {:inline 1} $MulU128(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} $IsValidU128(src1) && $IsValidU128(src2);
{
    if (i#$Integer(src1) * i#$Integer(src2) > $MAX_U128) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) * i#$Integer(src2));
}

procedure {:inline 1} $Div(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} is#$Integer(src1) && is#$Integer(src2);
{
    if (i#$Integer(src2) == 0) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) div i#$Integer(src2));
}

procedure {:inline 1} $Mod(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} is#$Integer(src1) && is#$Integer(src2);
{
    if (i#$Integer(src2) == 0) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) mod i#$Integer(src2));
}

procedure {:inline 1} $ArithBinaryUnimplemented(src1: $Value, src2: $Value) returns (dst: $Value);
{{backend.type_requires}} is#$Integer(src1) && is#$Integer(src2);
ensures is#$Integer(dst);

procedure {:inline 1} $Lt(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} is#$Integer(src1) && is#$Integer(src2);
{
    dst := $Boolean(i#$Integer(src1) < i#$Integer(src2));
}

procedure {:inline 1} $Gt(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} is#$Integer(src1) && is#$Integer(src2);
{
    dst := $Boolean(i#$Integer(src1) > i#$Integer(src2));
}

procedure {:inline 1} $Le(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} is#$Integer(src1) && is#$Integer(src2);
{
    dst := $Boolean(i#$Integer(src1) <= i#$Integer(src2));
}

procedure {:inline 1} $Ge(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} is#$Integer(src1) && is#$Integer(src2);
{
    dst := $Boolean(i#$Integer(src1) >= i#$Integer(src2));
}

procedure {:inline 1} $And(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} is#$Boolean(src1) && is#$Boolean(src2);
{
    dst := $Boolean(b#$Boolean(src1) && b#$Boolean(src2));
}

procedure {:inline 1} $Or(src1: $Value, src2: $Value) returns (dst: $Value)
{{backend.type_requires}} is#$Boolean(src1) && is#$Boolean(src2);
{
    dst := $Boolean(b#$Boolean(src1) || b#$Boolean(src2));
}

procedure {:inline 1} $Not(src: $Value) returns (dst: $Value)
{{backend.type_requires}} is#$Boolean(src);
{
    dst := $Boolean(!b#$Boolean(src));
}

// Pack and Unpack are auto-generated for each type T


// ==================================================================================
// Native Vector Type

function {:inline} $Vector_type_value(tv: $TypeValue): $TypeValue {
    $VectorType(tv)
}

{{#if backend.vector_using_sequences}}

// This uses the implementation of $ValueArray using sequences
function {:inline} $Vector_$is_well_formed(v: $Value): bool {
    (
        is#$Vector(v)
    )
}

{{else}}

// This is uses the implementation of $ValueArray using integer maps
function {:inline} $Vector_$is_well_formed(v: $Value): bool {
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

{{/if}}

procedure {:inline 1} $Vector_empty(ta: $TypeValue) returns (v: $Value) {
    v := $mk_vector();
}

function {:inline 1} $Vector_$empty(ta: $TypeValue): $Value {
    $mk_vector()
}

procedure {:inline 1} $Vector_is_empty(ta: $TypeValue, v: $Value) returns (b: $Value) {
    assume is#$Vector(v);
    b := $Boolean($vlen(v) == 0);
}

procedure {:inline 1} $Vector_push_back(ta: $TypeValue, v: $Value, val: $Value) returns (v': $Value) {
    assume is#$Vector(v);
    v' := $push_back_vector(v, val);
}

function {:inline 1} $Vector_$push_back(ta: $TypeValue, v: $Value, val: $Value): $Value {
    $push_back_vector(v, val)
}

procedure {:inline 1} $Vector_pop_back(ta: $TypeValue, v: $Value) returns (e: $Value, v': $Value) {
    var len: int;
    assume is#$Vector(v);
    len := $vlen(v);
    if (len == 0) {
        call $ExecFailureAbort();
        return;
    }
    e := $select_vector(v, len-1);
    v' := $pop_back_vector(v);
}

function {:inline 1} $Vector_$pop_back(ta: $TypeValue, v: $Value): $Value {
    $select_vector(v, $vlen(v)-1)
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

function {:inline 1} $Vector_$length(ta: $TypeValue, v: $Value): $Value {
    $Integer($vlen(v))
}

procedure {:inline 1} $Vector_borrow(ta: $TypeValue, v: $Value, i: $Value) returns (dst: $Value) {
    var i_ind: int;

    assume is#$Vector(v);
    assume is#$Integer(i);
    i_ind := i#$Integer(i);
    if (i_ind < 0 || i_ind >= $vlen(v)) {
        call $ExecFailureAbort();
        return;
    }
    dst := $select_vector(v, i_ind);
}

function {:inline 1} $Vector_$borrow(ta: $TypeValue, v: $Value, i: $Value): $Value {
    $select_vector(v, i#$Integer(i))
}

procedure {:inline 1} $Vector_borrow_mut(ta: $TypeValue, v: $Value, index: $Value) returns (dst: $Mutation, v': $Value)
{{backend.type_requires}} is#$Integer(index);
{
    var i_ind: int;

    i_ind := i#$Integer(index);
    assume is#$Vector(v);
    if (i_ind < 0 || i_ind >= $vlen(v)) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Mutation($Local(0), $Path(p#$Path($EmptyPath)[0 := i_ind], 1), $select_vector(v, i_ind));
    v' := v;
}

function {:inline 1} $Vector_$borrow_mut(ta: $TypeValue, v: $Value, i: $Value): $Value {
    $select_vector(v, i#$Integer(i))
}

procedure {:inline 1} $Vector_destroy_empty(ta: $TypeValue, v: $Value) {
    if ($vlen(v) != 0) {
      call $ExecFailureAbort();
    }
}

procedure {:inline 1} $Vector_swap(ta: $TypeValue, v: $Value, i: $Value, j: $Value) returns (v': $Value)
{{backend.type_requires}} is#$Integer(i) && is#$Integer(j);
{
    var i_ind: int;
    var j_ind: int;
    assume is#$Vector(v);
    i_ind := i#$Integer(i);
    j_ind := i#$Integer(j);
    if (i_ind >= $vlen(v) || j_ind >= $vlen(v) || i_ind < 0 || j_ind < 0) {
        call $ExecFailureAbort();
        return;
    }
    v' := $swap_vector(v, i_ind, j_ind);
}

function {:inline 1} $Vector_$swap(ta: $TypeValue, v: $Value, i: $Value, j: $Value): $Value {
    $swap_vector(v, i#$Integer(i), i#$Integer(j))
}

procedure {:inline 1} $Vector_remove(ta: $TypeValue, v: $Value, i: $Value) returns (e: $Value, v': $Value)
{{backend.type_requires}} is#$Integer(i);
{
    var i_ind: int;

    assume is#$Vector(v);
    i_ind := i#$Integer(i);
    if (i_ind < 0 || i_ind >= $vlen(v)) {
        call $ExecFailureAbort();
        return;
    }
    e := $select_vector(v, i_ind);
    v' := $remove_vector(v, i_ind);
}

procedure {:inline 1} $Vector_swap_remove(ta: $TypeValue, v: $Value, i: $Value) returns (e: $Value, v': $Value)
{{backend.type_requires}} is#$Integer(i);
{
    var i_ind: int;
    var len: int;

    assume is#$Vector(v);
    i_ind := i#$Integer(i);
    len := $vlen(v);
    if (i_ind < 0 || i_ind >= len) {
        call $ExecFailureAbort();
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


function {:inline} $Hash_sha2(val: $Value): $Value {
    $Hash_sha2_core(val)
}

function $Hash_sha2_core(val: $Value): $Value;

// This says that Hash_sha2 respects isEquals (this would be automatic if we had an
// extensional theory of arrays and used ==, which has the substitution property
// for functions).
axiom (forall v1,v2: $Value :: $Vector_$is_well_formed(v1) && $Vector_$is_well_formed(v2)
       && $IsEqual(v1, v2) ==> $IsEqual($Hash_sha2_core(v1), $Hash_sha2_core(v2)));

// This says that Hash_sha2 is an injection
axiom (forall v1,v2: $Value :: $Vector_$is_well_formed(v1) && $Vector_$is_well_formed(v2)
        && $IsEqual($Hash_sha2_core(v1), $Hash_sha2_core(v2)) ==> $IsEqual(v1, v2));

// This procedure has no body. We want Boogie to just use its requires
// and ensures properties when verifying code that calls it.
procedure $Hash_sha2_256(val: $Value) returns (res: $Value);
// It will still work without this, but this helps verifier find more reasonable counterexamples.
{{backend.type_requires}} $IsValidU8Vector(val);
ensures res == $Hash_sha2_core(val);     // returns Hash_sha2 Value
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.
ensures $vlen(res) == 32;               // result is 32 bytes.

// Spec version of Move native function.
function {:inline} $Hash_$sha2_256(val: $Value): $Value {
    $Hash_sha2_core(val)
}

// similarly for Hash_sha3
function {:inline} $Hash_sha3(val: $Value): $Value {
    $Hash_sha3_core(val)
}
function $Hash_sha3_core(val: $Value): $Value;

axiom (forall v1,v2: $Value :: $Vector_$is_well_formed(v1) && $Vector_$is_well_formed(v2)
       && $IsEqual(v1, v2) ==> $IsEqual($Hash_sha3_core(v1), $Hash_sha3_core(v2)));

axiom (forall v1,v2: $Value :: $Vector_$is_well_formed(v1) && $Vector_$is_well_formed(v2)
        && $IsEqual($Hash_sha3_core(v1), $Hash_sha3_core(v2)) ==> $IsEqual(v1, v2));

procedure $Hash_sha3_256(val: $Value) returns (res: $Value);
ensures res == $Hash_sha3_core(val);     // returns Hash_sha3 Value
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.
ensures $vlen(res) == 32;               // result is 32 bytes.

// Spec version of Move native function.
function {:inline} $Hash_$sha3_256(val: $Value): $Value {
    $Hash_sha3_core(val)
}

// ==================================================================================
// Native diem_account

procedure {:inline 1} $DiemAccount_create_signer(
  addr: $Value
) returns (signer: $Value) {
    // A signer is currently identical to an address.
    signer := addr;
}

procedure {:inline 1} $DiemAccount_destroy_signer(
  signer: $Value
) {
  return;
}

// ==================================================================================
// Native Signer

procedure {:inline 1} $Signer_borrow_address(signer: $Value) returns (res: $Value)
    {{backend.type_requires}} is#$Address(signer);
{
    res := signer;
}

// ==================================================================================
// Native signature

// Signature related functionality is handled via uninterpreted functions. This is sound
// currently because we verify every code path based on signature verification with
// an arbitrary interpretation.

function $Signature_$ed25519_validate_pubkey(public_key: $Value): $Value;
function $Signature_$ed25519_verify(signature: $Value, public_key: $Value, message: $Value): $Value;

axiom (forall public_key: $Value ::
        is#$Boolean($Signature_$ed25519_validate_pubkey(public_key)));

axiom (forall signature, public_key, message: $Value ::
        is#$Boolean($Signature_$ed25519_verify(signature, public_key, message)));


procedure {:inline 1} $Signature_ed25519_validate_pubkey(public_key: $Value) returns (res: $Value) {
    res := $Signature_$ed25519_validate_pubkey(public_key);
}

procedure {:inline 1} $Signature_ed25519_verify(
        signature: $Value, public_key: $Value, message: $Value) returns (res: $Value) {
    res := $Signature_$ed25519_verify(signature, public_key, message);
}

// ==================================================================================
// Native BCS::serialize

// native define serialize<MoveValue>(v: &MoveValue): vector<u8>;

// Serialize is modeled as an uninterpreted function, with an additional
// axiom to say it's an injection.

function {:inline} $BCS_serialize(ta: $TypeValue, v: $Value): $Value {
    $BCS_serialize_core(v)
}

function $BCS_serialize_core(v: $Value): $Value;
function $BCS_serialize_core_inv(v: $Value): $Value;
// Needed only because IsEqual(v1, v2) is weaker than v1 == v2 in case there is a vector nested inside v1 or v2.
axiom (forall v1, v2: $Value :: $IsEqual(v1, v2) ==> $BCS_serialize_core(v1) == $BCS_serialize_core(v2));
// Injectivity
axiom (forall v: $Value :: $BCS_serialize_core_inv($BCS_serialize_core(v)) == v);

// This says that serialize returns a non-empty vec<u8>
{{#if (eq backend.serialize_bound 0)}}
axiom (forall v: $Value :: ( var r := $BCS_serialize_core(v); $IsValidU8Vector(r) && $vlen(r) > 0 ));
{{else}}
axiom (forall v: $Value :: ( var r := $BCS_serialize_core(v); $IsValidU8Vector(r) && $vlen(r) > 0 &&
                            $vlen(r) <= {{backend.serialize_bound}} ));
{{/if}}

// Serialized addresses should have the same length
const $serialized_address_len: int;
axiom (forall v: $Value :: (var r := $BCS_serialize_core(v); is#$Address(v) ==> $vlen(r) == $serialized_address_len));

procedure $BCS_to_bytes(ta: $TypeValue, v: $Value) returns (res: $Value);
ensures res == $BCS_serialize(ta, v);
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.

function {:inline} $BCS_$to_bytes(ta: $TypeValue, v: $Value): $Value {
    $BCS_serialize_core(v)
}

// ==================================================================================
// Native Signer::spec_address_of

function {:inline} $Signer_spec_address_of(signer: $Value): $Value
{
    // A signer is currently identical to an address.
    signer
}

function {:inline} $Signer_$borrow_address(signer: $Value): $Value
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
