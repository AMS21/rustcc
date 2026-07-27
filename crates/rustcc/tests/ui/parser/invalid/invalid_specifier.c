// RUN: ${{rustcc}}
// EXPECT-FAILURE

int main(void) {
    int foo bar = 3;
    return bar;
}
