// RUN: ${{rustcc}}
// EXPECT-FAILURE

int main(void) {
    int b = 10;
    b *= a;
    return 0;
}