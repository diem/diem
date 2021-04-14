// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Boogie model for vectors, based on smt arrays.

// This extends the basic array theory by an axiom to make it extensional. Those axioms use
// the Z3 builtins for map operations on arrays, so this theory requires native array theory.

// TODO: this theory is currently not working well; reproduce
// by `mvp --vnext --vector-theory=SmtArrayExt simple_vector_client.move`

{% include "vector-array-theory" %}

axiom {:ctor "Vec"} (forall<T> v: Vec T :: {l#Vec(v)} l#Vec(v) >= 0);

axiom {:ctor "Vec"} (forall<T> v: Vec T :: {v#Vec(v)}{l#Vec(v)}
    MapIte(MapRange(0, l#Vec(v)), MapConstVec(DefaultVecElem()), v#Vec(v))
                == MapConstVec(DefaultVecElem()));

function {:builtin "MapIte"} MapIte<T,U>([T]bool, [T]U, [T]U) : [T]U;
function {:builtin "MapAnd"} MapAnd<T>([T]bool, [T]bool) : [T]bool;
function {:builtin "MapNot"} MapNot<T>([T]bool) : [T]bool;
function {:builtin "MapLe"} MapLe<T>([T]int, [T]int) : [T]bool;

const Identity: [int]int;
axiom (forall x: int :: Identity[x] == x);

function {:inline} AtLeast(x: int): [int]bool { MapLe(MapConstVec(x), Identity) }
function {:inline} MapDiff<T>(a: [T]bool, b: [T]bool) : [T]bool { MapAnd(a, MapNot(b)) }
function {:inline} MapRange(from: int, n: int): [int]bool { MapDiff(AtLeast(from), AtLeast(from + n)) }
