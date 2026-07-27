// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    return ~2 * -2 == 1 + 5;
}
