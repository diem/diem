// Script for adding a new validator
// Will only succeed when run by the Association address
fun main(new_validator: address) {
  0x0::LibraSystem::add_validator(new_validator);
}
