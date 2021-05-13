module 0x42::TestQuantInvariant {
    use Std::Vector;
    spec module {
        pragma verify = true;
    }

    fun vector_of_proper_positives(): vector<u64> {
        let v = Vector::empty();
        Vector::push_back(&mut v, 1);
        Vector::push_back(&mut v, 2);
        Vector::push_back(&mut v, 3);
        v
    }
    spec vector_of_proper_positives {
        aborts_if false;
        ensures forall n in result: n > 0;
        ensures forall i in 0..len(result), j in 0..len(result) where result[i] == result[j] : i == j;
        ensures forall i : u64, j : u64 { result[i], result[j] }
            where result[i] == result[j] && i >= 0 && i < len(result) && j >= 0 && j < len(result) : i == j;
        ensures forall i in 0..len(result) { result[i] }: {let i = result[i]; i > 0};
    }
}
