// RUN: ${{rustcc}}
// EXPECT-FAILURE

int main(void)
{
    int 10 = 0;
    return 10;
}