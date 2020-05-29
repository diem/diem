script {
fun main() {
  let x: u64;

  if (true) {
    if (true) return ();
  } else {
    if (true) return ();
  };
  x = 7;
  x;
}
}

// OLD check: INVALID_FALL_THROUGH
