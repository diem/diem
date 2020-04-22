module M {

  struct S {
    x: u64,
    y: bool
  }

  struct R {
    s: S,
  }

  spec struct S {
    invariant x > 0 == y;
    invariant update old(x) < x;
  }

  spec struct R {
    // Test that calling a recursive function in a data invariant is detected as pure.
    invariant less10(true, s.x);
  }

  spec module {
    define less10(c: bool, x: num): bool {
      if (c) {
        less10a(c, x)
      } else {
        x < 10
      }
    }
    define less10a(c: bool, x: num): bool {
       less10(!c, x)
    }
  }
}
