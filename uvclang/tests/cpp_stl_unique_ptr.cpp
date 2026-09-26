// Phase C stub: expected to SKIP (header not found) until libc++ is vendored
// and wired in via -cxx-isystem. See uvclang/plan_cpp_stdlib.md, Phase C.
#include <cassert>
#include <cstdio>
#include <memory>

struct Counter
{
    static int live;
    Counter() { live++; }
    ~Counter() { live--; }
    int value = 7;
};
int Counter::live = 0;

int main()
{
    {
        std::unique_ptr<Counter> p = std::make_unique<Counter>();
        printf("live=%d value=%d\n", Counter::live, p->value);
        assert(Counter::live == 1);
        assert(p->value == 7);
    }
    printf("live after scope=%d\n", Counter::live);
    assert(Counter::live == 0);
    return 0;
}
