

class Obj
{
    public:
     int a = 1;
};


int main()
{
    Obj * o = new Obj;
    int retVal = o->a;
    delete o;
    return retVal;
}