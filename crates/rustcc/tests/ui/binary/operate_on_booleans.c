// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    return ~(0 && 1) - -(4 || 3);
}