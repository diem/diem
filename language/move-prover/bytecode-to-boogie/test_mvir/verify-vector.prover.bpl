function {:inline 1} vector_length(v : Value) : Value {
    Integer(vlen(v))
}

function {:inline 1} vector_get(v : Value, i : Value) : Value {
    vmap(v)[i#Integer(i)]
}

function {:inline 1} vector_update(v : Value, i : Value, e : Value) : Value {
    update_vector(v, i#Integer(i), e)
}
