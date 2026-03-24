// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    int a;
    int b = a = 0;
    return b;
}