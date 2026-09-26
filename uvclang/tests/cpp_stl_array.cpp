// Phase C stub: expected to SKIP (header not found) until libc++ is vendored
// and wired in via -cxx-isystem. See uvclang/plan_cpp_stdlib.md, Phase C.
#include <array>
#include <cassert>
#include <cstdio>

int main()
{
    std::array<int, 4> a{1, 2, 3, 4};
    int sum = 0;
    for (int v : a)
        sum += v;
    printf("sum = %d, size = %zu\n", sum, a.size());
    assert(sum == 10);
    assert(a.size() == 4);
    return sum;
}
