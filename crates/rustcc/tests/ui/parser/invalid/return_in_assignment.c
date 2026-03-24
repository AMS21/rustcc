// RUN: ${{rustcc}}
// EXPECT-FAILURE

int main(void)
{
    int 10 = return 0;
}