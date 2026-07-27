// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    return 20 >> 4 <= 3 << 1;
}
