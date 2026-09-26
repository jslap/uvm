// Phase C stub: expected to SKIP (header not found) until libc++ is vendored
// and wired in via -cxx-isystem. See uvclang/plan_cpp_stdlib.md, Phase C.
// First real test of allocator growth (push_back reallocation).
#include <cassert>
#include <cstdio>
#include <vector>

int main()
{
    std::vector<int> v;
    for (int i = 0; i < 10; i++)
        v.push_back(i);

    int sum = 0;
    for (int x : v)
        sum += x;

    printf("size=%zu sum=%d\n", v.size(), sum);
    assert(v.size() == 10);
    assert(sum == 45);
    return sum;
}
