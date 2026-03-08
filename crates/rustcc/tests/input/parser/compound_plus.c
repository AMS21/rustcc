// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    int to_add = 0;
    to_add += 4;
    return to_add;
}