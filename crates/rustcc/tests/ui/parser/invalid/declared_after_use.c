// RUN: ${{rustcc}}
// EXPECT-FAILURE

int main(void) {
    a = 1 + 2;
    int a;
    return a;
}