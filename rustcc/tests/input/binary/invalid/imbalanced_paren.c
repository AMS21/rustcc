// RUN: ${{rustcc}}
// EXPECT-FAILURE

int main(void) {
    return 1 + (2;
}