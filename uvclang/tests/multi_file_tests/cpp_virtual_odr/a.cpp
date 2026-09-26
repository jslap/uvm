#include "shared.h"

int use_from_a()
{
    Greeter g;
    Greeter* p = &g;
    return p->greet();
}
