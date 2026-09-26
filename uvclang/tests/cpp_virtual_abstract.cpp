// Abstract base class (pure virtual method + virtual destructor), one
// concrete override, called and destroyed through a base pointer. Exercises
// the __cxa_pure_virtual vtable slot being present-but-never-called.
#include <assert.h>
#include <stdio.h>

class Iface
{
public:
    virtual ~Iface() {}
    virtual int value() = 0;
};

class Impl : public Iface
{
public:
    Impl(int v) : _v(v) {}
    ~Impl() override { printf("Impl destructed\n"); }
    int value() override { return _v; }
private:
    int _v;
};

int main()
{
    Iface* p = new Impl(99);
    printf("value = %d\n", p->value());
    assert(p->value() == 99);
    delete p;
    return 0;
}
