// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    return 0 || 0 && (1 / 1);
}
