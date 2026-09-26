// Phase C stub: expected to SKIP (header not found) until libc++ is vendored
// and wired in via -cxx-isystem. See uvclang/plan_cpp_stdlib.md, Phase C.
// Exercises shared_ptr's atomic refcounted control block (see the plan's
// note on confirming this lowers to ordinary atomicrmw/cmpxchg IR).
#include <cassert>
#include <cstdio>
#include <memory>

int main()
{
    std::shared_ptr<int> a = std::make_shared<int>(5);
    std::shared_ptr<int> b = a;
    printf("use_count=%ld value=%d\n", (long)a.use_count(), *a);
    assert(a.use_count() == 2);
    assert(*a == 5 && *b == 5);
    b.reset();
    printf("use_count after reset=%ld\n", (long)a.use_count());
    assert(a.use_count() == 1);
    return *a;
}
