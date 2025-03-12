// RUN: ${{rustcc}} --print-tokens
// EXPECT-FAILURE

int main(void)
{
    return @b;
}