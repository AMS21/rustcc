// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    return (1 > 2) + (1 > 1);
}