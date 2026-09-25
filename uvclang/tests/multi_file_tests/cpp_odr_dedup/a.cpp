#include "shared.h"

int use_from_a(int v)
{
    Point p{v, v};
    return square(v) + p.sum();
}
