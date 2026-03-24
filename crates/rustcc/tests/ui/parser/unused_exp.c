// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    2 + 2;
    return 0;
}