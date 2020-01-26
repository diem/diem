// Boogie helper functions for test-specs-translate.mvir

function {:inline} number_in_range(x: Value): Value {
  Boolean(i#Integer(x) >= 0 && i#Integer(x) < 128)
}
