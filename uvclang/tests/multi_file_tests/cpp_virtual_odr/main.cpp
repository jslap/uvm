#include "shared.h"
#include <stdio.h>

int use_from_a();

int main()
{
    Greeter g;
    printf("%d %d\n", g.greet(), use_from_a());
    return g.greet() + use_from_a();
}
