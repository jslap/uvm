#include <unordered_map>


int main()
{
    std::unordered_map<int, int> m;
    m.insert({45, 66});
    m.insert({100, 660});
    m.insert({145, 6600});

    int r = 0;
    r += (m.size() == 3);
    r += (m.count(45) == 1);
    r += (m.count(44) == 0);
    r += (m[45] == 66);
    r += (m[100] == 660);
    r += (m[145] == 6600);

    return r;
}