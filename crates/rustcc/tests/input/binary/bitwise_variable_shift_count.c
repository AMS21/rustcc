// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    return (4 << (2 * 2)) + (100 >> (1 + 2)); // 64 + 12 = 76
}