// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    return 80 >> 2 | 1 ^ 5 & 7 << 1;
}
