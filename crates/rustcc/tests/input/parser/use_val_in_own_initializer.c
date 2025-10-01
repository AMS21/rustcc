// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    int a = 0 && a;
    return a;
}