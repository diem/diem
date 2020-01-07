#include <new>

// compressor for cuckaroo nodes where edgetrimming
// has left at most a fraction 2^-compressbits nodes in each partition
template <typename word_t>
class compressor {
public:
  u32 NODEBITS;
  u32 SHIFTBITS;
  u32 SIZEBITS;
  word_t SIZE;
  word_t SIZE2;
  word_t MASK;
  word_t MASK2;
  word_t nnodes;
  const word_t NIL = ~(word_t)0;
  word_t *nodes;
  bool sharedmem;

  compressor(u32 nodebits, u32 compressbits, char *bytes) {
    NODEBITS = nodebits;
    SHIFTBITS = compressbits;
    SIZEBITS = NODEBITS-compressbits;
    SIZE = (word_t)1 << SIZEBITS;
    SIZE2 = (word_t)2 << SIZEBITS;
    nodes = new (bytes) word_t[SIZE2];
    sharedmem = true;
    MASK = SIZE-1;
    MASK2 = SIZE2-1;
  }

  compressor(u32 nodebits, u32 compressbits) {
    NODEBITS = nodebits;
    SHIFTBITS = compressbits;
    SIZEBITS = NODEBITS-compressbits;
    SIZE = (word_t)1 << SIZEBITS;
    SIZE2 = (word_t)2 << SIZEBITS;
    nodes = new word_t[SIZE2];
    sharedmem = false;
    MASK = SIZE-1;
    MASK2 = SIZE2-1;
  }

  ~compressor() {
    if (!sharedmem)
      delete[] nodes;
  }

  uint64_t bytes() {
    return sizeof(word_t[SIZE2]);
  }

  void reset() {
    memset(nodes, (char)NIL, sizeof(word_t[SIZE2]));
    nnodes = 0;
  }

  word_t compress(word_t u) {
    word_t ui = u >> SHIFTBITS;
    for (; ; ui = (ui+1) & MASK2) {
      word_t cu = nodes[ui];
      if (cu == NIL) {
        if (nnodes >= SIZE) {
          print_log("NODE OVERFLOW at %x\n", u);
          return 0;
        }
        nodes[ui] = u << SIZEBITS | nnodes;
        return nnodes++;
      }
      if ((cu & ~MASK) == u << SIZEBITS) {
        return cu & MASK;
      }
    }
  }
};
