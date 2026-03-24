// RUN: ${{rustcc}} --print-tokens
// EXPECT-FAILURE

int main()
{
    return 99999999999999999999;
}