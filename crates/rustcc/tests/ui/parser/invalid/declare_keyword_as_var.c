// RUN: ${{rustcc}}
// EXPECT-FAILURE

int main(void) {
    int return = 4;
    return return + 1;
}
