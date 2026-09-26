// Phase C stub: expected to SKIP (header not found) until libc++ is vendored
// and wired in via -cxx-isystem. See uvclang/plan_cpp_stdlib.md, Phase C.
#include <cassert>
#include <cstdio>
#include <string>

int main()
{
    std::string s = "hello";
    s += ", world";
    printf("%s (len=%zu)\n", s.c_str(), s.size());
    assert(s == "hello, world");
    assert(s.size() == 12);
    return (int)s.size();
}
