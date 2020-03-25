use 0x0::LibraVersion;

fun main(major: u64) {
    LibraVersion::set(major)
}
