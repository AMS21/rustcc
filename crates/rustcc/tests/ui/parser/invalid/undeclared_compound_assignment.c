// RUN: ${{rustcc}}
// EXPECT-FAILURE

int main(void) {
    a += 1;
    return 0;
}
