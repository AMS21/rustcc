// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    int a = a = 5;
    return a;
}