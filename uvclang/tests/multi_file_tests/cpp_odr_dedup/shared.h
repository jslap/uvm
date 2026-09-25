// Included by both a.cpp and main.cpp: clang emits `square` and
// `Point::sum` with linkonce_odr linkage in each translation unit that
// instantiates them. Multi-TU codegen must keep exactly one copy (the ODR
// guarantees every TU's copy is identical) rather than erroring on the
// "duplicate" definition.
inline int square(int x) { return x * x; }

struct Point {
    int x, y;
    int sum() const { return x + y; }
};
