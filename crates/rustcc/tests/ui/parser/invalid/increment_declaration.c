// RUN: ${{rustcc}}
// EXPECT-FAILURE

int main(void) {
    int a++;
    return 0;
}