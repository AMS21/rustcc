// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    return (0 <= 2) + (0 <= 0);
}