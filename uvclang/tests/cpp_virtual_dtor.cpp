// Virtual destructor dispatch: `delete basePtr` through a Base* pointing at
// a Derived object must run Derived's destructor body, then Base's, and free
// the object at its real (Derived) size via the D0 deleting-destructor vtable
// slot -- the integration point between vtables (Phase A) and operator delete
// (Phase B).
#include <assert.h>
#include <stdio.h>

int g_destructed = 0;

class Base
{
public:
    virtual ~Base() { g_destructed += 1; }
    virtual int id() { return 1; }
};

class Derived : public Base
{
public:
    ~Derived() override { g_destructed += 10; }
    int id() override { return 2; }
};

int main()
{
    Base* p = new Derived();
    printf("id = %d\n", p->id());
    assert(p->id() == 2);
    delete p;
    printf("destructed = %d\n", g_destructed);
    assert(g_destructed == 11);
    return g_destructed;
}
