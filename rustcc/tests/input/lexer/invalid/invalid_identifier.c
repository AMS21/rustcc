// RUN: ${{rustcc}} --print-tokens
// EXPECT-FAILURE

/* '1foo' is not a valid token, because identifier can't start with digits. */
int main(void) {
    return 1foo;
}