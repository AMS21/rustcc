// RUN: ${{rustcc}}
// EXPECT-FAILURE

int main(void) {
    int a = 10;
    (a += 1) -= 2;
}
