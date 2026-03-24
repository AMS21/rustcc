// RUN: ${{rustcc}}
// EXPECT-FAILURE

int main(void) {
    ints a = 1;
    return a;
}