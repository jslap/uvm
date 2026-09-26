// Phase C stub: expected to SKIP (header not found) until libc++ is vendored
// and wired in via -cxx-isystem. See uvclang/plan_cpp_stdlib.md, Phase C.
#include <cassert>
#include <cstdio>
#include <optional>

std::optional<int> maybe_half(int x)
{
    if (x % 2 == 0)
        return x / 2;
    return std::nullopt;
}

int main()
{
    auto a = maybe_half(10);
    auto b = maybe_half(7);
    printf("a.has_value=%d b.has_value=%d\n", a.has_value(), b.has_value());
    assert(a.has_value() && *a == 5);
    assert(!b.has_value());
    return a.value_or(-1);
}
