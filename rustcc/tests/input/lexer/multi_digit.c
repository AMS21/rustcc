// RUN: ${{rustcc}} --print-tokens

int main(void) {
    // test case w/ multi-digit constant
    return 100;
}