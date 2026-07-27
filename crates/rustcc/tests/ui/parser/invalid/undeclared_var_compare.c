// RUN: ${{rustcc}}
// EXPECT-FAILURE

int main(void) {
    return a < 5;
}
