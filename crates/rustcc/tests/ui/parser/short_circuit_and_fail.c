// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    int a = 0;
    0 && (a = 5);
    return a;
}