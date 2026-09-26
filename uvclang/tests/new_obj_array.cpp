// Array new/delete: exercises operator new[]/delete[] (_Znam/_ZdaPvm).
#include <assert.h>
#include <stdio.h>

int main()
{
    int* arr = new int[5];
    for (int i = 0; i < 5; i++)
        arr[i] = i * i;

    int sum = 0;
    for (int i = 0; i < 5; i++)
        sum += arr[i];

    printf("sum = %d\n", sum);
    assert(sum == 0 + 1 + 4 + 9 + 16);
    delete[] arr;
    return sum;
}
