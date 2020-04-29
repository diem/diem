script {
use 0x0::LibraSystem;

// Script for adding a new validator
// Will only succeed when run by the Association address
fun main(new_validator: address) {
  LibraSystem::add_validator(new_validator);
}
}
