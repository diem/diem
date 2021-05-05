// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Boogie model for vectors, based on Boogie arrays.

// This version of vectors uses an encoding of an internalization function to
// achieve extensional equality on vectors.

type {:datatype} Vec _;

function {:constructor} Vec<T>(v: [int]T, l: int): Vec T;

function {:builtin "MapConst"} MapConstVec<T>(T): [int]T;
function {:inline} DefaultVecElem<T>(): T;
function {:inline} DefaultVecMap<T>(): [int]T { MapConstVec(DefaultVecElem()) }

axiom {:ctor "Vec"} (forall<T> x: Vec T :: {l#Vec(x)} l#Vec(x) >= 0);

function {:inline} VecIntern<T>(v: Vec T): Vec T;

axiom {:ctor "Vec"} (forall<T> v: Vec T :: {VecIntern(v)} VecRepIsEqual(v, VecIntern(v)));

axiom {:ctor "Vec"} (forall<T> v1, v2: Vec T ::
    {VecRepIsEqual(v1, v2)}{VecIntern(v1), VecIntern(v2)}
    VecRepIsEqual(v1, v2) <==> VecIntern(v1) == VecIntern(v2));

function VecRepIsEqual<T>(v1: Vec T, v2: Vec T): bool {
    v1 == v2 ||
    (var l := l#Vec(v1);
     l == l#Vec(v2) &&
     (forall i: int :: //{v#Vec(v1)[i], v#Vec(v2)[i]}
         i >= 0 && i < l ==> v#Vec(v1)[i] == v#Vec(v2)[i]))
}



function {:inline} EmptyVec<T>(): Vec T {
    VecIntern(Vec(DefaultVecMap(), 0))
}

function {:inline} MakeVec1<T>(v: T): Vec T {
    VecIntern(Vec(DefaultVecMap()[0 := v], 1))
}

function {:inline} MakeVec2<T>(v1: T, v2: T): Vec T {
    VecIntern(Vec(DefaultVecMap()[0 := v1][1 := v2], 2))
}

function {:inline} MakeVec3<T>(v1: T, v2: T, v3: T): Vec T {
    VecIntern(Vec(DefaultVecMap()[0 := v1][1 := v2][2 := v3], 3))
}

function {:inline} MakeVec4<T>(v1: T, v2: T, v3: T, v4: T): Vec T {
    VecIntern(Vec(DefaultVecMap()[0 := v1][1 := v2][2 := v3][3 := v4], 4))
}

function {:inline} ExtendVec<T>(v: Vec T, elem: T): Vec T {
    (var l := l#Vec(v);
    VecIntern(Vec(v#Vec(v)[l := elem], l + 1)))
}

function {:inline} ReadVec<T>(v: Vec T, i: int): T {
    v#Vec(v)[i]
}

function {:inline} LenVec<T>(v: Vec T): int {
    l#Vec(v)
}

function {:inline} IsEmptyVec<T>(v: Vec T): bool {
    l#Vec(v) == 0
}

function {:inline} RemoveVec<T>(v: Vec T): Vec T {
    (var l := l#Vec(v) - 1;
    VecIntern(Vec(v#Vec(v)[l := DefaultVecElem()], l)))
}

function {:inline} RemoveAtVec<T>(v: Vec T, i: int): Vec T {
    (var l := l#Vec(v) - 1;
    VecIntern(Vec(
        (lambda j: int ::
           if j >= 0 && j < l then
               if j < i then v#Vec(v)[j] else v#Vec(v)[j+1]
           else DefaultVecElem()),
        l)))
}

function {:inline} ConcatVec<T>(v1: Vec T, v2: Vec T): Vec T {
    (var l1, m1, l2, m2 := l#Vec(v1), v#Vec(v1), l#Vec(v2), v#Vec(v2);
    VecIntern(Vec(
        (lambda i: int ::
          if i >= 0 && i < l1 + l2 then
            if i < l1 then m1[i] else m2[i - l1]
          else DefaultVecElem()),
        l1 + l2)))
}

function {:inline} ReverseVec<T>(v: Vec T): Vec T {
    (var l := l#Vec(v);
    VecIntern(Vec(
        (lambda i: int :: if 0 <= i && i < l then v#Vec(v)[l - i - 1] else DefaultVecElem()),
        l)))
}

function {:inline} SliceVec<T>(v: Vec T, i: int, j: int): Vec T {
    (var m := v#Vec(v);
    VecIntern(Vec(
        (lambda k:int ::
          if 0 <= k && k < j - i then
            m[i + k]
          else
            DefaultVecElem()),
        (if j - i < 0 then 0 else j - i))))
}


function {:inline} UpdateVec<T>(v: Vec T, i: int, elem: T): Vec T {
    VecIntern(Vec(v#Vec(v)[i := elem], l#Vec(v)))
}

function {:inline} SwapVec<T>(v: Vec T, i: int, j: int): Vec T {
    (var m := v#Vec(v);
    VecIntern(Vec(m[i := m[j]][j := m[i]], l#Vec(v))))
}

function {:inline} ContainsVec<T>(v: Vec T, e: T): bool {
    (var l := l#Vec(v);
    (exists i: int :: i >= 0 && i < l && v#Vec(v)[i] == e))
}

function IndexOfVec<T>(v: Vec T, e: T): int;
axiom {:ctor "Vec"} (forall<T> v: Vec T, e: T :: {IndexOfVec(v, e)}
    (var i := IndexOfVec(v,e);
     if (!ContainsVec(v, e)) then i == -1
     else InRangeVec(v, i) && ReadVec(v, i) == e &&
        (forall j: int :: j >= 0 && j < i ==> ReadVec(v, j) != e)));

function {:inline} InRangeVec<T>(v: Vec T, i: int): bool {
    i >= 0 && i < LenVec(v)
}
