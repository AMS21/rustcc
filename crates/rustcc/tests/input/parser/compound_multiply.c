// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    int to_multiply = 4;
    to_multiply *= 3;
    return to_multiply;
}