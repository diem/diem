// ================================================================================
// Domains

type TypeName;
type FieldName = int;
type LocalName;
type Address = int;
type ByteArray;
type String;
type Location = int;
type {:datatype} Edge;
function {:constructor} Field(f: FieldName): Edge;
function {:constructor} Index(i: int): Edge;
function {:constructor} String(s: String): Edge;

type {:datatype} Path;
function {:constructor} Path(p: [int]Edge, size: int): Path;

const DefaultPath: [int]Edge;

type {:datatype} TypeValue;
function {:constructor} BooleanType() : TypeValue;
function {:constructor} IntegerType() : TypeValue;
function {:constructor} AddressType() : TypeValue;
function {:constructor} ByteArrayType() : TypeValue;
function {:constructor} StrType() : TypeValue;
function {:constructor} ReferenceType(t: TypeValue) : TypeValue;
function {:constructor} VectorType(t: TypeValue) : TypeValue;
function {:constructor} StructType(name: TypeName, ts: TypeValueArray) : TypeValue;

type TypeValueArray;
function TypeValueArray(v: [int]TypeValue, l: int): TypeValueArray;
function v#TypeValueArray(tv: TypeValueArray): [int]TypeValue;
function l#TypeValueArray(tv: TypeValueArray): int;
axiom (forall v: [int]TypeValue, l: int :: v#TypeValueArray(TypeValueArray(v, l)) == v);
axiom (forall v: [int]TypeValue, l: int :: l >= 0 ==> l#TypeValueArray(TypeValueArray(v, l)) == l);
axiom (forall tv: TypeValueArray :: l#TypeValueArray(tv) >= 0);
axiom (forall v1, v2: [int]TypeValue, l1, l2: int ::
         TypeValueArray(v1, l1) == TypeValueArray(v2, l2) <==>
         l1 == l2 && (forall i: int :: i >= 0 && i < l1 ==> v1[i] == v2[i]));
const EmptyTypeValueArray: TypeValueArray;
axiom l#TypeValueArray(EmptyTypeValueArray) == 0;

type {:datatype} Value;
function {:constructor} Boolean(b: bool): Value;
function {:constructor} Integer(i: int): Value;
function {:constructor} Address(a: Address): Value;
function {:constructor} ByteArray(b: ByteArray): Value;
function {:constructor} Str(a: String): Value;
function {:constructor} Vector(v: ValueArray): Value;
function {:constructor} Struct(v: ValueArray): Value;

type  ValueArray;
function ValueArray(v: [int]Value, l: int): ValueArray;
function v#ValueArray(tv: ValueArray): [int]Value;
function l#ValueArray(tv: ValueArray): int;
axiom (forall v: [int]Value, l: int :: v#ValueArray(ValueArray(v, l)) == v);
axiom (forall v: [int]Value, l: int :: l >= 0 ==> l#ValueArray(ValueArray(v, l)) == l);
axiom (forall tv: ValueArray :: l#ValueArray(tv) >= 0);
axiom (forall v1, v2: [int]Value, l1, l2: int ::
         ValueArray(v1, l1) == ValueArray(v2, l2) <==>
         l1 == l2 && (forall i: int :: i >= 0 && i < l1 ==> v1[i] == v2[i]));
const EmptyValueArray: ValueArray;
axiom l#ValueArray(EmptyValueArray) == 0;


function {:inline} vlen(v: Value): int { l#ValueArray(v#Vector(v)) }
function {:inline} vmap(v: Value): [int]Value { v#ValueArray(v#Vector(v)) }
function {:inline} mk_vector(v: [int]Value, l: int): Value { Vector(ValueArray(v, l)) }
function {:inline} slen(v: Value): int { l#ValueArray(v#Struct(v)) }
function {:inline} smap(v: Value): [int]Value { v#ValueArray(v#Struct(v)) }
function {:inline} mk_struct(v: [int]Value, l: int): Value { Struct(ValueArray(v, l)) }


// Axiomatic definition of type test and equality.
function get_type(v: Value): TypeValue;
function {:inline} has_type(t: TypeValue, v: Value): bool;
function {:inline} is_equal(t: TypeValue, v1: Value, v2: Value): bool;

axiom (forall x: bool :: get_type(Boolean(x)) == BooleanType());
axiom (forall x: int :: get_type(Integer(x)) == IntegerType());
axiom (forall x: Address :: get_type(Address(x)) == AddressType());
axiom (forall x: ByteArray :: get_type(ByteArray(x)) == ByteArrayType());
axiom (forall x: String :: get_type(Str(x)) == StrType());
axiom (forall va: ValueArray :: (exists tv: TypeValue ::
        (l#ValueArray(va) > 0 ==> tv == get_type(v#ValueArray(va)[0]))
        && get_type(Vector(va)) == VectorType(tv)));
axiom (forall va: ValueArray :: (exists tn: TypeName, tva: TypeValueArray ::
        l#ValueArray(va) == l#TypeValueArray(tva)
        && (forall i: int :: i >= 0 && i < l#ValueArray(va) ==>
              v#TypeValueArray(tva)[i] == get_type(v#ValueArray(va)[i]))
        && get_type(Struct(va)) == StructType(tn, tva)));

axiom (forall v: Value, tv: TypeValue :: has_type(tv, v) <==> get_type(v) == tv);
axiom (forall tv: TypeValue, v1, v2: Value :: is_equal(tv, v1, v2) <==> v1 == v2);


const DefaultEdgeMap: [Edge]Value;
const DefaultIntMap: [int]Value;
const DefaultTypeMap: [int]TypeValue;

type {:datatype} Reference;
function {:constructor} Reference(rt: RefType, p: Path, l: Location): Reference;

type {:datatype} RefType;
function {:constructor} Global(a: Address, t: TypeName): RefType;
function {:constructor} Local(): RefType;

type {:datatype} TypeStore;
function {:constructor} TypeStore(domain: [TypeName]bool, contents: [TypeName]Location): TypeStore;

type {:datatype} GlobalStore;
function {:constructor} GlobalStore(domain: [Address]bool, contents: [Address]TypeStore): GlobalStore;

type {:datatype} Memory;
function {:constructor} Memory(domain: [Location]bool, contents: [Location]Value): Memory;

var gs : GlobalStore;
var m : Memory;
var m_size : int;


// ============================================================================================
// Spec support

// TODO: fill this in
function ExistsTxnSenderAccount(): bool;

// Tests whether resource exists.
function {:inline 1} ExistsResource(gs: GlobalStore, addr: Address, resource: TypeName): Value {
  Boolean(domain#TypeStore(contents#GlobalStore(gs)[addr])[resource])
}

// Obtains reference to the given resource.
function {:inline 1} GetResourceReference(gs: GlobalStore, addr: Address, resource: TypeName): Reference {
  Reference(
    Global(addr, resource),
    Path(DefaultPath, 0),
    contents#TypeStore(contents#GlobalStore(gs)[addr])[resource]
  )
}

// Obtains reference to local.
function {:inline 1} GetLocalReference(frame_idx: int, idx: int): Reference {
  Reference(
    Local(),
    Path(DefaultPath, 0),
    frame_idx + idx
  )
}

// Applies a field selection to the reference.
function {:inline 1} SelectField(ref: Reference, field: FieldName): Reference {
  Reference(
    rt#Reference(ref),
    Path(p#Path(p#Reference(ref))[size#Path(p#Reference(ref)) := Field(field)], size#Path(p#Reference(ref)) + 1),
    l#Reference(ref)
  )
}

// Dereferences a reference.
// TODO: this is currently not implemented, it would require the stratified functions to deal with
//   going through the Path. As we are refactoring this right now, it is kept open.
function {:inline 1} Dereference(m: Memory, ref: Reference): Value;



// ============================================================================================
// Instructions

procedure {:inline 1} Exists(address: Value, t: TypeValue) returns (dst: Value)
requires is#Address(address);
{
    assume is#StructType(t);
    dst := Boolean(domain#GlobalStore(gs)[a#Address(address)]
           && domain#TypeStore(contents#GlobalStore(gs)[a#Address(address)])[name#StructType(t)]);
}

procedure {:inline 1} MoveToSender(ta: TypeValue, v: Value)
{
    var a: Address;
    var ts: TypeStore;
    var t: TypeName;
    assume is#StructType(ta);
    t := name#StructType(ta);
    a := sender#Transaction_cons(txn);
    assert domain#GlobalStore(gs)[a];
    ts := contents#GlobalStore(gs)[a];
    assert !domain#TypeStore(ts)[t];
    ts := TypeStore(domain#TypeStore(ts)[t := true], contents#TypeStore(ts)[t := m_size]);
    gs := GlobalStore(domain#GlobalStore(gs), contents#GlobalStore(gs)[a := ts]);
    m := Memory(domain#Memory(m)[m_size := true], contents#Memory(m)[m_size := v]);
    m_size := m_size + 1;
}

procedure {:inline 1} MoveFrom(address: Value, ta: TypeValue) returns (dst: Value)
requires is#Address(address);
{
    var a: Address;
    var ts: TypeStore;
    var l: Location;
    var t: TypeName;
    a := a#Address(address);
    assert domain#GlobalStore(gs)[a];
    ts := contents#GlobalStore(gs)[a];
    assume is#StructType(ta);
    t := name#StructType(ta);
    assert domain#TypeStore(ts)[t];
    l := contents#TypeStore(ts)[t];
    assert domain#Memory(m)[l];
    dst := contents#Memory(m)[l];
    m := Memory(domain#Memory(m)[l := false], contents#Memory(m));
    ts := TypeStore(domain#TypeStore(ts)[t := false], contents#TypeStore(ts));
    gs := GlobalStore(domain#GlobalStore(gs), contents#GlobalStore(gs)[a := ts]);
}

procedure {:inline 1} BorrowGlobal(address: Value, ta: TypeValue) returns (dst: Reference)
requires is#Address(address);
{
    var a: Address;
    var v: Value;
    var ts: TypeStore;
    var l: Location;
    var t: TypeName;
    a := a#Address(address);
    assert domain#GlobalStore(gs)[a];
    ts := contents#GlobalStore(gs)[a];
    assume is#StructType(ta);
    t := name#StructType(ta);
    assert domain#TypeStore(ts)[t];
    l := contents#TypeStore(ts)[t];
    assert domain#Memory(m)[l];
    dst := Reference(Global(a, t), Path(DefaultPath, 0), l);
}

procedure {:inline 1} BorrowLoc(l: int) returns (dst: Reference)
{
    dst := Reference(Local(), Path(DefaultPath, 0), l);
}

procedure {:inline 1} BorrowField(src: Reference, f: FieldName) returns (dst: Reference)
{
    var p: Path;
    var size: int;
    p := p#Reference(src);
    size := size#Path(p);
	p := Path(p#Path(p)[size := Field(f)], size+1);
    dst := Reference(rt#Reference(src), p, l#Reference(src));
}

procedure {:inline 1} WriteRef(to: Reference, new_v: Value)
{
    var a: Address;
    var ts: TypeStore;
    var t: TypeName;
    var v: Value;
    var l: Location;
    if (is#Global(rt#Reference(to))) {
        a := a#Global(rt#Reference(to));
        assert domain#GlobalStore(gs)[a];
        ts := contents#GlobalStore(gs)[a];
        t := t#Global(rt#Reference(to));
        assert domain#TypeStore(ts)[t];
        l := l#Reference(to);
        v := contents#Memory(m)[l];
        call v := UpdateValueMax(p#Reference(to), 0, v, new_v);
        m := Memory(domain#Memory(m), contents#Memory(m)[l := v]);
    } else {
        v := contents#Memory(m)[l#Reference(to)];
        call v := UpdateValueMax(p#Reference(to), 0, v, new_v);
        m := Memory(domain#Memory(m), contents#Memory(m)[l#Reference(to) := v]);
    }
}

procedure {:inline 1} ReadRef(from: Reference) returns (v: Value)
{
    var a: Address;
    var ts: TypeStore;
    var t: TypeName;
    if (is#Global(rt#Reference(from))) {
        a := a#Global(rt#Reference(from));
        assert domain#GlobalStore(gs)[a];
        ts := contents#GlobalStore(gs)[a];
        t := t#Global(rt#Reference(from));
        assert domain#TypeStore(ts)[t];
        call v := ReadValueMax(p#Reference(from), 0, contents#Memory(m)[contents#TypeStore(ts)[t]]);
    } else {
        call v := ReadValueMax(p#Reference(from), 0, contents#Memory(m)[l#Reference(from)]);
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

function Vector_T_type_value(tv: TypeValue): TypeValue {
  VectorType(tv)
}

procedure {:inline 1} Vector_empty(ta: TypeValue) returns (v: Value) {
    v := mk_vector(DefaultIntMap, 0);
    assume has_type(VectorType(ta), v);
}

procedure {:inline 1} Vector_is_empty(ta: TypeValue, r: Reference) returns (b: Value) {
    var v: Value;
    call v := ReadRef(r);
    b := Boolean(vlen(v) == 0);
}

procedure {:inline 1} Vector_push_back(ta: TypeValue, r: Reference, val: Value) {
    var old_v: Value;
    var old_len: int;
    assume has_type(ta, val);
    call old_v := ReadRef(r);
    assume has_type(VectorType(ta), old_v);
    old_len := vlen(old_v);
    call WriteRef(r, mk_vector(vmap(old_v)[old_len := val], old_len+1));
}

procedure {:inline 1} Vector_pop_back(ta: TypeValue, r: Reference) returns (e: Value){
    var v: Value;
    var old_len: int;
    call v := ReadRef(r);
    assume has_type(VectorType(ta), v);
    old_len := vlen(v);
    e := vmap(v)[old_len-1];
    assume has_type(ta, e);
    call WriteRef(r, mk_vector(vmap(v), old_len-1));
}

procedure {:inline 1} Vector_append(ta: TypeValue, r: Reference, other_v: Value) {
    var v: Value;
    var old_len: int;
    var other_len: int;
    var result: Value;
    call v := ReadRef(r);
    assume has_type(VectorType(ta), v);
    assume has_type(VectorType(ta), other_v);
    old_len := vlen(v);
    other_len := vlen(other_v);
    result := mk_vector(
        (lambda i:int ::
            if i < old_len then
		        vmap(v)[i]
		    else
		        vmap(other_v)[i - old_len]
        ),
	    old_len + other_len);
	assume has_type(VectorType(ta), result);
    call WriteRef(r, result);
}

procedure {:inline 1} Vector_reverse(ta: TypeValue, r: Reference) {
    var v: Value;
    var result: Value;
    var len: int;

    call v := ReadRef(r);
    assume has_type(VectorType(ta), v);
    len := vlen(v);
    result := mk_vector(
        (lambda i:int ::
            vmap(v)[len-i-1]
        ),
	    len);
    assume has_type(VectorType(ta), result);
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
    dst := Reference(rt#Reference(src), p, l#Reference(src));
}

procedure {:inline 1} Vector_borrow_mut(ta: TypeValue, src: Reference, index: Value) returns (dst: Reference) {
    var p: Path;
    var size: int;
    p := p#Reference(src);
    size := size#Path(p);
	  p := Path(p#Path(p)[size := Index(i#Integer(index))], size+1);
    dst := Reference(rt#Reference(src), p, l#Reference(src));
}

procedure {:inline 1} Vector_destroy_empty(ta: TypeValue, v: Value) {
    assert (vlen(v) == 0);
}

procedure {:inline 1} Vector_swap(ta: TypeValue, src: Reference, i: Value, j: Value) {
    var i_val: Value;
    var j_val: Value;
    var i_ind: int;
    var j_ind: int;
    var v: Value;
    i_ind := i#Integer(i);
    j_ind := i#Integer(j);
    call v := ReadRef(src);
    assume has_type(VectorType(ta), v);
    assert (vlen(v) > i_ind && vlen(v) > j_ind);
    i_val := vmap(v)[i_ind];
    j_val := vmap(v)[j_ind];
    v := mk_vector(vmap(v)[i_ind := j_val][j_ind := i_val], vlen(v));
    call WriteRef(src, v);
}

procedure {:inline 1} Vector_get(ta: TypeValue, src: Reference, i: Value) returns (e: Value) {
    var i_ind: int;
    var v: Value;
    call v := ReadRef(src);
    assume has_type(VectorType(ta), v);
    i_ind := i#Integer(i);
    assert (i_ind < vlen(v));
    e := vmap(v)[i_ind];
    assume has_type(ta, e);
}

procedure {:inline 1} Vector_set(ta: TypeValue, src: Reference, i: Value, e: Value) {
    var i_ind: int;
    var v: Value;
    assume has_type(ta, e);
    i_ind := i#Integer(i);
    call v := ReadRef(src);
    assume has_type(VectorType(ta), v);
    assert (vlen(v) > i_ind);
    v := mk_vector(vmap(v)[i_ind := e], vlen(v));
    assume has_type(VectorType(ta), v);
    call WriteRef(src, v);
}
