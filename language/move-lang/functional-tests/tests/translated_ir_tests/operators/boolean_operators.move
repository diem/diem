script {
fun main() {
  assert((true && false) == false, 99);
  assert((true || false) == true, 100);
  assert(!true == false, 101);
  assert(!false == true, 102);
  assert(!!true == true, 103);
  assert(!!false == false, 104);
}
}
