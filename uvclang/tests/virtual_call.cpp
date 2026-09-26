#include <stdio.h>

class ObjA
{
    public:
    virtual int f() {return a;}
     int a = 1;
};

class ObjB : public ObjA
{
    public:
    virtual int f() {return b;}
     int b = 99;
};


int main()
{
    ObjA * oa = new ObjA;
    oa->a = 10;
    printf("a = %d\n", oa->f());
    delete oa;

    ObjA * ob = new ObjB;
    ob->a = 10;
    printf("a = %d\n", ob->f());
    delete ob;

    return 0;
}