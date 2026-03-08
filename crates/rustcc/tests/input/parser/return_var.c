// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    int a = 2;
    return a;
}