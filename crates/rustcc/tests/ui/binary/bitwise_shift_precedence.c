// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    return 40 << 4 + 12 >> 1;
}
