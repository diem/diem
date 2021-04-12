// ================================================================================
// Notation

// This files contains a Handlebars Rust template for the prover's Boogie prelude.
// The template language constructs allow the prelude to adjust the actual content
// to multiple options. We only use a few selected template constructs which are
// mostly self-explaining. See the handlebars crate documentation for more information.
//
// The object passed in as context for templates is the struct `crate::TemplateOptions` and
// which contains a field `options` which points `crate::options::Options`, that is the
// backend command line options.

// ================================================================================
// Included theories

// Which theory implementation is bound to these includes is determined
// by the function `crate::add_prelude`, based on options provided to the prover.

{{> vector-theory }}
{{> multiset-theory }}


// ============================================================================================
// Type Values

type $TypeName;
type $FieldName = int;
type $LocalName;
type {:datatype} $TypeValue;
function {:constructor} $BooleanType() : $TypeValue;
function {:constructor} $IntegerType() : $TypeValue;
function {:constructor} $AddressType() : $TypeValue;
function {:constructor} $StrType() : $TypeValue;
function {:constructor} $VectorType(t: $TypeValue) : $TypeValue;
function {:constructor} $StructType(name: $TypeName, ts: Vec $TypeValue) : $TypeValue;
function {:constructor} $TypeType(): $TypeValue;
function {:constructor} $ErrorType() : $TypeValue;


// ============================================================================================
// Values

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
function {:constructor} $Vector(v: Vec $Value): $Value; // used to both represent Move Struct and Vector
function {:constructor} $Range(lb: int, ub: int): $Value;
function {:constructor} $Type(t: $TypeValue): $Value;
function {:constructor} $Error(): $Value;

function {:inline} $DefaultValue(): $Value { $Error() }

function {:inline} $IsValidBox(v: $Value): bool {
    true
}

function {:inline} $IsValidBox_int(v: $Value): bool {
  is#$Integer(v)
}

function {:inline} $IsValidBox_bool(v: $Value): bool {
  is#$Boolean(v)
}

function {:inline} $IsValidBox_addr(v: $Value): bool {
  is#$Address(v)
}

function {:inline} $IsValidBox_vec(v: $Value): bool {
  is#$Vector(v)
}

function {:inline} $IsValidU8Boxed(v: $Value): bool {
  $IsValidBox_int(v) && $IsValidU8($Unbox_int(v))
}

function {:inline} $IsValidBool(v: bool): bool {
  true
}

function {:inline} $IsValidU8(v: int): bool {
  $TagU8(v) && v >= 0 && v <= $MAX_U8
}

function {:inline} $IsValidU8Vector(v: Vec $Value): bool {
  $Vector_$is_well_formed($IntegerType(), v) &&
  (forall i: int :: {ReadVec(v, i)} 0 <= i && i < LenVec(v) ==> $IsValidU8Boxed(ReadVec(v, i)))
}

function {:inline} $IsValidU64(v: int): bool {
  $TagU64(v) && v >= 0 && v <= $MAX_U64
}

function {:inline} $IsValidU128(v: int): bool {
  $TagU128(v) && v >= 0 && v <= $MAX_U128
}

function {:inline} $IsValidNum(v: int): bool {
  $TagNum(v) && true
}

function {:inline} $IsValidAddress(v: int): bool {
  // TODO: restrict max to representable addresses?
  $TagAddr(v) && v >= 0
}

// Non-inlined type tagging functions. Those are added to type assumptions to provide
// a trigger for quantifier instantiation based on type information.
function $TagBool(x: bool): bool { true }
function $TagU8(x: int): bool { true }
function $TagU64(x: int): bool { true }
function $TagU128(x: int): bool { true }
function $TagNum(x: int): bool { true }
function $TagAddr(x: int): bool { true }
function $TagType(x: $TypeValue): bool { true }
function $TagVec(et: $TypeValue, x: Vec $Value): bool { true }


