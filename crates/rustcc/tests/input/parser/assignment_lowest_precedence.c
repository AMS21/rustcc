// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    int a;
    a = 0 || 5;
    return a;
}