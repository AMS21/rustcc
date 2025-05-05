// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    return (3 / 2 * 4) + (5 - 4 + 3);
}