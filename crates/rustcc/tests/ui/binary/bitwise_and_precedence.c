// RUN: ${{rustcc}} --print-ast --print-ir

int main(void) {
    // & has lower precedence than ==
    return 5 & 7 == 5;
}