// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    int var0;
    var0 = 2;
    return var0;
}
