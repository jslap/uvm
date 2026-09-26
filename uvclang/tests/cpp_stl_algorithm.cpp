// Phase C stub: expected to SKIP (header not found) until libc++ is vendored
// and wired in via -cxx-isystem. See uvclang/plan_cpp_stdlib.md, Phase C.
#include <algorithm>
#include <array>
#include <cassert>
#include <cstdio>

int main()
{
    std::array<int, 5> a{5, 3, 4, 1, 2};
    std::sort(a.begin(), a.end());
    auto it = std::find(a.begin(), a.end(), 3);
    printf("sorted[0]=%d found_idx=%ld\n", a[0], (long)(it - a.begin()));
    assert(a[0] == 1 && a[4] == 5);
    assert(it != a.end() && *it == 3);
    return a[0] + a[4];
}
