// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    return 33 >> 2 << 1;
}
