// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    return 5 >= 0 > 1 <= 0;
}
