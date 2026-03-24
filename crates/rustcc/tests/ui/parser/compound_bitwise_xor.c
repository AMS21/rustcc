// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    int to_xor = 7;
    to_xor ^= 5;
    return to_xor;
}