// Minimal virtual-dispatch case: one class, one virtual method, no
// inheritance, no destructor. First thing that should compile once vtable
// support lands.
#include <assert.h>
#include <stdio.h>

class Shape
{
public:
    int tag = 7;
    virtual int area() { return 42; }
};

int main()
{
    Shape s;
    Shape* p = &s;
    printf("area = %d, tag = %d\n", p->area(), p->tag);
    assert(p->area() == 42);
    assert(p->tag == 7);
    return p->area();
}
