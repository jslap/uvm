// Phase C stub: expected to SKIP (header not found) until libc++ is vendored
// and wired in via -cxx-isystem. See uvclang/plan_cpp_stdlib.md, Phase C.
#include <cassert>
#include <cstdio>
#include <utility>

int main()
{
    std::pair<int, int> p{3, 4};
    std::swap(p.first, p.second);
    int x = 10;
    int y = std::move(x);
    printf("%d %d %d\n", p.first, p.second, y);
    assert(p.first == 4 && p.second == 3);
    return p.first + p.second + y;
}
