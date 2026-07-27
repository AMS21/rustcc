// RUN: ${{rustcc}}
// EXPECT-FAILURE

int main(void)
{
    10 <= !;
}
