#include <stdio.h>

void say () {
    
}

int add(int a, int b) {
    say();
    return a + b;
}

int main() {
    int a = 1;
    int b = 2;
    int c = add(a, b);
    return 0;
}