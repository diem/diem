function {:inline 1} len(v : Value) : Value {
    Integer(vlen(v))
}

// deprecated
function {:inline 1} vector_length(v : Value) : Value {
    Integer(vlen(v))
}

function {:inline 1} vector_get(v : Value, i : Value) : Value {
    select_vector(v, i#Integer(i))
}

// deprecated
function {:inline 1} vector_get2(v : Value, i : Value) : Value {
    vmap(v)[i#Integer(i)]
}

function {:inline 1} vector_update(v : Value, i : Value, e : Value) : Value {
    update_vector(v, i#Integer(i), e)
}

function {:inline 1} vector_slice(v : Value, i : Value, j : Value) : Value {
    slice_vector(v, i#Integer(i), i#Integer(j))
}