function {:inline} $IsValidRange(r: $Value): bool {
   $IsValidU64(lb#$Range(r)) &&  $IsValidU64(ub#$Range(r))
}

function {:inline} $InRange(r: $Value, i: int): bool {
   lb#$Range(r) <= i && i < ub#$Range(r)
}

// Canonical API to constructors/selectors, supporting codegen.

function {:inline} $Box(x: $Value): $Value {
    x
}
function {:inline} $Box_int(x: int): $Value {
    $Integer(x)
}
function {:inline} $Box_bool(x: bool): $Value {
    $Boolean(x)
}
function {:inline} $Box_addr(x: int): $Value {
    $Address(x)
}
function {:inline} $Box_vec(x: Vec $Value): $Value {
    $Vector(x)
}

function {:inline} $Unbox(x: $Value): $Value {
    x
}
function {:inline} $Unbox_int(x: $Value): int {
    i#$Integer(x)
}
function {:inline} $Unbox_bool(x: $Value): bool {
    b#$Boolean(x)
}
function {:inline} $Unbox_addr(x: $Value): int {
    a#$Address(x)
}
function {:inline} $Unbox_vec(x: $Value): Vec $Value {
    v#$Vector(x)
}

// ============================================================================================
// Helpers for Vectors

function {:inline} $SliceVecByRange(v: Vec $Value, r: $Value): Vec $Value {
    SliceVec(v, lb#$Range(r), ub#$Range(r))
}

{{#if options.native_equality}}

function {:inline} $ContainsVec(v: Vec $Value, e: $Value): bool {
    ContainsVec(v, e)
}

function {:inline} $IndexOfVec(v: Vec $Value, e: $Value): int {
    IndexOfVec(v, e)
}

{{else}}

// Because the vector implementation does not support extensional equality,
// we need to redefine some functions here using $IsEqual.

function {:inline} $ContainsVec(v: Vec $Value, e: $Value): bool {
    (exists i: int :: $InRangeVec(v, i) && $IsEqual(ReadVec(v, i), e))
}

function $IndexOfVec(v: Vec $Value, e: $Value): int;
axiom (forall v: Vec $Value, e: $Value:: {$IndexOfVec(v, e)}
    (var i := $IndexOfVec(v,e);
     if (!$ContainsVec(v, e)) then i == -1
     else $InRangeVec(v, i) && $IsEqual(ReadVec(v, i), e) &&
        (forall j: int :: j >= 0 && j < i ==> !$IsEqual(ReadVec(v, j), e))));

{{/if}}

// Specialize function InRangeVec. Currently there is no difference to the one
// provided by the vectory theory, but we may want to refine this for triggering.
function {:inline} $InRangeVec(v: Vec $Value, i: int): bool {
    InRangeVec(v, i)
}

// This funtion is used to avoid type inference ambiguity for expressions
// like `LenVec(EmptyVec())`. Boogie cannot infer this and also does no allow
// type annotations.
function {:inline} $EmptyValueVec(): Vec $Value { EmptyVec() }

// ================================================================================
// Path type

type {:datatype} $Path;
function {:constructor} $Path(p: [int]int, size: int): $Path;
const $EmptyPath: $Path;
axiom size#$Path($EmptyPath) == 0;

function {:inline} $path_index_at(p: $Path, i: int): int {
    p#$Path(p)[i]
}

const $StratificationDepth: int;
axiom $StratificationDepth == {{options.stratification_depth}};


// Generate stratified $UpdateValue for the depth of {{options.stratification_depth}}.

{{#stratified}}
function {{options.aggressive_func_inline}} $UpdateValue_{{@this_suffix}}(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    (var poffset := offset + {{@this_level}};
    if (poffset == size#$Path(p)) then
        new_v
    else
        (var va := $Unbox_vec(v);
         $Box_vec(UpdateVec(va, $path_index_at(p, poffset),
                       $UpdateValue_{{@next_suffix}}(p, offset, ReadVec(va, $path_index_at(p, poffset)), new_v)))))
}
{{else}}
function {:inline} $UpdateValue_{{@this_suffix}}(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    new_v
}
{{/stratified}}

function {:inline} $UpdateValue(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    $UpdateValue_stratified(p, offset, v, new_v)
}

// Generate stratified $IsPathPrefix for the depth of {{options.stratification_depth}}.

{{#stratified}}
function {{options.aggressive_func_inline}} $IsPathPrefix_{{@this_suffix}}(p1: $Path, p2: $Path): bool {
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

// Generate stratified $ConcatPath for the depth of {{options.stratification_depth}}.

{{#stratified}}
function {{options.aggressive_func_inline}} $ConcatPath_{{@this_suffix}}(p1: $Path, p2: $Path): $Path {
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

// ============================================================================================
// Equality

{{#if options.native_equality}}

// Map IsEqual to native Boogie equality. This only works with vector theories with
// extensional equality.
function {:inline} $IsEqual(v1: $Value, v2: $Value): bool {
    v1 == v2
}

function {:inline} $IsEqual_vec(x: Vec $Value, y: Vec $Value): bool {
    x == y
}

{{else}}

// Generate a stratified version of IsEqual for depth of {{options.stratification_depth}}.

function {:inline} $IsEqual(v1: $Value, v2: $Value): bool {
    $IsEqual_stratified(v1, v2)
}

function {:inline} $IsEqual_vec(v1: Vec $Value, v2: Vec $Value): bool {
    v1 == v2 ||
    (LenVec(v1) == LenVec(v2) &&
     (forall i: int :: 0 <= i && i < LenVec(v1) ==> $IsEqual(ReadVec(v1, i), ReadVec(v2, i))))
}

{{/if}}

function {:inline} $IsEqual_int(x: int, y: int): bool {
    x == y
}

function {:inline} $IsEqual_addr(x: int, y: int): bool {
    x == y
}

function {:inline} $IsEqual_bool(x: bool, y: bool): bool {
    x == y
}

function {:inline} $IsEqual_type(x: $TypeValue, y: $TypeValue): bool {
    x == y
}




// ============================================================================================
// Memory

type {:datatype} $Location;

// A global resource location within the statically known resource type's memory.
// `ts` are the type parameters for the outer type, and `a` is the address.
function {:constructor} $Global(ts: Vec $TypeValue, a: int): $Location;

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

// Representation of memory for a given type. The maps take the content of a Global location.
type {:datatype} $Memory _;
function {:constructor} $Memory<T>(domain: [Vec $TypeValue, int]bool, contents: [Vec $TypeValue, int]T): $Memory T;

function {:builtin "MapConst"} $ConstMemoryDomain(v: bool): [Vec $TypeValue, int]bool;
function {:builtin "MapConst"} $ConstMemoryContent<T>(v: T): [Vec $TypeValue, int]T;
axiom $ConstMemoryDomain(false) == (lambda ta: Vec $TypeValue, i: int :: false);
axiom $ConstMemoryDomain(true) == (lambda ta: Vec $TypeValue, i: int :: true);


// Dereferences a mutation.
function {:inline} $Dereference(ref: $Mutation): $Value {
    v#$Mutation(ref)
}

// Update the value of a mutation.
function {:inline} $UpdateMutation(m: $Mutation, v: $Value): $Mutation {
    $Mutation(l#$Mutation(m), p#$Mutation(m), v)
}

// Havoc the value of a mutation.
procedure {:inline 1} $HavocMutation(m: $Mutation) returns (m': $Mutation) {
  var v': $Value;
  m' := $Mutation(l#$Mutation(m), p#$Mutation(m), v');
}

// Tests whether resource exists.
function {:inline} $ResourceExists<T>(m: $Memory T, args: Vec $TypeValue, addr: int): bool {
    domain#$Memory(m)[args, addr]
}

// Obtains Value of given resource.
function {:inline} $ResourceValue<T>(m: $Memory T, args: Vec $TypeValue, addr: int): T {
    contents#$Memory(m)[args, addr]
}

// Update resource.
function {:inline} $ResourceUpdate<T>(m: $Memory T, ta: Vec $TypeValue, a: int, v: T): $Memory T {
    $Memory(domain#$Memory(m)[ta, a := true], contents#$Memory(m)[ta, a := v])
}

// Remove resource.
function {:inline} $ResourceRemove<T>(m: $Memory T, ta: Vec $TypeValue, a: int): $Memory T {
    $Memory(domain#$Memory(m)[ta, a := false], contents#$Memory(m))
}

// Copies resource from memory s to m.
function {:inline} $ResourceCopy<T>(m: $Memory T, s: $Memory T, ta: Vec $TypeValue, a: int): $Memory T {
    $Memory(domain#$Memory(m)[ta, a := domain#$Memory(s)[ta,a]],
            contents#$Memory(m)[ta, a := contents#$Memory(s)[ta,a]])
}



// ============================================================================================
// Abort Handling

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
    // Assume that the EventStore is initially empty.
    assume $EventStore__is_empty($es);
}

// ============================================================================================
// Instructions

procedure {:inline 1} $BorrowLoc(l: int, v: $Value) returns (dst: $Mutation)
{
    dst := $Mutation($Local(l), $EmptyPath, v);
}

procedure {:inline 1} $WritebackToValue(src: $Mutation, idx: int, vdst: $Value) returns (vdst': $Value)
{
    if (l#$Mutation(src) == $Local(idx)) {
        vdst' := v#$Mutation(src);
    } else {
        vdst' := vdst;
    }
}

procedure {:inline 1} $WritebackToValue_int(src: $Mutation, idx: int, vdst: int) returns (vdst': int)
{
    if (l#$Mutation(src) == $Local(idx)) {
        vdst' := $Unbox_int(v#$Mutation(src));
    } else {
        vdst' := vdst;
    }
}

procedure {:inline 1} $WritebackToValue_bool(src: $Mutation, idx: int, vdst: bool) returns (vdst': bool)
{
    if (l#$Mutation(src) == $Local(idx)) {
        vdst' := $Unbox_bool(v#$Mutation(src));
    } else {
        vdst' := vdst;
    }
}

procedure {:inline 1} $WritebackToValue_addr(src: $Mutation, idx: int, vdst: int) returns (vdst': int)
{
    if (l#$Mutation(src) == $Local(idx)) {
        vdst' := $Unbox_addr(v#$Mutation(src));
    } else {
        vdst' := vdst;
    }
}

procedure {:inline 1} $WritebackToValue_vec(src: $Mutation, idx: int, vdst: Vec $Value) returns (vdst': Vec $Value)
{
    if (l#$Mutation(src) == $Local(idx)) {
        vdst' := $Unbox_vec(v#$Mutation(src));
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
                    v#$Mutation(src));
    } else {
        dst' := dst;
    }
}

procedure {:inline 1} $WritebackToVec(src: $Mutation, dst: $Mutation)
returns (dst': $Mutation)
{
    var srcPath, dstPath: $Path;

    srcPath := p#$Mutation(src);
    dstPath := p#$Mutation(dst);
    if (l#$Mutation(dst) == l#$Mutation(src)) {
        dst' := $Mutation(
                    l#$Mutation(dst),
                    dstPath,
                    $Box_vec(UpdateVec($Unbox_vec(v#$Mutation(dst)),
                    $path_index_at(srcPath, size#$Path(srcPath) - 1), v#$Mutation(src)))
                    );
    } else {
        dst' := dst;
    }
}

procedure {:inline 1} $Splice1(idx1: int, src1: $Mutation, dst: $Mutation) returns (dst': $Mutation) {
    dst' := $Mutation(l#$Mutation(src1), $ConcatPath(p#$Mutation(src1), p#$Mutation(dst)), v#$Mutation(dst));
}

procedure {:inline 1} $CastU8(src: int) returns (dst: int)
{
    if (src > $MAX_U8) {
        call $ExecFailureAbort();
        return;
    }
    dst := src;
}

procedure {:inline 1} $CastU64(src: int) returns (dst: int)
{
    if (src > $MAX_U64) {
        call $ExecFailureAbort();
        return;
    }
    dst := src;
}

procedure {:inline 1} $CastU128(src: int) returns (dst: int)
{
    if (src > $MAX_U128) {
        call $ExecFailureAbort();
        return;
    }
    dst := src;
}

procedure {:inline 1} $AddU8(src1: int, src2: int) returns (dst: int)
{
    if (src1 + src2 > $MAX_U8) {
        call $ExecFailureAbort();
        return;
    }
    dst := src1 + src2;
}

procedure {:inline 1} $AddU64(src1: int, src2: int) returns (dst: int)
{
    if (src1 + src2 > $MAX_U64) {
        call $ExecFailureAbort();
        return;
    }
    dst := src1 + src2;
}

procedure {:inline 1} $AddU64_unchecked(src1: int, src2: int) returns (dst: int)
{
    dst := src1 + src2;
}

procedure {:inline 1} $AddU128(src1: int, src2: int) returns (dst: int)
{
    if (src1 + src2 > $MAX_U128) {
        call $ExecFailureAbort();
        return;
    }
    dst := src1 + src2;
}

procedure {:inline 1} $AddU128_unchecked(src1: int, src2: int) returns (dst: int)
{
    dst := src1 + src2;
}

procedure {:inline 1} $Sub(src1: int, src2: int) returns (dst: int)
{
    if (src1 < src2) {
        call $ExecFailureAbort();
        return;
    }
    dst := src1 - src2;
}

// Note that *not* inlining the shl/shr functions avoids timeouts. It appears that Z3 can reason
// better about this if it is an axiomatized function.
function $shl(src1: int, p: int): int {
    if p == 8 then src1 * 256
    else if p == 16 then src1 * 65536
    else if p == 32 then src1 * 4294967296
    else if p == 64 then src1 * 18446744073709551616
    // Value is undefined, otherwise.
    else -1
}

function $shr(src1: int, p: int): int {
    if p == 8 then src1 div 256
    else if p == 16 then src1 div 65536
    else if p == 32 then src1 div 4294967296
    else if p == 64 then src1 div 18446744073709551616
    // Value is undefined, otherwise.
    else -1
}

// TODO: fix this and $Shr to drop bits on overflow. Requires $Shl8, $Shl64, and $Shl128
procedure {:inline 1} $Shl(src1: int, src2: int) returns (dst: int)
{
    var res: int;
    res := $shl(src1, src2);
    assert res >= 0;   // restriction: shift argument must be 8, 16, 32, or 64
    dst := res;
}

procedure {:inline 1} $Shr(src1: int, src2: int) returns (dst: int)
{
    var res: int;
    res := $shr(src1, src2);
    assert res >= 0;   // restriction: shift argument must be 8, 16, 32, or 64
    dst := res;
}

procedure {:inline 1} $MulU8(src1: int, src2: int) returns (dst: int)
{
    if (src1 * src2 > $MAX_U8) {
        call $ExecFailureAbort();
        return;
    }
    dst := src1 * src2;
}

procedure {:inline 1} $MulU64(src1: int, src2: int) returns (dst: int)
{
    if (src1 * src2 > $MAX_U64) {
        call $ExecFailureAbort();
        return;
    }
    dst := src1 * src2;
}

procedure {:inline 1} $MulU128(src1: int, src2: int) returns (dst: int)
{
    if (src1 * src2 > $MAX_U128) {
        call $ExecFailureAbort();
        return;
    }
    dst := src1 * src2;
}

procedure {:inline 1} $Div(src1: int, src2: int) returns (dst: int)
{
    if (src2 == 0) {
        call $ExecFailureAbort();
        return;
    }
    dst := src1 div src2;
}

procedure {:inline 1} $Mod(src1: int, src2: int) returns (dst: int)
{
    if (src2 == 0) {
        call $ExecFailureAbort();
        return;
    }
    dst := src1 mod src2;
}

procedure {:inline 1} $ArithBinaryUnimplemented(src1: int, src2: int) returns (dst: int);

procedure {:inline 1} $Lt(src1: int, src2: int) returns (dst: bool)
{
    dst := src1 < src2;
}

procedure {:inline 1} $Gt(src1: int, src2: int) returns (dst: bool)
{
    dst := src1 > src2;
}

procedure {:inline 1} $Le(src1: int, src2: int) returns (dst: bool)
{
    dst := src1 <= src2;
}

procedure {:inline 1} $Ge(src1: int, src2: int) returns (dst: bool)
{
    dst := src1 >= src2;
}

procedure {:inline 1} $And(src1: bool, src2: bool) returns (dst: bool)
{
    dst := src1 && src2;
}

procedure {:inline 1} $Or(src1: bool, src2: bool) returns (dst: bool)
{
    dst := src1 || src2;
}

procedure {:inline 1} $Not(src: bool) returns (dst: bool)
{
    dst := !src;
}

// Pack and Unpack are auto-generated for each type T


// ==================================================================================
// Native Vector Type

function {:inline} $Vector_type_value(tv: $TypeValue): $TypeValue {
    $VectorType(tv)
}

function {:inline} $Vector_$is_well_formed(et: $TypeValue, v: Vec $Value): bool {
    $TagVec(et, v) && LenVec(v) >= 0 && $IsValidU64(LenVec(v))
}

procedure {:inline 1} $Vector_empty(ta: $TypeValue) returns (v: Vec $Value) {
    v := EmptyVec();
}

function {:inline} $Vector_$empty(ta: $TypeValue): Vec $Value {
    EmptyVec()
}

procedure {:inline 1} $Vector_is_empty(ta: $TypeValue, v: Vec $Value) returns (b: bool) {
    b := LenVec(v) == 0;
}

procedure {:inline 1} $Vector_push_back(ta: $TypeValue, m: $Mutation, val: $Value) returns (m': $Mutation) {
    var v: Vec $Value;
    v := $Unbox_vec($Dereference(m));
    m' := $UpdateMutation(m, $Box_vec(ExtendVec(v, val)));
}

function {:inline} $Vector_$push_back(ta: $TypeValue, v: Vec $Value, val: $Value): Vec $Value {
    ExtendVec(v, val)
}

procedure {:inline 1} $Vector_pop_back(ta: $TypeValue, m: $Mutation) returns (e: $Value, m': $Mutation) {
    var v: Vec $Value;
    var len: int;
    v := $Unbox_vec($Dereference(m));
    len := LenVec(v);
    if (len == 0) {
        call $ExecFailureAbort();
        return;
    }
    e := ReadVec(v, len-1);
    m' := $UpdateMutation(m, $Box_vec(RemoveVec(v)));
}

procedure {:inline 1} $Vector_append(ta: $TypeValue, m: $Mutation, other: Vec $Value) returns (m': $Mutation) {
    var v: Vec $Value;
    v := $Unbox_vec($Dereference(m));
    m' := $UpdateMutation(m, $Box_vec(ConcatVec(v, other)));
}

procedure {:inline 1} $Vector_reverse(ta: $TypeValue, m: $Mutation) returns (m': $Mutation) {
    var v: Vec $Value;
    v := $Unbox_vec($Dereference(m));
    m' := $UpdateMutation(m, $Box_vec(ReverseVec(v)));
}

procedure {:inline 1} $Vector_length(ta: $TypeValue, v: Vec $Value) returns (l: int) {
    l := LenVec(v);
}

function {:inline} $Vector_$length(ta: $TypeValue, v: Vec $Value): int {
    LenVec(v)
}

procedure {:inline 1} $Vector_borrow(ta: $TypeValue, v: Vec $Value, i: int) returns (dst: $Value) {
    if (i < 0 || i >= LenVec(v)) {
        call $ExecFailureAbort();
        return;
    }
    dst := ReadVec(v, i);
}

function {:inline} $Vector_$borrow(ta: $TypeValue, v: Vec $Value, i: int): $Value {
    ReadVec(v, i)
}

procedure {:inline 1} $Vector_borrow_mut(ta: $TypeValue, m: $Mutation, index: int) returns (dst: $Mutation, m': $Mutation)
{
    var v: Vec $Value;
    var p: $Path;
    var size: int;

    v := $Unbox_vec($Dereference(m));
    if (index < 0 || index >= LenVec(v)) {
        call $ExecFailureAbort();
        return;
    }
    p := p#$Mutation(m);
    size := size#$Path(p);
    p := $Path(p#$Path(p)[size := index], size+1);
    dst := $Mutation(l#$Mutation(m), p, ReadVec(v, index));
    m' := m;
}

function {:inline} $Vector_$borrow_mut(ta: $TypeValue, v: Vec $Value, i: int): $Value {
    ReadVec(v, i)
}

procedure {:inline 1} $Vector_destroy_empty(ta: $TypeValue, v: Vec $Value) {
    if (!IsEmptyVec(v)) {
      call $ExecFailureAbort();
    }
}

procedure {:inline 1} $Vector_swap(ta: $TypeValue, m: $Mutation, i: int, j: int) returns (m': $Mutation)
{
    var v: Vec $Value;
    v := $Unbox_vec($Dereference(m));
    if (i >= LenVec(v) || j >= LenVec(v) || i < 0 || j < 0) {
        call $ExecFailureAbort();
        return;
    }
    m' := $UpdateMutation(m, $Box_vec(SwapVec(v, i, j)));
}

function {:inline} $Vector_$swap(ta: $TypeValue, v: Vec $Value, i: int, j: int): Vec $Value {
    SwapVec(v, i, j)
}

procedure {:inline 1} $Vector_remove(ta: $TypeValue, m: $Mutation, i: int) returns (e: $Value, m': $Mutation)
{
    var v: Vec $Value;

    v := $Unbox_vec($Dereference(m));

    if (i < 0 || i >= LenVec(v)) {
        call $ExecFailureAbort();
        return;
    }
    e := ReadVec(v, i);
    m' := $UpdateMutation(m, $Box_vec(RemoveAtVec(v, i)));
}

procedure {:inline 1} $Vector_swap_remove(ta: $TypeValue, m: $Mutation, i: int) returns (e: $Value, m': $Mutation)
{
    var len: int;
    var v: Vec $Value;

    v := $Unbox_vec($Dereference(m));

    len := LenVec(v);
    if (i < 0 || i >= len) {
        call $ExecFailureAbort();
        return;
    }
    e := ReadVec(v, i);
    m' := $UpdateMutation(m, $Box_vec(RemoveVec(SwapVec(v, i, len-1))));
}

procedure {:inline 1} $Vector_contains(ta: $TypeValue, v: Vec $Value, e: $Value) returns (res: bool)  {
    res := $ContainsVec(v, e);
}

/*
procedure {:inline 1}
$Vector_index_of(ta: $TypeValue, v: Vec $Value, e: $Value) returns (res1: bool, res2: int) {
    res2 := $IndexOfVec(v, e);
    if (res2 >= 0) {
        res1 := true;
    } else {
        res1 := false;
        res2 := 0;
    }
}
*/

// FIXME: This procedure sometimes (not always) make the test (performance_200511) very slow (> 10 mins) or hang
// although this is not used in the test script (performance_200511). The test finishes in 20 secs when it works fine.
procedure {:inline 1} $Vector_index_of(ta: $TypeValue, v: Vec $Value, e: $Value) returns (res1: bool, res2: int);
ensures 0 <= res2 && res2 < LenVec(v);
ensures res1 == $ContainsVec(v, e);
ensures res1 ==> $IsEqual(ReadVec(v, res2), e);
ensures res1 ==> (forall i:int :: 0 <= i && i < res2 ==> !$IsEqual(ReadVec(v, i), e));
ensures !res1 ==> res2 == 0;


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


function {:inline} $Hash_sha2(val: Vec $Value): Vec $Value {
    $Hash_sha2_core(val)
}

function $Hash_sha2_core(val: Vec $Value): Vec $Value;

// This says that Hash_sha2 is bijective.
axiom (forall v1,v2: Vec $Value :: {$Hash_sha2_core(v1), $Hash_sha2_core(v2)}
       $IsEqual_vec(v1, v2) <==> $IsEqual_vec($Hash_sha2_core(v1), $Hash_sha2_core(v2)));

// This procedure has no body. We want Boogie to just use its requires
// and ensures properties when verifying code that calls it.
procedure $Hash_sha2_256(val: Vec $Value) returns (res: Vec $Value);
// It will still work without this, but this helps verifier find more reasonable counterexamples.
{{options.type_requires}} $IsValidU8Vector(val);
ensures res == $Hash_sha2_core(val);     // returns Hash_sha2 Value
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.
ensures LenVec(res) == 32;               // result is 32 bytes.

// Spec version of Move native function.
function {:inline} $Hash_$sha2_256(val: Vec $Value): Vec $Value {
    $Hash_sha2_core(val)
}

// similarly for Hash_sha3
function {:inline} $Hash_sha3(val: Vec $Value): Vec $Value {
    $Hash_sha3_core(val)
}
function $Hash_sha3_core(val: Vec $Value): Vec $Value;

axiom (forall v1,v2: Vec $Value :: {$Hash_sha3_core(v1), $Hash_sha3_core(v2)}
       $IsEqual_vec(v1, v2) <==> $IsEqual_vec($Hash_sha3_core(v1), $Hash_sha3_core(v2)));

procedure $Hash_sha3_256(val: Vec $Value) returns (res: Vec $Value);
ensures res == $Hash_sha3_core(val);     // returns Hash_sha3 Value
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.
ensures LenVec(res) == 32;               // result is 32 bytes.

// Spec version of Move native function.
function {:inline} $Hash_$sha3_256(val: Vec $Value): Vec $Value {
    $Hash_sha3_core(val)
}

// ==================================================================================
// Native diem_account

procedure {:inline 1} $DiemAccount_create_signer(
  addr: int
) returns (signer: int) {
    // A signer is currently identical to an address.
    signer := addr;
}

procedure {:inline 1} $DiemAccount_destroy_signer(
  signer: int
) {
  return;
}

// ==================================================================================
// Native Signer

procedure {:inline 1} $Signer_borrow_address(signer: int) returns (res: int) {
    res := signer;
}

// ==================================================================================
// Native signature

// Signature related functionality is handled via uninterpreted functions. This is sound
// currently because we verify every code path based on signature verification with
// an arbitrary interpretation.

function $Signature_$ed25519_validate_pubkey(public_key: Vec $Value): bool;
function $Signature_$ed25519_verify(signature: Vec $Value, public_key: Vec $Value, message: Vec $Value): bool;

// Needed because we do not have extensional equality:
axiom (forall k1, k2: Vec $Value ::
    {$Signature_$ed25519_validate_pubkey(k1), $Signature_$ed25519_validate_pubkey(k2)}
    $IsEqual_vec(k1, k2) ==> $Signature_$ed25519_validate_pubkey(k1) == $Signature_$ed25519_validate_pubkey(k2));
axiom (forall s1, s2, k1, k2, m1, m2: Vec $Value ::
    {$Signature_$ed25519_verify(s1, k1, m1), $Signature_$ed25519_verify(s2, k2, m2)}
    $IsEqual_vec(s1, s2) && $IsEqual_vec(k1, k2) && $IsEqual_vec(m1, m2)
    ==> $Signature_$ed25519_verify(s1, k1, m1) == $Signature_$ed25519_verify(s2, k2, m2));


procedure {:inline 1} $Signature_ed25519_validate_pubkey(public_key: Vec $Value) returns (res: bool) {
    res := $Signature_$ed25519_validate_pubkey(public_key);
}

procedure {:inline 1} $Signature_ed25519_verify(
        signature: Vec $Value, public_key: Vec $Value, message: Vec $Value) returns (res: bool) {
    res := $Signature_$ed25519_verify(signature, public_key, message);
}

// ==================================================================================
// Native BCS::serialize

// native define serialize<MoveValue>(v: &MoveValue): vector<u8>;

// Serialize is modeled as an uninterpreted function, with an additional
// axiom to say it's an injection.

function {:inline} $BCS_serialize(ta: $TypeValue, v: $Value): Vec $Value {
    $BCS_serialize_core(v)
}

function $BCS_serialize_core(v: $Value): Vec $Value;
axiom (forall v1, v2: $Value :: {$BCS_serialize_core(v1), $BCS_serialize_core(v2)}
   $IsEqual(v1, v2) <==> $IsEqual_vec($BCS_serialize_core(v1), $BCS_serialize_core(v2)));

// This says that serialize returns a non-empty vec<u8>
{{#if (eq options.serialize_bound 0)}}
axiom (forall v: $Value :: {$BCS_serialize_core(v)}
     ( var r := $BCS_serialize_core(v); $IsValidU8Vector(r) && LenVec(r) > 0 ));
{{else}}
axiom (forall v: $Value :: {$BCS_serialize_core(v)}
     ( var r := $BCS_serialize_core(v); $IsValidU8Vector(r) && LenVec(r) > 0 &&
                            LenVec(r) <= {{options.serialize_bound}} ));
{{/if}}

// Serialized addresses should have the same length
const $serialized_address_len: int;
axiom (forall v: $Value :: {$BCS_serialize_core(v)}
     ( var r := $BCS_serialize_core(v); is#$Address(v) ==> LenVec(r) == $serialized_address_len));

procedure $BCS_to_bytes(ta: $TypeValue, v: $Value) returns (res: Vec $Value);
ensures res == $BCS_serialize_core(v);

function {:inline} $BCS_$to_bytes(ta: $TypeValue, v: $Value): Vec $Value {
    $BCS_serialize_core(v)
}

// ==================================================================================
// Native Signer::spec_address_of

function {:inline} $Signer_spec_address_of(signer: int): int
{
    // A signer is currently identical to an address.
    signer
}

function {:inline} $Signer_$borrow_address(signer: int): int
{
    // A signer is currently identical to an address.
    signer
}

// ==================================================================================
// Mocked out Event module

// Abstract type of event handles.
type $Event_EventHandle;

// Global state to implement uniqueness of event handles.
var $Event_EventHandles: [$Event_EventHandle]bool;

// Embedding of event handle type into $Value
function {:constructor} $Box_$Event_EventHandle(s: $Event_EventHandle): $Value;
function {:inline} $Unbox_$Event_EventHandle(x: $Value): $Event_EventHandle {
    s#$Box_$Event_EventHandle(x)
}
function {:inline} $IsValidBox_$Event_EventHandle(x: $Value): bool {
    is#$Box_$Event_EventHandle(x)
}

// Type value for event handles and equality.
const unique $Event_EventHandle_name: $TypeName;
function {:inline} $Event_EventHandle_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Event_EventHandle_name, MakeVec1($tv0))
}

function {:inline} $IsEqual_$Event_EventHandle(a: $Event_EventHandle, b: $Event_EventHandle): bool {
    a == b
}

// Publishing a generator does nothing. Currently we just ignore this function and do not represent generators
// at all because they are not publicly exposed by the Event module.
// TODO: we should check (and abort with the right code) if a generator already exists for
// the signer.
procedure {:inline 1} $Event_publish_generator(account: int) {
}

// We need to implement the $Modifies for EventHandleGenerator. It does nothing because they are mocked out.
procedure {:inline} $Modifies_$Event_EventHandleGenerator(ta: Vec $TypeValue, a: int) { }

// Creates a new event handle. This ensures each time it is called that a unique new abstract event handler is
// returned.
// TODO: we should check (and abort with the right code) if not generator exists for the signer.
procedure {:inline 1} $Event_new_event_handle(t: $TypeValue, signer: int) returns (res: $Event_EventHandle) {
    assume $Event_EventHandles[res] == false;
    $Event_EventHandles := $Event_EventHandles[res := true];
}

// This boogie procedure is the model of `emit_event`. This model abstracts away the `counter` behavior, thus not
// mutating (or increasing) `counter`.
procedure {:inline 1} $Event_emit_event(t: $TypeValue, handle_mut: $Mutation, msg: $Value) returns (res: $Mutation) {
    var handle: $Event_EventHandle;
    handle := $Unbox_$Event_EventHandle($Dereference(handle_mut));
    $es := $ExtendEventStore($es, handle, msg);
    res := handle_mut;
}

procedure {:inline 1} $Event_destroy_handle(t: $TypeValue, handle: $Event_EventHandle) {
}

// EventStore
// ==========

{{#if options.native_equality}}

// TODO: like to have this as aliases, but currently blocked by boogie issue #364
type {:datatype} $EventRep;
function {:constructor} $ToEventRep(event: $Value): $EventRep;

{{else}}

// Because we do not have extensional equality we need to encode events and guids
// in abstract types which have equality, so they can be used as indices for Boogie arrays.

type $EventRep;

function $ToEventRep(event: $Value): $EventRep;

axiom (forall v1, v2: $Value :: {$ToEventRep(v1), $ToEventRep(v2)}
    $IsEqual(v1, v2) <==> $ToEventRep(v1) == $ToEventRep(v2));

{{/if}}

// Representation of EventStore that consists of event streams.
type {:datatype} $EventStore;
function {:constructor} $EventStore(counter: int, streams: [$Event_EventHandle]Multiset $EventRep): $EventStore;

function {:inline} $EventStore__is_well_formed(es: $EventStore): bool {
    true
}

function {:inline} $EventStore__is_empty(es: $EventStore): bool {
    (counter#$EventStore(es) == 0) &&
    (forall handle: $Event_EventHandle ::
        (var stream := streams#$EventStore(es)[handle];
        IsEmptyMultiset(stream)))
}

// This function returns (es1 - es2). This function assumes that es2 is a subset of es1.
function {:inline} $EventStore__subtract(es1: $EventStore, es2: $EventStore): $EventStore {
    $EventStore(counter#$EventStore(es1)-counter#$EventStore(es2),
        (lambda handle: $Event_EventHandle ::
        SubtractMultiset(
            streams#$EventStore(es1)[handle],
            streams#$EventStore(es2)[handle])))
}

function {:inline} $EventStore__is_subset(es1: $EventStore, es2: $EventStore): bool {
    (counter#$EventStore(es1) <= counter#$EventStore(es2)) &&
    (forall handle: $Event_EventHandle ::
        IsSubsetMultiset(
            streams#$EventStore(es1)[handle],
            streams#$EventStore(es2)[handle]
        )
    )
}

procedure {:inline 1} $EventStore__diverge(es: $EventStore) returns (es': $EventStore) {
    assume $EventStore__is_subset(es, es');
}

const $EmptyEventStore: $EventStore;
axiom $EventStore__is_empty($EmptyEventStore);

function {:inline} $ExtendEventStore(es: $EventStore, handle: $Event_EventHandle, msg: $Value): $EventStore {
    (var stream := streams#$EventStore(es)[handle];
    (var stream_new := ExtendMultiset(stream, $ToEventRep(msg));
    $EventStore(counter#$EventStore(es)+1, streams#$EventStore(es)[handle := stream_new])))
}

function {:inline} $CondExtendEventStore(es: $EventStore, handle: $Event_EventHandle, msg: $Value, cond: bool): $EventStore {
    if cond then
        $ExtendEventStore(es, handle, msg)
    else
        es
}

var $es: $EventStore;
