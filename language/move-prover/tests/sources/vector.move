// dep: ../stdlib/modules/vector.move
module TestVector {
    use 0x0::Vector;

    // succeeds. [] == [].
    fun test_empty1() : (vector<u64>, vector<u64>)
    {
        let ev1 = Vector::empty<u64>();
        let ev2 = Vector::empty<u64>();
        (ev1, ev2)
    }
    spec fun test_empty1 {
        ensures result_1 == result_2;
    }

    // succeeds. len([1]) = len([]) + 1.
    fun test_length1() : (u64, u64)
    {
        let ev1 = Vector::empty();
        let ev2 = Vector::empty();
        Vector::push_back(&mut ev1, 1);
        (Vector::length<u64>(&ev1), Vector::length<u64>(&ev2))
    }

    spec fun test_length1 {
        ensures result_1 == result_2 + 1;
    }

    fun vector_of_proper_positives(): vector<u64> {
        let v = Vector::empty();
        Vector::push_back(&mut v, 1);
        Vector::push_back(&mut v, 2);
        Vector::push_back(&mut v, 3);
        v
    }

    spec fun vector_of_proper_positives {
      ensures all(result, |n| n > 0);
    }
}
