// Included by both a.cpp and main.cpp: a virtual class instantiated in more
// than one TU, so its vtable gets linkonce_odr linkage exactly like the
// inline-function/method case in cpp_odr_dedup. Multi-TU codegen must keep
// exactly one copy of the vtable rather than erroring on "duplicate
// definition".
class Greeter
{
public:
    virtual int greet() { return 42; }
};
