#include "shared.h"
#include <stdio.h>

int use_from_a(int v);

int main()
{
    Point p{3, 4};
    printf("%d %d %d\n", square(5), p.sum(), use_from_a(2));
    return square(5) + p.sum() + use_from_a(2);
}
