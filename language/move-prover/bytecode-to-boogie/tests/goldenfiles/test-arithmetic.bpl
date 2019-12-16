

// ** structs of module TestArithmetic



// ** stratified functions

procedure {:inline 1} ReadValue0(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        assert false;
    }
}

procedure {:inline 1} ReadValueMax(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        e := p#Path(p)[i];
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
        call v' := ReadValue0(p, i+1, v');
    }
}

procedure {:inline 1} UpdateValue0(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        assert false;
    }
}

procedure {:inline 1} UpdateValueMax(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        e := p#Path(p)[i];
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
        call v' := UpdateValue0(p, i+1, v', new_v);
        if (is#Struct(v)) { v' := mk_struct(smap(v)[f#Field(e) := v'], slen(v));}
        if (is#Vector(v)) { v' := mk_vector(vmap(v)[i#Index(e) := v'], vlen(v));}
    }
}



// ** functions of module TestArithmetic

procedure {:inline 1} TestArithmetic_add_two_number (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);
    assume has_type(IntegerType(), arg1);

    old_size := m_size;
    m_size := m_size + 10;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+4], contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := LdConst(3);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+8];
    ret1 := contents#Memory(m)[old_size+9];
    return;

}

procedure TestArithmetic_add_two_number_verify (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
{
    call ret0, ret1 := TestArithmetic_add_two_number(arg0, arg1);
}

procedure {:inline 1} TestArithmetic_multiple_ops (arg0: Value, arg1: Value, arg2: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);
    assume has_type(IntegerType(), arg1);
    assume has_type(IntegerType(), arg2);

    old_size := m_size;
    m_size := m_size + 10;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size :=  arg2]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+4], contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call tmp := Mul(contents#Memory(m)[old_size+6], contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+9];
    return;

}

procedure TestArithmetic_multiple_ops_verify (arg0: Value, arg1: Value, arg2: Value) returns (ret0: Value)
{
    call ret0 := TestArithmetic_multiple_ops(arg0, arg1, arg2);
}

procedure {:inline 1} TestArithmetic_bool_ops (arg0: Value, arg1: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // BooleanType()
    var t3: Value; // BooleanType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // BooleanType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // BooleanType()
    var t10: Value; // BooleanType()
    var t11: Value; // IntegerType()
    var t12: Value; // IntegerType()
    var t13: Value; // BooleanType()
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var t16: Value; // BooleanType()
    var t17: Value; // BooleanType()
    var t18: Value; // BooleanType()
    var t19: Value; // BooleanType()
    var t20: Value; // BooleanType()
    var t21: Value; // BooleanType()
    var t22: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);
    assume has_type(IntegerType(), arg1);

    old_size := m_size;
    m_size := m_size + 23;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := Gt(contents#Memory(m)[old_size+4], contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := Ge(contents#Memory(m)[old_size+7], contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := And(contents#Memory(m)[old_size+6], contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+10]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call tmp := Lt(contents#Memory(m)[old_size+11], contents#Memory(m)[old_size+12]);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    call tmp := Le(contents#Memory(m)[old_size+14], contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call tmp := Or(contents#Memory(m)[old_size+13], contents#Memory(m)[old_size+16]);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+17]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    tmp := Boolean(!is_equal(BooleanType(), contents#Memory(m)[old_size+18], contents#Memory(m)[old_size+19]));
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+20]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 21];
    if (!b#Boolean(tmp)) { goto Label_23; }

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[22+old_size := true], contents#Memory(m)[22+old_size := tmp]);

    assert false;

Label_23:
    return;

}

procedure TestArithmetic_bool_ops_verify (arg0: Value, arg1: Value) returns ()
{
    call TestArithmetic_bool_ops(arg0, arg1);
}

procedure {:inline 1} TestArithmetic_arithmetic_ops (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()
    var t10: Value; // IntegerType()
    var t11: Value; // IntegerType()
    var t12: Value; // IntegerType()
    var t13: Value; // IntegerType()
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var t16: Value; // BooleanType()
    var t17: Value; // BooleanType()
    var t18: Value; // IntegerType()
    var t19: Value; // IntegerType()
    var t20: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);
    assume has_type(IntegerType(), arg1);

    old_size := m_size;
    m_size := m_size + 21;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := LdConst(6);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := LdConst(4);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+3], contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := Sub(contents#Memory(m)[old_size+5], contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call tmp := LdConst(2);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := Mul(contents#Memory(m)[old_size+7], contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := LdConst(3);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := Div(contents#Memory(m)[old_size+9], contents#Memory(m)[old_size+10]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := LdConst(4);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call tmp := Mod(contents#Memory(m)[old_size+11], contents#Memory(m)[old_size+12]);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+13]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := LdConst(2);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    tmp := Boolean(is_equal(IntegerType(), contents#Memory(m)[old_size+14], contents#Memory(m)[old_size+15]));
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+16]);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 17];
    if (!b#Boolean(tmp)) { goto Label_19; }

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    assert false;

Label_19:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+19];
    ret1 := contents#Memory(m)[old_size+20];
    return;

}

procedure TestArithmetic_arithmetic_ops_verify (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
{
    call ret0, ret1 := TestArithmetic_arithmetic_ops(arg0, arg1);
}

procedure {:inline 1} TestArithmetic_overflow () returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 6;

    // bytecode translation starts here
    call tmp := LdConst(9223372036854775807);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+3], contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    return;

}

procedure TestArithmetic_overflow_verify () returns ()
{
    call TestArithmetic_overflow();
}

procedure {:inline 1} TestArithmetic_underflow () returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 6;

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := Sub(contents#Memory(m)[old_size+3], contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    return;

}

procedure TestArithmetic_underflow_verify () returns ()
{
    call TestArithmetic_underflow();
}

procedure {:inline 1} TestArithmetic_div_by_zero () returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 6;

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := Div(contents#Memory(m)[old_size+3], contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    return;

}

procedure TestArithmetic_div_by_zero_verify () returns ()
{
    call TestArithmetic_div_by_zero();
}
