// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    int x = 3;
    return x << 3;
}