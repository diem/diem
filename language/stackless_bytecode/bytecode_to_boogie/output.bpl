type TypeName;
type FieldName;
type LocalName;
type Address;
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
        dst := GlobalReference(a#GlobalReference(src), t#GlobalReference(src), Cons(p#GlobalReference(src), Field(f)), m#Map(v#GlobalReference(src))[Field(f)]);
    } else {
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

procedure {:inline 1} Add(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Integer(i#Integer(src1) + i#Integer(src2));
}

procedure {:inline 1} Sub(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Integer(i#Integer(src1) - i#Integer(src2));
}

procedure {:inline 1} Mul(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Integer(i#Integer(src1) * i#Integer(src2));
}

procedure {:inline 1} Div(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Integer(i#Integer(src1) div i#Integer(src2));
}

procedure {:inline 1} Mod(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Integer(i#Integer(src1) mod i#Integer(src2));
}

procedure {:inline 1} Lt(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Boolean(i#Integer(src1) < i#Integer(src2));
}

procedure {:inline 1} Gt(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Boolean(i#Integer(src1) > i#Integer(src2));
}

procedure {:inline 1} Le(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Boolean(i#Integer(src1) <= i#Integer(src2));
}

procedure {:inline 1} Ge(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Boolean(i#Integer(src1) >= i#Integer(src2));
}

procedure {:inline 1} And(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Boolean(b#Boolean(src1) && b#Boolean(src2));
}

procedure {:inline 1} Or(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Boolean(b#Boolean(src1) || b#Boolean(src2));
}

procedure {:inline 1} Not(src: Value) returns (dst: Value)
{
    dst := Boolean(!b#Boolean(src));
}

procedure {:inline 1} Eq_int(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Boolean(i#Integer(src1) == i#Integer(src2));
}

procedure {:inline 1} Neq_int(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Boolean(i#Integer(src1) != i#Integer(src2));
}

procedure {:inline 1} Eq_bool(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Boolean(b#Boolean(src1) == b#Boolean(src2));
}

procedure {:inline 1} Neq_bool(src1: Value, src2: Value) returns (dst: Value)
{
    dst := Boolean(b#Boolean(src1) != b#Boolean(src2));
}

procedure {:inline 1} LdConst(val: int) returns (ret: Value)
{
    ret := Integer(val);
}

procedure {:inline 1} LdTrue() returns (ret: Value)
{
    ret := Boolean(true);
}

procedure {:inline 1} LdFalse() returns (ret: Value)
{
    ret := Boolean(false);
}


// everything below is auto generated

const unique Test3_T: TypeName;
const unique Test3_T_f: FieldName;
const unique Test3_T_g: FieldName;

procedure {:inline 1} Pack_Test3_T(v0: Value, v1: Value) returns (v: Value)
{
    v := Map(DefaultMap[Field(Test3_T_f) := v0][Field(Test3_T_g) := v1]);
}

procedure {:inline 1} Unpack_Test3_T(v: Value) returns (v0: Value, v1: Value)
{
    v0 := m#Map(v)[Field(Test3_T_f)];
    v1 := m#Map(v)[Field(Test3_T_g)];
}

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
const unique t24_LocalName: LocalName;
const unique t25_LocalName: LocalName;
const unique t26_LocalName: LocalName;
const unique t27_LocalName: LocalName;
const unique t28_LocalName: LocalName;
const unique t29_LocalName: LocalName;
const unique t30_LocalName: LocalName;
const unique t31_LocalName: LocalName;
const unique t32_LocalName: LocalName;
const unique t33_LocalName: LocalName;
const unique t34_LocalName: LocalName;
const unique t35_LocalName: LocalName;
const unique t36_LocalName: LocalName;
const unique t37_LocalName: LocalName;
const unique t38_LocalName: LocalName;
const unique t39_LocalName: LocalName;
const unique t40_LocalName: LocalName;
const unique t41_LocalName: LocalName;
const unique t42_LocalName: LocalName;
const unique t43_LocalName: LocalName;
const unique t44_LocalName: LocalName;
const unique t45_LocalName: LocalName;
const unique t46_LocalName: LocalName;
const unique t47_LocalName: LocalName;
const unique t48_LocalName: LocalName;
const unique t49_LocalName: LocalName;
const unique t50_LocalName: LocalName;
const unique t51_LocalName: LocalName;
const unique t52_LocalName: LocalName;
const unique t53_LocalName: LocalName;
const unique t54_LocalName: LocalName;
const unique t55_LocalName: LocalName;
const unique t56_LocalName: LocalName;
const unique t57_LocalName: LocalName;
const unique t58_LocalName: LocalName;
const unique t59_LocalName: LocalName;
const unique t60_LocalName: LocalName;
const unique t61_LocalName: LocalName;
const unique t62_LocalName: LocalName;
const unique t63_LocalName: LocalName;

procedure {:inline 1} IsPrefix0(dstPath: Path, srcPath: Path) returns (isPrefix: bool)
{
    if (srcPath == dstPath) {
        isPrefix := true;
    } else if (srcPath == Nil()) {
        isPrefix := false;
    } else {
        assert false;
    }
}

procedure {:inline 1} IsPrefixMax(dstPath: Path, srcPath: Path) returns (isPrefix: bool)
{
    if (srcPath == dstPath) {
        isPrefix := true;
    } else if (srcPath == Nil()) {
        isPrefix := false;
    } else {
        call isPrefix := IsPrefix0(dstPath, p#Cons(srcPath));
    }
}

procedure {:inline 1} UpdateValue0(srcPath: Path, srcValue: Value, dstPath: Path, dstValue: Value) returns (dstValue': Value)
{
    var e: Edge;
    var v': Value;
    if (srcPath == dstPath) {
        dstValue' := srcValue;
    } else {
        assume false;
    }
}

procedure {:inline 1} UpdateValueMax(srcPath: Path, srcValue: Value, dstPath: Path, dstValue: Value) returns (dstValue': Value)
{
    var e: Edge;
    var v': Value;
    if (srcPath == dstPath) {
        dstValue' := srcValue;
    } else {
        call v' := UpdateValue0(srcPath, srcValue, Cons(dstPath, e), m#Map(dstValue)[e]);
        dstValue' := Map(m#Map(dstValue)[e := v']);
    }
}

procedure Test3_test3 (c: CreationTime, t0: Value, rs_Test3_T: ResourceStore) returns (rs_Test3_T': ResourceStore) {
    // declare local variables
    var t1: Value; // Test3_T
    var t2: Reference; // Test3_T_ref
    var t3: Reference; // int_ref
    var t4: Reference; // int_ref
    var t5: Reference; // int_ref
    var t6: Value; // int
    var t7: Value; // int
    var t8: Value; // int
    var t9: Value; // int
    var t10: Value; // Test3_T
    var t11: Reference; // Test3_T_ref
    var t12: Value; // bool
    var t13: Reference; // Test3_T_ref
    var t14: Reference; // int_ref
    var t15: Reference; // Test3_T_ref
    var t16: Reference; // int_ref
    var t17: Value; // int
    var t18: Reference; // int_ref
    var t19: Reference; // Test3_T_ref
    var t20: Reference; // int_ref
    var t21: Reference; // int_ref
    var t22: Reference; // Test3_T_ref
    var t23: Reference; // int_ref
    var t24: Reference; // int_ref
    var t25: Reference; // int_ref
    var t26: Value; // int
    var t27: Reference; // int_ref
    var t28: Value; // int
    var t29: Value; // bool
    var t30: Value; // int
    var t31: Value; // int
    var t32: Value; // bool
    var t33: Value; // bool
    var t34: Value; // int
    var t35: Value; // int
    var t36: Value; // int
    var t37: Value; // bool
    var t38: Value; // bool
    var t39: Value; // int
    var t40: Value; // int
    var t41: Value; // int
    var t42: Value; // bool
    var t43: Value; // bool
    var t44: Value; // int
    var t45: Value; // int
    var t46: Value; // int
    var t47: Value; // bool
    var t48: Value; // bool
    var t49: Value; // int

    // declare a new creation time for calls inside this functions;
    var c': CreationTime;
    assume c' > c;
    rs_Test3_T' := rs_Test3_T;

    // bytecode translation starts here
    call t8 := LdConst(0);

    call t9 := LdConst(0);

    call t10 := Pack_Test3_T(t8, t9);

    call t1 := CopyOrMoveValue(t10);

    call t11 := BorrowLoc(c, t1_LocalName, t1);

    call t2 := CopyOrMoveRef(t11);

    call t12 := CopyOrMoveValue(t0);

    if (!b#Boolean(t12)) { goto Label_12; }

    call t13 := CopyOrMoveRef(t2);

    call t14 := BorrowField(t13, Test3_T_f);

    call t3 := CopyOrMoveRef(t14);

    goto Label_15;

Label_12:
    call t15 := CopyOrMoveRef(t2);

    call t16 := BorrowField(t15, Test3_T_g);

    call t3 := CopyOrMoveRef(t16);

Label_15:
    call t17 := LdConst(10);

    call t18 := CopyOrMoveRef(t3);

    call t18 := WriteRef(t18, t17);

    call t2 := DeepUpdateReference(t18, t2);
    call t3 := DeepUpdateReference(t18, t3);
    call t4 := DeepUpdateReference(t18, t4);
    call t5 := DeepUpdateReference(t18, t5);
    call t11 := DeepUpdateReference(t18, t11);
    call t13 := DeepUpdateReference(t18, t13);
    call t14 := DeepUpdateReference(t18, t14);
    call t15 := DeepUpdateReference(t18, t15);
    call t16 := DeepUpdateReference(t18, t16);
    call t19 := DeepUpdateReference(t18, t19);
    call t20 := DeepUpdateReference(t18, t20);
    call t21 := DeepUpdateReference(t18, t21);
    call t22 := DeepUpdateReference(t18, t22);
    call t23 := DeepUpdateReference(t18, t23);
    call t24 := DeepUpdateReference(t18, t24);
    call t25 := DeepUpdateReference(t18, t25);
    call t27 := DeepUpdateReference(t18, t27);
    call rs_Test3_T' := DeepUpdateGlobal(Test3_T, t18, rs_Test3_T');
    call t1 := DeepUpdateLocal(c, t1_LocalName, t18, t1);
    call t6 := DeepUpdateLocal(c, t6_LocalName, t18, t6);
    call t7 := DeepUpdateLocal(c, t7_LocalName, t18, t7);
    call t8 := DeepUpdateLocal(c, t8_LocalName, t18, t8);
    call t9 := DeepUpdateLocal(c, t9_LocalName, t18, t9);
    call t10 := DeepUpdateLocal(c, t10_LocalName, t18, t10);
    call t12 := DeepUpdateLocal(c, t12_LocalName, t18, t12);
    call t26 := DeepUpdateLocal(c, t26_LocalName, t18, t26);
    call t28 := DeepUpdateLocal(c, t28_LocalName, t18, t28);
    call t29 := DeepUpdateLocal(c, t29_LocalName, t18, t29);
    call t30 := DeepUpdateLocal(c, t30_LocalName, t18, t30);
    call t31 := DeepUpdateLocal(c, t31_LocalName, t18, t31);
    call t32 := DeepUpdateLocal(c, t32_LocalName, t18, t32);
    call t33 := DeepUpdateLocal(c, t33_LocalName, t18, t33);
    call t34 := DeepUpdateLocal(c, t34_LocalName, t18, t34);
    call t35 := DeepUpdateLocal(c, t35_LocalName, t18, t35);
    call t36 := DeepUpdateLocal(c, t36_LocalName, t18, t36);
    call t37 := DeepUpdateLocal(c, t37_LocalName, t18, t37);
    call t38 := DeepUpdateLocal(c, t38_LocalName, t18, t38);
    call t39 := DeepUpdateLocal(c, t39_LocalName, t18, t39);
    call t40 := DeepUpdateLocal(c, t40_LocalName, t18, t40);
    call t41 := DeepUpdateLocal(c, t41_LocalName, t18, t41);
    call t42 := DeepUpdateLocal(c, t42_LocalName, t18, t42);
    call t43 := DeepUpdateLocal(c, t43_LocalName, t18, t43);
    call t44 := DeepUpdateLocal(c, t44_LocalName, t18, t44);
    call t45 := DeepUpdateLocal(c, t45_LocalName, t18, t45);
    call t46 := DeepUpdateLocal(c, t46_LocalName, t18, t46);
    call t47 := DeepUpdateLocal(c, t47_LocalName, t18, t47);
    call t48 := DeepUpdateLocal(c, t48_LocalName, t18, t48);
    call t49 := DeepUpdateLocal(c, t49_LocalName, t18, t49);

    call t19 := CopyOrMoveRef(t2);

    call t20 := BorrowField(t19, Test3_T_f);

    call t21 := FreezeRef(t20);

    call t4 := CopyOrMoveRef(t21);

    call t22 := CopyOrMoveRef(t2);

    call t23 := BorrowField(t22, Test3_T_g);

    call t24 := FreezeRef(t23);

    call t5 := CopyOrMoveRef(t24);

    call t25 := CopyOrMoveRef(t4);

    call t26 := ReadRef(t25);

    call t6 := CopyOrMoveValue(t26);

    call t27 := CopyOrMoveRef(t5);

    call t28 := ReadRef(t27);

    call t7 := CopyOrMoveValue(t28);

    call t29 := CopyOrMoveValue(t0);

    if (!b#Boolean(t29)) { goto Label_49; }

    call t30 := CopyOrMoveValue(t6);

    call t31 := LdConst(10);

    call t32 := Eq_int(t30, t31);

    call t33 := Not(t32);

    if (!b#Boolean(t33)) { goto Label_41; }

    call t34 := LdConst(42);

    // abort not supported

Label_41:
    call t35 := CopyOrMoveValue(t7);

    call t36 := LdConst(0);

    call t37 := Eq_int(t35, t36);

    call t38 := Not(t37);

    if (!b#Boolean(t38)) { goto Label_48; }

    call t39 := LdConst(42);

    // abort not supported

Label_48:
    goto Label_63;

Label_49:
    call t40 := CopyOrMoveValue(t6);

    call t41 := LdConst(0);

    call t42 := Eq_int(t40, t41);

    call t43 := Not(t42);

    if (!b#Boolean(t43)) { goto Label_56; }

    call t44 := LdConst(42);

    // abort not supported

Label_56:
    call t45 := CopyOrMoveValue(t7);

    call t46 := LdConst(10);

    call t47 := Eq_int(t45, t46);

    call t48 := Not(t47);

    if (!b#Boolean(t48)) { goto Label_63; }

    call t49 := LdConst(42);

    // abort not supported

Label_63:
    return;

}
