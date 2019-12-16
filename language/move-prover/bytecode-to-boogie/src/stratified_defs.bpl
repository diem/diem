function {:inline} IsEqual2(v1: Value, v2: Value): bool {
    (v1 == v2)
}
function {:inline} IsEqual1(v1: Value, v2: Value): bool {
    (v1 == v2) ||
    (is#Vector(v1) &&
     is#Vector(v2) &&
     vlen(v1) == vlen(v2) &&
     (forall i: int :: 0 <= i && i < vlen(v1) ==> IsEqual2(vmap(v1)[i], vmap(v2)[i])))
}
function {:inline} IsEqual0(v1: Value, v2: Value): bool {
    (v1 == v2) ||
    (is#Vector(v1) &&
     is#Vector(v2) &&
     vlen(v1) == vlen(v2) &&
     (forall i: int :: 0 <= i && i < vlen(v1) ==> IsEqual1(vmap(v1)[i], vmap(v2)[i])))
}

function {:inline} ReadValue2(p: Path, v: Value) : Value
{
    v
}
function {:inline} ReadValue1(p: Path, v: Value) : Value
{
    if (1 == size#Path(p))
        then v
    else
        ReadValue2(p, vmap(v)[vector_index(p, 1)])
}
function {:inline} ReadValue0(p: Path, v: Value) : Value
{
    if (0 == size#Path(p))
        then v
    else
        ReadValue1(p, vmap(v)[vector_index(p, 0)])
}

function {:inline} UpdateValue2(p: Path, v: Value, new_v: Value): Value
{
    new_v
}
function {:inline} UpdateValue1(p: Path, v: Value, new_v: Value): Value
{
    if (1 == size#Path(p))
        then new_v
    else
        update_vector(v, vector_index(p, 1), UpdateValue2(p, vmap(v)[vector_index(p, 1)], new_v))
}
function {:inline} UpdateValue0(p: Path, v: Value, new_v: Value): Value
{
    if (0 == size#Path(p))
        then new_v
    else
        update_vector(v, vector_index(p, 0), UpdateValue1(p, vmap(v)[vector_index(p, 0)], new_v))
}
