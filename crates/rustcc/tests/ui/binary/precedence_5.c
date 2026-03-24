// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    return (0 == 0 && 3 == 2 + 1 > 1) + 1;
}