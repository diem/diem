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
procedure {:inline 1} CreateAccount(addr_val: Value, addr_exists: [Address]bool)
returns (addr_exists': [Address]bool)
{
  var addr: Address;
  addr := a#Address(addr_val);
  assert !addr_exists[addr];
  addr_exists' := addr_exists[addr := true];
}


// everything below is auto generated

const unique t0_LocalName: LocalName;
const unique t1_LocalName: LocalName;
const unique t2_LocalName: LocalName;
const unique t3_LocalName: LocalName;
const unique t4_LocalName: LocalName;
const unique t5_LocalName: LocalName;
const unique t6_LocalName: LocalName;
const unique t7_LocalName: LocalName;
const unique t8_LocalName: LocalName;
const unique t9_LocalName: LocalName;
const unique t10_LocalName: LocalName;
const unique t11_LocalName: LocalName;
const unique t12_LocalName: LocalName;
const unique t13_LocalName: LocalName;
const unique t14_LocalName: LocalName;
const unique t15_LocalName: LocalName;
const unique t16_LocalName: LocalName;
const unique t17_LocalName: LocalName;
const unique t18_LocalName: LocalName;
const unique t19_LocalName: LocalName;
const unique t20_LocalName: LocalName;
const unique t21_LocalName: LocalName;
const unique t22_LocalName: LocalName;
const unique t23_LocalName: LocalName;

procedure {:inline 1} IsPrefixMax(dstPath: Path, srcPath: Path) returns (isPrefix: bool)
{
    if (srcPath == dstPath) {
        isPrefix := true;
    } else if (srcPath == Nil()) {
        isPrefix := false;
    } else {
        assert false;
    }
}

procedure {:inline 1} UpdateValueMax(srcPath: Path, srcValue: Value, dstPath: Path, dstValue: Value) returns (dstValue': Value)
{
    var e: Edge;
    var v': Value;
    if (srcPath == dstPath) {
        dstValue' := srcValue;
    } else {
        assume false;
    }
}

procedure TestArithmetic_add_two_number (c: CreationTime, addr_exists: [Address]bool, arg0: Value, arg1: Value) returns (addr_exists': [Address]bool, ret0: Value, ret1: Value)
ensures !abort_flag;
{
    // declare local variables
    var t0: Value; // int
    var t1: Value; // int
    var t2: Value; // int
    var t3: Value; // int
    var t4: Value; // int
    var t5: Value; // int
    var t6: Value; // int
    var t7: Value; // int
    var t8: Value; // int
    var t9: Value; // int

    // declare a new creation time for calls inside this function
    var c': CreationTime;
    assume c' > c;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#Integer(arg1);

    // assign arguments to locals so they can be modified
    t0 := arg0;
    t1 := arg1;

    // assign ResourceStores to locals so they can be modified
    addr_exists' := addr_exists;

    // bytecode translation starts here
    call t4 := CopyOrMoveValue(t0);

    call t5 := CopyOrMoveValue(t1);

    call t6 := Add(t4, t5);

    call t2 := CopyOrMoveValue(t6);

    call t7 := LdConst(3);

    call t3 := CopyOrMoveValue(t7);

    call t8 := CopyOrMoveValue(t3);

    call t9 := CopyOrMoveValue(t2);

    ret0 := t8;
    ret1 := t9;
    return;

}
procedure TestArithmetic_multiple_ops (c: CreationTime, addr_exists: [Address]bool, arg0: Value, arg1: Value, arg2: Value) returns (addr_exists': [Address]bool, ret0: Value)
ensures !abort_flag;
{
    // declare local variables
    var t0: Value; // int
    var t1: Value; // int
    var t2: Value; // int
    var t3: Value; // int
    var t4: Value; // int
    var t5: Value; // int
    var t6: Value; // int
    var t7: Value; // int
    var t8: Value; // int
    var t9: Value; // int

    // declare a new creation time for calls inside this function
    var c': CreationTime;
    assume c' > c;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#Integer(arg1);
    assume is#Integer(arg2);

    // assign arguments to locals so they can be modified
    t0 := arg0;
    t1 := arg1;
    t2 := arg2;

    // assign ResourceStores to locals so they can be modified
    addr_exists' := addr_exists;

    // bytecode translation starts here
    call t4 := CopyOrMoveValue(t0);

    call t5 := CopyOrMoveValue(t1);

    call t6 := Add(t4, t5);

    call t7 := CopyOrMoveValue(t2);

    call t8 := Mul(t6, t7);

    call t3 := CopyOrMoveValue(t8);

    call t9 := CopyOrMoveValue(t3);

    ret0 := t9;
    return;

}
procedure TestArithmetic_bool_ops (c: CreationTime, addr_exists: [Address]bool, arg0: Value, arg1: Value) returns (addr_exists': [Address]bool)
ensures !abort_flag;
{
    // declare local variables
    var t0: Value; // int
    var t1: Value; // int
    var t2: Value; // bool
    var t3: Value; // bool
    var t4: Value; // int
    var t5: Value; // int
    var t6: Value; // bool
    var t7: Value; // int
    var t8: Value; // int
    var t9: Value; // bool
    var t10: Value; // bool
    var t11: Value; // int
    var t12: Value; // int
    var t13: Value; // bool
    var t14: Value; // int
    var t15: Value; // int
    var t16: Value; // bool
    var t17: Value; // bool
    var t18: Value; // bool
    var t19: Value; // bool
    var t20: Value; // bool
    var t21: Value; // bool
    var t22: Value; // int

    // declare a new creation time for calls inside this function
    var c': CreationTime;
    assume c' > c;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#Integer(arg1);

    // assign arguments to locals so they can be modified
    t0 := arg0;
    t1 := arg1;

    // assign ResourceStores to locals so they can be modified
    addr_exists' := addr_exists;

    // bytecode translation starts here
    call t4 := CopyOrMoveValue(t0);

    call t5 := CopyOrMoveValue(t1);

    call t6 := Gt(t4, t5);

    call t7 := CopyOrMoveValue(t0);

    call t8 := CopyOrMoveValue(t1);

    call t9 := Ge(t7, t8);

    call t10 := And(t6, t9);

    call t2 := CopyOrMoveValue(t10);

    call t11 := CopyOrMoveValue(t0);

    call t12 := CopyOrMoveValue(t1);

    call t13 := Lt(t11, t12);

    call t14 := CopyOrMoveValue(t0);

    call t15 := CopyOrMoveValue(t1);

    call t16 := Le(t14, t15);

    call t17 := Or(t13, t16);

    call t3 := CopyOrMoveValue(t17);

    call t18 := CopyOrMoveValue(t2);

    call t19 := CopyOrMoveValue(t3);

    call t20 := Neq_bool(t18, t19);

    call t21 := Not(t20);

    if (!b#Boolean(t21)) { goto Label_23; }

    call t22 := LdConst(42);

    assert false;

Label_23:
    return;

}
procedure TestArithmetic_arithmetic_ops (c: CreationTime, addr_exists: [Address]bool, arg0: Value, arg1: Value) returns (addr_exists': [Address]bool, ret0: Value, ret1: Value)
ensures !abort_flag;
{
    // declare local variables
    var t0: Value; // int
    var t1: Value; // int
    var t2: Value; // int
    var t3: Value; // int
    var t4: Value; // int
    var t5: Value; // int
    var t6: Value; // int
    var t7: Value; // int
    var t8: Value; // int
    var t9: Value; // int
    var t10: Value; // int
    var t11: Value; // int
    var t12: Value; // int
    var t13: Value; // int
    var t14: Value; // int
    var t15: Value; // int
    var t16: Value; // bool
    var t17: Value; // bool
    var t18: Value; // int
    var t19: Value; // int
    var t20: Value; // int

    // declare a new creation time for calls inside this function
    var c': CreationTime;
    assume c' > c;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#Integer(arg1);

    // assign arguments to locals so they can be modified
    t0 := arg0;
    t1 := arg1;

    // assign ResourceStores to locals so they can be modified
    addr_exists' := addr_exists;

    // bytecode translation starts here
    call t3 := LdConst(6);

    call t4 := LdConst(4);

    call t5 := Add(t3, t4);

    call t6 := LdConst(1);

    call t7 := Sub(t5, t6);

    call t8 := LdConst(2);

    call t9 := Mul(t7, t8);

    call t10 := LdConst(3);

    call t11 := Div(t9, t10);

    call t12 := LdConst(4);

    call t13 := Mod(t11, t12);

    call t2 := CopyOrMoveValue(t13);

    call t14 := CopyOrMoveValue(t2);

    call t15 := LdConst(2);

    call t16 := Eq_int(t14, t15);

    call t17 := Not(t16);

    if (!b#Boolean(t17)) { goto Label_19; }

    call t18 := LdConst(42);

    assert false;

Label_19:
    call t19 := CopyOrMoveValue(t2);

    call t20 := CopyOrMoveValue(t0);

    ret0 := t19;
    ret1 := t20;
    return;

}
procedure TestArithmetic_overflow (c: CreationTime, addr_exists: [Address]bool) returns (addr_exists': [Address]bool)
ensures !abort_flag;
{
    // declare local variables
    var t0: Value; // int
    var t1: Value; // int
    var t2: Value; // int
    var t3: Value; // int
    var t4: Value; // int
    var t5: Value; // int

    // declare a new creation time for calls inside this function
    var c': CreationTime;
    assume c' > c;
    assume !abort_flag;

    // assume arguments are of correct types

    // assign arguments to locals so they can be modified

    // assign ResourceStores to locals so they can be modified
    addr_exists' := addr_exists;

    // bytecode translation starts here
    call t2 := LdConst(9223372036854775807);

    call t0 := CopyOrMoveValue(t2);

    call t3 := CopyOrMoveValue(t0);

    call t4 := LdConst(1);

    call t5 := Add(t3, t4);

    call t1 := CopyOrMoveValue(t5);

    return;

}
procedure TestArithmetic_underflow (c: CreationTime, addr_exists: [Address]bool) returns (addr_exists': [Address]bool)
ensures !abort_flag;
{
    // declare local variables
    var t0: Value; // int
    var t1: Value; // int
    var t2: Value; // int
    var t3: Value; // int
    var t4: Value; // int
    var t5: Value; // int

    // declare a new creation time for calls inside this function
    var c': CreationTime;
    assume c' > c;
    assume !abort_flag;

    // assume arguments are of correct types

    // assign arguments to locals so they can be modified

    // assign ResourceStores to locals so they can be modified
    addr_exists' := addr_exists;

    // bytecode translation starts here
    call t2 := LdConst(0);

    call t0 := CopyOrMoveValue(t2);

    call t3 := CopyOrMoveValue(t0);

    call t4 := LdConst(1);

    call t5 := Sub(t3, t4);

    call t1 := CopyOrMoveValue(t5);

    return;

}
procedure TestArithmetic_div_by_zero (c: CreationTime, addr_exists: [Address]bool) returns (addr_exists': [Address]bool)
ensures !abort_flag;
{
    // declare local variables
    var t0: Value; // int
    var t1: Value; // int
    var t2: Value; // int
    var t3: Value; // int
    var t4: Value; // int
    var t5: Value; // int

    // declare a new creation time for calls inside this function
    var c': CreationTime;
    assume c' > c;
    assume !abort_flag;

    // assume arguments are of correct types

    // assign arguments to locals so they can be modified

    // assign ResourceStores to locals so they can be modified
    addr_exists' := addr_exists;

    // bytecode translation starts here
    call t2 := LdConst(0);

    call t0 := CopyOrMoveValue(t2);

    call t3 := LdConst(1);

    call t4 := CopyOrMoveValue(t0);

    call t5 := Div(t3, t4);

    call t1 := CopyOrMoveValue(t5);

    return;

}
