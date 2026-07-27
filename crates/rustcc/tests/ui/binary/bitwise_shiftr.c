// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    return 1000 >> 4;
}
